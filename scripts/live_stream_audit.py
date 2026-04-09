#!/usr/bin/env python3
"""Audit recent Roblox bootstrap telemetry served by the planetary worker.

Fetches the latest N records from the Cloudflare Worker telemetry endpoint,
filters them to a time window / place version, runs a battery of per-record
and aggregate assertions, and prints a human or JSON report. Designed to be
safe to run in CI: stdlib only, no third-party deps, deterministic exit codes.

Exit codes:
    0 - all assertions passed
    1 - one or more assertions failed
    2 - no records remained after filtering
    3 - HTTP / transport error fetching telemetry
"""
from __future__ import annotations

import argparse
import json
import math
import statistics
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass, field
from typing import Any, Callable, Dict, Iterable, List, Optional, Sequence, Tuple


DEFAULT_BASE_URL = "https://planetary.adpena.workers.dev"
DEFAULT_LIMIT = 25
MAX_LIMIT = 100
DEFAULT_SINCE_SECONDS = 600
# Thresholds calibrated for the current production profile:
#   - LOAD_RADIUS = 768 → 9-18 chunks in the spawn ring + background
#     prefetch of the outer ring
#   - KV-first chunk cache with Cloudflare edge caching (24h immutable)
#   - tertiary harness importing on an 8GB M1
# A cold-cache first-run in a new Cloudflare colo legitimately sees
# ~1s per chunk, so the per-record chunk_avg_latency threshold has to
# tolerate that. Once the edge cache warms, avg drops to <200ms and
# a future tightening cycle can pull these numbers back down.
# Post-osm2world port: chunks include roof + wall geometry (avg ~1.5MB vs
# ~500KB before). Fetch latency scaled proportionally. The 4+ story window
# threshold limits blowup but avg chunks are still ~2x previous. Thresholds
# recalibrated to accommodate richer geometry without masking real regressions.
# 60s startup streaming timeout + 10s initial import = ~70-75s bootstrap.
DEFAULT_MAX_BOOTSTRAP_SECONDS = 90.0
DEFAULT_MAX_AVG_LATENCY_MS = 4000.0
DEFAULT_MAX_SLOWEST_LATENCY_MS = 10000.0
DEFAULT_MAX_P95_BOOTSTRAP_SECONDS = 95.0
DEFAULT_MIN_CHUNKS = 4
DEFAULT_MIN_RECORDS = 1
DEFAULT_MIN_SUCCESS_RATE = 0.9

EXIT_OK = 0
EXIT_ASSERTION_FAILED = 1
EXIT_NO_RECORDS = 2
EXIT_HTTP_ERROR = 3


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def percentile(values: Sequence[float], pct: float) -> Optional[float]:
    """Return the ``pct`` percentile of ``values`` using linear interpolation.

    ``pct`` is a float in the inclusive range [0, 100]. Returns ``None`` for an
    empty input. Mirrors numpy's default ("linear") interpolation so the result
    is stable for small sample sizes that ``statistics.quantiles`` rejects.
    """
    if not values:
        return None
    if pct < 0 or pct > 100:
        raise ValueError(f"pct must be in [0, 100], got {pct}")
    ordered = sorted(float(v) for v in values)
    if len(ordered) == 1:
        return ordered[0]
    rank = (pct / 100.0) * (len(ordered) - 1)
    lo = int(math.floor(rank))
    hi = int(math.ceil(rank))
    if lo == hi:
        return ordered[lo]
    frac = rank - lo
    return ordered[lo] + (ordered[hi] - ordered[lo]) * frac


def _safe_get(record: Dict[str, Any], path: Sequence[str], default: Any = None) -> Any:
    cursor: Any = record
    for key in path:
        if not isinstance(cursor, dict):
            return default
        cursor = cursor.get(key, default)
        if cursor is default and cursor is None:
            return default
    return cursor


