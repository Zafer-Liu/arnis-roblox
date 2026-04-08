#!/usr/bin/env python3
"""Cross-reference compile-time chunk profile data with runtime telemetry.

Runs ``arbx_cli profile <index.json> --split --json`` to collect per-chunk
static cost estimates, fetches the latest runtime telemetry records from the
planetary Cloudflare Worker, and flags chunks whose compile cost and/or
runtime fetch latency exceed 2 standard deviations above the mean.

The intent is to close the signal gap between "what is expensive to import"
(compile-time: byte size, feature count, polygon vertices, estimated cost)
and "what is slow at runtime" (live: slowestChunkId, slowestLatencyMs,
avgLatencyMs) so ops can pinpoint chunks that are heavy AND slow rather than
one or the other.

Exit codes:
    0 - analysis ran, no assertion failures
    1 - --assert-hot-count N exceeded
    2 - no data to analyze (empty profile or no overlapping chunks)
    3 - subprocess or HTTP error
"""
from __future__ import annotations

import argparse
import json
import os
import statistics
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass, field
from typing import Any, Callable, Dict, Iterable, List, Optional, Sequence, Tuple


DEFAULT_INDEX_PATH = "/tmp/austin-mfst/index.json"
DEFAULT_ARBX_CLI = "/Users/adpena/Projects/arnis-roblox/rust/target/release/arbx_cli"
DEFAULT_WORKER_URL = "https://planetary.adpena.workers.dev"
DEFAULT_LIMIT = 25
DEFAULT_SINCE_SECONDS = 3600
DEFAULT_TOP = 10
DEFAULT_STDDEV_THRESHOLD = 2.0
MAX_LIMIT = 100

EXIT_OK = 0
EXIT_ASSERTION_FAILED = 1
EXIT_NO_DATA = 2
EXIT_SUBPROCESS_ERROR = 3


# ---------------------------------------------------------------------------
# Errors
# ---------------------------------------------------------------------------


class ProfileSubprocessError(RuntimeError):
    """Raised when invoking arbx_cli fails or returns invalid JSON."""


class TelemetryFetchError(RuntimeError):
    """Raised when fetching telemetry from the worker fails."""


# ---------------------------------------------------------------------------
# Configuration / data model
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class CrossConfig:
    index_path: str = DEFAULT_INDEX_PATH
    arbx_cli: str = DEFAULT_ARBX_CLI
    worker_url: str = DEFAULT_WORKER_URL
    limit: int = DEFAULT_LIMIT
    since_seconds: int = DEFAULT_SINCE_SECONDS
    top: int = DEFAULT_TOP
    stddev_threshold: float = DEFAULT_STDDEV_THRESHOLD
    assert_hot_count: Optional[int] = None


@dataclass
class ChunkProfile:
    id: str
    bytes: int
    feature_count: int
    polygon_vertices: int
    terrain_filled_cells: int
    estimated_import_cost: float

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ChunkProfile":
        return cls(
            id=str(data.get("id", "")),
            bytes=int(data.get("bytes", 0) or 0),
            feature_count=int(data.get("feature_count", 0) or 0),
            polygon_vertices=int(data.get("polygon_vertices", 0) or 0),
            terrain_filled_cells=int(data.get("terrain_filled_cells", 0) or 0),
            estimated_import_cost=float(data.get("estimated_import_cost", 0.0) or 0.0),
        )


@dataclass
class RuntimeSample:
    chunk_id: str
    latency_ms: float
    run_id: str
    timestamp: Optional[float]
    source_url: Optional[str]


@dataclass
class HotChunk:
    chunk_id: str
    profile: ChunkProfile
    runtime: RuntimeSample
    cross_score: float
    cost_z: float
    latency_z: float
    flag_reasons: List[str] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "chunk_id": self.chunk_id,
            "cross_score": self.cross_score,
            "cost_z": self.cost_z,
            "latency_z": self.latency_z,
            "flag_reasons": list(self.flag_reasons),
            "profile": {
                "bytes": self.profile.bytes,
                "feature_count": self.profile.feature_count,
                "polygon_vertices": self.profile.polygon_vertices,
                "terrain_filled_cells": self.profile.terrain_filled_cells,
                "estimated_import_cost": self.profile.estimated_import_cost,
            },
            "runtime": {
                "latency_ms": self.runtime.latency_ms,
                "run_id": self.runtime.run_id,
                "timestamp": self.runtime.timestamp,
                "source_url": self.runtime.source_url,
            },
        }


