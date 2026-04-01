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
            "roadsMs": int(last_slow_chunk.get("roadsMs", 0)),
            "landuseTerrainFillMs": int(last_slow_chunk.get("landuseTerrainFillMs", 0)),
            "artifactCount": int(last_slow_chunk.get("artifactCount", 0)),
        }
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
        project_parts.extend(
            [
                f"slow_chunk={slow_chunk.get('chunkId')}",
                f"slow_chunk_phase={slow_chunk.get('phase', 'unknown')}",
                f"slow_chunk_total_ms={slow_chunk.get('totalMs', 0)}",
                f"slow_chunk_buildings_ms={slow_chunk.get('buildingsMs', 0)}",
                f"slow_chunk_terrain_ms={slow_chunk.get('terrainMs', 0)}",
                f"slow_chunk_terrain_material_kind_count={slow_chunk.get('terrainMaterialKindCount', 0)}",
                f"slow_chunk_terrain_dominant_material={slow_chunk.get('terrainDominantMaterial', 'unknown')}",
                f"slow_chunk_terrain_dominant_material_cells={slow_chunk.get('terrainDominantMaterialCellCount', 0)}",
                f"slow_chunk_terrain_non_grass_cells={slow_chunk.get('terrainNonGrassCellCount', 0)}",
                f"slow_chunk_roads_ms={slow_chunk.get('roadsMs', 0)}",
                f"slow_chunk_landuse_terrain_fill_ms={slow_chunk.get('landuseTerrainFillMs', 0)}",
                f"slow_chunk_artifacts={slow_chunk.get('artifactCount', 0)}",
            ]
        )
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