def _record_id(record: Dict[str, Any]) -> str:
    run_id = record.get("runId")
    if isinstance(run_id, str) and run_id:
        return run_id
    ts = record.get("timestamp")
    if ts is not None:
        return f"ts={ts}"
    return "<unknown>"


# ---------------------------------------------------------------------------
# Configuration / assertion definitions
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class AuditConfig:
    base_url: str = DEFAULT_BASE_URL
    limit: int = DEFAULT_LIMIT
    since_seconds: int = DEFAULT_SINCE_SECONDS
    place_version: Optional[int] = None
    max_bootstrap_seconds: float = DEFAULT_MAX_BOOTSTRAP_SECONDS
    max_avg_latency_ms: float = DEFAULT_MAX_AVG_LATENCY_MS
    max_slowest_latency_ms: float = DEFAULT_MAX_SLOWEST_LATENCY_MS
    max_p95_bootstrap_seconds: float = DEFAULT_MAX_P95_BOOTSTRAP_SECONDS
    min_chunks: int = DEFAULT_MIN_CHUNKS
    min_records: int = DEFAULT_MIN_RECORDS
    min_success_rate: float = DEFAULT_MIN_SUCCESS_RATE
    allow_failures: bool = False
    allow_fetch_failures: bool = False


@dataclass
class AssertionFailure:
    record_id: str
    assertion: str
    message: str

    def to_dict(self) -> Dict[str, Any]:
        return {
            "record_id": self.record_id,
            "assertion": self.assertion,
            "message": self.message,
        }


# Each per-record check is a pure function returning an optional failure
# message. Keeping them as small functions makes them trivial to unit test
# without instantiating the full audit pipeline.
PerRecordCheck = Callable[[Dict[str, Any], AuditConfig], Optional[Tuple[str, str]]]


# Status values that count as a successful bootstrap. "success" is the
# canonical value; "success_post_walk" is emitted by HarnessWalkPath
# after the scripted walk completes and re-reports the flicker
# aggregates (see ServerScriptService/HarnessWalkPath.server.lua).
SUCCESS_STATUSES = frozenset(
    {
        "success",
        "success_post_walk",
        # stationary_baseline is the T+20s clean-window re-report fired
        # from BootstrapAustin.server.lua after gameplay_ready. It carries
        # the bootstrap phase timings verbatim but its flicker block is a
        # pure 20s stationary observation window — the diagnosis signal
        # for "the world flickers while I'm not moving".
        "stationary_baseline",
        # heartbeat fires every 30s for the lifetime of the session so we
        # get continuous flicker visibility rather than a single bootstrap
        # snapshot. The audit treats it as a success so a long-running
        # session doesn't fail the loop.
        "heartbeat",
    }
)

# Status values that represent a re-report of a previously-successful
# bootstrap (as opposed to a fresh bootstrap). These records carry a
# chunksImported=0 by design because the re-report is just a telemetry
# refresh, not a new import run. Skip min_chunks on these.
POST_REPORT_STATUSES = frozenset(
    {
        "success_post_walk",
        "stationary_baseline",
        "heartbeat",
    }
)


def check_bootstrap_status(record: Dict[str, Any], cfg: AuditConfig) -> Optional[Tuple[str, str]]:
    if cfg.allow_failures:
        return None
    status = _safe_get(record, ("bootstrap", "status"))
    if status not in SUCCESS_STATUSES:
        err = _safe_get(record, ("bootstrap", "errorMessage")) or "<no errorMessage>"
        return ("bootstrap_status", f"status={status!r} errorMessage={err!r}")
    return None


def check_bootstrap_elapsed(record: Dict[str, Any], cfg: AuditConfig) -> Optional[Tuple[str, str]]:
    elapsed = _safe_get(record, ("bootstrap", "totalElapsedSeconds"))
    if elapsed is None:
        return ("bootstrap_elapsed", "missing bootstrap.totalElapsedSeconds")
    try:
        elapsed_f = float(elapsed)
    except (TypeError, ValueError):
        return ("bootstrap_elapsed", f"non-numeric value {elapsed!r}")
    if elapsed_f > cfg.max_bootstrap_seconds:
        return (
            "bootstrap_elapsed",
            f"{elapsed_f:.3f}s > max {cfg.max_bootstrap_seconds:.3f}s",
        )
    return None


