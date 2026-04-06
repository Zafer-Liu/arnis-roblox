#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any

from route_slice_parity_artifacts import build_route_slice_manifest, _extract_chunk_ids_from_log
import scene_fidelity_audit as fidelity_audit


ICONIC_ROOF_SHAPES = {
    "dome",
    "onion",
    "cone",
    "pyramidal",
    "gabled",
    "hipped",
    "mansard",
    "gambrel",
    "skillion",
}
LANDMARK_USAGES = {
    "government",
    "office",
    "tower",
    "roof",
    "school",
    "commercial",
    "apartments",
    "residential",
    "parking",
}


TRUNCATED_STRING_FIELD_PATTERNS = {
    "groundMaterial": re.compile(r'"groundMaterial":"([^"]*)"'),
    "supportSurfaceRole": re.compile(r'"supportSurfaceRole":"([^"]*)"'),
    "localSupportSurfaceRole": re.compile(r'"localSupport":\{[^}]*"surfaceRole":"([^"]*)"'),
}
TRUNCATED_INT_FIELD_PATTERNS = {
    "nearbyBuildingModels": re.compile(r'"nearbyBuildingModels":(-?\d+)'),
    "nearbyRoofParts": re.compile(r'"nearbyRoofParts":(-?\d+)'),
    "nearbyReadableFacadeCueParts": re.compile(r'"nearbyReadableFacadeCueParts":(-?\d+)'),
    "nearbyWallParts": re.compile(r'"nearbyWallParts":(-?\d+)'),
    "collidableWallPartsNearby": re.compile(r'"collidableWallPartsNearby":(-?\d+)'),
    "overheadRoofParts": re.compile(r'"overheadRoofParts":(-?\d+)'),
    "localEnclosureNearbyWallParts": re.compile(r'"localEnclosure":\{[^}]*"nearbyWallParts":(-?\d+)'),
    "localEnclosureCollidableWallPartsNearby": re.compile(r'"localEnclosure":\{[^}]*"collidableWallPartsNearby":(-?\d+)'),
    "localEnclosureReadableFacadeCueParts": re.compile(r'"localEnclosure":\{[^}]*"readableFacadeCueParts":(-?\d+)'),
    "localRoofCoverNearbyRoofParts": re.compile(r'"localRoofCover":\{[^}]*"nearbyRoofParts":(-?\d+)'),
    "localRoofCoverOverheadRoofParts": re.compile(r'"localRoofCover":\{[^}]*"overheadRoofParts":(-?\d+)'),
}
TRUNCATED_LIST_FIELD_PATTERNS = {
    "nearestBuildingSourceIds": re.compile(r'"nearestBuildingSourceIds":\[(.*?)\]'),
    "nearestNamedBuildingSourceIds": re.compile(r'"nearestNamedBuildingSourceIds":\[(.*?)\]'),
    "nearestNamedBuildingNames": re.compile(r'"nearestNamedBuildingNames":\[(.*?)\]'),
}


def _write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")


