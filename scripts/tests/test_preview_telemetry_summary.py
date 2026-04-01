#!/usr/bin/env python3
from __future__ import annotations

import unittest

import scripts.preview_telemetry_summary as preview_telemetry_summary


class PreviewTelemetrySummaryTests(unittest.TestCase):
    def test_build_plugin_state_summary_returns_compact_structured_blocks(self) -> None:
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
                "full_bake": {"active": False, "last_result": "success"},
            },
            "preview_project_snapshot": {
                "counters": {
                    "build_scheduled": 1,
                    "sync_complete": 2,
                    "sync_cancelled": 0,
                    "state_apply_succeeded": 1,
                    "state_apply_failed": 0,
                },
                "chunkTotals": {"imported": 52, "skipped": 1, "unloaded": 0},
                "lastSync": {"elapsedMs": 17384},
                "lastSlowChunk": {
                    "chunkId": "7_5",
                    "phase": "preview",
                    "totalMs": 166,
                    "buildingsMs": 121,
                    "buildingMeshCreateMs": 97,
                    "buildingShellDetailMs": 20,
                    "buildingInteriorMs": 4,
                    "buildingRoofBuildMs": 8,
                    "buildingFacadeDetailMs": 6,
                    "buildingPerimeterDetailMs": 3,
                    "buildingTerrainFillMs": 2,
                    "buildingRooftopDetailMs": 1,
                    "buildingNameLabelMs": 0,
                    "buildingMeshPartCount": 14,
                    "buildingRoofMeshPartCount": 6,
                    "buildingMeshTriangleCount": 4096,
                    "terrainMs": 18,
                    "terrainMaterialKindCount": 1,
                    "terrainDominantMaterial": "Grass",
                    "terrainDominantMaterialCellCount": 64,
                    "terrainNonGrassCellCount": 0,
                    "terrainCellCount": 64,
                    "terrainSubsampleCount": 16,
                    "roadsMs": 9,
                    "landuseTerrainFillMs": 6,
                    "buildingFeatureCount": 14,
                    "artifactCount": 2,
                },
            },
        }

        self.assertEqual(
            preview_telemetry_summary.build_plugin_state_summary(
                payload, telemetry_families=" terrain ,roads,roads,water ,,"
            ),
            {
                "runtime": {
                    "connected": True,
                    "attached": True,
                    "projectLoaded": True,
                    "syncStatus": "connected",
                    "wsConnected": True,
                },
                "project": {
                    "syncState": "idle",
                    "buildActive": False,
                    "stateApplyPending": False,
                    "fullBakeActive": False,
                    "fullBakeLastResult": "success",
                    "counters": {
                        "build_scheduled": 1,
                        "sync_complete": 2,
                        "sync_cancelled": 0,
                        "state_apply_succeeded": 1,
                        "state_apply_failed": 0,
                    },
                    "chunkTotals": {"imported": 52, "skipped": 1, "unloaded": 0},
                },
                "hotspot": {
                    "status": "present",
                    "lastSyncElapsedMs": 17384,
                    "slowChunk": {
                        "chunkId": "7_5",
                        "phase": "preview",
                        "totalMs": 166,
                        "buildingsMs": 121,
                        "buildingMeshCreateMs": 97,
                        "buildingShellDetailMs": 20,
                        "buildingInteriorMs": 4,
                        "buildingRoofBuildMs": 8,
                        "buildingFacadeDetailMs": 6,
                        "buildingPerimeterDetailMs": 3,
                        "buildingTerrainFillMs": 2,
                        "buildingRooftopDetailMs": 1,
                        "buildingNameLabelMs": 0,
                        "buildingMeshPartCount": 14,
                        "buildingRoofMeshPartCount": 6,
                        "buildingMeshTriangleCount": 4096,
                        "terrainMs": 18,
                        "terrainMaterialKindCount": 1,
                        "terrainDominantMaterial": "Grass",
                        "terrainDominantMaterialCellCount": 64,
                        "terrainNonGrassCellCount": 0,
                        "terrainCellCount": 64,
                        "terrainSubsampleCount": 16,
                        "roadsMs": 9,
                        "landuseTerrainFillMs": 6,
                        "buildingFeatureCount": 14,
                        "buildingResidualMs": 24,
                        "buildingMeshCreateRatio": 0.8017,
                        "buildingResidualRatio": 0.1983,
                        "buildingMeshPartsPerFeature": 1.0,
                        "buildingMeshTrianglesPerFeature": 292.57,
                        "buildingShellDominantDetailPhase": "roof_build",
                        "buildingShellDominantDetailMs": 8,
                        "dominantCostCenter": "buildings",
                        "dominantCostMs": 121,
                        "dominantCostRatio": 0.7289,
                        "terrainSignalStatus": "present",
                        "artifactCount": 2,
                    },
                },
                "telemetryFamilies": ["terrain", "roads", "water"],
            },
        )

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
            preview_telemetry_summary.summarize_plugin_state(payload),
            "runtime=connected=1 attached=1 project_loaded=1 sync_status=connected ws_connected=1; "
            "project=sync_state=idle build_active=0 state_apply_pending=0 full_bake_active=0 "
            "build=1 sync_complete=1 sync_cancelled=0 state_apply_succeeded=1 state_apply_failed=0 "
            "imported=52 skipped=0 unloaded=0 hotspot_status=absent",
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
            preview_telemetry_summary.summarize_plugin_state(payload),
            "runtime=connected=1 attached=1 project_loaded=0 sync_status=connecting ws_connected=0; "
            "project=sync_state=syncing build_active=1 state_apply_pending=1 full_bake_active=1 "
            "hotspot_status=missing_snapshot full_bake_last_result=pending",
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
            preview_telemetry_summary.summarize_plugin_state(payload),
            "runtime=connected=1 attached=1 project_loaded=1 sync_status=connected ws_connected=1; "
            "project=sync_state=idle build_active=0 state_apply_pending=0 full_bake_active=0 "
            "build=1 sync_complete=1 sync_cancelled=0 state_apply_succeeded=1 state_apply_failed=0 "
            "imported=52 skipped=0 unloaded=0 hotspot_status=absent",
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
            preview_telemetry_summary.summarize_plugin_state(payload, telemetry_families=" terrain ,roads,roads,water ,,"),
            "runtime=connected=1 attached=1 project_loaded=1 sync_status=connected ws_connected=1; "
            "project=sync_state=idle build_active=0 state_apply_pending=0 full_bake_active=0 "
            "build=1 sync_complete=1 sync_cancelled=0 state_apply_succeeded=1 state_apply_failed=0 "
            "imported=80 skipped=0 unloaded=0 hotspot_status=absent telemetry_families=terrain,roads,water",
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
                    "buildingMeshCreateMs": 97,
                    "buildingShellDetailMs": 20,
                    "buildingInteriorMs": 4,
                    "buildingRoofBuildMs": 8,
                    "buildingFacadeDetailMs": 6,
                    "buildingPerimeterDetailMs": 3,
                    "buildingTerrainFillMs": 2,
                    "buildingRooftopDetailMs": 1,
                    "buildingNameLabelMs": 0,
                    "buildingMeshPartCount": 14,
                    "buildingRoofMeshPartCount": 6,
                    "buildingMeshTriangleCount": 4096,
                    "terrainMs": 18,
                    "terrainMaterialKindCount": 1,
                    "terrainDominantMaterial": "Grass",
                    "terrainDominantMaterialCellCount": 64,
                    "terrainNonGrassCellCount": 0,
                    "terrainCellCount": 64,
                    "terrainSubsampleCount": 16,
                    "roadsMs": 9,
                    "landuseTerrainFillMs": 6,
                    "buildingFeatureCount": 14,
                    "artifactCount": 2,
                },
            },
        }

        self.assertEqual(
            preview_telemetry_summary.summarize_plugin_state(payload),
            "runtime=connected=1 attached=1 project_loaded=1 sync_status=connected ws_connected=1; "
            "project=sync_state=idle build_active=0 state_apply_pending=0 full_bake_active=0 "
            "build=1 sync_complete=1 sync_cancelled=0 state_apply_succeeded=1 state_apply_failed=0 "
            "imported=80 skipped=0 unloaded=0 hotspot_status=present last_sync_elapsed_ms=17384 slow_chunk=7_5 "
            "slow_chunk_phase=preview slow_chunk_total_ms=166 slow_chunk_buildings_ms=121 "
            "slow_chunk_building_mesh_create_ms=97 slow_chunk_building_mesh_parts=14 "
            "slow_chunk_building_roof_mesh_parts=6 slow_chunk_building_mesh_triangles=4096 "
            "slow_chunk_building_shell_detail_ms=20 slow_chunk_building_interior_ms=4 "
            "slow_chunk_building_roof_build_ms=8 slow_chunk_building_facade_detail_ms=6 "
            "slow_chunk_building_perimeter_detail_ms=3 slow_chunk_building_terrain_fill_ms=2 "
            "slow_chunk_building_rooftop_detail_ms=1 slow_chunk_building_name_label_ms=0 "
            "slow_chunk_building_residual_ms=24 slow_chunk_building_mesh_create_ratio=0.8017 "
            "slow_chunk_building_residual_ratio=0.1983 slow_chunk_building_mesh_parts_per_feature=1.0 "
            "slow_chunk_building_mesh_triangles_per_feature=292.57 "
            "slow_chunk_building_shell_dominant_detail_phase=roof_build "
            "slow_chunk_building_shell_dominant_detail_ms=8 "
            "slow_chunk_terrain_ms=18 slow_chunk_terrain_material_kind_count=1 "
            "slow_chunk_terrain_dominant_material=Grass "
            "slow_chunk_terrain_dominant_material_cells=64 slow_chunk_terrain_non_grass_cells=0 "
            "slow_chunk_terrain_cells=64 slow_chunk_terrain_subsamples=16 slow_chunk_roads_ms=9 "
            "slow_chunk_landuse_terrain_fill_ms=6 slow_chunk_building_features=14 "
            "slow_chunk_dominant_cost_center=buildings slow_chunk_dominant_cost_ms=121 "
            "slow_chunk_dominant_cost_ratio=0.7289 slow_chunk_terrain_signal_status=present "
            "slow_chunk_artifacts=2",
        )

    def test_summarize_plugin_state_marks_sync_error_hotspot_unavailable(self) -> None:
        payload = {
            "preview_runtime": {
                "studio_connected": True,
                "plugin_attached": True,
                "project_loaded": False,
                "sync_status": "error",
                "connection": {"ws_connected": False},
            },
            "preview_project": {
                "preview": {
                    "build_active": False,
                    "state_apply_pending": False,
                    "sync_state": "idle",
                },
                "full_bake": {"active": False, "last_result": None},
            },
        }

        self.assertEqual(
            preview_telemetry_summary.summarize_plugin_state(payload),
            "runtime=connected=1 attached=1 project_loaded=0 sync_status=error ws_connected=0; "
            "project=sync_state=idle build_active=0 state_apply_pending=0 full_bake_active=0 "
            "hotspot_status=sync_error",
        )

    def test_build_plugin_state_summary_distinguishes_absent_vs_missing_terrain_signal(self) -> None:
        absent_payload = {
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
                "lastSlowChunk": {
                    "chunkId": "-1_0",
                    "phase": "foreground",
                    "totalMs": 155,
                    "buildingsMs": 153,
                    "terrainMs": 0,
                    "terrainMaterialKindCount": 0,
                    "terrainCellCount": 0,
                    "terrainSubsampleCount": 16,
                    "roadsMs": 0,
                    "landuseTerrainFillMs": 0,
                    "buildingFeatureCount": 28,
                    "artifactCount": 28,
                },
            },
        }
        missing_payload = {
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
                "lastSlowChunk": {
                    "chunkId": "3_2",
                    "phase": "foreground",
                    "totalMs": 90,
                    "buildingsMs": 10,
                    "terrainMs": 0,
                    "terrainMaterialKindCount": 0,
                    "terrainCellCount": 64,
                    "terrainSubsampleCount": 16,
                    "roadsMs": 4,
                    "landuseTerrainFillMs": 0,
                    "buildingFeatureCount": 2,
                    "artifactCount": 8,
                },
            },
        }

        absent_summary = preview_telemetry_summary.build_plugin_state_summary(absent_payload)
        missing_summary = preview_telemetry_summary.build_plugin_state_summary(missing_payload)

        self.assertEqual(absent_summary["hotspot"]["slowChunk"]["terrainSignalStatus"], "not_authored")
        self.assertEqual(absent_summary["hotspot"]["slowChunk"]["dominantCostCenter"], "buildings")
        self.assertAlmostEqual(absent_summary["hotspot"]["slowChunk"]["dominantCostRatio"], 0.9871, places=4)

        self.assertEqual(missing_summary["hotspot"]["slowChunk"]["terrainSignalStatus"], "missing")
        self.assertEqual(missing_summary["hotspot"]["slowChunk"]["dominantCostCenter"], "buildings")


if __name__ == "__main__":
    unittest.main()
