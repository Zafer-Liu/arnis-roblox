#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


def _write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")


def _load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def _append_unique(rows: list[str], value: str | None) -> None:
    if not isinstance(value, str) or not value:
        return
    if value not in rows:
        rows.append(value)


def build_landmark_roof_matrix(proof_paths: list[Path], artifact_dir: Path) -> dict[str, Any]:
    proofs: list[dict[str, Any]] = []
    for path in proof_paths:
        proofs.append(_load_json(path))

    slices: list[dict[str, Any]] = []
    named_buildings: dict[str, dict[str, Any]] = {}
    iconic_roofs: dict[str, dict[str, Any]] = {}

    for proof_path, proof in zip(proof_paths, proofs, strict=False):
        slice_id = proof_path.parent.name
        client = proof.get("clientWorldSummary") if isinstance(proof.get("clientWorldSummary"), dict) else {}
        slice_row = {
            "sliceId": slice_id,
            "proofPath": str(proof_path),
            "manifestSourceKind": proof.get("manifestSourceKind"),
            "manifestSourceName": proof.get("manifestSourceName"),
            "chunkIds": proof.get("chunkIds") or [],
            "namedBuildings": [row.get("name") for row in proof.get("namedBuildingsInSlice", []) if row.get("name")],
            "playerNearbyNamedBuildings": [row.get("name") for row in proof.get("playerNearbyNamedBuildings", []) if row.get("name")],
            "nearbyReadableFacadeCueParts": client.get("nearbyReadableFacadeCueParts"),
            "nearbyRoofParts": client.get("nearbyRoofParts"),
            "overheadRoofParts": client.get("overheadRoofParts"),
            "nearestBuildingSourceIds": client.get("nearestBuildingSourceIds") or [],
            "iconicRoofShapes": sorted(
                {
                    str(row.get("roofShape"))
                    for row in proof.get("iconicRoofBuildingsInSlice", [])
                    if isinstance(row, dict) and row.get("roofShape")
                }
            ),
        }
        slices.append(slice_row)

        for row in proof.get("namedBuildingsInSlice", []):
            if not isinstance(row, dict):
                continue
            name = row.get("name")
            if not isinstance(name, str) or not name:
                continue
            entry = named_buildings.setdefault(
                name,
                {
                    "name": name,
                    "sourceIds": [],
                    "usages": [],
                    "roofShapes": [],
                    "roofMaterials": [],
                    "wallMaterials": [],
                    "sliceIds": [],
                    "playerNearbySliceIds": [],
                },
            )
            _append_unique(entry["sourceIds"], row.get("id"))
            _append_unique(entry["usages"], row.get("usage"))
            _append_unique(entry["roofShapes"], row.get("roofShape"))
            _append_unique(entry["roofMaterials"], row.get("roofMaterial"))
            _append_unique(entry["wallMaterials"], row.get("wallMaterial"))
            _append_unique(entry["sliceIds"], slice_id)
        for row in proof.get("playerNearbyNamedBuildings", []):
            if not isinstance(row, dict):
                continue
            name = row.get("name")
            if not isinstance(name, str) or not name:
                continue
            entry = named_buildings.setdefault(
                name,
                {
                    "name": name,
                    "sourceIds": [],
                    "usages": [],
                    "roofShapes": [],
                    "roofMaterials": [],
                    "wallMaterials": [],
                    "sliceIds": [],
                    "playerNearbySliceIds": [],
                },
            )
            _append_unique(entry["playerNearbySliceIds"], slice_id)

        for row in proof.get("iconicRoofBuildingsInSlice", []):
            if not isinstance(row, dict):
                continue
            building_id = row.get("id")
            if not isinstance(building_id, str) or not building_id:
                continue
            entry = iconic_roofs.setdefault(
                building_id,
                {
                    "id": building_id,
                    "name": row.get("name"),
                    "usage": row.get("usage"),
                    "roofShape": row.get("roofShape"),
                    "roofMaterial": row.get("roofMaterial"),
                    "wallMaterial": row.get("wallMaterial"),
                    "sliceIds": [],
                },
            )
            _append_unique(entry["sliceIds"], slice_id)

    matrix = {
        "sliceCount": len(slices),
        "slices": slices,
        "namedBuildings": sorted(named_buildings.values(), key=lambda row: row["name"]),
        "iconicRoofBuildings": sorted(iconic_roofs.values(), key=lambda row: row["id"]),
    }
    matrix["namedBuildingsWithoutPlayerNearbyCoverage"] = [
        row["name"]
        for row in matrix["namedBuildings"]
        if not row.get("playerNearbySliceIds")
    ]
    matrix["namedBuildingsWithPlayerNearbyCoverage"] = [
        row["name"]
        for row in matrix["namedBuildings"]
        if row.get("playerNearbySliceIds")
    ]

    _write_json(artifact_dir / "landmark-roof-matrix.json", matrix)
    md_lines = [
        "# Landmark Roof Matrix",
        "",
        f"- sliceCount: `{matrix['sliceCount']}`",
        f"- namedBuildingsWithPlayerNearbyCoverage: `{', '.join(matrix['namedBuildingsWithPlayerNearbyCoverage']) or 'none'}`",
        f"- namedBuildingsWithoutPlayerNearbyCoverage: `{', '.join(matrix['namedBuildingsWithoutPlayerNearbyCoverage']) or 'none'}`",
        "",
        "## Slices",
    ]
    for row in slices:
        md_lines.append(
            f"- `{row['sliceId']}` | named={', '.join(row['namedBuildings']) or 'none'} | nearbyNamed={', '.join(row['playerNearbyNamedBuildings']) or 'none'} | iconicRoofShapes={', '.join(row['iconicRoofShapes']) or 'none'} | nearbyFacade={row['nearbyReadableFacadeCueParts']} | nearbyRoofParts={row['nearbyRoofParts']} | overheadRoofParts={row['overheadRoofParts']}"
        )
    md_lines.append("")
    md_lines.append("## Named Buildings")
    for row in matrix["namedBuildings"]:
        md_lines.append(
            f"- {row['name']} | sourceIds={', '.join(row['sourceIds']) or 'none'} | sliceIds={', '.join(row['sliceIds']) or 'none'} | playerNearbySliceIds={', '.join(row['playerNearbySliceIds']) or 'none'} | roofMaterials={', '.join([item for item in row['roofMaterials'] if item]) or 'none'}"
        )
    (artifact_dir / "landmark-roof-matrix.md").write_text("\n".join(md_lines) + "\n", encoding="utf-8")
    return matrix


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Aggregate multiple landmark/roof proof artifacts into one matrix.")
    parser.add_argument("--artifact-dir", type=Path, required=True)
    parser.add_argument("proofs", nargs="+", type=Path)
    args = parser.parse_args(argv)

    matrix = build_landmark_roof_matrix(args.proofs, args.artifact_dir)
    print(json.dumps(matrix, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
