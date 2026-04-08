#!/usr/bin/env python3
"""Unit tests for scripts/profile_telemetry_cross.py."""
from __future__ import annotations

import io
import json
import os
import sys
import unittest
from typing import Any, Dict, List, Tuple

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
SCRIPTS_DIR = os.path.join(REPO_ROOT, "scripts")
if SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, SCRIPTS_DIR)

import profile_telemetry_cross as ptc  # noqa: E402


# ---------------------------------------------------------------------------
# Fixture helpers
# ---------------------------------------------------------------------------


def make_profile_payload(chunks: List[Dict[str, Any]]) -> str:
    payload = {
        "manifest_path": "/tmp/test/index.json",
        "chunk_count": len(chunks),
        "total_bytes": sum(int(c.get("bytes", 0)) for c in chunks),
        "total_features": 0,
        "total_polygon_vertices": 0,
        "total_terrain_cells": 0,
        "total_terrain_filled_cells": 0,
        "avg_chunk_bytes": 0,
        "p50_chunk_bytes": 0,
        "p95_chunk_bytes": 0,
        "max_chunk_bytes": 0,
        "chunks": chunks,
    }
    return json.dumps(payload)


def default_profile_chunks() -> List[Dict[str, Any]]:
    return [
        {
            "id": "0_0",
            "bytes": 1024,
            "feature_count": 10,
            "polygon_vertices": 50,
            "terrain_filled_cells": 20,
            "estimated_import_cost": 100.0,
        },
        {
            "id": "1_0",
            "bytes": 2048,
            "feature_count": 20,
            "polygon_vertices": 80,
            "terrain_filled_cells": 40,
            "estimated_import_cost": 120.0,
        },
        {
            "id": "2_0",
            "bytes": 60_000,
            "feature_count": 500,
            "polygon_vertices": 5000,
            "terrain_filled_cells": 800,
            "estimated_import_cost": 5000.0,
        },
    ]


def make_telemetry_record(
    chunk_id: str,
    latency_ms: float,
    *,
    run_id: str = "run-xyz",
    ts: float = 10_000.0,
    source_url: str = "https://example.invalid/chunks/0_0.json",
) -> Dict[str, Any]:
    return {
        "runId": run_id,
        "timestamp": ts,
        "chunkFetch": {
            "fetchCount": 4,
            "totalBytes": 123456,
            "avgLatencyMs": latency_ms / 2.0,
            "slowestLatencyMs": latency_ms,
            "slowestChunkId": chunk_id,
            "sourceUrl": source_url,
        },
    }


def install_fake_fetchers(
    test_case: unittest.TestCase,
    *,
    profile_chunks: List[Dict[str, Any]],
    telemetry_records: List[Dict[str, Any]],
    profile_rc: int = 0,
    profile_stderr: str = "",
    telemetry_error: Exception = None,
):
    stdout_payload = make_profile_payload(profile_chunks)

    def fake_runner(cmd: List[str]) -> Tuple[int, str, str]:
        test_case.assertEqual(cmd[1], "profile")
        test_case.assertIn("--split", cmd)
        test_case.assertIn("--json", cmd)
        return (profile_rc, stdout_payload, profile_stderr)

    def profile_runner(arbx_cli: str, index_path: str) -> List[ptc.ChunkProfile]:
        return ptc.run_arbx_profile(arbx_cli, index_path, runner=fake_runner)

    def telemetry_fetcher(worker_url: str, limit: int) -> List[Dict[str, Any]]:
        if telemetry_error is not None:
            raise telemetry_error
        return list(telemetry_records)

    return profile_runner, telemetry_fetcher


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


