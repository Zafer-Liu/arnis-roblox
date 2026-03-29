from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
GENERATOR_PATH = ROOT / "scripts" / "generate_synthetic_manifest.py"


class GenerateSyntheticManifestTests(unittest.TestCase):
    def _load_module(self):
        spec = importlib.util.spec_from_file_location("generate_synthetic_manifest", GENERATOR_PATH)
        if spec is None or spec.loader is None:
            raise AssertionError("failed to load generator module")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def test_main_writes_current_0_4_0_contract_shape(self) -> None:
        module = self._load_module()

        with tempfile.TemporaryDirectory() as tmpdir:
            out_path = Path(tmpdir) / "synthetic-manifest.json"
            module.OUT = out_path

            module.main()

            manifest = json.loads(out_path.read_text(encoding="utf-8"))

        self.assertEqual(manifest["schemaVersion"], "0.4.0")
        self.assertEqual(
            manifest["meta"],
            {
                "worldName": "SyntheticGrid",
                "generator": "scripts/generate_synthetic_manifest.py",
                "source": "synthetic",
                "metersPerStud": 1.0,
                "chunkSizeStuds": 256,
                "bbox": {
                    "minLat": 0,
                    "minLon": 0,
                    "maxLat": 1,
                    "maxLon": 1,
                },
                "totalFeatures": 4,
                "notes": ["Generated synthetic manifest"],
            },
        )

        self.assertEqual(len(manifest["chunks"]), 4)
        first_chunk = manifest["chunks"][0]
        self.assertEqual(
            set(first_chunk.keys()),
            {
                "id",
                "originStuds",
                "terrain",
                "roads",
                "rails",
                "buildings",
                "water",
                "props",
                "landuse",
                "barriers",
            },
        )
        self.assertEqual(
            first_chunk["terrain"],
            {
                "cellSizeStuds": 2,
                "width": 1,
                "depth": 1,
                "heights": [0.0],
                "materials": ["Grass"],
                "material": "Grass",
            },
        )
        self.assertEqual(first_chunk["roads"], [])
        self.assertEqual(first_chunk["rails"], [])
        self.assertEqual(first_chunk["buildings"], [])
        self.assertEqual(first_chunk["water"], [])
        self.assertEqual(first_chunk["props"], [])
        self.assertEqual(first_chunk["landuse"], [])
        self.assertEqual(first_chunk["barriers"], [])

        self.assertEqual(len(manifest["chunkRefs"]), 4)
        first_chunk_ref = manifest["chunkRefs"][0]
        self.assertEqual(
            first_chunk_ref,
            {
                "id": "0_0",
                "originStuds": {"x": 0, "y": 0, "z": 0},
                "featureCount": 1,
                "streamingCost": 8.0,
                "partitionVersion": "subplans.v1",
                "subplans": [
                    {
                        "id": "terrain",
                        "layer": "terrain",
                        "featureCount": 1,
                        "streamingCost": 8.0,
                    },
                    {
                        "id": "roads",
                        "layer": "roads",
                        "featureCount": 0,
                        "streamingCost": 0.0,
                    },
                    {
                        "id": "landuse",
                        "layer": "landuse",
                        "featureCount": 0,
                        "streamingCost": 0.0,
                    },
                    {
                        "id": "buildings",
                        "layer": "buildings",
                        "featureCount": 0,
                        "streamingCost": 0.0,
                    },
                    {
                        "id": "water",
                        "layer": "water",
                        "featureCount": 0,
                        "streamingCost": 0.0,
                    },
                    {
                        "id": "props",
                        "layer": "props",
                        "featureCount": 0,
                        "streamingCost": 0.0,
                    },
                ],
            },
        )


if __name__ == "__main__":
    unittest.main()
