#!/usr/bin/env python3
"""Fetch and pretty-print bootstrap telemetry from the planetary Cloudflare Worker.

The companion Cloudflare Worker exposes ``GET /telemetry/latest?limit=N`` which
returns the N most recent runtime bootstrap records. Each record is the client
payload merged with a server-side ``edge`` block (receivedAtIso, country, etc).

This CLI is stdlib-only and read-only: it never POSTs and only issues GET
requests against the configured worker URL.

Usage:
    python3 scripts/fetch_telemetry.py                  # latest 5, table
    python3 scripts/fetch_telemetry.py --limit 25 --json
    python3 scripts/fetch_telemetry.py --watch          # 5s poll, dedup
    python3 scripts/fetch_telemetry.py --since 120      # last 2 minutes
    python3 scripts/fetch_telemetry.py --run-id ab12cd34
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from datetime import datetime, timezone
from typing import Any, Iterable, Optional
from urllib import error as urllib_error
from urllib import parse as urllib_parse
from urllib import request as urllib_request


DEFAULT_BASE_URL = "https://planetary.adpena.workers.dev"
DEFAULT_LIMIT = 5
WATCH_INTERVAL_SECONDS = 5
HTTP_TIMEOUT_SECONDS = 15
USER_AGENT = "arnis-roblox-fetch-telemetry/1.0"

# Table column widths (kept here so format_record_row stays declarative).
COL_RECV = 11      # HH:MM:SS CC
COL_RUN = 7        # short runId + ellipsis
COL_STATUS = 7
COL_PLACEV = 7
COL_ELAPSED = 7
COL_CHUNKS = 6
COL_AVG_MS = 12
COL_WORST_MS = 16
COL_FAIL = 4


# ----- HTTP -------------------------------------------------------------------


def fetch_latest(
    base_url: str,
    limit: int,
    *,
    opener: Optional[urllib_request.OpenerDirector] = None,
) -> list[dict[str, Any]]:
    """GET ``/telemetry/latest?limit=N`` and return the parsed JSON list.

    Raises ``RuntimeError`` on any HTTP/transport/JSON failure so callers can
    convert the error into the documented exit codes without leaking tracebacks.
    """

    if limit < 1 or limit > 100:
        raise ValueError(f"limit must be between 1 and 100, got {limit}")

    url = base_url.rstrip("/") + "/telemetry/latest?" + urllib_parse.urlencode({"limit": limit})
    http_request = urllib_request.Request(
        url,
        method="GET",
        headers={"Accept": "application/json", "User-Agent": USER_AGENT},
    )

    try:
        opener_fn = opener.open if opener is not None else urllib_request.urlopen
        response = opener_fn(http_request, timeout=HTTP_TIMEOUT_SECONDS)  # noqa: S310
    except urllib_error.HTTPError as exc:
        detail = ""
        try:
            detail = exc.read().decode("utf-8", errors="replace")
        except Exception:  # noqa: BLE001
            pass
        raise RuntimeError(
            f"telemetry fetch failed: HTTP {exc.code} {exc.reason} {detail}".strip()
        ) from exc
    except urllib_error.URLError as exc:
        raise RuntimeError(f"telemetry fetch failed: {exc.reason}") from exc

    with response:
        raw = response.read().decode("utf-8", errors="replace")
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"telemetry endpoint returned non-JSON body: {raw[:200]}") from exc
    if not isinstance(payload, list):
        raise RuntimeError(f"telemetry endpoint returned non-list body: {type(payload).__name__}")
    return payload


# ----- Filters ---------------------------------------------------------------


def parse_iso_timestamp(value: str) -> Optional[datetime]:
    """Parse an ISO-8601 timestamp from the worker (handles trailing 'Z')."""

    if not value:
        return None
    candidate = value.strip()
    if candidate.endswith("Z"):
        candidate = candidate[:-1] + "+00:00"
    try:
        parsed = datetime.fromisoformat(candidate)
    except ValueError:
        return None
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed


def filter_since(
    records: Iterable[dict[str, Any]],
    seconds: float,
    *,
    now: Optional[datetime] = None,
) -> list[dict[str, Any]]:
    """Return only records whose ``edge.receivedAtIso`` is within ``seconds``."""
    if seconds <= 0:
        return list(records)
    cutoff_now = now or datetime.now(timezone.utc)
    out: list[dict[str, Any]] = []
    for record in records:
        received = parse_iso_timestamp((record.get("edge") or {}).get("receivedAtIso", ""))
        if received is not None and (cutoff_now - received).total_seconds() <= seconds:
            out.append(record)
    return out


def filter_run_id(records: Iterable[dict[str, Any]], run_id: str) -> list[dict[str, Any]]:
    return [record for record in records if str(record.get("runId", "")) == run_id]


def dedupe_by_run_id(records: Iterable[dict[str, Any]]) -> list[dict[str, Any]]:
    """Keep only the first occurrence of each runId, preserving order."""

    seen: set[str] = set()
    out: list[dict[str, Any]] = []
    for record in records:
        run_id = str(record.get("runId", ""))
        if not run_id or run_id in seen:
            continue
        seen.add(run_id)
        out.append(record)
    return out


# ----- Pretty printing -------------------------------------------------------


def _short_run_id(run_id: str, width: int = COL_RUN) -> str:
    if len(run_id) <= width:
        return run_id.ljust(width)
    keep = width - 1
    return (run_id[:keep] + "\u2026").ljust(width)


def _short_recv(received_iso: str, country: str) -> str:
    parsed = parse_iso_timestamp(received_iso)
    if parsed is None:
        clock = "--:--:--"
    else:
        clock = parsed.strftime("%H:%M:%S")
    cc = (country or "--")[:2].rjust(2)
    return f"{clock} {cc}".ljust(COL_RECV)


_HEADER_COLS = [
    ("recv", COL_RECV, "ljust"), ("runId", COL_RUN, "ljust"),
    ("status", COL_STATUS, "ljust"), ("place v", COL_PLACEV, "rjust"),
    ("elapsed", COL_ELAPSED, "rjust"), ("chunks", COL_CHUNKS, "rjust"),
    ("fetch ms avg", COL_AVG_MS, "rjust"), ("fetch ms p-worst", COL_WORST_MS, "rjust"),
    ("fail", COL_FAIL, "rjust"),
]


def _join_row(cells: list[str]) -> str:
    return " " + " | ".join(cells)


def format_table_header() -> list[str]:
    width = sum(c[1] for c in _HEADER_COLS) + 3 * (len(_HEADER_COLS) - 1) + 1
    bar = "=" * width
    cells = [getattr(label, side)(w) for label, w, side in _HEADER_COLS]
    return [bar, _join_row(cells), bar]


def format_record_row(record: dict[str, Any]) -> str:
    """Render one record as a single table row. Pure function for tests."""

    edge = record.get("edge") or {}
    bootstrap = record.get("bootstrap") or {}
    chunk = record.get("chunkFetch") or {}
    env = record.get("environment") or {}

    elapsed_value = bootstrap.get("totalElapsedSeconds")
    if isinstance(elapsed_value, (int, float)):
        elapsed_text = f"{float(elapsed_value):.2f}s"
    else:
        elapsed_text = "-"
    place_version = env.get("placeVersion")

    cells = [
        _short_recv(str(edge.get("receivedAtIso", "")), str(edge.get("country", ""))),
        _short_run_id(str(record.get("runId", ""))),
        str(bootstrap.get("status", "?"))[:COL_STATUS].ljust(COL_STATUS),
        (str(place_version) if place_version is not None else "-").rjust(COL_PLACEV),
        elapsed_text.rjust(COL_ELAPSED),
        str(int(chunk.get("fetchCount") or 0)).rjust(COL_CHUNKS),
        str(int(chunk.get("avgLatencyMs") or 0)).rjust(COL_AVG_MS),
        str(int(chunk.get("slowestLatencyMs") or 0)).rjust(COL_WORST_MS),
        str(int(chunk.get("failureCount") or 0)).rjust(COL_FAIL),
    ]
    return _join_row(cells)


def format_run_detail(record: dict[str, Any]) -> list[str]:
    """Verbose detail block, used for ``--run-id`` lookups."""

    bootstrap = record.get("bootstrap") or {}
    phases = bootstrap.get("phases") or {}
    lines: list[str] = []
    lines.append("")
    lines.append(f"runId={record.get('runId', '?')}")
    if phases:
        lines.append("  phases:")
        for phase in sorted(phases.keys()):
            value = phases[phase]
            if isinstance(value, (int, float)):
                lines.append(f"    {phase:<20s} {float(value):>7.3f}s")
            else:
                lines.append(f"    {phase:<20s} {value}")
    err_message = bootstrap.get("errorMessage")
    err_detail = bootstrap.get("errorDetail")
    if err_message:
        lines.append(f"  errorMessage: {err_message}")
    if err_detail:
        lines.append(f"  errorDetail:  {err_detail}")
    return lines


def render_table(records: list[dict[str, Any]]) -> str:
    out = format_table_header()
    for record in records:
        out.append(format_record_row(record))
    out.append(out[0])  # closing bar
    return "\n".join(out)


# ----- CLI -------------------------------------------------------------------


def parse_args(argv: Optional[list[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Fetch bootstrap telemetry from the planetary Cloudflare Worker.",
    )
    parser.add_argument("--url", default=DEFAULT_BASE_URL,
                        help=f"Base worker URL (default: {DEFAULT_BASE_URL}).")
    parser.add_argument("--limit", type=int, default=DEFAULT_LIMIT,
                        help="How many records to request (1-100, default 5).")
    parser.add_argument("--json", action="store_true", dest="as_json",
                        help="Emit raw JSON instead of the human-readable table.")
    parser.add_argument("--watch", action="store_true",
                        help="Poll every 5 seconds and print only newly-seen records.")
    parser.add_argument("--run-id", default=None,
                        help="Filter to a single run and print its phase detail.")
    parser.add_argument("--since", type=float, default=None,
                        help="Only show records received within the last N seconds.")
    return parser.parse_args(argv)


def _emit_records(
    records: list[dict[str, Any]],
    *,
    as_json: bool,
    run_id: Optional[str],
    stream,
) -> None:
    if as_json:
        json.dump(records, stream, indent=2, sort_keys=True)
        stream.write("\n")
        return
    stream.write(render_table(records) + "\n")
    if run_id:
        for record in records:
            for line in format_run_detail(record):
                stream.write(line + "\n")


def main(argv: Optional[list[str]] = None) -> int:
    args = parse_args(argv)

    if args.limit < 1 or args.limit > 100:
        print("[fetch_telemetry] --limit must be between 1 and 100", file=sys.stderr)
        return 1

    def fetch_once() -> list[dict[str, Any]]:
        records = fetch_latest(args.url, args.limit)
        if args.run_id:
            records = filter_run_id(records, args.run_id)
        if args.since is not None:
            records = filter_since(records, args.since)
        return records

    if not args.watch:
        try:
            records = fetch_once()
        except (RuntimeError, ValueError) as exc:
            print(f"[fetch_telemetry] {exc}", file=sys.stderr)
            return 1
        if args.since is not None and not records:
            print(f"[fetch_telemetry] no records received in the last {args.since:g}s",
                  file=sys.stderr)
            return 2
        _emit_records(records, as_json=args.as_json, run_id=args.run_id, stream=sys.stdout)
        return 0

    # --watch loop: stream new records as they appear, dedup by runId.
    seen: set[str] = set()
    try:
        while True:
            try:
                records = fetch_once()
            except (RuntimeError, ValueError) as exc:
                print(f"[fetch_telemetry] {exc}", file=sys.stderr)
                return 1
            fresh = [r for r in records if str(r.get("runId", "")) not in seen]
            for record in fresh:
                seen.add(str(record.get("runId", "")))
            if fresh:
                _emit_records(fresh, as_json=args.as_json, run_id=args.run_id, stream=sys.stdout)
                sys.stdout.flush()
            time.sleep(WATCH_INTERVAL_SECONDS)
    except KeyboardInterrupt:
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