def check_chunk_failure_count(record: Dict[str, Any], cfg: AuditConfig) -> Optional[Tuple[str, str]]:
    if cfg.allow_fetch_failures:
        return None
    failures = _safe_get(record, ("chunkFetch", "failureCount"))
    if failures is None:
        return None
    try:
        failures_i = int(failures)
    except (TypeError, ValueError):
        return ("chunk_failure_count", f"non-numeric value {failures!r}")
    if failures_i != 0:
        return ("chunk_failure_count", f"failureCount={failures_i} (expected 0)")
    return None


def check_chunk_avg_latency(record: Dict[str, Any], cfg: AuditConfig) -> Optional[Tuple[str, str]]:
    avg = _safe_get(record, ("chunkFetch", "avgLatencyMs"))
    if avg is None:
        return None
    try:
        avg_f = float(avg)
    except (TypeError, ValueError):
        return ("chunk_avg_latency", f"non-numeric value {avg!r}")
    if avg_f > cfg.max_avg_latency_ms:
        return (
            "chunk_avg_latency",
            f"avgLatencyMs={avg_f:.1f} > max {cfg.max_avg_latency_ms:.1f}",
        )
    return None


def check_chunk_slowest_latency(
    record: Dict[str, Any], cfg: AuditConfig
) -> Optional[Tuple[str, str]]:
    slowest = _safe_get(record, ("chunkFetch", "slowestLatencyMs"))
    if slowest is None:
        return None
    try:
        slowest_f = float(slowest)
    except (TypeError, ValueError):
        return ("chunk_slowest_latency", f"non-numeric value {slowest!r}")
    if slowest_f > cfg.max_slowest_latency_ms:
        return (
            "chunk_slowest_latency",
            f"slowestLatencyMs={slowest_f:.1f} > max {cfg.max_slowest_latency_ms:.1f}",
        )
    return None


def check_min_chunks(record: Dict[str, Any], cfg: AuditConfig) -> Optional[Tuple[str, str]]:
    # Post-walk re-reports carry chunksImported=0 by design — the walk
    # script only fires TelemetryReporter.Report to refresh the flicker
    # aggregate, not to announce a new import run. Skip the min_chunks
    # check on those records so the audit doesn't flag them as broken.
    status = _safe_get(record, ("bootstrap", "status"))
    if status in POST_REPORT_STATUSES:
        return None
    chunks = _safe_get(record, ("import", "chunksImported"))
    if chunks is None:
        return ("min_chunks", "missing import.chunksImported")
    try:
        chunks_i = int(chunks)
    except (TypeError, ValueError):
        return ("min_chunks", f"non-numeric value {chunks!r}")
    if chunks_i < cfg.min_chunks:
        return (
            "min_chunks",
            f"chunksImported={chunks_i} < min {cfg.min_chunks}",
        )
    return None


PER_RECORD_CHECKS: Tuple[PerRecordCheck, ...] = (
    check_bootstrap_status,
    check_bootstrap_elapsed,
    check_chunk_failure_count,
    check_chunk_avg_latency,
    check_chunk_slowest_latency,
    check_min_chunks,
)


# ---------------------------------------------------------------------------
# Fetching
# ---------------------------------------------------------------------------


class TelemetryFetchError(RuntimeError):
    """Raised when fetching telemetry from the worker fails."""


