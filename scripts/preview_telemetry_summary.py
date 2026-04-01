#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
from typing import Any


def _as_bool_flag(value: Any) -> int:
    return 1 if bool(value) else 0


def _normalize_telemetry_families(value: Any) -> list[str]:
    if not value:
        return []

    if isinstance(value, str):
        raw_families = value.split(",")
    elif isinstance(value, (list, tuple)):
        raw_families = value
    else:
        raw_families = [value]

    normalized: list[str] = []
    seen: set[str] = set()
    for family in raw_families:
        family_name = str(family).strip()
        if not family_name or family_name in seen:
            continue
        seen.add(family_name)
        normalized.append(family_name)
    return normalized


def _slow_chunk_dominant_cost_center(slow_chunk: dict[str, Any]) -> tuple[str, int, float]:
    cost_candidates = {
        "buildings": int(slow_chunk.get("buildingsMs", 0) or 0),
        "terrain": int(slow_chunk.get("terrainMs", 0) or 0),
        "roads": int(slow_chunk.get("roadsMs", 0) or 0),
        "landuse": int(slow_chunk.get("landuseTerrainFillMs", 0) or 0),
    }
    dominant_center = "unknown"
    dominant_ms = 0
    for center, cost_ms in cost_candidates.items():
        if cost_ms > dominant_ms or (cost_ms == dominant_ms and dominant_center == "unknown"):
            dominant_center = center
            dominant_ms = cost_ms
    total_ms = int(slow_chunk.get("totalMs", 0) or 0)
    dominant_ratio = round(dominant_ms / total_ms, 4) if total_ms > 0 else 0.0
    return dominant_center, dominant_ms, dominant_ratio


def _terrain_signal_status(slow_chunk: dict[str, Any]) -> str:
    terrain_cell_count = int(slow_chunk.get("terrainCellCount", 0) or 0)
    terrain_material_kind_count = int(slow_chunk.get("terrainMaterialKindCount", 0) or 0)
    terrain_ms = int(slow_chunk.get("terrainMs", 0) or 0)
    if terrain_cell_count <= 0 and terrain_material_kind_count <= 0 and terrain_ms <= 0:
        return "not_authored"
    if terrain_material_kind_count <= 0:
        return "missing"
    return "present"


def _round_ratio(numerator: int, denominator: int, *, digits: int = 4) -> float:
    if denominator <= 0:
        return 0.0
    return round(numerator / denominator, digits)


def _building_hotspot_metrics(slow_chunk: dict[str, Any]) -> dict[str, int | float]:
    buildings_ms = int(slow_chunk.get("buildingsMs", 0) or 0)
    mesh_create_ms = int(slow_chunk.get("buildingMeshCreateMs", 0) or 0)
    shell_detail_ms = int(slow_chunk.get("buildingShellDetailMs", 0) or 0)
    interior_ms = int(slow_chunk.get("buildingInteriorMs", 0) or 0)
    feature_count = int(slow_chunk.get("buildingFeatureCount", 0) or 0)
    mesh_part_count = int(slow_chunk.get("buildingMeshPartCount", 0) or 0)
    triangle_count = int(slow_chunk.get("buildingMeshTriangleCount", 0) or 0)
    residual_ms = max(
        shell_detail_ms + interior_ms if shell_detail_ms > 0 or interior_ms > 0 else buildings_ms - mesh_create_ms,
        0,
    )

    metrics: dict[str, int | float] = {
        "buildingResidualMs": residual_ms,
        "buildingMeshCreateRatio": _round_ratio(mesh_create_ms, buildings_ms),
        "buildingResidualRatio": _round_ratio(residual_ms, buildings_ms),
    }
    if feature_count > 0:
        metrics["buildingMeshPartsPerFeature"] = round(mesh_part_count / feature_count, 2)
        metrics["buildingMeshTrianglesPerFeature"] = round(triangle_count / feature_count, 2)
    return metrics


def _building_detail_phase_metrics(slow_chunk: dict[str, Any]) -> dict[str, int | str]:
    detail_phase_values = {
        "roof_build": int(slow_chunk.get("buildingRoofBuildMs", 0) or 0),
        "facade_detail": int(slow_chunk.get("buildingFacadeDetailMs", 0) or 0),
        "perimeter_detail": int(slow_chunk.get("buildingPerimeterDetailMs", 0) or 0),
        "terrain_fill": int(slow_chunk.get("buildingTerrainFillMs", 0) or 0),
        "rooftop_detail": int(slow_chunk.get("buildingRooftopDetailMs", 0) or 0),
        "name_label": int(slow_chunk.get("buildingNameLabelMs", 0) or 0),
    }
    dominant_phase = "unknown"
    dominant_ms = 0
    for phase_name, elapsed_ms in detail_phase_values.items():
        if elapsed_ms > dominant_ms or (elapsed_ms == dominant_ms and dominant_phase == "unknown" and elapsed_ms > 0):
            dominant_phase = phase_name
            dominant_ms = elapsed_ms

    metrics: dict[str, int | str] = {
        "buildingShellDominantDetailPhase": dominant_phase,
        "buildingShellDominantDetailMs": dominant_ms,
    }
    for phase_name, elapsed_ms in detail_phase_values.items():
        token_name = "".join(part.capitalize() for part in phase_name.split("_"))
        metrics[f"building{token_name}Ms"] = elapsed_ms
    return metrics


