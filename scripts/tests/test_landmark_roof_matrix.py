from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "landmark_roof_matrix.py"


def load_module():
    scripts_dir = str(MODULE_PATH.parent)
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
    spec = importlib.util.spec_from_file_location("landmark_roof_matrix", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load module spec from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def write_proof(path: Path, *, slice_name: str, named: list[dict], nearby_named: list[dict], iconic: list[dict], client: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "manifestSourceKind": "route_catalog",
        "manifestSourceName": "PlanetaryRouteBundle.route-catalog",
        "chunkIds": ["0_0"],
        "namedBuildingsInSlice": named,
        "playerNearbyNamedBuildings": nearby_named,
        "iconicRoofBuildingsInSlice": iconic,
        "clientWorldSummary": client,
    }
    path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")


class LandmarkRoofMatrixTests(unittest.TestCase):
    def test_build_landmark_roof_matrix_aggregates_named_buildings_and_slices(self) -> None:
        module = load_module()
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            proof_a = root / "slice-a" / "landmark-roof-proof.json"
            proof_b = root / "slice-b" / "landmark-roof-proof.json"
            artifact_dir = root / "matrix"

            write_proof(
                proof_a,
                slice_name="slice-a",
                named=[
                    {
                        "id": "capitol",
                        "name": "Texas State Capitol",
                        "usage": "government",
                        "roofShape": "flat",
                        "roofMaterial": "copper",
                        "wallMaterial": "Limestone",
                    }
                ],
                nearby_named=[
                    {
                        "id": "capitol",
                        "name": "Texas State Capitol",
                        "usage": "government",
                        "roofShape": "flat",
                        "roofMaterial": "copper",
                        "wallMaterial": "Limestone",
                    }
                ],
                iconic=[
                    {
                        "id": "dome_1",
                        "name": None,
                        "usage": "office",
                        "roofShape": "dome",
                        "roofMaterial": None,
                        "wallMaterial": "Concrete",
                    }
                ],
                client={
                    "nearbyReadableFacadeCueParts": 10,
                    "nearbyRoofParts": 5,
                    "overheadRoofParts": 2,
                    "nearestBuildingSourceIds": ["capitol"],
                },
            )
            write_proof(
                proof_b,
                slice_name="slice-b",
                named=[
                    {
                        "id": "sam_houston",
                        "name": "Sam Houston Building",
                        "usage": "government",
                        "roofShape": "flat",
                        "roofMaterial": None,
                        "wallMaterial": "Limestone",
                    },
                    {
                        "id": "capitol",
                        "name": "Texas State Capitol",
                        "usage": "government",
                        "roofShape": "flat",
                        "roofMaterial": "copper",
                        "wallMaterial": "Limestone",
                    },
                ],
                nearby_named=[],
                iconic=[
                    {
                        "id": "onion_1",
                        "name": None,
                        "usage": "office",
                        "roofShape": "onion",
                        "roofMaterial": None,
                        "wallMaterial": "Concrete",
                    }
                ],
                client={
                    "nearbyReadableFacadeCueParts": 20,
                    "nearbyRoofParts": 7,
                    "overheadRoofParts": 3,
                    "nearestBuildingSourceIds": ["sam_houston", "capitol"],
                },
            )

            matrix = module.build_landmark_roof_matrix([proof_a, proof_b], artifact_dir)

            self.assertEqual(matrix["sliceCount"], 2)
            self.assertEqual([row["sliceId"] for row in matrix["slices"]], ["slice-a", "slice-b"])
            capitol = next(row for row in matrix["namedBuildings"] if row["name"] == "Texas State Capitol")
            self.assertEqual(capitol["sliceIds"], ["slice-a", "slice-b"])
            self.assertEqual(capitol["playerNearbySliceIds"], ["slice-a"])
            self.assertEqual(matrix["namedBuildingsWithPlayerNearbyCoverage"], ["Texas State Capitol"])
            self.assertEqual(matrix["namedBuildingsWithoutPlayerNearbyCoverage"], ["Sam Houston Building"])
            self.assertEqual(sorted(row["roofShape"] for row in matrix["iconicRoofBuildings"]), ["dome", "onion"])
            self.assertTrue((artifact_dir / "landmark-roof-matrix.json").exists())
            self.assertTrue((artifact_dir / "landmark-roof-matrix.md").exists())


if __name__ == "__main__":
    unittest.main()