def fetch_telemetry_records(
    base_url: str,
    limit: int,
    *,
    opener: Optional[Any] = None,
    timeout: float = 20.0,
) -> List[Dict[str, Any]]:
    """Fetch up to ``limit`` telemetry records from ``base_url``.

    ``opener`` is an optional ``urllib.request.OpenerDirector``-like object,
    primarily so unit tests can substitute a fake transport without monkey
    patching ``urllib.request.urlopen``.
    """
    if limit <= 0:
        raise ValueError("limit must be positive")
    if limit > MAX_LIMIT:
        limit = MAX_LIMIT
    base = base_url.rstrip("/")
    qs = urllib.parse.urlencode({"limit": str(limit)})
    url = f"{base}/telemetry/latest?{qs}"
    request = urllib.request.Request(
        url,
        headers={"Accept": "application/json", "User-Agent": "live-stream-audit/1.0"},
    )
    try:
        if opener is not None:
            response = opener.open(request)
        else:
            response = urllib.request.urlopen(request, timeout=timeout)  # noqa: S310
    except urllib.error.HTTPError as exc:
        raise TelemetryFetchError(
            f"HTTP {exc.code} fetching {url}: {exc.reason}"
        ) from exc
    except urllib.error.URLError as exc:
        raise TelemetryFetchError(f"URL error fetching {url}: {exc.reason}") from exc
    except (TimeoutError, OSError) as exc:
        raise TelemetryFetchError(f"transport error fetching {url}: {exc}") from exc

    try:
        with response:
            raw = response.read()
    except Exception as exc:  # noqa: BLE001
        raise TelemetryFetchError(f"failed to read telemetry body: {exc}") from exc

    try:
        payload = json.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise TelemetryFetchError(f"invalid JSON body from {url}: {exc}") from exc

    if not isinstance(payload, list):
        raise TelemetryFetchError(
            f"expected JSON array from {url}, got {type(payload).__name__}"
        )
    out: List[Dict[str, Any]] = []
    for entry in payload:
        if isinstance(entry, dict):
            out.append(entry)
    return out


# ---------------------------------------------------------------------------
# Filtering & auditing
# ---------------------------------------------------------------------------


def filter_records(
    records: Iterable[Dict[str, Any]],
    cfg: AuditConfig,
    *,
    now: Optional[float] = None,
) -> List[Dict[str, Any]]:
    """Apply --since-seconds and --place-version filters to ``records``."""
    if now is None:
        now = time.time()
    cutoff = now - cfg.since_seconds if cfg.since_seconds > 0 else None
    out: List[Dict[str, Any]] = []
    for record in records:
        if cutoff is not None:
            ts = record.get("timestamp")
            try:
                ts_f = float(ts) if ts is not None else None
            except (TypeError, ValueError):
                ts_f = None
            if ts_f is None or ts_f < cutoff:
                continue
        if cfg.place_version is not None:
            version = _safe_get(record, ("environment", "placeVersion"))
            try:
                version_i = int(version) if version is not None else None
            except (TypeError, ValueError):
                version_i = None
            if version_i != cfg.place_version:
                continue
        out.append(record)
    return out


def _collect_floats(records: Sequence[Dict[str, Any]], path: Sequence[str]) -> List[float]:
    out: List[float] = []
    for record in records:
        value = _safe_get(record, path)
        if value is None:
            continue
        try:
            out.append(float(value))
        except (TypeError, ValueError):
            continue
    return out