@dataclass
class CrossReport:
    config: CrossConfig
    profile_count: int
    runtime_sample_count: int
    overlap_count: int
    cost_mean: float
    cost_stdev: float
    latency_mean: float
    latency_stdev: float
    hot_chunks: List[HotChunk]
    all_crossed: List[HotChunk]
    assertion_failure: Optional[str] = None

    @property
    def passed(self) -> bool:
        return self.assertion_failure is None


# ---------------------------------------------------------------------------
# Profile subprocess
# ---------------------------------------------------------------------------


def run_arbx_profile(
    arbx_cli: str,
    index_path: str,
    *,
    runner: Optional[Callable[[List[str]], Tuple[int, str, str]]] = None,
) -> List[ChunkProfile]:
    """Invoke ``arbx_cli profile <index> --split --json`` and parse output."""
    if runner is None and not os.path.exists(arbx_cli):
        raise ProfileSubprocessError(f"arbx_cli binary not found at {arbx_cli}")
    cmd = [arbx_cli, "profile", index_path, "--split", "--json"]
    if runner is not None:
        try:
            rc, stdout, stderr = runner(cmd)
        except Exception as exc:  # noqa: BLE001
            raise ProfileSubprocessError(f"runner raised: {exc}") from exc
    else:
        try:
            proc = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                check=False,
                timeout=120,
            )
        except FileNotFoundError as exc:
            raise ProfileSubprocessError(f"failed to exec {arbx_cli}: {exc}") from exc
        except subprocess.TimeoutExpired as exc:
            raise ProfileSubprocessError(f"arbx_cli profile timed out: {exc}") from exc
        rc, stdout, stderr = proc.returncode, proc.stdout, proc.stderr
    if rc != 0:
        raise ProfileSubprocessError(
            f"arbx_cli profile exited {rc}: {stderr.strip() or '<no stderr>'}"
        )
    try:
        payload = json.loads(stdout)
    except json.JSONDecodeError as exc:
        raise ProfileSubprocessError(f"arbx_cli profile emitted invalid JSON: {exc}") from exc
    chunks = payload.get("chunks")
    if not isinstance(chunks, list):
        raise ProfileSubprocessError("arbx_cli profile JSON missing 'chunks' array")
    out: List[ChunkProfile] = []
    for entry in chunks:
        if isinstance(entry, dict):
            out.append(ChunkProfile.from_dict(entry))
    return out


# ---------------------------------------------------------------------------
# Telemetry fetch
# ---------------------------------------------------------------------------


def fetch_telemetry_records(
    worker_url: str,
    limit: int,
    *,
    opener: Optional[Any] = None,
    timeout: float = 20.0,
) -> List[Dict[str, Any]]:
    if limit <= 0:
        raise ValueError("limit must be positive")
    if limit > MAX_LIMIT:
        limit = MAX_LIMIT
    base = worker_url.rstrip("/")
    qs = urllib.parse.urlencode({"limit": str(limit)})
    url = f"{base}/telemetry/latest?{qs}"
    request = urllib.request.Request(
        url,
        headers={
            "Accept": "application/json",
            "User-Agent": "profile-telemetry-cross/1.0",
        },
    )
    try:
        if opener is not None:
            response = opener.open(request)
        else:
            response = urllib.request.urlopen(request, timeout=timeout)  # noqa: S310
    except urllib.error.HTTPError as exc:
        raise TelemetryFetchError(f"HTTP {exc.code} fetching {url}: {exc.reason}") from exc
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
    return [e for e in payload if isinstance(e, dict)]


# ---------------------------------------------------------------------------
# Filtering / extraction
# ---------------------------------------------------------------------------


def _safe_get(record: Dict[str, Any], path: Sequence[str]) -> Any:
    cursor: Any = record
    for key in path:
        if not isinstance(cursor, dict):
            return None
        cursor = cursor.get(key)
    return cursor


