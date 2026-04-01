#!/usr/bin/env python3
from __future__ import annotations

import unittest

from scripts.preview_telemetry_summary import summarize_plugin_state


class PreviewTelemetrySummaryTests(unittest.TestCase):
    def test_summarize_plugin_state_prefers_snapshot_counters_when_present(self) -> None:
        payload = {
            "preview_runtime": {
                "studio_connected": True,
                "plugin_attached": True,
                "project_loaded": True,
                "sync_status": "connected",
                "connection": {"ws_connected": True},
            },
            "preview_project": {
                "preview": {
                    "build_active": False,
                    "state_apply_pending": False,
                    "sync_state": "idle",
                },
                "full_bake": {"active": False, "last_result": None},
            },
            "preview_project_snapshot": {
                "counters": {
                    "build_scheduled": 1,
                    "sync_complete": 1,
                    "sync_cancelled": 0,
                    "state_apply_succeeded": 1,
                    "state_apply_failed": 0,
                },
                "chunkTotals": {"imported": 52, "skipped": 0, "unloaded": 0},
            },
        }

        self.assertEqual(
            summarize_plugin_state(payload),
            "runtime=connected=1 attached=1 project_loaded=1 sync_status=connected ws_connected=1; "
            "project=sync_state=idle build_active=0 state_apply_pending=0 full_bake_active=0 "
            "build=1 sync_complete=1 sync_cancelled=0 state_apply_succeeded=1 state_apply_failed=0 "
            "imported=52 skipped=0 unloaded=0",
        )

    def test_summarize_plugin_state_falls_back_to_compact_project_facts(self) -> None:
        payload = {
            "preview_runtime": {
                "studio_connected": True,
                "plugin_attached": True,
                "project_loaded": False,
                "sync_status": "connecting",
                "connection": {"ws_connected": False},
            },
            "preview_project": {
                "preview": {
                    "build_active": True,
                    "state_apply_pending": True,
                    "sync_state": "syncing",
                },
                "full_bake": {"active": True, "last_result": "pending"},
            },
        }

        self.assertEqual(
            summarize_plugin_state(payload),
            "runtime=connected=1 attached=1 project_loaded=0 sync_status=connecting ws_connected=0; "
            "project=sync_state=syncing build_active=1 state_apply_pending=1 full_bake_active=1 "
            "full_bake_last_result=pending",
        )

    def test_summarize_plugin_state_keeps_default_output_compact_without_requested_families(self) -> None:
        payload = {
            "preview_runtime": {
                "studio_connected": True,
                "plugin_attached": True,
                "project_loaded": True,
                "sync_status": "connected",
                "connection": {"ws_connected": True},
            },
            "preview_project": {
                "preview": {
                    "build_active": False,
                    "state_apply_pending": False,
                    "sync_state": "idle",
                },
                "full_bake": {"active": False, "last_result": None},
            },
            "preview_project_snapshot": {
                "counters": {
                    "build_scheduled": 1,
                    "sync_complete": 1,
                    "sync_cancelled": 0,
                    "state_apply_succeeded": 1,
                    "state_apply_failed": 0,
                },
                "chunkTotals": {"imported": 52, "skipped": 0, "unloaded": 0},
            },
        }

        self.assertEqual(
            summarize_plugin_state(payload),
            "runtime=connected=1 attached=1 project_loaded=1 sync_status=connected ws_connected=1; "
            "project=sync_state=idle build_active=0 state_apply_pending=0 full_bake_active=0 "
            "build=1 sync_complete=1 sync_cancelled=0 state_apply_succeeded=1 state_apply_failed=0 "
            "imported=52 skipped=0 unloaded=0",
        )

    def test_summarize_plugin_state_surfaces_requested_telemetry_families_compactly(self) -> None:
        payload = {
            "preview_runtime": {
                "studio_connected": True,
                "plugin_attached": True,
                "project_loaded": True,
                "sync_status": "connected",
                "connection": {"ws_connected": True},
            },
            "preview_project": {
                "preview": {
                    "build_active": False,
                    "state_apply_pending": False,
                    "sync_state": "idle",
                },
                "full_bake": {"active": False, "last_result": None},
            },
            "preview_project_snapshot": {
                "counters": {
                    "build_scheduled": 1,
                    "sync_complete": 1,
                    "sync_cancelled": 0,
                    "state_apply_succeeded": 1,
                    "state_apply_failed": 0,
                },
                "chunkTotals": {"imported": 80, "skipped": 0, "unloaded": 0},
            },
        }

        self.assertEqual(
            summarize_plugin_state(payload, telemetry_families=" terrain ,roads,roads,water ,,"),
            "runtime=connected=1 attached=1 project_loaded=1 sync_status=connected ws_connected=1; "
            "project=sync_state=idle build_active=0 state_apply_pending=0 full_bake_active=0 "
            "build=1 sync_complete=1 sync_cancelled=0 state_apply_succeeded=1 state_apply_failed=0 "
            "imported=80 skipped=0 unloaded=0 telemetry_families=terrain,roads,water",
        )

    def test_summarize_plugin_state_includes_last_sync_elapsed_and_slow_chunk(self) -> None:
        payload = {
            "preview_runtime": {
                "studio_connected": True,
                "plugin_attached": True,
                "project_loaded": True,
                "sync_status": "connected",
                "connection": {"ws_connected": True},
            },
            "preview_project": {
                "preview": {
                    "build_active": False,
                    "state_apply_pending": False,
                    "sync_state": "idle",
                },
                "full_bake": {"active": False, "last_result": None},
            },
            "preview_project_snapshot": {
                "counters": {
                    "build_scheduled": 1,
                    "sync_complete": 1,
                    "sync_cancelled": 0,
                    "state_apply_succeeded": 1,
                    "state_apply_failed": 0,
                },
                "chunkTotals": {"imported": 80, "skipped": 0, "unloaded": 0},
                "lastSync": {"elapsedMs": 17384},
                "lastSlowChunk": {
                    "chunkId": "7_5",
                    "phase": "preview",
                    "totalMs": 166,
                    "buildingsMs": 121,
                    "terrainMs": 18,
                    "roadsMs": 9,
                    "landuseTerrainFillMs": 6,
                    "artifactCount": 2,
                },
            },
        }

        self.assertEqual(
            summarize_plugin_state(payload),
            "runtime=connected=1 attached=1 project_loaded=1 sync_status=connected ws_connected=1; "
            "project=sync_state=idle build_active=0 state_apply_pending=0 full_bake_active=0 "
            "build=1 sync_complete=1 sync_cancelled=0 state_apply_succeeded=1 state_apply_failed=0 "
            "imported=80 skipped=0 unloaded=0 last_sync_elapsed_ms=17384 slow_chunk=7_5 "
            "slow_chunk_phase=preview slow_chunk_total_ms=166 slow_chunk_buildings_ms=121 "
            "slow_chunk_terrain_ms=18 slow_chunk_roads_ms=9 slow_chunk_landuse_terrain_fill_ms=6 "
            "slow_chunk_artifacts=2",
        )


if __name__ == "__main__":
    unittest.main()