def build_plugin_state_summary(data: dict[str, Any], telemetry_families: Any | None = None) -> dict[str, Any]:
    runtime = data.get("preview_runtime") or {}
    runtime_connection = runtime.get("connection") or {}
    project = data.get("preview_project") or {}
    project_preview = project.get("preview") or {}
    project_full_bake = project.get("full_bake") or {}
    project_snapshot = data.get("preview_project_snapshot") or {}
    if not isinstance(project_snapshot, dict):
        project_snapshot = {}
    project_counters = project_snapshot.get("counters") or {}
    project_chunks = project_snapshot.get("chunkTotals") or {}
    last_sync = project_snapshot.get("lastSync") or {}
    last_slow_chunk = project_snapshot.get("lastSlowChunk") or {}
    if not isinstance(project_counters, dict):
        project_counters = {}
    if not isinstance(project_chunks, dict):
        project_chunks = {}
    if not isinstance(last_sync, dict):
        last_sync = {}
    if not isinstance(last_slow_chunk, dict):
        last_slow_chunk = {}
    hotspot_status = "sync_error" if runtime.get("sync_status") == "error" else None
    if hotspot_status is None:
        if not isinstance(data.get("preview_project_snapshot"), dict):
            hotspot_status = "missing_snapshot"
        elif isinstance(last_slow_chunk, dict) and last_slow_chunk.get("chunkId") is not None:
            hotspot_status = "present"
        else:
            hotspot_status = "absent"

    requested_families = _normalize_telemetry_families(telemetry_families)
    summary: dict[str, Any] = {
        "runtime": {
            "connected": bool(runtime.get("studio_connected")),
            "attached": bool(runtime.get("plugin_attached")),
            "projectLoaded": bool(runtime.get("project_loaded")),
            "syncStatus": str(runtime.get("sync_status", "unknown")),
            "wsConnected": bool(runtime_connection.get("ws_connected")),
        },
        "project": {
            "syncState": str(project_preview.get("sync_state", "unknown")),
            "buildActive": bool(project_preview.get("build_active")),
            "stateApplyPending": bool(project_preview.get("state_apply_pending")),
            "fullBakeActive": bool(project_full_bake.get("active")),
            "fullBakeLastResult": project_full_bake.get("last_result"),
        },
        "hotspot": {
            "status": hotspot_status,
        },
    }
    if project_counters or project_chunks:
        summary["project"]["counters"] = {
            "build_scheduled": int(project_counters.get("build_scheduled", 0)),
            "sync_complete": int(project_counters.get("sync_complete", 0)),
            "sync_cancelled": int(project_counters.get("sync_cancelled", 0)),
            "state_apply_succeeded": int(project_counters.get("state_apply_succeeded", 0)),
            "state_apply_failed": int(project_counters.get("state_apply_failed", 0)),
        }
        summary["project"]["chunkTotals"] = {
            "imported": int(project_chunks.get("imported", 0)),
            "skipped": int(project_chunks.get("skipped", 0)),
            "unloaded": int(project_chunks.get("unloaded", 0)),
        }
    if last_sync.get("elapsedMs") is not None:
        summary["hotspot"]["lastSyncElapsedMs"] = int(last_sync.get("elapsedMs", 0))
    slow_chunk_id = last_slow_chunk.get("chunkId")
    if slow_chunk_id is not None:
        dominant_cost_center, dominant_cost_ms, dominant_cost_ratio = _slow_chunk_dominant_cost_center(last_slow_chunk)
        summary["hotspot"]["slowChunk"] = {
            "chunkId": str(slow_chunk_id),
            "phase": str(last_slow_chunk.get("phase", "unknown")),
            "totalMs": int(last_slow_chunk.get("totalMs", 0)),
            "buildingsMs": int(last_slow_chunk.get("buildingsMs", 0)),
            "terrainMs": int(last_slow_chunk.get("terrainMs", 0)),
            "terrainMaterialKindCount": int(last_slow_chunk.get("terrainMaterialKindCount", 0)),
            "terrainDominantMaterial": str(last_slow_chunk.get("terrainDominantMaterial", "unknown")),
            "terrainDominantMaterialCellCount": int(last_slow_chunk.get("terrainDominantMaterialCellCount", 0)),
            "terrainNonGrassCellCount": int(last_slow_chunk.get("terrainNonGrassCellCount", 0)),
            "terrainCellCount": int(last_slow_chunk.get("terrainCellCount", 0)),
            "terrainSubsampleCount": int(last_slow_chunk.get("terrainSubsampleCount", 0)),
            "roadsMs": int(last_slow_chunk.get("roadsMs", 0)),
            "landuseTerrainFillMs": int(last_slow_chunk.get("landuseTerrainFillMs", 0)),
            "buildingFeatureCount": int(last_slow_chunk.get("buildingFeatureCount", 0)),
            "buildingShellDetailMs": int(last_slow_chunk.get("buildingShellDetailMs", 0)),
            "buildingInteriorMs": int(last_slow_chunk.get("buildingInteriorMs", 0)),
            "dominantCostCenter": dominant_cost_center,
            "dominantCostMs": dominant_cost_ms,
            "dominantCostRatio": dominant_cost_ratio,
            "terrainSignalStatus": _terrain_signal_status(last_slow_chunk),
            "artifactCount": int(last_slow_chunk.get("artifactCount", 0)),
        }
        summary["hotspot"]["slowChunk"].update(_building_hotspot_metrics(last_slow_chunk))
        summary["hotspot"]["slowChunk"].update(_building_detail_phase_metrics(last_slow_chunk))
        for source_key in (
            "buildingMeshCreateMs",
            "buildingMeshPartCount",
            "buildingRoofMeshPartCount",
            "buildingMeshVertexCount",
            "buildingMeshTriangleCount",
        ):
            if last_slow_chunk.get(source_key) is not None:
                summary["hotspot"]["slowChunk"][source_key] = int(last_slow_chunk.get(source_key, 0))
    if requested_families:
        summary["telemetryFamilies"] = requested_families
    return summary