def extract_runtime_samples(
    records: Iterable[Dict[str, Any]],
    *,
    since_seconds: int,
    now: Optional[float] = None,
) -> List[RuntimeSample]:
    """Pull (slowestChunkId, slowestLatencyMs) pairs from filtered records."""
    if now is None:
        now = time.time()
    cutoff = now - since_seconds if since_seconds > 0 else None
    out: List[RuntimeSample] = []
    for record in records:
        ts_raw = record.get("timestamp")
        try:
            ts = float(ts_raw) if ts_raw is not None else None
        except (TypeError, ValueError):
            ts = None
        if cutoff is not None and (ts is None or ts < cutoff):
            continue
        chunk_id = _safe_get(record, ("chunkFetch", "slowestChunkId"))
        latency = _safe_get(record, ("chunkFetch", "slowestLatencyMs"))
        if not isinstance(chunk_id, str) or not chunk_id:
            continue
        try:
            latency_f = float(latency)
        except (TypeError, ValueError):
            continue
        run_id = record.get("runId")
        source_url = _safe_get(record, ("chunkFetch", "sourceUrl"))
        out.append(
            RuntimeSample(
                chunk_id=chunk_id,
                latency_ms=latency_f,
                run_id=str(run_id) if isinstance(run_id, str) else "<unknown>",
                timestamp=ts,
                source_url=source_url if isinstance(source_url, str) else None,
            )
        )
    return out


def _aggregate_worst_sample(samples: Iterable[RuntimeSample]) -> Dict[str, RuntimeSample]:
    """Collapse multiple samples per chunk to the slowest observed."""
    out: Dict[str, RuntimeSample] = {}
    for sample in samples:
        existing = out.get(sample.chunk_id)
        if existing is None or sample.latency_ms > existing.latency_ms:
            out[sample.chunk_id] = sample
    return out


# ---------------------------------------------------------------------------
# Analysis
# ---------------------------------------------------------------------------


def _mean_stdev(values: Sequence[float]) -> Tuple[float, float]:
    if not values:
        return 0.0, 0.0
    mean = statistics.fmean(values)
    stdev = statistics.pstdev(values) if len(values) > 1 else 0.0
    return mean, stdev


def _z(value: float, mean: float, stdev: float) -> float:
    if stdev <= 0.0:
        return 0.0
    return (value - mean) / stdev


def analyze(
    profiles: Sequence[ChunkProfile],
    samples: Sequence[RuntimeSample],
    cfg: CrossConfig,
) -> CrossReport:
    profile_map: Dict[str, ChunkProfile] = {p.id: p for p in profiles}
    worst_runtime = _aggregate_worst_sample(samples)

    costs = [p.estimated_import_cost for p in profiles]
    latencies = [s.latency_ms for s in worst_runtime.values()]
    cost_mean, cost_stdev = _mean_stdev(costs)
    latency_mean, latency_stdev = _mean_stdev(latencies)

    overlap_ids = sorted(set(profile_map.keys()) & set(worst_runtime.keys()))
    crossed: List[HotChunk] = []
    for chunk_id in overlap_ids:
        profile = profile_map[chunk_id]
        sample = worst_runtime[chunk_id]
        cost_z = _z(profile.estimated_import_cost, cost_mean, cost_stdev)
        latency_z = _z(sample.latency_ms, latency_mean, latency_stdev)
        norm_cost = profile.estimated_import_cost / cost_mean if cost_mean > 0 else 0.0
        norm_latency = sample.latency_ms / latency_mean if latency_mean > 0 else 0.0
        cross_score = norm_cost * norm_latency
        reasons: List[str] = []
        if cost_z > cfg.stddev_threshold:
            reasons.append(
                f"cost_z={cost_z:.2f}>+{cfg.stddev_threshold:.1f}"
            )
        if latency_z > cfg.stddev_threshold:
            reasons.append(
                f"latency_z={latency_z:.2f}>+{cfg.stddev_threshold:.1f}"
            )
        crossed.append(
            HotChunk(
                chunk_id=chunk_id,
                profile=profile,
                runtime=sample,
                cross_score=cross_score,
                cost_z=cost_z,
                latency_z=latency_z,
                flag_reasons=reasons,
            )
        )

    crossed.sort(key=lambda h: h.cross_score, reverse=True)
    hot = [h for h in crossed if h.flag_reasons]

    report = CrossReport(
        config=cfg,
        profile_count=len(profiles),
        runtime_sample_count=len(worst_runtime),
        overlap_count=len(overlap_ids),
        cost_mean=cost_mean,
        cost_stdev=cost_stdev,
        latency_mean=latency_mean,
        latency_stdev=latency_stdev,
        hot_chunks=hot,
        all_crossed=crossed,
    )

    if cfg.assert_hot_count is not None and len(hot) > cfg.assert_hot_count:
        report.assertion_failure = (
            f"hot_chunks={len(hot)} > assert_hot_count={cfg.assert_hot_count}"
        )
    return report