def _build_building_index(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    index: dict[str, dict[str, Any]] = {}
    for chunk in manifest.get("chunks", []):
        if not isinstance(chunk, dict):
            continue
        chunk_id = str(chunk.get("id") or "")
        for building in chunk.get("buildings", []):
            if not isinstance(building, dict):
                continue
            source_id = building.get("id")
            if not isinstance(source_id, str) or not source_id:
                continue
            row = {
                "id": source_id,
                "name": building.get("name"),
                "usage": building.get("usage"),
                "roofShape": building.get("roofShape") or building.get("roof"),
                "roofMaterial": building.get("roofMaterial"),
                "wallMaterial": building.get("material"),
                "chunkId": chunk_id,
            }
            index[source_id] = row
    return index


def _is_notable_building(row: dict[str, Any]) -> bool:
    name = row.get("name")
    usage = str(row.get("usage") or "")
    roof_shape = str(row.get("roofShape") or "")
    roof_material = row.get("roofMaterial")
    return bool(name) or usage in LANDMARK_USAGES or roof_shape in ICONIC_ROOF_SHAPES or bool(roof_material)


def _select_sorted_rows(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    def sort_key(row: dict[str, Any]) -> tuple[str, str]:
        return (str(row.get("name") or ""), str(row.get("id") or ""))

    return sorted(rows, key=sort_key)


def _collect_player_near_buildings(building_index: dict[str, dict[str, Any]], client_world: dict[str, Any]) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    seen: set[str] = set()
    for source_id in client_world.get("nearestBuildingSourceIds") or []:
        if not isinstance(source_id, str) or source_id in seen:
            continue
        seen.add(source_id)
        row = building_index.get(source_id)
        if row is not None:
            rows.append(row)
    return rows


def _collect_player_near_named_buildings(
    building_index: dict[str, dict[str, Any]], client_world: dict[str, Any]
) -> list[dict[str, Any]]:
    rows = _collect_player_near_buildings(
        building_index,
        {"nearestBuildingSourceIds": client_world.get("nearestNamedBuildingSourceIds") or []},
    )
    if rows:
        return rows
    names = client_world.get("nearestNamedBuildingNames") or []
    if not isinstance(names, list):
        return []
    fallback_rows: list[dict[str, Any]] = []
    seen: set[str] = set()
    for row in building_index.values():
        name = row.get("name")
        if not isinstance(name, str) or name not in names or name in seen:
            continue
        seen.add(name)
        fallback_rows.append(row)
    return fallback_rows


def _parse_truncated_client_fields(line: str) -> dict[str, Any]:
    payload: dict[str, Any] = {}
    for key, pattern in TRUNCATED_STRING_FIELD_PATTERNS.items():
        match = pattern.search(line)
        if match:
            payload[key] = match.group(1)
    for key, pattern in TRUNCATED_INT_FIELD_PATTERNS.items():
        match = pattern.search(line)
        if match:
            payload[key] = int(match.group(1))
    for key, pattern in TRUNCATED_LIST_FIELD_PATTERNS.items():
        match = pattern.search(line)
        if not match:
            continue
        raw_items = match.group(1).strip()
        if not raw_items:
            payload[key] = []
            continue
        payload[key] = re.findall(r'"([^"]+)"', raw_items)
    return payload


def _extract_latest_truncated_client_world(log_path: Path) -> dict[str, Any]:
    latest_compact_line = ""
    latest_local_line = ""
    with log_path.open(encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if "ARNIS_CLIENT_WORLD_COMPACT " in line:
                latest_compact_line = line
            elif "ARNIS_CLIENT_LOCAL_EXPERIENCE " in line:
                latest_local_line = line
    merged: dict[str, Any] = {}
    if latest_compact_line:
        merged.update(_parse_truncated_client_fields(latest_compact_line))
    if latest_local_line:
        merged.update(_parse_truncated_client_fields(latest_local_line))
    return merged


def build_landmark_roof_proof(
    manifest_sqlite: Path | None,
    log_path: Path,
    artifact_dir: Path,
    *,
    route_runtime_index: Path | None = None,
    marker: str = "ARNIS_SCENE_PLAY",
) -> dict[str, Any]:
    chunk_ids = _extract_chunk_ids_from_log(log_path, marker)
    manifest = build_route_slice_manifest(
        manifest_sqlite,
        chunk_ids,
        route_runtime_index=route_runtime_index,
    )

    manifest_json = artifact_dir / "route-slice-manifest.json"
    proof_json = artifact_dir / "landmark-roof-proof.json"
    proof_md = artifact_dir / "landmark-roof-proof.md"

    _write_json(manifest_json, manifest)

    report = fidelity_audit.build_report(
        manifest_json,
        log_path,
        marker=marker,
        truth_pack=None,
        preview_plugin_state=None,
    )
    report = fidelity_audit._enrich_report_with_log_markers(report, log_path)

    building_index = _build_building_index(manifest)
    all_rows = list(building_index.values())
    named_rows = _select_sorted_rows([row for row in all_rows if row.get("name")])
    notable_rows = _select_sorted_rows([row for row in all_rows if _is_notable_building(row)])
    iconic_rows = _select_sorted_rows(
        [row for row in notable_rows if str(row.get("roofShape") or "") in ICONIC_ROOF_SHAPES]
    )

    client_world = fidelity_audit._merge_client_world_markers(
        fidelity_audit._parse_latest_simple_marker(log_path, "ARNIS_CLIENT_WORLD_COMPACT"),
        fidelity_audit._parse_latest_simple_marker(log_path, "ARNIS_CLIENT_LOCAL_EXPERIENCE"),
    )
    if not client_world:
        client_world = report.get("clientWorld") if isinstance(report.get("clientWorld"), dict) else {}
    truncated_client_world = _extract_latest_truncated_client_world(log_path)
    if truncated_client_world:
        client_world.update({k: v for k, v in truncated_client_world.items() if v not in (None, "", [])})
    if client_world.get("supportSurfaceRole") in (None, "", "unknown") and client_world.get("localSupportSurfaceRole"):
        client_world["supportSurfaceRole"] = client_world["localSupportSurfaceRole"]
    if not client_world.get("nearbyWallParts") and client_world.get("localEnclosureNearbyWallParts") is not None:
        client_world["nearbyWallParts"] = client_world["localEnclosureNearbyWallParts"]
    if not client_world.get("collidableWallPartsNearby") and client_world.get("localEnclosureCollidableWallPartsNearby") is not None:
        client_world["collidableWallPartsNearby"] = client_world["localEnclosureCollidableWallPartsNearby"]
    if not client_world.get("nearbyReadableFacadeCueParts") and client_world.get("localEnclosureReadableFacadeCueParts") is not None:
        client_world["nearbyReadableFacadeCueParts"] = client_world["localEnclosureReadableFacadeCueParts"]
    if not client_world.get("nearbyRoofParts") and client_world.get("localRoofCoverNearbyRoofParts") is not None:
        client_world["nearbyRoofParts"] = client_world["localRoofCoverNearbyRoofParts"]
    if not client_world.get("overheadRoofParts") and client_world.get("localRoofCoverOverheadRoofParts") is not None:
        client_world["overheadRoofParts"] = client_world["localRoofCoverOverheadRoofParts"]
    nearest_rows = _collect_player_near_buildings(building_index, client_world)
    nearest_named_rows = _collect_player_near_named_buildings(building_index, client_world)

    scene = report.get("scene") if isinstance(report.get("scene"), dict) else {}
    summary = report.get("summary") if isinstance(report.get("summary"), dict) else {}

    proof = {
        "marker": marker,
        "log": str(log_path),
        "manifestSourceKind": report.get("manifestSourceKind"),
        "manifestSourceName": report.get("manifestSourceName"),
        "chunkIds": chunk_ids,
        "namedBuildingsInSlice": named_rows,
        "notableBuildingsInSlice": notable_rows,
        "iconicRoofBuildingsInSlice": iconic_rows,
        "playerNearbyBuildings": nearest_rows,
        "playerNearbyNamedBuildings": nearest_named_rows,
        "clientWorldSummary": {
            "nearbyBuildingModels": client_world.get("nearbyBuildingModels"),
            "nearbyRoofParts": client_world.get("nearbyRoofParts"),
            "overheadRoofParts": client_world.get("overheadRoofParts"),
            "nearbyReadableFacadeCueParts": client_world.get("nearbyReadableFacadeCueParts"),
            "nearbyWallParts": client_world.get("nearbyWallParts"),
            "collidableWallPartsNearby": client_world.get("collidableWallPartsNearby"),
            "nearestBuildingSourceIds": client_world.get("nearestBuildingSourceIds"),
            "nearestNamedBuildingSourceIds": client_world.get("nearestNamedBuildingSourceIds"),
            "nearestNamedBuildingNames": client_world.get("nearestNamedBuildingNames"),
            "supportSurfaceRole": client_world.get("supportSurfaceRole"),
            "groundMaterial": client_world.get("groundMaterial"),
        },
        "roofCoverageByShape": scene.get("buildingRoofCoverageByShape"),
        "roofCoverageByUsage": scene.get("buildingRoofCoverageByUsage"),
        "sceneBuildingRoofMaterials": summary.get("sceneBuildingRoofMaterials"),
        "sceneBuildingWallMaterials": summary.get("sceneBuildingWallMaterials"),
    }
    _write_json(proof_json, proof)

    lines = [
        f"# Landmark Roof Proof",
        "",
        f"- marker: `{marker}`",
        f"- manifestSourceKind: `{proof['manifestSourceKind']}`",
        f"- manifestSourceName: `{proof['manifestSourceName']}`",
        f"- chunkIds: `{', '.join(chunk_ids)}`",
        f"- nearbyBuildingModels: `{proof['clientWorldSummary']['nearbyBuildingModels']}`",
        f"- nearbyRoofParts: `{proof['clientWorldSummary']['nearbyRoofParts']}`",
        f"- overheadRoofParts: `{proof['clientWorldSummary']['overheadRoofParts']}`",
        f"- nearestBuildingSourceIds: `{', '.join(proof['clientWorldSummary']['nearestBuildingSourceIds'] or [])}`",
        "",
        "## Named Buildings In Slice",
    ]
    if named_rows:
        for row in named_rows:
            lines.append(
                f"- `{row['id']}`: {row.get('name')} | usage=`{row.get('usage')}` roof=`{row.get('roofShape')}` roofMaterial=`{row.get('roofMaterial')}` wallMaterial=`{row.get('wallMaterial')}`"
            )
    else:
        lines.append("- none")
    lines.append("")
    lines.append("## Iconic Roof Buildings In Slice")
    if iconic_rows:
        for row in iconic_rows:
            lines.append(
                f"- `{row['id']}` | usage=`{row.get('usage')}` roof=`{row.get('roofShape')}` roofMaterial=`{row.get('roofMaterial')}` wallMaterial=`{row.get('wallMaterial')}`"
            )
    else:
        lines.append("- none")
    proof_md.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return proof


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Build bounded non-visual landmark/roof proof artifacts from authoritative Studio logs.")
    parser.add_argument("--manifest-sqlite", type=Path)
    parser.add_argument("--route-runtime-index", type=Path)
    parser.add_argument("--log", type=Path, required=True)
    parser.add_argument("--artifact-dir", type=Path, required=True)
    parser.add_argument("--marker", default="ARNIS_SCENE_PLAY")
    args = parser.parse_args(argv)

    if bool(args.manifest_sqlite) == bool(args.route_runtime_index):
        parser.error("provide exactly one of --manifest-sqlite or --route-runtime-index")

    proof = build_landmark_roof_proof(
        args.manifest_sqlite,
        args.log,
        args.artifact_dir,
        route_runtime_index=args.route_runtime_index,
        marker=args.marker,
    )
    print(json.dumps(proof, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