class RunArbxProfileTests(unittest.TestCase):
    def test_parses_valid_output(self):
        stdout = make_profile_payload(default_profile_chunks())

        def runner(cmd):
            return (0, stdout, "")

        profiles = ptc.run_arbx_profile("/fake/arbx_cli", "/fake/index.json", runner=runner)
        self.assertEqual(len(profiles), 3)
        self.assertEqual(profiles[2].id, "2_0")
        self.assertAlmostEqual(profiles[2].estimated_import_cost, 5000.0)

    def test_missing_binary_raises(self):
        with self.assertRaises(ptc.ProfileSubprocessError) as ctx:
            ptc.run_arbx_profile("/definitely/not/a/real/binary/arbx_cli", "/tmp/idx.json")
        self.assertIn("not found", str(ctx.exception))

    def test_nonzero_exit_raises(self):
        def runner(cmd):
            return (7, "", "boom")

        with self.assertRaises(ptc.ProfileSubprocessError) as ctx:
            ptc.run_arbx_profile("/fake", "/fake", runner=runner)
        self.assertIn("exited 7", str(ctx.exception))
        self.assertIn("boom", str(ctx.exception))

    def test_invalid_json_raises(self):
        def runner(cmd):
            return (0, "not json", "")

        with self.assertRaises(ptc.ProfileSubprocessError):
            ptc.run_arbx_profile("/fake", "/fake", runner=runner)

    def test_missing_chunks_raises(self):
        def runner(cmd):
            return (0, json.dumps({"manifest_path": "x"}), "")

        with self.assertRaises(ptc.ProfileSubprocessError):
            ptc.run_arbx_profile("/fake", "/fake", runner=runner)


class ExtractRuntimeSamplesTests(unittest.TestCase):
    def test_filters_by_since_seconds(self):
        records = [
            make_telemetry_record("0_0", 100.0, ts=9000.0),
            make_telemetry_record("1_0", 200.0, ts=9999.0),
        ]
        samples = ptc.extract_runtime_samples(records, since_seconds=600, now=10_000.0)
        self.assertEqual([s.chunk_id for s in samples], ["1_0"])

    def test_skips_missing_chunk_id(self):
        records = [
            {"runId": "r", "timestamp": 10_000.0, "chunkFetch": {"slowestLatencyMs": 1.0}}
        ]
        samples = ptc.extract_runtime_samples(records, since_seconds=0)
        self.assertEqual(samples, [])


class AnalyzeHappyPathTests(unittest.TestCase):
    def test_three_chunks_one_hot(self):
        profiles = [ptc.ChunkProfile.from_dict(c) for c in default_profile_chunks()]
        samples = [
            ptc.RuntimeSample("0_0", 100.0, "r1", 10_000.0, None),
            ptc.RuntimeSample("1_0", 120.0, "r2", 10_000.0, None),
            ptc.RuntimeSample("2_0", 4000.0, "r3", 10_000.0, None),
        ]
        cfg = ptc.CrossConfig(stddev_threshold=1.0)
        report = ptc.analyze(profiles, samples, cfg)
        self.assertEqual(report.overlap_count, 3)
        self.assertEqual(len(report.hot_chunks), 1)
        self.assertEqual(report.hot_chunks[0].chunk_id, "2_0")
        # Hottest chunk should also be highest cross-score.
        self.assertEqual(report.all_crossed[0].chunk_id, "2_0")
        self.assertTrue(report.passed)
        # Flag reasons should cite both cost_z and latency_z.
        reasons = report.hot_chunks[0].flag_reasons
        self.assertTrue(any("cost_z" in r for r in reasons))
        self.assertTrue(any("latency_z" in r for r in reasons))


class AnalyzeAssertionTests(unittest.TestCase):
    def test_assert_hot_count_zero_fails_when_hot(self):
        profiles = [ptc.ChunkProfile.from_dict(c) for c in default_profile_chunks()]
        samples = [
            ptc.RuntimeSample("0_0", 100.0, "r1", 10_000.0, None),
            ptc.RuntimeSample("1_0", 120.0, "r2", 10_000.0, None),
            ptc.RuntimeSample("2_0", 4000.0, "r3", 10_000.0, None),
        ]
        cfg = ptc.CrossConfig(stddev_threshold=1.0, assert_hot_count=0)
        report = ptc.analyze(profiles, samples, cfg)
        self.assertFalse(report.passed)
        self.assertIsNotNone(report.assertion_failure)
        self.assertIn("hot_chunks=1", report.assertion_failure)

    def test_no_hot_chunks_pass(self):
        # Uniform profile + uniform latency = no outliers.
        profiles = [
            ptc.ChunkProfile("0_0", 1024, 10, 50, 20, 100.0),
            ptc.ChunkProfile("1_0", 1024, 10, 50, 20, 100.0),
            ptc.ChunkProfile("2_0", 1024, 10, 50, 20, 100.0),
        ]
        samples = [
            ptc.RuntimeSample("0_0", 100.0, "r1", 10_000.0, None),
            ptc.RuntimeSample("1_0", 100.0, "r2", 10_000.0, None),
            ptc.RuntimeSample("2_0", 100.0, "r3", 10_000.0, None),
        ]
        cfg = ptc.CrossConfig(assert_hot_count=0)
        report = ptc.analyze(profiles, samples, cfg)
        self.assertEqual(len(report.hot_chunks), 0)
        self.assertTrue(report.passed)


