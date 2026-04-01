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

            payload = out_path.read_text(encoding="utf-8")
            manifest = json.loads(payload)

        self.assertEqual(manifest["schemaVersion"], "0.4.0")
        self.assertNotIn('"schemaVersion": "0.1.0"', payload)
        self.assertNotIn('"schemaVersion": "0.2.0"', payload)
        self.assertNotIn("chunkRefs", manifest)

        meta = manifest["meta"]
        for key in (
            "worldName",
            "generator",
            "source",
            "metersPerStud",
            "chunkSizeStuds",
            "bbox",
            "totalFeatures",
            "notes",
        ):
            self.assertIn(key, meta)

        self.assertEqual(meta["worldName"], "SyntheticGrid")
        self.assertEqual(meta["generator"], "scripts/generate_synthetic_manifest.py")
        self.assertEqual(meta["source"], "synthetic")
        self.assertEqual(meta["metersPerStud"], 1.0)
        self.assertEqual(meta["chunkSizeStuds"], 256)
        self.assertEqual(meta["totalFeatures"], 4)
        self.assertEqual(meta["notes"], ["Generated synthetic manifest"])

        bbox = meta["bbox"]
        self.assertEqual(
            {key: bbox[key] for key in ("minLat", "minLon", "maxLat", "maxLon")},
            {"minLat": 0, "minLon": 0, "maxLat": 1, "maxLon": 1},
        )

        self.assertEqual(len(manifest["chunks"]), 4)
        total_features = 0
        required_chunk_keys = {
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
        }

        for chunk in manifest["chunks"]:
            self.assertEqual(set(chunk.keys()), required_chunk_keys)
            self.assertEqual(set(chunk["originStuds"].keys()), {"x", "y", "z"})
            self.assertEqual(chunk["roads"], [])
            self.assertEqual(chunk["rails"], [])
            self.assertEqual(chunk["buildings"], [])
            self.assertEqual(chunk["water"], [])
            self.assertEqual(chunk["props"], [])
            self.assertEqual(chunk["landuse"], [])
            self.assertEqual(chunk["barriers"], [])

            terrain = chunk["terrain"]
            self.assertEqual(
                set(terrain.keys()),
                {"cellSizeStuds", "width", "depth", "heights", "materials", "material"},
            )
            self.assertEqual(terrain["cellSizeStuds"], 2)
            self.assertEqual(terrain["width"], 1)
            self.assertEqual(terrain["depth"], 1)
            self.assertEqual(terrain["heights"], [0.0])
            self.assertEqual(terrain["materials"], ["Grass"])
            self.assertEqual(terrain["material"], "Grass")

            total_features += 1

        self.assertEqual(meta["totalFeatures"], total_features)


if __name__ == "__main__":
    unittest.main()
