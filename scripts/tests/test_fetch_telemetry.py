from __future__ import annotations

import importlib.util
import io
import json
import sys
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Optional
from unittest import mock


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "fetch_telemetry.py"


def load_module():
    scripts_dir = str(MODULE_PATH.parent)
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
    spec = importlib.util.spec_from_file_location("fetch_telemetry", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load module spec from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules["fetch_telemetry"] = module
    spec.loader.exec_module(module)
    return module


def make_record(
    *,
    run_id: str = "ab12cd34ef",
    received_iso: str = "2026-04-08T13:42:05Z",
    country: str = "US",
    status: str = "success",
    place_version: int = 35,
    elapsed: float = 5.23,
    fetch_count: int = 7,
    avg_ms: int = 145,
    worst_ms: int = 230,
    fail: int = 0,
    error_message: Optional[str] = None,
    error_detail: Optional[str] = None,
) -> dict[str, Any]:
    return {
        "runId": run_id,
        "timestamp": 1712600000.0,
        "place": {"placeId": 1, "universeId": 2, "serverJobId": "job"},
        "bootstrap": {
            "status": status,
            "totalElapsedSeconds": elapsed,
            "errorMessage": error_message,
            "errorDetail": error_detail,
            "phases": {
                "loading_manifest": 0.3,
                "importing_startup": 2.1,
                "world_ready": 2.5,
                "streaming_ready": 4.5,
                "minimap_ready": 4.8,
                "gameplay_ready": elapsed,
            },
        },
        "chunkFetch": {
            "sourceUrl": "https://example/manifest.json",
            "fetchCount": fetch_count,
            "failureCount": fail,
            "totalBytes": 3500000,
            "avgLatencyMs": avg_ms,
            "slowestLatencyMs": worst_ms,
            "slowestChunkId": "chunk-0",
        },
        "import": {"chunksImported": fetch_count, "totalInstances": 1234, "totalFeatures": 245},
        "environment": {"placeVersion": place_version, "sourceUrl": "https://example/m.json"},
        "edge": {
            "receivedAtIso": received_iso,
            "ipHash": "abc1234567890def",
            "country": country,
            "cfRay": "ray",
            "colo": "DFW",
        },
    }


class _FakeResponse:
    def __init__(self, payload: bytes, status: int = 200) -> None:
        self._buf = io.BytesIO(payload)
        self.status = status

    def read(self) -> bytes:
        return self._buf.read()

    def __enter__(self) -> "_FakeResponse":
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        self._buf.close()


class FormatRecordRowTests(unittest.TestCase):
    def test_format_record_row_basic_fields(self) -> None:
        module = load_module()
        record = make_record()
        row = module.format_record_row(record)
        # Time + country header
        self.assertIn("13:42:05 US", row)
        self.assertIn("success", row)
        self.assertIn("35", row)
        self.assertIn("5.23s", row)
        # 7 chunks, 145 avg, 230 worst, 0 failures, all in their columns.
        self.assertIn("7", row)
        self.assertIn("145", row)
        self.assertIn("230", row)
        self.assertTrue(row.rstrip().endswith("0"))

    def test_format_record_row_truncates_long_run_id(self) -> None:
        module = load_module()
        row = module.format_record_row(make_record(run_id="abcdefghijklmnop"))
        self.assertIn("abcdef", row)
        self.assertIn("\u2026", row)

    def test_format_record_row_handles_failed_status(self) -> None:
        module = load_module()
        row = module.format_record_row(
            make_record(
                status="failed",
                elapsed=0.41,
                fetch_count=0,
                avg_ms=0,
                worst_ms=0,
                error_message="boom",
            )
        )
        self.assertIn("failed", row)
        self.assertIn("0.41s", row)


class FilterSinceTests(unittest.TestCase):
    def test_filter_since_drops_old_records(self) -> None:
        module = load_module()
        now = datetime(2026, 4, 8, 13, 42, 5, tzinfo=timezone.utc)
        recent = make_record(
            run_id="recent",
            received_iso=(now - timedelta(seconds=10)).strftime("%Y-%m-%dT%H:%M:%SZ"),
        )
        old = make_record(
            run_id="old",
            received_iso=(now - timedelta(seconds=600)).strftime("%Y-%m-%dT%H:%M:%SZ"),
        )
        out = module.filter_since([recent, old], 60, now=now)
        self.assertEqual([record["runId"] for record in out], ["recent"])

    def test_filter_since_zero_passthrough(self) -> None:
        module = load_module()
        records = [make_record(run_id="a"), make_record(run_id="b")]
        out = module.filter_since(records, 0)
        self.assertEqual(len(out), 2)

    def test_filter_since_skips_unparseable_iso(self) -> None:
        module = load_module()
        record = make_record(received_iso="not-a-real-iso")
        out = module.filter_since([record], 9999)
        self.assertEqual(out, [])


class DedupeByRunIdTests(unittest.TestCase):
    def test_dedupe_keeps_first_occurrence(self) -> None:
        module = load_module()
        records = [
            make_record(run_id="a"),
            make_record(run_id="b"),
            make_record(run_id="a"),
            make_record(run_id="c"),
            make_record(run_id="b"),
        ]
        out = module.dedupe_by_run_id(records)
        self.assertEqual([record["runId"] for record in out], ["a", "b", "c"])

    def test_dedupe_drops_blank_run_ids(self) -> None:
        module = load_module()
        out = module.dedupe_by_run_id([{"runId": ""}, {"runId": "x"}])
        self.assertEqual([record["runId"] for record in out], ["x"])


class ParseArgsTests(unittest.TestCase):
    def test_defaults(self) -> None:
        module = load_module()
        args = module.parse_args([])
        self.assertEqual(args.limit, module.DEFAULT_LIMIT)
        self.assertFalse(args.as_json)
        self.assertFalse(args.watch)
        self.assertIsNone(args.run_id)
        self.assertIsNone(args.since)
        self.assertEqual(args.url, module.DEFAULT_BASE_URL)

    def test_overrides(self) -> None:
        module = load_module()
        args = module.parse_args(
            [
                "--limit", "20",
                "--json",
                "--since", "120",
                "--url", "https://example.test",
                "--run-id", "ab12",
            ]
        )
        self.assertEqual(args.limit, 20)
        self.assertTrue(args.as_json)
        self.assertEqual(args.since, 120.0)
        self.assertEqual(args.url, "https://example.test")
        self.assertEqual(args.run_id, "ab12")


class MainTests(unittest.TestCase):
    def _patch_fetch(self, module, payload):
        body = json.dumps(payload).encode("utf-8")
        return mock.patch.object(
            module.urllib_request,
            "urlopen",
            return_value=_FakeResponse(body),
        )

    def test_main_returns_zero_on_success(self) -> None:
        module = load_module()
        records = [make_record(run_id="ab12cd34"), make_record(run_id="ef34gh56")]
        stdout = io.StringIO()
        with self._patch_fetch(module, records), mock.patch("sys.stdout", stdout):
            exit_code = module.main(["--limit", "2"])
        self.assertEqual(exit_code, 0)
        output = stdout.getvalue()
        self.assertIn("runId", output)
        self.assertIn("ab12cd", output)

    def test_main_returns_two_when_since_filters_everything(self) -> None:
        module = load_module()
        # received well before our --since window
        old = make_record(run_id="old", received_iso="2000-01-01T00:00:00Z")
        stderr = io.StringIO()
        with self._patch_fetch(module, [old]), mock.patch("sys.stderr", stderr):
            exit_code = module.main(["--since", "60"])
        self.assertEqual(exit_code, 2)
        self.assertIn("no records", stderr.getvalue())

    def test_main_returns_one_on_http_error(self) -> None:
        module = load_module()
        from urllib import error as urllib_error  # local import for clarity

        def boom(*_args, **_kwargs):
            raise urllib_error.URLError("connection refused")

        stderr = io.StringIO()
        with mock.patch.object(module.urllib_request, "urlopen", side_effect=boom), mock.patch(
            "sys.stderr", stderr
        ):
            exit_code = module.main([])
        self.assertEqual(exit_code, 1)
        self.assertIn("telemetry fetch failed", stderr.getvalue())

    def test_main_json_mode_emits_valid_json(self) -> None:
        module = load_module()
        records = [make_record(run_id="ab12cd34")]
        stdout = io.StringIO()
        with self._patch_fetch(module, records), mock.patch("sys.stdout", stdout):
            exit_code = module.main(["--json"])
        self.assertEqual(exit_code, 0)
        parsed = json.loads(stdout.getvalue())
        self.assertEqual(parsed[0]["runId"], "ab12cd34")


if __name__ == "__main__":
    unittest.main()
