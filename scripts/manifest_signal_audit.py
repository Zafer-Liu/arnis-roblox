#!/usr/bin/env python3
"""Manifest Signal Audit — measures how much source data is preserved vs dropped.

Reads a compiled manifest (JSON or SQLite) and reports:
- Per-feature-type: how many fields are populated vs empty
- Signal preservation rate: % of compiled fields actually consumed by builders
- Data richness distribution: histograms of key fields
- Drop audit: which fields are compiled but never rendered

Usage:
    python3 scripts/manifest_signal_audit.py --manifest rust/out/austin-manifest.json
    python3 scripts/manifest_signal_audit.py --manifest rust/out/austin-manifest.json --format json
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


# Fields consumed by each builder (from the data gap audit)
CONSUMED_FIELDS = {
    "buildings": {
        "id", "footprint", "baseY", "height", "material", "wallColor",
        "roofColor", "roofShape", "roofMaterial", "usage", "minHeight",
        "roofHeight", "name", "roof", "holes", "levels", "rooms",
        "facadeStyle", "roofLevels",
    },
    "roads": {
        "id", "kind", "widthStuds", "points", "material", "hasSidewalk",
        "elevated", "tunnel", "surface", "oneway", "maxspeed", "lanes",
        "lit", "color", "sidewalk", "layer", "subkind",
    },
    "water": {
        "id", "kind", "material", "surfaceY", "intermittent", "points",
        "footprint", "holes", "widthStuds", "color",
    },
    "props": {
        "id", "kind", "position", "yawDegrees", "scale", "species",
        "height", "leafType", "circumference",
    },
    "terrain": {
        "cellSizeStuds", "width", "depth", "heights", "material", "materials",
    },
    "landuse": {"id", "kind", "material", "footprint"},
    "barriers": {"id", "kind", "points"},
    "rails": {"id", "kind", "widthStuds", "points", "material"},
}

# Fields compiled but historically not consumed (now fixed in this sprint)
RECENTLY_WIRED = {
    "buildings": {"facadeStyle", "roofLevels"},
    "roads": {"sidewalk", "layer", "subkind"},
    "water": {"color"},
    "terrain": {"materials"},
}


def audit_feature_list(features: list[dict[str, Any]], feature_type: str) -> dict:
    """Audit a list of features for signal preservation."""
    consumed = CONSUMED_FIELDS.get(feature_type, set())
    recently = RECENTLY_WIRED.get(feature_type, set())

    field_populated = Counter()
    field_total = Counter()
    total_features = len(features)

    for feature in features:
        for key, value in feature.items():
            field_total[key] += 1
            if value is not None and value != "" and value != [] and value != {}:
                field_populated[key] += 1

    all_fields = sorted(set(field_total.keys()))
    field_stats = []
    for field in all_fields:
        populated = field_populated.get(field, 0)
        total = field_total.get(field, 0)
        rate = populated / total if total > 0 else 0
        status = "consumed" if field in consumed else "unused"
        if field in recently:
            status = "recently_wired"
        field_stats.append({
            "field": field,
            "populated": populated,
            "total": total,
            "population_rate": round(rate, 3),
            "status": status,
        })

    consumed_populated = sum(
        1 for f in features
        for k in consumed
        if k in f and f[k] is not None and f[k] != "" and f[k] != [] and f[k] != {}
    )
    total_possible = total_features * len(consumed) if consumed else 1

    return {
        "feature_type": feature_type,
        "feature_count": total_features,
        "total_fields_seen": len(all_fields),
        "consumed_fields": len(consumed & set(all_fields)),
        "unused_fields": len(set(all_fields) - consumed),
        "signal_preservation_rate": round(consumed_populated / total_possible, 3) if total_possible > 0 else 0,
        "fields": field_stats,
    }


def audit_terrain(terrain: dict[str, Any]) -> dict:
    """Audit terrain grid data."""
    has_heights = bool(terrain.get("heights"))
    has_materials = bool(terrain.get("materials"))
    height_count = len(terrain.get("heights", []))
    material_count = 0
    material_distribution: Counter = Counter()

    materials_list = terrain.get("materials", [])
    if materials_list:
        for mat in materials_list:
            if mat:
                material_count += 1
                material_distribution[mat] += 1

    return {
        "feature_type": "terrain",
        "has_heights": has_heights,
        "height_cell_count": height_count,
        "has_satellite_materials": has_materials,
        "satellite_material_cell_count": material_count,
        "satellite_material_coverage": round(material_count / height_count, 3) if height_count > 0 else 0,
        "material_distribution": dict(material_distribution.most_common(20)),
        "grid_width": terrain.get("width", 0),
        "grid_depth": terrain.get("depth", 0),
        "cell_size_studs": terrain.get("cellSizeStuds", 0),
    }


def audit_manifest(manifest: dict[str, Any]) -> dict:
    """Audit a full manifest for signal preservation."""
    results = {"chunks": []}

    chunks = manifest.get("chunks", [])
    aggregate = defaultdict(lambda: {"features": 0, "signal_rate": 0.0})

    for chunk in chunks:
        chunk_id = chunk.get("id", "unknown")
        chunk_result = {"chunk_id": chunk_id, "audits": []}

        for feature_type in ["buildings", "roads", "water", "props", "landuse", "barriers", "rails"]:
            features = chunk.get(feature_type, [])
            if features:
                audit = audit_feature_list(features, feature_type)
                chunk_result["audits"].append(audit)
                aggregate[feature_type]["features"] += audit["feature_count"]
                aggregate[feature_type]["signal_rate"] += audit["signal_preservation_rate"] * audit["feature_count"]

        terrain = chunk.get("terrain")
        if terrain:
            chunk_result["terrain"] = audit_terrain(terrain)

        results["chunks"].append(chunk_result)

    # Compute aggregate signal rates
    results["aggregate"] = {}
    for ft, data in aggregate.items():
        count = data["features"]
        results["aggregate"][ft] = {
            "total_features": count,
            "avg_signal_preservation_rate": round(data["signal_rate"] / count, 3) if count > 0 else 0,
        }

    return results


def format_text(results: dict) -> str:
    """Format audit results as human-readable text."""
    lines = ["=== Manifest Signal Audit ===", ""]

    agg = results.get("aggregate", {})
    if agg:
        lines.append("Feature Type           | Features | Signal Preservation")
        lines.append("-" * 60)
        for ft in sorted(agg.keys()):
            data = agg[ft]
            lines.append(f"{ft:<22} | {data['total_features']:>8} | {data['avg_signal_preservation_rate']:.1%}")
        lines.append("")

    # Terrain summary across chunks
    terrain_chunks = [c.get("terrain") for c in results.get("chunks", []) if c.get("terrain")]
    if terrain_chunks:
        total_cells = sum(t.get("height_cell_count", 0) for t in terrain_chunks)
        sat_cells = sum(t.get("satellite_material_cell_count", 0) for t in terrain_chunks)
        coverage = sat_cells / total_cells if total_cells > 0 else 0
        lines.append(f"Terrain: {total_cells} cells, {sat_cells} with satellite materials ({coverage:.1%} coverage)")

        all_mats: Counter = Counter()
        for t in terrain_chunks:
            for mat, count in t.get("material_distribution", {}).items():
                all_mats[mat] += count
        if all_mats:
            lines.append("  Material distribution:")
            for mat, count in all_mats.most_common(15):
                lines.append(f"    {mat}: {count}")
        lines.append("")

    # Per-chunk field details (first chunk only for brevity)
    if results.get("chunks"):
        first = results["chunks"][0]
        lines.append(f"Sample chunk: {first.get('chunk_id', 'unknown')}")
        for audit in first.get("audits", []):
            lines.append(f"  {audit['feature_type']}: {audit['feature_count']} features, {audit['signal_preservation_rate']:.1%} signal preserved")
            unused = [f for f in audit["fields"] if f["status"] == "unused" and f["populated"] > 0]
            if unused:
                lines.append(f"    Unused but populated: {', '.join(f['field'] for f in unused)}")

    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description="Audit manifest signal preservation")
    parser.add_argument("--manifest", required=True, help="Path to compiled manifest JSON")
    parser.add_argument("--format", choices=["text", "json"], default="text")
    args = parser.parse_args()

    manifest_path = Path(args.manifest)
    if not manifest_path.exists():
        print(f"Manifest not found: {manifest_path}", file=sys.stderr)
        sys.exit(1)

    with open(manifest_path, encoding="utf-8") as f:
        manifest = json.load(f)

    results = audit_manifest(manifest)

    if args.format == "json":
        print(json.dumps(results, indent=2))
    else:
        print(format_text(results))


if __name__ == "__main__":
    main()
