from __future__ import annotations

import importlib.util
import io
import json
import sys
import unittest
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence
from unittest import mock


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "live_stream_audit.py"


def load_module():
    scripts_dir = str(MODULE_PATH.parent)
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
    spec = importlib.util.spec_from_file_location("live_stream_audit", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load module spec from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules["live_stream_audit"] = module
    spec.loader.exec_module(module)
    return module


# A timestamp roughly "now" used in test fixtures. Tests pin --now via the
# main() now= kwarg so the value just needs to be deterministic and recent
# enough to fall inside the default --since-seconds window.
FIXED_NOW = 1_712_600_000.0


def make_record(
    *,
    run_id: str = "run-1",
    timestamp: float = FIXED_NOW - 30.0,
    status: str = "success",
    elapsed: float = 5.2,
    error_message: Optional[str] = None,
    fetch_count: int = 7,
    failure_count: int = 0,
    avg_latency: float = 145.0,
    slowest_latency: float = 230.0,
    chunks_imported: int = 7,
    place_version: int = 35,
) -> Dict[str, Any]:
    return {
        "runId": run_id,
        "timestamp": timestamp,
        "place": {
            "placeId": 108781748738397,
            "universeId": 10006866306,
            "serverJobId": "job-abc",
        },
        "bootstrap": {
            "status": status,
            "totalElapsedSeconds": elapsed,
            "errorMessage": error_message,
            "errorDetail": None,
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
            "sourceUrl": "https://example.test/manifest.json",
            "fetchCount": fetch_count,
            "failureCount": failure_count,
            "totalBytes": 3_500_000,
            "avgLatencyMs": avg_latency,
            "slowestLatencyMs": slowest_latency,
            "slowestChunkId": "chunk-7",
        },
        "import": {
            "chunksImported": chunks_imported,
            "totalInstances": 1234,
            "totalFeatures": 245,
        },
        "environment": {
            "placeVersion": place_version,
            "sourceUrl": "https://example.test/manifest.json",
        },
        "edge": {
            "receivedAtIso": "2026-04-08T16:00:00Z",
            "ipHash": "abc1234567890def",
            "country": "US",
            "cfRay": "ray-1",
            "colo": "DFW",
        },
    }


def make_passing_records(n: int = 3) -> List[Dict[str, Any]]:
    return [make_record(run_id=f"run-{i}", timestamp=FIXED_NOW - (i * 10)) for i in range(n)]


def fake_fetcher(records: Sequence[Dict[str, Any]]):
    def _fetch(base_url: str, limit: int) -> List[Dict[str, Any]]:  # noqa: ARG001
        return list(records)
    return _fetch


def run_main(
    module,
    records: Sequence[Dict[str, Any]],
    extra_args: Optional[Sequence[str]] = None,
):
    stdout = io.StringIO()
    stderr = io.StringIO()
    argv = list(extra_args or [])
    exit_code = module.main(
        argv,
        fetcher=fake_fetcher(records),
        now=FIXED_NOW,
        stdout=stdout,
        stderr=stderr,
    )
    return exit_code, stdout.getvalue(), stderr.getvalue()


class PercentileTests(unittest.TestCase):
    def test_empty_returns_none(self) -> None:
        module = load_module()
        self.assertIsNone(module.percentile([], 50.0))

    def test_single_value(self) -> None:
        module = load_module()
        self.assertEqual(module.percentile([4.2], 95.0), 4.2)

    def test_small_list_p50_p95(self) -> None:
        module = load_module()
        values = [1.0, 2.0, 3.0, 4.0, 5.0]
        self.assertAlmostEqual(module.percentile(values, 50.0), 3.0)
        self.assertAlmostEqual(module.percentile(values, 95.0), 4.8)
        self.assertAlmostEqual(module.percentile(values, 0.0), 1.0)
        self.assertAlmostEqual(module.percentile(values, 100.0), 5.0)

    def test_large_list_p95(self) -> None:
        module = load_module()
        values = [float(i) for i in range(1, 101)]  # 1..100
        # Linear interpolation: rank = 0.95 * 99 = 94.05 -> 95.05
        self.assertAlmostEqual(module.percentile(values, 95.0), 95.05)
        self.assertAlmostEqual(module.percentile(values, 50.0), 50.5)

    def test_invalid_percent(self) -> None:
        module = load_module()
        with self.assertRaises(ValueError):
            module.percentile([1.0, 2.0], 150.0)


class PassingRunTests(unittest.TestCase):
    def test_defaults_pass_clean_dataset(self) -> None:
        module = load_module()
        records = make_passing_records(3)
        exit_code, stdout, stderr = run_main(module, records)
        self.assertEqual(exit_code, 0, msg=f"stdout={stdout}\nstderr={stderr}")
        self.assertIn("PASS", stdout)
        self.assertIn("records_audited    = 3", stdout)
        self.assertNotIn("FAIL", stdout)


class SlowAvgLatencyTests(unittest.TestCase):
    def test_slow_avg_latency_fails_with_message(self) -> None:
        module = load_module()
        # Raised to 1500ms (above the new 1200ms default threshold) after
        # LOAD_RADIUS bumped to 768 and cold-cache colos legitimately see
        # ~1s/chunk on first load. Still well above a healthy warm-cache
        # reading so the test continues to catch real regressions.
        records = [
            make_record(run_id="ok", avg_latency=120.0),
            make_record(run_id="slow", avg_latency=1500.0),
        ]
        exit_code, stdout, _ = run_main(module, records)
        self.assertEqual(exit_code, 1)
        self.assertIn("FAIL", stdout)
        self.assertIn("chunk_avg_latency", stdout)
        self.assertIn("slow", stdout)
        self.assertIn("1500", stdout)


class FailureRecordTests(unittest.TestCase):
    def test_failure_record_fails_by_default(self) -> None:
        module = load_module()
        records = [
            make_record(run_id="ok"),
            make_record(
                run_id="boom",
                status="failed",
                error_message="manifest blew up",
            ),
        ]
        exit_code, stdout, _ = run_main(module, records)
        self.assertEqual(exit_code, 1)
        self.assertIn("bootstrap_status", stdout)
        self.assertIn("boom", stdout)

    def test_allow_failures_flag_passes(self) -> None:
        module = load_module()
        # 9 successes + 1 failure keeps success_rate at 0.9, satisfying the
        # default --min-success-rate threshold.
        records = [make_record(run_id=f"ok-{i}") for i in range(9)]
        records.append(
            make_record(
                run_id="boom",
                status="failed",
                error_message="oh no",
            )
        )
        exit_code, stdout, _ = run_main(module, records, ["--allow-failures"])
        self.assertEqual(exit_code, 0, msg=stdout)
        self.assertIn("PASS", stdout)


class WindowFilterTests(unittest.TestCase):
    def test_empty_after_filter_returns_exit_2(self) -> None:
        module = load_module()
        # All records older than the default 600s window.
        old_records = [
            make_record(run_id=f"old-{i}", timestamp=FIXED_NOW - 10_000.0 - i)
            for i in range(3)
        ]
        exit_code, _, stderr = run_main(module, old_records)
        self.assertEqual(exit_code, 2)
        self.assertIn("no records matched filters", stderr)

    def test_empty_after_filter_json_emits_payload(self) -> None:
        module = load_module()
        old_records = [make_record(run_id="old", timestamp=FIXED_NOW - 99_999.0)]
        exit_code, stdout, _ = run_main(module, old_records, ["--json"])
        self.assertEqual(exit_code, 2)
        payload = json.loads(stdout)
        self.assertFalse(payload["pass"])
        self.assertEqual(payload["record_count"], 0)
        self.assertEqual(payload["failures"][0]["assertion"], "no_records")


class HttpErrorTests(unittest.TestCase):
    def test_fetch_error_returns_exit_3(self) -> None:
        module = load_module()

        def boom(base_url: str, limit: int):  # noqa: ARG001
            raise module.TelemetryFetchError("HTTP 500 fetching telemetry")

        stdout = io.StringIO()
        stderr = io.StringIO()
        exit_code = module.main(
            [],
            fetcher=boom,
            now=FIXED_NOW,
            stdout=stdout,
            stderr=stderr,
        )
        self.assertEqual(exit_code, 3)
        self.assertIn("HTTP 500", stderr.getvalue())

    def test_fetcher_real_path_handles_http_error(self) -> None:
        module = load_module()

        class _FakeOpener:
            def open(self, request):  # noqa: ARG002
                import io as _io
                import urllib.error
                raise urllib.error.HTTPError(
                    url=request.full_url,
                    code=500,
                    msg="Internal Server Error",
                    hdrs=None,
                    fp=_io.BytesIO(b""),
                )

        with self.assertRaises(module.TelemetryFetchError) as ctx:
            module.fetch_telemetry_records(
                "https://example.test", 5, opener=_FakeOpener()
            )
        self.assertIn("HTTP 500", str(ctx.exception))

    def test_main_does_not_call_real_urlopen(self) -> None:
        module = load_module()
        records = make_passing_records(2)
        with mock.patch("urllib.request.urlopen") as fake_urlopen:
            exit_code, _, _ = run_main(module, records)
        self.assertEqual(exit_code, 0)
        fake_urlopen.assert_not_called()


class JsonOutputTests(unittest.TestCase):
    def test_json_output_has_expected_top_level_keys(self) -> None:
        module = load_module()
        records = make_passing_records(3)
        exit_code, stdout, _ = run_main(module, records, ["--json"])
        self.assertEqual(exit_code, 0)
        payload = json.loads(stdout)
        for key in ("pass", "record_count", "failures", "stats"):
            self.assertIn(key, payload)
        self.assertTrue(payload["pass"])
        self.assertEqual(payload["record_count"], 3)
        self.assertIsInstance(payload["failures"], list)
        self.assertIn("bootstrap_elapsed_seconds", payload["stats"])

    def test_json_output_failure_includes_failure_objects(self) -> None:
        module = load_module()
        records = [
            make_record(run_id="ok"),
            make_record(run_id="slow", avg_latency=2_000.0),
        ]
        exit_code, stdout, _ = run_main(module, records, ["--json"])
        self.assertEqual(exit_code, 1)
        payload = json.loads(stdout)
        self.assertFalse(payload["pass"])
        assertions = {f["assertion"] for f in payload["failures"]}
        self.assertIn("chunk_avg_latency", assertions)


class PlaceVersionFilterTests(unittest.TestCase):
    def test_place_version_filter_drops_mismatches(self) -> None:
        module = load_module()
        records = [
            make_record(run_id="v34", place_version=34),
            make_record(run_id="v35-a", place_version=35),
            make_record(run_id="v35-b", place_version=35),
        ]
        exit_code, stdout, _ = run_main(
            module, records, ["--place-version", "35", "--json"]
        )
        self.assertEqual(exit_code, 0)
        payload = json.loads(stdout)
        self.assertEqual(payload["record_count"], 2)
        # Confirm a place_version=34 filter alone yields the single matching
        # record (and that the dropped records were indeed filtered out).
        exit_code2, stdout2, _ = run_main(
            module, records, ["--place-version", "34", "--json"]
        )
        self.assertEqual(exit_code2, 0)
        payload2 = json.loads(stdout2)
        self.assertEqual(payload2["record_count"], 1)


if __name__ == "__main__":
    unittest.main()