class MainEndToEndTests(unittest.TestCase):
    def _run(self, argv, *, telemetry_records, profile_chunks=None,
             telemetry_error=None, profile_rc=0) -> Tuple[int, str, str]:
        profile_chunks = profile_chunks if profile_chunks is not None else default_profile_chunks()
        profile_runner, telemetry_fetcher = install_fake_fetchers(
            self,
            profile_chunks=profile_chunks,
            telemetry_records=telemetry_records,
            profile_rc=profile_rc,
            telemetry_error=telemetry_error,
        )
        out = io.StringIO()
        err = io.StringIO()
        rc = ptc.main(
            argv,
            profile_runner=profile_runner,
            telemetry_fetcher=telemetry_fetcher,
            now=10_000.0,
            stdout=out,
            stderr=err,
        )
        return rc, out.getvalue(), err.getvalue()

    def test_happy_path_pass(self):
        records = [
            make_telemetry_record("2_0", 4000.0, run_id="r3", ts=9_900.0),
            make_telemetry_record("0_0", 100.0, run_id="r1", ts=9_900.0),
        ]
        rc, out, err = self._run(
            ["--stddev-threshold", "1.0", "--json"],
            telemetry_records=records,
        )
        self.assertEqual(rc, ptc.EXIT_OK, err)
        payload = json.loads(out)
        self.assertTrue(payload["pass"])
        self.assertEqual(payload["stats"]["overlap_count"], 2)
        hot_ids = [h["chunk_id"] for h in payload["hot_chunks"]]
        self.assertIn("2_0", hot_ids)

    def test_assertion_failure_when_hot_exceeds(self):
        records = [
            make_telemetry_record("2_0", 4000.0, run_id="r3", ts=9_900.0),
            make_telemetry_record("0_0", 100.0, run_id="r1", ts=9_900.0),
            make_telemetry_record("1_0", 120.0, run_id="r2", ts=9_900.0),
        ]
        rc, out, err = self._run(
            ["--stddev-threshold", "1.0", "--assert-hot-count", "0"],
            telemetry_records=records,
        )
        self.assertEqual(rc, ptc.EXIT_ASSERTION_FAILED)
        self.assertIn("FAIL", out)

    def test_http_error_surfaces(self):
        rc, out, err = self._run(
            [],
            telemetry_records=[],
            telemetry_error=ptc.TelemetryFetchError("HTTP 503 fetching: Service Unavailable"),
        )
        self.assertEqual(rc, ptc.EXIT_SUBPROCESS_ERROR)
        self.assertIn("telemetry fetch failed", err)

    def test_empty_profile_is_no_data(self):
        rc, out, err = self._run(
            [],
            telemetry_records=[make_telemetry_record("0_0", 100.0, ts=9_900.0)],
            profile_chunks=[],
        )
        self.assertEqual(rc, ptc.EXIT_NO_DATA)

    def test_missing_arbx_cli_binary_error(self):
        out = io.StringIO()
        err = io.StringIO()
        rc = ptc.main(
            ["--arbx-cli", "/definitely/not/a/real/binary/arbx_cli"],
            now=10_000.0,
            stdout=out,
            stderr=err,
            telemetry_fetcher=lambda u, n: [],
        )
        self.assertEqual(rc, ptc.EXIT_SUBPROCESS_ERROR)
        self.assertIn("profile failed", err.getvalue())

    def test_no_overlap_returns_no_data(self):
        records = [make_telemetry_record("99_99", 100.0, ts=9_900.0)]
        rc, out, err = self._run([], telemetry_records=records)
        self.assertEqual(rc, ptc.EXIT_NO_DATA)


if __name__ == "__main__":
    unittest.main()