# ---------------------------------------------------------------------------
# Reporting
# ---------------------------------------------------------------------------


def render_human_report(report: CrossReport) -> str:
    cfg = report.config
    lines: List[str] = []
    lines.append("profile_telemetry_cross report")
    lines.append("==============================")
    lines.append(f"index_path          = {cfg.index_path}")
    lines.append(f"worker_url          = {cfg.worker_url}")
    lines.append(f"limit / since       = {cfg.limit} / {cfg.since_seconds}s")
    lines.append(f"profile_chunks      = {report.profile_count}")
    lines.append(f"runtime_samples     = {report.runtime_sample_count}")
    lines.append(f"overlap_chunks      = {report.overlap_count}")
    lines.append(
        f"cost    mean/stdev  = {report.cost_mean:.2f} / {report.cost_stdev:.2f}"
    )
    lines.append(
        f"latency mean/stdev  = {report.latency_mean:.1f}ms / {report.latency_stdev:.1f}ms"
    )
    lines.append(f"stddev_threshold    = +{cfg.stddev_threshold:.1f}sigma")
    lines.append("")
    top = report.all_crossed[: cfg.top]
    if not top:
        lines.append("no overlapping chunks between profile and telemetry")
    else:
        lines.append(f"top {len(top)} cross-scored chunks:")
        header = (
            f"{'rank':>4} {'chunk_id':<14} {'score':>8} {'cost':>10} "
            f"{'cost_z':>8} {'latency_ms':>11} {'lat_z':>8}  flags"
        )
        lines.append(header)
        lines.append("-" * len(header))
        for idx, h in enumerate(top, start=1):
            flags = ",".join(h.flag_reasons) if h.flag_reasons else "-"
            lines.append(
                f"{idx:>4} {h.chunk_id:<14} {h.cross_score:>8.2f} "
                f"{h.profile.estimated_import_cost:>10.1f} "
                f"{h.cost_z:>+8.2f} {h.runtime.latency_ms:>11.1f} "
                f"{h.latency_z:>+8.2f}  {flags}"
            )
    lines.append("")
    lines.append(f"hot_chunks flagged: {len(report.hot_chunks)}")
    for h in report.hot_chunks:
        lines.append(f"  - {h.chunk_id}: {', '.join(h.flag_reasons)}")
    lines.append("")
    if report.assertion_failure:
        lines.append(f"assertion FAIL: {report.assertion_failure}")
        lines.append("FAIL")
    else:
        lines.append("PASS")
    return "\n".join(lines)


def render_json_report(report: CrossReport) -> str:
    cfg = report.config
    payload = {
        "pass": report.passed,
        "config": {
            "index_path": cfg.index_path,
            "arbx_cli": cfg.arbx_cli,
            "worker_url": cfg.worker_url,
            "limit": cfg.limit,
            "since_seconds": cfg.since_seconds,
            "top": cfg.top,
            "stddev_threshold": cfg.stddev_threshold,
            "assert_hot_count": cfg.assert_hot_count,
        },
        "stats": {
            "profile_count": report.profile_count,
            "runtime_sample_count": report.runtime_sample_count,
            "overlap_count": report.overlap_count,
            "cost_mean": report.cost_mean,
            "cost_stdev": report.cost_stdev,
            "latency_mean": report.latency_mean,
            "latency_stdev": report.latency_stdev,
        },
        "top": [h.to_dict() for h in report.all_crossed[: cfg.top]],
        "hot_chunks": [h.to_dict() for h in report.hot_chunks],
        "assertion_failure": report.assertion_failure,
    }
    return json.dumps(payload, indent=2, sort_keys=True)


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="profile_telemetry_cross",
        description=(
            "Cross-reference compile-time chunk profile data (via arbx_cli "
            "profile) with runtime telemetry (via the planetary Cloudflare "
            "Worker) and flag chunks that are both heavy and slow."
        ),
    )
    parser.add_argument("--index", dest="index_path", default=DEFAULT_INDEX_PATH,
                        help=f"split-manifest index.json (default {DEFAULT_INDEX_PATH})")
    parser.add_argument("--arbx-cli", dest="arbx_cli", default=DEFAULT_ARBX_CLI,
                        help=f"arbx_cli binary path (default {DEFAULT_ARBX_CLI})")
    parser.add_argument("--worker-url", dest="worker_url", default=DEFAULT_WORKER_URL,
                        help=f"telemetry worker base URL (default {DEFAULT_WORKER_URL})")
    parser.add_argument("--limit", type=int, default=DEFAULT_LIMIT,
                        help=f"telemetry records to fetch (1..{MAX_LIMIT}, default {DEFAULT_LIMIT})")
    parser.add_argument("--since-seconds", type=int, default=DEFAULT_SINCE_SECONDS,
                        help=f"only consider records newer than this (default {DEFAULT_SINCE_SECONDS}s)")
    parser.add_argument("--top", type=int, default=DEFAULT_TOP,
                        help=f"top N hot chunks to display (default {DEFAULT_TOP})")
    parser.add_argument("--stddev-threshold", type=float, default=DEFAULT_STDDEV_THRESHOLD,
                        help=f"z-score flag threshold (default {DEFAULT_STDDEV_THRESHOLD})")
    parser.add_argument("--assert-hot-count", type=int, default=None,
                        help="fail (exit 1) if more than N chunks flagged hot")
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
    if args.top < 1:
        parser.error("--top must be >= 1")
    if args.stddev_threshold < 0:
        parser.error("--stddev-threshold must be >= 0")
    if args.assert_hot_count is not None and args.assert_hot_count < 0:
        parser.error("--assert-hot-count must be >= 0")
    return args