def compute_stats(records: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    bootstrap_elapsed = _collect_floats(records, ("bootstrap", "totalElapsedSeconds"))
    avg_latency = _collect_floats(records, ("chunkFetch", "avgLatencyMs"))
    slowest_latency = _collect_floats(records, ("chunkFetch", "slowestLatencyMs"))

    def _series(values: Sequence[float]) -> Dict[str, Optional[float]]:
        if not values:
            return {"count": 0, "avg": None, "p50": None, "p95": None}
        return {
            "count": len(values),
            "avg": statistics.fmean(values),
            "p50": percentile(values, 50.0),
            "p95": percentile(values, 95.0),
        }

    success = sum(
        1
        for r in records
        if _safe_get(r, ("bootstrap", "status")) in SUCCESS_STATUSES
    )
    total = len(records)
    failure = total - success
    success_rate = (success / total) if total > 0 else 0.0
    return {
        "total": total,
        "success": success,
        "failure": failure,
        "success_rate": success_rate,
        "bootstrap_elapsed_seconds": _series(bootstrap_elapsed),
        "chunk_avg_latency_ms": _series(avg_latency),
        "chunk_slowest_latency_ms": _series(slowest_latency),
    }


@dataclass
class AuditResult:
    config: AuditConfig
    records: List[Dict[str, Any]]
    stats: Dict[str, Any]
    failures: List[AssertionFailure] = field(default_factory=list)

    @property
    def passed(self) -> bool:
        return not self.failures


def run_per_record_checks(
    records: Sequence[Dict[str, Any]], cfg: AuditConfig
) -> List[AssertionFailure]:
    out: List[AssertionFailure] = []
    for record in records:
        rid = _record_id(record)
        for check in PER_RECORD_CHECKS:
            result = check(record, cfg)
            if result is None:
                continue
            assertion, message = result
            out.append(AssertionFailure(rid, assertion, message))
    return out


def run_aggregate_checks(
    stats: Dict[str, Any], cfg: AuditConfig
) -> List[AssertionFailure]:
    out: List[AssertionFailure] = []
    if stats["total"] < cfg.min_records:
        out.append(
            AssertionFailure(
                "<aggregate>",
                "min_records",
                f"records={stats['total']} < min {cfg.min_records}",
            )
        )
    p95 = stats["bootstrap_elapsed_seconds"]["p95"]
    if p95 is not None and p95 > cfg.max_p95_bootstrap_seconds:
        out.append(
            AssertionFailure(
                "<aggregate>",
                "p95_bootstrap_elapsed",
                f"p95={p95:.3f}s > max {cfg.max_p95_bootstrap_seconds:.3f}s",
            )
        )
    if stats["total"] > 0 and stats["success_rate"] < cfg.min_success_rate:
        out.append(
            AssertionFailure(
                "<aggregate>",
                "min_success_rate",
                f"success_rate={stats['success_rate']:.3f} < min {cfg.min_success_rate:.3f}",
            )
        )
    return out


def audit_records(
    records: Sequence[Dict[str, Any]], cfg: AuditConfig
) -> AuditResult:
    stats = compute_stats(records)
    failures: List[AssertionFailure] = []
    failures.extend(run_per_record_checks(records, cfg))
    failures.extend(run_aggregate_checks(stats, cfg))
    return AuditResult(config=cfg, records=list(records), stats=stats, failures=failures)


# ---------------------------------------------------------------------------
# Reporting
# ---------------------------------------------------------------------------


def _fmt_optional(value: Optional[float], unit: str = "") -> str:
    if value is None:
        return "n/a"
    return f"{value:.3f}{unit}" if unit else f"{value:.3f}"


def render_human_report(result: AuditResult) -> str:
    cfg = result.config
    stats = result.stats
    lines: List[str] = []
    lines.append("live_stream_audit report")
    lines.append("========================")
    lines.append(f"base_url           = {cfg.base_url}")
    lines.append(f"limit              = {cfg.limit}")
    lines.append(f"since_seconds      = {cfg.since_seconds}")
    lines.append(
        f"place_version      = {cfg.place_version if cfg.place_version is not None else '<any>'}"
    )
    lines.append(f"records_audited    = {stats['total']}")
    lines.append(
        f"success / failure  = {stats['success']} / {stats['failure']} "
        f"(rate={stats['success_rate']:.3f})"
    )

    def _emit_series(label: str, series: Dict[str, Any], unit: str = "") -> None:
        lines.append(
            f"{label:<30} count={series['count']} "
            f"avg={_fmt_optional(series['avg'], unit)} "
            f"p50={_fmt_optional(series['p50'], unit)} "
            f"p95={_fmt_optional(series['p95'], unit)}"
        )

    _emit_series("bootstrap_elapsed (s)", stats["bootstrap_elapsed_seconds"], "s")
    _emit_series("chunk_avg_latency (ms)", stats["chunk_avg_latency_ms"], "ms")
    _emit_series("chunk_slowest_latency (ms)", stats["chunk_slowest_latency_ms"], "ms")

    if result.failures:
        lines.append("")
        lines.append(f"failures ({len(result.failures)}):")
        for failure in result.failures:
            lines.append(
                f"  - [{failure.assertion}] {failure.record_id}: {failure.message}"
            )
    else:
        lines.append("")
        lines.append("failures: none")

    lines.append("")
    lines.append("PASS" if result.passed else "FAIL")
    return "\n".join(lines)


def render_json_report(result: AuditResult) -> str:
    cfg = result.config
    payload = {
        "pass": result.passed,
        "record_count": result.stats["total"],
        "config": {
            "base_url": cfg.base_url,
            "limit": cfg.limit,
            "since_seconds": cfg.since_seconds,
            "place_version": cfg.place_version,
            "max_bootstrap_seconds": cfg.max_bootstrap_seconds,
            "max_avg_latency_ms": cfg.max_avg_latency_ms,
            "max_slowest_latency_ms": cfg.max_slowest_latency_ms,
            "max_p95_bootstrap_seconds": cfg.max_p95_bootstrap_seconds,
            "min_chunks": cfg.min_chunks,
            "min_records": cfg.min_records,
            "min_success_rate": cfg.min_success_rate,
            "allow_failures": cfg.allow_failures,
            "allow_fetch_failures": cfg.allow_fetch_failures,
        },
        "stats": result.stats,
        "failures": [f.to_dict() for f in result.failures],
    }
    return json.dumps(payload, indent=2, sort_keys=True)


# ---------------------------------------------------------------------------
# CLI plumbing
# ---------------------------------------------------------------------------


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="live_stream_audit",
        description=(
            "Audit recent Roblox bootstrap telemetry posted to the planetary "
            "Cloudflare Worker. Exits non-zero if any per-record or aggregate "
            "assertion fails."
        ),
    )
    parser.add_argument("--url", dest="base_url", default=DEFAULT_BASE_URL,
                        help=f"Worker base URL (default: {DEFAULT_BASE_URL})")
    parser.add_argument("--limit", type=int, default=DEFAULT_LIMIT,
                        help=f"records to fetch from /telemetry/latest (1..{MAX_LIMIT}, default {DEFAULT_LIMIT})")
    parser.add_argument("--since-seconds", type=int, default=DEFAULT_SINCE_SECONDS,
                        help=f"only audit records newer than this (default {DEFAULT_SINCE_SECONDS}s; 0 disables)")
    parser.add_argument("--place-version", type=int, default=None,
                        help="optional placeVersion filter")
    parser.add_argument("--max-bootstrap-seconds", type=float,
                        default=DEFAULT_MAX_BOOTSTRAP_SECONDS,
                        help=f"per-record bootstrap.totalElapsedSeconds ceiling (default {DEFAULT_MAX_BOOTSTRAP_SECONDS})")
    parser.add_argument("--max-avg-latency-ms", type=float,
                        default=DEFAULT_MAX_AVG_LATENCY_MS,
                        help=f"per-record chunkFetch.avgLatencyMs ceiling (default {DEFAULT_MAX_AVG_LATENCY_MS})")
    parser.add_argument("--max-slowest-latency-ms", type=float,
                        default=DEFAULT_MAX_SLOWEST_LATENCY_MS,
                        help=f"per-record chunkFetch.slowestLatencyMs ceiling (default {DEFAULT_MAX_SLOWEST_LATENCY_MS})")
    parser.add_argument("--max-p95-bootstrap-seconds", type=float,
                        default=DEFAULT_MAX_P95_BOOTSTRAP_SECONDS,
                        help=f"aggregate p95 bootstrap.totalElapsedSeconds ceiling (default {DEFAULT_MAX_P95_BOOTSTRAP_SECONDS})")
    parser.add_argument("--min-chunks", type=int, default=DEFAULT_MIN_CHUNKS,
                        help=f"per-record import.chunksImported floor (default {DEFAULT_MIN_CHUNKS})")
    parser.add_argument("--min-records", type=int, default=DEFAULT_MIN_RECORDS,
                        help=f"aggregate minimum number of records (default {DEFAULT_MIN_RECORDS})")
    parser.add_argument("--min-success-rate", type=float, default=DEFAULT_MIN_SUCCESS_RATE,
                        help=f"aggregate minimum success rate 0..1 (default {DEFAULT_MIN_SUCCESS_RATE})")
    parser.add_argument("--allow-failures", action="store_true",
                        help="do not fail on bootstrap.status != success")
    parser.add_argument("--allow-fetch-failures", action="store_true",
                        help="do not fail on chunkFetch.failureCount > 0")
    parser.add_argument("--json", dest="emit_json", action="store_true",
                        help="emit machine-readable JSON report")
    return parser


