#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "specs" / "generated" / "synthetic-manifest.json"

def main() -> None:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    chunks = []
    chunk_refs = []
    manifest = {
        "schemaVersion": "0.4.0",
        "meta": {
            "worldName": "SyntheticGrid",
            "generator": "scripts/generate_synthetic_manifest.py",
            "source": "synthetic",
            "metersPerStud": 1.0,
            "chunkSizeStuds": 256,
            "bbox": {
                "minLat": 0,
                "minLon": 0,
                "maxLat": 1,
                "maxLon": 1
            },
            "notes": ["Generated synthetic manifest"]
        },
        "chunks": chunks,
        "chunkRefs": chunk_refs,
    }

    for x in range(2):
        for z in range(2):
            chunk_id = f"{x}_{z}"
            chunks.append({
                "id": chunk_id,
                "originStuds": {"x": x * 256, "y": 0, "z": z * 256},
                "terrain": {
                    "cellSizeStuds": 2,
                    "width": 1,
                    "depth": 1,
                    "heights": [0.0],
                    "materials": ["Grass"],
                    "material": "Grass"
                },
                "roads": [],
                "rails": [],
                "buildings": [],
                "water": [],
                "props": [],
                "landuse": [],
                "barriers": []
            })
            chunk_refs.append({
                "id": chunk_id,
                "originStuds": {"x": x * 256, "y": 0, "z": z * 256},
                "featureCount": 1,
                "streamingCost": 8.0,
                "partitionVersion": "subplans.v1",
                "subplans": [
                    {
                        "id": "terrain",
                        "layer": "terrain",
                        "featureCount": 1,
                        "streamingCost": 8.0
                    },
                    {
                        "id": "roads",
                        "layer": "roads",
                        "featureCount": 0,
                        "streamingCost": 0.0
                    },
                    {
                        "id": "landuse",
                        "layer": "landuse",
                        "featureCount": 0,
                        "streamingCost": 0.0
                    },
                    {
                        "id": "buildings",
                        "layer": "buildings",
                        "featureCount": 0,
                        "streamingCost": 0.0
                    },
                    {
                        "id": "water",
                        "layer": "water",
                        "featureCount": 0,
                        "streamingCost": 0.0
                    },
                    {
                        "id": "props",
                        "layer": "props",
                        "featureCount": 0,
                        "streamingCost": 0.0
                    }
                ]
            })

    manifest["meta"]["totalFeatures"] = len(chunks)

    OUT.write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    print(f"Wrote {OUT}")

if __name__ == "__main__":
    main()
