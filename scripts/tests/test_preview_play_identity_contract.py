from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PREVIEW_MODULE_PATH = ROOT / "scripts" / "refresh_preview_from_sample_data.py"
RUNTIME_MODULE_PATH = ROOT / "scripts" / "refresh_runtime_harness_from_sample_data.py"
PARITY_SPEC_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "Tests" / "CanonicalWorldParity.spec.lua"
PREVIEW_BUILDER_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "StudioPreview" / "AustinPreviewBuilder.lua"
RUN_AUSTIN_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "RunAustin.lua"


def load_module(module_path: Path, module_name: str):
    scripts_dir = str(module_path.parent)
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
    spec = importlib.util.spec_from_file_location(module_name, module_path)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load module spec from {module_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class PreviewPlayIdentityContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.preview_module = load_module(PREVIEW_MODULE_PATH, "refresh_preview_from_sample_data")
        cls.runtime_module = load_module(RUNTIME_MODULE_PATH, "refresh_runtime_harness_from_sample_data")
        cls.parity_spec_text = PARITY_SPEC_PATH.read_text(encoding="utf-8")
        cls.preview_builder_text = PREVIEW_BUILDER_PATH.read_text(encoding="utf-8")
        cls.run_austin_text = RUN_AUSTIN_PATH.read_text(encoding="utf-8")

    def test_preview_and_runtime_refresh_paths_share_identity_summary_contract(self) -> None:
        source_chunks = {
            "0_0": {
                "id": "0_0",
                "terrain": {"cellSizeStuds": 4},
                "roads": [{"id": "road_a"}, {"id": "road_b"}],
                "buildings": [{"id": "building_a"}],
            },
            "1_0": {
                "id": "1_0",
                "water": [{"id": "water_a"}],
                "props": [{"id": "prop_a"}],
            },
        }

        preview_summary = self.preview_module.build_identity_summary(["0_0", "1_0"], source_chunks)
        runtime_summary = self.runtime_module.build_identity_summary(["0_0", "1_0"], source_chunks)

        self.assertEqual(preview_summary, runtime_summary)
        self.assertEqual(preview_summary["byFamily"]["roads"], ["road_a", "road_b"])
        self.assertEqual(preview_summary["byChunk"]["1_0"]["water"], ["water_a"])

    def test_preview_and_runtime_refresh_paths_share_minimap_basis_contract(self) -> None:
        preview_basis = self.preview_module.build_minimap_basis_summary(
            ["0_0", "1_0"],
            canonical_anchor_position=(10.5, 20.25, -30.75),
            chunk_size_studs=256,
        )
        runtime_basis = self.runtime_module.build_minimap_basis_summary(
            ["0_0", "1_0"],
            canonical_anchor_position=(10.5, 20.25, -30.75),
            chunk_size_studs=256,
        )

        self.assertEqual(preview_basis, runtime_basis)
        self.assertEqual(preview_basis["canonicalAnchor"]["lookDirectionStuds"], {"x": 0, "y": 0, "z": 1})

    def test_canonical_world_parity_spec_asserts_identity_and_minimap_basis_parity(self) -> None:
        self.assertIn("identitySummary", self.parity_spec_text)
        self.assertIn("minimapBasis", self.parity_spec_text)
        self.assertIn("expected preview and full-bake routes to preserve the same source identities", self.parity_spec_text)
        self.assertIn(
            "expected preview and full-bake routes to preserve the same minimap transform basis",
            self.parity_spec_text,
        )

    def test_preview_fixture_radius_matches_builder_and_runtime_radius_matches_play_owner(self) -> None:
        self.assertIn("PREVIEW_LOAD_RADIUS_STUDS = 1500", PREVIEW_MODULE_PATH.read_text(encoding="utf-8"))
        self.assertIn("AustinPreviewBuilder.LOAD_RADIUS = 1500", self.preview_builder_text)
        self.assertIn("RunAustin.LOAD_RADIUS = 1500", self.run_austin_text)


if __name__ == "__main__":
    unittest.main()