def parse_args(argv: Optional[Sequence[str]] = None) -> argparse.Namespace:
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.limit < 1:
        parser.error("--limit must be >= 1")
    if args.limit > MAX_LIMIT:
        args.limit = MAX_LIMIT
    if args.since_seconds < 0:
        parser.error("--since-seconds must be >= 0")
    if not 0.0 <= args.min_success_rate <= 1.0:
        parser.error("--min-success-rate must be in [0.0, 1.0]")
    return args


def args_to_config(args: argparse.Namespace) -> AuditConfig:
    return AuditConfig(
        base_url=args.base_url,
        limit=args.limit,
        since_seconds=args.since_seconds,
        place_version=args.place_version,
        max_bootstrap_seconds=args.max_bootstrap_seconds,
        max_avg_latency_ms=args.max_avg_latency_ms,
        max_slowest_latency_ms=args.max_slowest_latency_ms,
        max_p95_bootstrap_seconds=args.max_p95_bootstrap_seconds,
        min_chunks=args.min_chunks,
        min_records=args.min_records,
        min_success_rate=args.min_success_rate,
        allow_failures=args.allow_failures,
        allow_fetch_failures=args.allow_fetch_failures,
    )


def main(
    argv: Optional[Sequence[str]] = None,
    *,
    fetcher: Optional[Callable[[str, int], List[Dict[str, Any]]]] = None,
    now: Optional[float] = None,
    stdout: Optional[Any] = None,
    stderr: Optional[Any] = None,
) -> int:
    args = parse_args(argv)
    cfg = args_to_config(args)

    out_stream = stdout if stdout is not None else sys.stdout
    err_stream = stderr if stderr is not None else sys.stderr

    fetch = fetcher if fetcher is not None else fetch_telemetry_records
    try:
        raw_records = fetch(cfg.base_url, cfg.limit)
    except TelemetryFetchError as exc:
        print(f"live_stream_audit: telemetry fetch failed: {exc}", file=err_stream)
        return EXIT_HTTP_ERROR

    filtered = filter_records(raw_records, cfg, now=now)
    if not filtered:
        message = (
            f"live_stream_audit: no records matched filters "
            f"(fetched={len(raw_records)}, since={cfg.since_seconds}s, "
            f"place_version={cfg.place_version})"
        )
        if args.emit_json:
            payload = {
                "pass": False,
                "record_count": 0,
                "stats": compute_stats([]),
                "failures": [
                    {
                        "record_id": "<aggregate>",
                        "assertion": "no_records",
                        "message": message,
                    }
                ],
            }
            print(json.dumps(payload, indent=2, sort_keys=True), file=out_stream)
        else:
            print(message, file=err_stream)
        return EXIT_NO_RECORDS

    result = audit_records(filtered, cfg)
    if args.emit_json:
        print(render_json_report(result), file=out_stream)
    else:
        print(render_human_report(result), file=out_stream)
    return EXIT_OK if result.passed else EXIT_ASSERTION_FAILED


if __name__ == "__main__":
    sys.exit(main())