def args_to_config(args: argparse.Namespace) -> CrossConfig:
    return CrossConfig(
        index_path=args.index_path,
        arbx_cli=args.arbx_cli,
        worker_url=args.worker_url,
        limit=args.limit,
        since_seconds=args.since_seconds,
        top=args.top,
        stddev_threshold=args.stddev_threshold,
        assert_hot_count=args.assert_hot_count,
    )


def main(
    argv: Optional[Sequence[str]] = None,
    *,
    profile_runner: Optional[Callable[[str, str], List[ChunkProfile]]] = None,
    telemetry_fetcher: Optional[Callable[[str, int], List[Dict[str, Any]]]] = None,
    now: Optional[float] = None,
    stdout: Optional[Any] = None,
    stderr: Optional[Any] = None,
) -> int:
    args = parse_args(argv)
    cfg = args_to_config(args)

    out_stream = stdout if stdout is not None else sys.stdout
    err_stream = stderr if stderr is not None else sys.stderr

    try:
        if profile_runner is not None:
            profiles = profile_runner(cfg.arbx_cli, cfg.index_path)
        else:
            profiles = run_arbx_profile(cfg.arbx_cli, cfg.index_path)
    except ProfileSubprocessError as exc:
        print(f"profile_telemetry_cross: profile failed: {exc}", file=err_stream)
        return EXIT_SUBPROCESS_ERROR

    if not profiles:
        message = "profile_telemetry_cross: empty profile (no chunks)"
        if args.emit_json:
            print(json.dumps({"pass": False, "error": message}, indent=2), file=out_stream)
        else:
            print(message, file=err_stream)
        return EXIT_NO_DATA

    try:
        fetch = telemetry_fetcher if telemetry_fetcher is not None else fetch_telemetry_records
        raw_records = fetch(cfg.worker_url, cfg.limit)
    except TelemetryFetchError as exc:
        print(f"profile_telemetry_cross: telemetry fetch failed: {exc}", file=err_stream)
        return EXIT_SUBPROCESS_ERROR

    samples = extract_runtime_samples(raw_records, since_seconds=cfg.since_seconds, now=now)
    report = analyze(profiles, samples, cfg)

    if report.overlap_count == 0:
        message = (
            f"profile_telemetry_cross: no overlap between profile "
            f"({report.profile_count} chunks) and telemetry "
            f"({report.runtime_sample_count} chunks)"
        )
        if args.emit_json:
            print(render_json_report(report), file=out_stream)
        else:
            print(render_human_report(report), file=out_stream)
            print(message, file=err_stream)
        return EXIT_NO_DATA

    if args.emit_json:
        print(render_json_report(report), file=out_stream)
    else:
        print(render_human_report(report), file=out_stream)

    return EXIT_OK if report.passed else EXIT_ASSERTION_FAILED


if __name__ == "__main__":
    sys.exit(main())
