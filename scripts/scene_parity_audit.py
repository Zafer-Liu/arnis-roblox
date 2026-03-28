#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from html import escape
from pathlib import Path
from typing import Any


def _load_json(path: Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as handle:
        data = json.load(handle)
    if not isinstance(data, dict):
        raise ValueError(f"{path} did not contain a top-level JSON object")
    return data


def _normalize_chunk_ids(scene: dict[str, Any]) -> list[str]:
    values = scene.get("chunkIds")
    if not isinstance(values, list):
        return []
    return [value for value in values if isinstance(value, str)]


def _normalize_scalar(scene: dict[str, Any], key: str) -> int:
    try:
        return int(scene.get(key) or 0)
    except (TypeError, ValueError):
        return 0


def _normalize_string(value: Any) -> str:
    return str(value or "")


def _normalize_world_identity(report: dict[str, Any]) -> str:
    world_identity = report.get("worldIdentity")
    if isinstance(world_identity, str) and world_identity:
        return world_identity
    return _normalize_string(report.get("rootName"))


def _normalize_client_world(report: dict[str, Any]) -> dict[str, Any] | None:
    client_world = report.get("clientWorld")
    if not isinstance(client_world, dict):
        return None
    normalized: dict[str, Any] = {}
    for key in (
        "worldRootName",
        "worldRootExists",
        "nearbyBuildingModels",
        "nearbyMergedBuildingMeshParts",
        "nearbyRoofParts",
        "overheadRoofParts",
        "groundMaterial",
        "bootstrapState",
    ):
        if key not in client_world:
            continue
        value = client_world.get(key)
        if key in {
            "nearbyBuildingModels",
            "nearbyMergedBuildingMeshParts",
            "nearbyRoofParts",
            "overheadRoofParts",
        }:
            normalized[key] = _normalize_json_value(int(value or 0))
        else:
            normalized[key] = _normalize_json_value(value)
    return normalized


def _chunk_ids_match(
    edit_chunk_ids: list[str],
    play_chunk_ids: list[str],
    *,
    edit_report: dict[str, Any],
    play_report: dict[str, Any],
) -> bool:
    if edit_chunk_ids == play_chunk_ids:
        return True
    if _normalize_json_value(edit_report.get("focus")) != _normalize_json_value(play_report.get("focus")):
        return False
    if _normalize_json_value(edit_report.get("radius")) != _normalize_json_value(play_report.get("radius")):
        return False
    edit_envelope_kind = _normalize_string(edit_report.get("chunkEnvelopeKind"))
    play_envelope_kind = _normalize_string(play_report.get("chunkEnvelopeKind"))
    if edit_envelope_kind == "bounded_preview" and play_envelope_kind == "runtime_resident":
        return set(edit_chunk_ids).issubset(play_chunk_ids)
    return False


def _normalize_json_value(value: Any) -> Any:
    if isinstance(value, dict):
        return {str(key): _normalize_json_value(nested) for key, nested in sorted(value.items())}
    if isinstance(value, list):
        return [_normalize_json_value(item) for item in value]
    if isinstance(value, (str, int, float, bool)) or value is None:
        return value
    return str(value)


def _normalize_roof_shape_coverage(scene: dict[str, Any]) -> dict[str, dict[str, int]]:
    coverage = scene.get("buildingRoofCoverageByShape")
    if not isinstance(coverage, dict):
        return {}
    normalized: dict[str, dict[str, int]] = {}
    for bucket, row in coverage.items():
        if not isinstance(bucket, str) or not isinstance(row, dict):
            continue
        normalized[bucket] = {
            "buildingModelCount": _normalize_scalar(row, "buildingModelCount"),
            "withRoofCount": _normalize_scalar(row, "withRoofCount"),
            "withoutRoofCount": _normalize_scalar(row, "withoutRoofCount"),
        }
    return normalized


def _normalize_source_ids(row: dict[str, Any]) -> list[str]:
    source_ids = row.get("sourceIds")
    if not isinstance(source_ids, list):
        return []
    return sorted(value for value in source_ids if isinstance(value, str))


def _normalize_metric_bucket(
    scene: dict[str, Any],
    key: str,
    *,
    count_keys: tuple[str, ...],
) -> dict[str, dict[str, Any]]:
    buckets = scene.get(key)
    if not isinstance(buckets, dict):
        return {}
    normalized: dict[str, dict[str, Any]] = {}
    for bucket, row in buckets.items():
        if not isinstance(bucket, str) or not isinstance(row, dict):
            continue
        normalized[bucket] = {
            **{count_key: _normalize_scalar(row, count_key) for count_key in count_keys},
            "sourceIds": _normalize_source_ids(row),
        }
    return normalized


def _normalize_material_bucket(scene: dict[str, Any], key: str) -> dict[str, dict[str, Any]]:
    buckets = scene.get(key)
    if not isinstance(buckets, dict):
        return {}
    normalized: dict[str, dict[str, Any]] = {}
    for bucket, row in buckets.items():
        if not isinstance(bucket, str) or not isinstance(row, dict):
            continue
        source_ids = row.get("sourceIds")
        normalized[bucket] = {
            "buildingModelCount": _normalize_scalar(row, "buildingModelCount"),
            "sourceIds": sorted(value for value in source_ids if isinstance(value, str))
            if isinstance(source_ids, list)
            else [],
        }
    return normalized


def _normalize_road_kind_surface(scene: dict[str, Any]) -> dict[str, dict[str, Any]]:
    buckets = scene.get("roadSurfacePartCountByKind")
    if not isinstance(buckets, dict):
        return {}
    normalized: dict[str, dict[str, Any]] = {}
    for bucket, row in buckets.items():
        if not isinstance(bucket, str) or not isinstance(row, dict):
            continue
        normalized[bucket] = {
            "surfacePartCount": _normalize_scalar(row, "surfacePartCount"),
            "featureCount": _normalize_scalar(row, "featureCount"),
            "sourceIds": _normalize_source_ids(row),
        }
    return normalized


def _is_subset_envelope_mode(
    edit_report: dict[str, Any],
    play_report: dict[str, Any],
    *,
    edit_chunk_ids: list[str],
    play_chunk_ids: list[str],
) -> bool:
    if _normalize_json_value(edit_report.get("focus")) != _normalize_json_value(play_report.get("focus")):
        return False
    if _normalize_json_value(edit_report.get("radius")) != _normalize_json_value(play_report.get("radius")):
        return False
    if _normalize_string(edit_report.get("chunkEnvelopeKind")) != "bounded_preview":
        return False
    if _normalize_string(play_report.get("chunkEnvelopeKind")) != "runtime_resident":
        return False
    return set(edit_chunk_ids).issubset(play_chunk_ids)


def _bucket_source_ids_match_as_subset(edit_bucket: Any, play_bucket: Any) -> bool:
    if not isinstance(edit_bucket, dict) or not isinstance(play_bucket, dict):
        return False
    for bucket, edit_row in edit_bucket.items():
        if not isinstance(bucket, str) or not isinstance(edit_row, dict):
            return False
        play_row = play_bucket.get(bucket)
        if not isinstance(play_row, dict):
            return False
        edit_source_ids = edit_row.get("sourceIds")
        play_source_ids = play_row.get("sourceIds")
        if not isinstance(edit_source_ids, list) or not isinstance(play_source_ids, list):
            return False
        if not edit_source_ids:
            return False
        if not set(edit_source_ids).issubset(play_source_ids):
            return False
    return True


def _scalar_matches_as_monotonic_subset(edit_value: Any, play_value: Any) -> bool:
    try:
        return int(edit_value) <= int(play_value)
    except (TypeError, ValueError):
        return False


def _bucket_matches_as_monotonic_subset(edit_bucket: Any, play_bucket: Any) -> bool:
    if not isinstance(edit_bucket, dict) or not isinstance(play_bucket, dict):
        return False
    for bucket, edit_row in edit_bucket.items():
        if not isinstance(bucket, str) or not isinstance(edit_row, dict):
            return False
        play_row = play_bucket.get(bucket)
        if not isinstance(play_row, dict):
            return False
        for key, value in edit_row.items():
            if key == "sourceIds":
                continue
            if not _scalar_matches_as_monotonic_subset(value, play_row.get(key)):
                return False
    return True


def build_report(edit_report: dict[str, Any], play_report: dict[str, Any]) -> dict[str, Any]:
    edit_scene = edit_report.get("scene") if isinstance(edit_report.get("scene"), dict) else {}
    play_scene = play_report.get("scene") if isinstance(play_report.get("scene"), dict) else {}

    comparisons = {
        "worldIdentity": {
            "edit": _normalize_world_identity(edit_report),
            "play": _normalize_world_identity(play_report),
        },
        "focus": {
            "edit": _normalize_json_value(edit_report.get("focus")),
            "play": _normalize_json_value(play_report.get("focus")),
        },
        "radius": {
            "edit": _normalize_json_value(edit_report.get("radius")),
            "play": _normalize_json_value(play_report.get("radius")),
        },
        "chunkIds": {
            "edit": _normalize_chunk_ids(edit_scene),
            "play": _normalize_chunk_ids(play_scene),
        },
        "buildingModelCount": {
            "edit": _normalize_scalar(edit_scene, "buildingModelCount"),
            "play": _normalize_scalar(play_scene, "buildingModelCount"),
        },
        "buildingModelsWithDirectShell": {
            "edit": _normalize_scalar(edit_scene, "buildingModelsWithDirectShell"),
            "play": _normalize_scalar(play_scene, "buildingModelsWithDirectShell"),
        },
        "buildingModelsWithRoofClosureDeck": {
            "edit": _normalize_scalar(edit_scene, "buildingModelsWithRoofClosureDeck"),
            "play": _normalize_scalar(play_scene, "buildingModelsWithRoofClosureDeck"),
        },
        "roadSurfacePartCount": {
            "edit": _normalize_scalar(edit_scene, "roadSurfacePartCount"),
            "play": _normalize_scalar(play_scene, "roadSurfacePartCount"),
        },
        "waterSurfacePartCount": {
            "edit": _normalize_scalar(edit_scene, "waterSurfacePartCount"),
            "play": _normalize_scalar(play_scene, "waterSurfacePartCount"),
        },
        "propInstanceCount": {
            "edit": _normalize_scalar(edit_scene, "propInstanceCount"),
            "play": _normalize_scalar(play_scene, "propInstanceCount"),
        },
        "roadSurfacePartCountBySubkind": {
            "edit": _normalize_metric_bucket(
                edit_scene,
                "roadSurfacePartCountBySubkind",
                count_keys=("surfacePartCount", "featureCount"),
            ),
            "play": _normalize_metric_bucket(
                play_scene,
                "roadSurfacePartCountBySubkind",
                count_keys=("surfacePartCount", "featureCount"),
            ),
        },
        "waterSurfacePartCountByType": {
            "edit": _normalize_metric_bucket(
                edit_scene,
                "waterSurfacePartCountByType",
                count_keys=("surfacePartCount", "featureCount"),
            ),
            "play": _normalize_metric_bucket(
                play_scene,
                "waterSurfacePartCountByType",
                count_keys=("surfacePartCount", "featureCount"),
            ),
        },
        "waterSurfacePartCountByKind": {
            "edit": _normalize_metric_bucket(
                edit_scene,
                "waterSurfacePartCountByKind",
                count_keys=("surfacePartCount", "featureCount"),
            ),
            "play": _normalize_metric_bucket(
                play_scene,
                "waterSurfacePartCountByKind",
                count_keys=("surfacePartCount", "featureCount"),
            ),
        },
        "railReceiptCountByKind": {
            "edit": _normalize_metric_bucket(
                edit_scene,
                "railReceiptCountByKind",
                count_keys=("receiptCount", "featureCount"),
            ),
            "play": _normalize_metric_bucket(
                play_scene,
                "railReceiptCountByKind",
                count_keys=("receiptCount", "featureCount"),
            ),
        },
        "vegetationInstanceCountByKind": {
            "edit": _normalize_metric_bucket(
                edit_scene,
                "vegetationInstanceCountByKind",
                count_keys=("instanceCount", "featureCount"),
            ),
            "play": _normalize_metric_bucket(
                play_scene,
                "vegetationInstanceCountByKind",
                count_keys=("instanceCount", "featureCount"),
            ),
        },
        "treeInstanceCountBySpecies": {
            "edit": _normalize_metric_bucket(
                edit_scene,
                "treeInstanceCountBySpecies",
                count_keys=("instanceCount", "featureCount"),
            ),
            "play": _normalize_metric_bucket(
                play_scene,
                "treeInstanceCountBySpecies",
                count_keys=("instanceCount", "featureCount"),
            ),
        },
        "buildingRoofCoverageByUsage": {
            "edit": _normalize_metric_bucket(
                edit_scene,
                "buildingRoofCoverageByUsage",
                count_keys=("buildingModelCount", "withRoofCount", "withoutRoofCount"),
            ),
            "play": _normalize_metric_bucket(
                play_scene,
                "buildingRoofCoverageByUsage",
                count_keys=("buildingModelCount", "withRoofCount", "withoutRoofCount"),
            ),
        },
        "buildingRoofCoverageByShape": {
            "edit": _normalize_roof_shape_coverage(edit_scene),
            "play": _normalize_roof_shape_coverage(play_scene),
        },
        "buildingModelCountByWallMaterial": {
            "edit": _normalize_material_bucket(edit_scene, "buildingModelCountByWallMaterial"),
            "play": _normalize_material_bucket(play_scene, "buildingModelCountByWallMaterial"),
        },
        "buildingModelCountByRoofMaterial": {
            "edit": _normalize_material_bucket(edit_scene, "buildingModelCountByRoofMaterial"),
            "play": _normalize_material_bucket(play_scene, "buildingModelCountByRoofMaterial"),
        },
        "roadSurfacePartCountByKind": {
            "edit": _normalize_road_kind_surface(edit_scene),
            "play": _normalize_road_kind_surface(play_scene),
        },
    }
    edit_client_world = _normalize_client_world(edit_report)
    play_client_world = _normalize_client_world(play_report)
    if edit_client_world is not None or play_client_world is not None:
        comparisons["clientWorld"] = {
            "edit": edit_client_world or {},
            "play": play_client_world or {},
        }

    code_by_metric = {
        "worldIdentity": "world_identity_mismatch",
        "focus": "focus_mismatch",
        "radius": "radius_mismatch",
        "chunkIds": "chunk_ids_mismatch",
        "buildingModelCount": "building_model_count_mismatch",
        "buildingModelsWithDirectShell": "building_direct_shell_count_mismatch",
        "buildingModelsWithRoofClosureDeck": "building_roof_closure_deck_count_mismatch",
        "roadSurfacePartCount": "road_surface_part_count_mismatch",
        "waterSurfacePartCount": "water_surface_part_count_mismatch",
        "propInstanceCount": "prop_instance_count_mismatch",
        "roadSurfacePartCountBySubkind": "road_subkind_surface_mismatch",
        "waterSurfacePartCountByType": "water_type_surface_mismatch",
        "waterSurfacePartCountByKind": "water_kind_surface_mismatch",
        "railReceiptCountByKind": "rail_kind_receipt_mismatch",
        "vegetationInstanceCountByKind": "vegetation_kind_instance_mismatch",
        "treeInstanceCountBySpecies": "tree_species_instance_mismatch",
        "buildingRoofCoverageByUsage": "roof_usage_coverage_mismatch",
        "buildingRoofCoverageByShape": "roof_shape_coverage_mismatch",
        "buildingModelCountByWallMaterial": "wall_material_count_mismatch",
        "buildingModelCountByRoofMaterial": "roof_material_count_mismatch",
        "roadSurfacePartCountByKind": "road_kind_surface_mismatch",
        "clientWorld": "client_world_mismatch",
    }
    severity_by_metric = {
        "worldIdentity": "high",
        "focus": "high",
        "radius": "high",
        "chunkIds": "high",
        "buildingModelCount": "high",
        "buildingModelsWithDirectShell": "medium",
        "buildingModelsWithRoofClosureDeck": "medium",
        "roadSurfacePartCount": "high",
        "waterSurfacePartCount": "medium",
        "propInstanceCount": "medium",
        "roadSurfacePartCountBySubkind": "high",
        "waterSurfacePartCountByType": "medium",
        "waterSurfacePartCountByKind": "medium",
        "railReceiptCountByKind": "medium",
        "vegetationInstanceCountByKind": "medium",
        "treeInstanceCountBySpecies": "medium",
        "buildingRoofCoverageByUsage": "medium",
        "buildingRoofCoverageByShape": "medium",
        "buildingModelCountByWallMaterial": "medium",
        "buildingModelCountByRoofMaterial": "medium",
        "roadSurfacePartCountByKind": "medium",
        "clientWorld": "high",
    }

    findings: list[dict[str, Any]] = []
    matching = 0
    mismatched = 0
    subset_envelope_mode = _is_subset_envelope_mode(
        edit_report,
        play_report,
        edit_chunk_ids=comparisons["chunkIds"]["edit"],
        play_chunk_ids=comparisons["chunkIds"]["play"],
    )
    source_id_subset_metrics = {
        "roadSurfacePartCountBySubkind",
        "waterSurfacePartCountByType",
        "waterSurfacePartCountByKind",
        "railReceiptCountByKind",
        "vegetationInstanceCountByKind",
        "treeInstanceCountBySpecies",
        "buildingModelCountByWallMaterial",
        "buildingModelCountByRoofMaterial",
        "roadSurfacePartCountByKind",
    }
    monotonic_subset_scalar_metrics = {
        "buildingModelCount",
        "buildingModelsWithDirectShell",
        "buildingModelsWithRoofClosureDeck",
        "roadSurfacePartCount",
        "waterSurfacePartCount",
        "propInstanceCount",
    }
    monotonic_subset_bucket_metrics = {
        "buildingRoofCoverageByUsage",
        "buildingRoofCoverageByShape",
    }
    for metric, payload in comparisons.items():
        matched = payload["edit"] == payload["play"]
        if metric == "chunkIds":
            matched = _chunk_ids_match(payload["edit"], payload["play"], edit_report=edit_report, play_report=play_report)
        elif not matched and subset_envelope_mode and metric in source_id_subset_metrics:
            matched = _bucket_source_ids_match_as_subset(payload["edit"], payload["play"])
        elif not matched and subset_envelope_mode and metric in monotonic_subset_scalar_metrics:
            matched = _scalar_matches_as_monotonic_subset(payload["edit"], payload["play"])
        elif not matched and subset_envelope_mode and metric in monotonic_subset_bucket_metrics:
            matched = _bucket_matches_as_monotonic_subset(payload["edit"], payload["play"])
        if matched:
            matching += 1
            continue
        mismatched += 1
        findings.append(
            {
                "severity": severity_by_metric[metric],
                "code": code_by_metric[metric],
                "message": f"edit/play mismatch for {metric}",
                "metric": metric,
                "edit": payload["edit"],
                "play": payload["play"],
            }
        )

    return {
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "editMarker": (edit_report.get("summary") or {}).get("marker"),
        "playMarker": (play_report.get("summary") or {}).get("marker"),
        "summary": {
            "matching": matching,
            "mismatched": mismatched,
            "totalChecks": len(comparisons),
        },
        "comparisons": comparisons,
        "findings": findings,
    }


def write_html_report(report: dict[str, Any], html_path: Path) -> None:
    summary = report.get("summary") or {}
    rows = []
    for metric, payload in (report.get("comparisons") or {}).items():
        match = payload.get("edit") == payload.get("play")
        rows.append(
            "<tr>"
            f"<td>{escape(metric)}</td>"
            f"<td>{escape(json.dumps(payload.get('edit'), sort_keys=True))}</td>"
            f"<td>{escape(json.dumps(payload.get('play'), sort_keys=True))}</td>"
            f"<td>{'match' if match else 'mismatch'}</td>"
            "</tr>"
        )
    findings_rows = []
    for finding in report.get("findings") or []:
        findings_rows.append(
            "<tr>"
            f"<td>{escape(str(finding.get('severity', '')))}</td>"
            f"<td>{escape(str(finding.get('code', '')))}</td>"
            f"<td>{escape(str(finding.get('message', '')))}</td>"
            "</tr>"
        )
    html = "\n".join(
        [
            "<!doctype html>",
            "<html><head><meta charset=\"utf-8\"><title>Scene Parity Audit</title></head><body>",
            "<h1>Scene Parity Audit</h1>",
            f"<p>matching={escape(str(summary.get('matching', 0)))} mismatched={escape(str(summary.get('mismatched', 0)))} totalChecks={escape(str(summary.get('totalChecks', 0)))}</p>",
            "<h2>Comparisons</h2>",
            "<table border=\"1\"><thead><tr><th>Metric</th><th>Edit</th><th>Play</th><th>Status</th></tr></thead><tbody>",
            *rows,
            "</tbody></table>",
            "<h2>Findings</h2>",
            "<table border=\"1\"><thead><tr><th>Severity</th><th>Code</th><th>Message</th></tr></thead><tbody>",
            *findings_rows,
            "</tbody></table>",
            "</body></html>",
        ]
    )
    html_path.write_text(html, encoding="utf-8")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Compare edit and play scene fidelity reports.")
    parser.add_argument("--edit-report", type=Path)
    parser.add_argument("--play-report", type=Path)
    parser.add_argument("--json-out", type=Path)
    parser.add_argument("--html-out", type=Path)
    parser.add_argument("--report-json", type=Path)
    args = parser.parse_args(argv)

    if args.report_json is not None:
        report = _load_json(args.report_json)
    else:
        if args.edit_report is None or args.play_report is None:
            parser.error("--edit-report and --play-report are required unless --report-json is provided")
        report = build_report(_load_json(args.edit_report), _load_json(args.play_report))

    if args.json_out is not None:
        args.json_out.write_text(json.dumps(report, indent=2, sort_keys=True), encoding="utf-8")
    if args.html_out is not None:
        write_html_report(report, args.html_out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