def summarize_plugin_state(data: dict[str, Any], telemetry_families: Any | None = None) -> str:
    summary = build_plugin_state_summary(data, telemetry_families)
    runtime = summary.get("runtime", {})
    project = summary.get("project", {})
    hotspot = summary.get("hotspot", {})
    counters = project.get("counters") if isinstance(project.get("counters"), dict) else {}
    chunk_totals = project.get("chunkTotals") if isinstance(project.get("chunkTotals"), dict) else {}

    runtime_parts = [
        f"connected={_as_bool_flag(runtime.get('connected'))}",
        f"attached={_as_bool_flag(runtime.get('attached'))}",
        f"project_loaded={_as_bool_flag(runtime.get('projectLoaded'))}",
        f"sync_status={runtime.get('syncStatus', 'unknown')}",
        f"ws_connected={_as_bool_flag(runtime.get('wsConnected'))}",
    ]

    project_parts = [
        f"sync_state={project.get('syncState', 'unknown')}",
        f"build_active={_as_bool_flag(project.get('buildActive'))}",
        f"state_apply_pending={_as_bool_flag(project.get('stateApplyPending'))}",
        f"full_bake_active={_as_bool_flag(project.get('fullBakeActive'))}",
    ]

    has_snapshot_counters = bool(counters or chunk_totals)
    if has_snapshot_counters:
        project_parts.extend(
            [
                f"build={counters.get('build_scheduled', 0)}",
                f"sync_complete={counters.get('sync_complete', 0)}",
                f"sync_cancelled={counters.get('sync_cancelled', 0)}",
                f"state_apply_succeeded={counters.get('state_apply_succeeded', 0)}",
                f"state_apply_failed={counters.get('state_apply_failed', 0)}",
                f"imported={chunk_totals.get('imported', 0)}",
                f"skipped={chunk_totals.get('skipped', 0)}",
                f"unloaded={chunk_totals.get('unloaded', 0)}",
            ]
        )
    project_parts.append(f"hotspot_status={hotspot.get('status', 'unknown')}")
    if hotspot.get("lastSyncElapsedMs") is not None:
        project_parts.append(f"last_sync_elapsed_ms={hotspot.get('lastSyncElapsedMs')}")
    slow_chunk = hotspot.get("slowChunk") if isinstance(hotspot.get("slowChunk"), dict) else {}
    if slow_chunk:
        slow_chunk_parts = [
            f"slow_chunk={slow_chunk.get('chunkId')}",
            f"slow_chunk_phase={slow_chunk.get('phase', 'unknown')}",
            f"slow_chunk_total_ms={slow_chunk.get('totalMs', 0)}",
            f"slow_chunk_buildings_ms={slow_chunk.get('buildingsMs', 0)}",
        ]
        optional_hotspot_tokens = (
            ("buildingMeshCreateMs", "slow_chunk_building_mesh_create_ms"),
            ("buildingMeshPartCount", "slow_chunk_building_mesh_parts"),
            ("buildingRoofMeshPartCount", "slow_chunk_building_roof_mesh_parts"),
            ("buildingMeshVertexCount", "slow_chunk_building_mesh_vertices"),
            ("buildingMeshTriangleCount", "slow_chunk_building_mesh_triangles"),
        )
        for source_key, token_key in optional_hotspot_tokens:
            if source_key in slow_chunk:
                slow_chunk_parts.append(f"{token_key}={slow_chunk.get(source_key, 0)}")
        derived_hotspot_tokens = (
            ("buildingShellDetailMs", "slow_chunk_building_shell_detail_ms"),
            ("buildingInteriorMs", "slow_chunk_building_interior_ms"),
            ("buildingRoofBuildMs", "slow_chunk_building_roof_build_ms"),
            ("buildingFacadeDetailMs", "slow_chunk_building_facade_detail_ms"),
            ("buildingPerimeterDetailMs", "slow_chunk_building_perimeter_detail_ms"),
            ("buildingTerrainFillMs", "slow_chunk_building_terrain_fill_ms"),
            ("buildingRooftopDetailMs", "slow_chunk_building_rooftop_detail_ms"),
            ("buildingNameLabelMs", "slow_chunk_building_name_label_ms"),
            ("buildingResidualMs", "slow_chunk_building_residual_ms"),
            ("buildingMeshCreateRatio", "slow_chunk_building_mesh_create_ratio"),
            ("buildingResidualRatio", "slow_chunk_building_residual_ratio"),
            ("buildingMeshPartsPerFeature", "slow_chunk_building_mesh_parts_per_feature"),
            ("buildingMeshTrianglesPerFeature", "slow_chunk_building_mesh_triangles_per_feature"),
            ("buildingShellDominantDetailPhase", "slow_chunk_building_shell_dominant_detail_phase"),
            ("buildingShellDominantDetailMs", "slow_chunk_building_shell_dominant_detail_ms"),
        )
        for source_key, token_key in derived_hotspot_tokens:
            if source_key in slow_chunk:
                slow_chunk_parts.append(f"{token_key}={slow_chunk.get(source_key, 0)}")
        slow_chunk_parts.extend(
            [
                f"slow_chunk_terrain_ms={slow_chunk.get('terrainMs', 0)}",
                f"slow_chunk_terrain_material_kind_count={slow_chunk.get('terrainMaterialKindCount', 0)}",
                f"slow_chunk_terrain_dominant_material={slow_chunk.get('terrainDominantMaterial', 'unknown')}",
                f"slow_chunk_terrain_dominant_material_cells={slow_chunk.get('terrainDominantMaterialCellCount', 0)}",
                f"slow_chunk_terrain_non_grass_cells={slow_chunk.get('terrainNonGrassCellCount', 0)}",
                f"slow_chunk_terrain_cells={slow_chunk.get('terrainCellCount', 0)}",
                f"slow_chunk_terrain_subsamples={slow_chunk.get('terrainSubsampleCount', 0)}",
                f"slow_chunk_roads_ms={slow_chunk.get('roadsMs', 0)}",
                f"slow_chunk_landuse_terrain_fill_ms={slow_chunk.get('landuseTerrainFillMs', 0)}",
                f"slow_chunk_building_features={slow_chunk.get('buildingFeatureCount', 0)}",
                f"slow_chunk_dominant_cost_center={slow_chunk.get('dominantCostCenter', 'unknown')}",
                f"slow_chunk_dominant_cost_ms={slow_chunk.get('dominantCostMs', 0)}",
                f"slow_chunk_dominant_cost_ratio={slow_chunk.get('dominantCostRatio', 0)}",
                f"slow_chunk_terrain_signal_status={slow_chunk.get('terrainSignalStatus', 'unknown')}",
                f"slow_chunk_artifacts={slow_chunk.get('artifactCount', 0)}",
            ]
        )
        project_parts.extend(slow_chunk_parts)
    elif not has_snapshot_counters and project.get("fullBakeLastResult") is not None:
        project_parts.append(f"full_bake_last_result={project.get('fullBakeLastResult')}")
    telemetry_families = summary.get("telemetryFamilies")
    if telemetry_families:
        project_parts.append(f"telemetry_families={','.join(str(item) for item in telemetry_families)}")

    return f"runtime={' '.join(runtime_parts)}; project={' '.join(project_parts)}"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Summarize Vertigo Sync preview telemetry from /plugin/state JSON."
    )
    parser.add_argument("plugin_state_json", type=Path)
    parser.add_argument("summary_out", type=Path)
    args = parser.parse_args()

    data = json.loads(args.plugin_state_json.read_text(encoding="utf-8"))
    summary = summarize_plugin_state(data, telemetry_families=os.environ.get("ARNIS_TELEMETRY_FAMILIES"))
    args.summary_out.write_text(summary, encoding="utf-8")
    print(summary)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
