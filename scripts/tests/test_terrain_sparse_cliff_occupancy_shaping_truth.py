from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TERRAIN_BUILDER = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "TerrainBuilder.lua"


class TerrainSparseCliffOccupancyShapingTruthTests(unittest.TestCase):
    def test_sparse_cliff_voxel_columns_compute_a_bounded_uniform_occupancy_scale(self) -> None:
        source = TERRAIN_BUILDER.read_text(encoding="utf-8")

        self.assertIn("local sparseCliffOccupancyScale =", source)
        self.assertIn("local sparseCliffCoverageBias = sparsePeakCoverageDamping * sparsePeakCoverageDamping", source)
        self.assertRegex(
            source,
            r"local sparseCliffOccupancyScale = if heightRange > 0\s+then math\.clamp\(edgeOccupancyScale \* sparseCliffCoverageBias, 0\.2, 1\)\s+else 1",
        )
        self.assertIn("sparseCliffOccupancyScale = sparseCliffOccupancyScale", source)

    def test_sparse_cliff_voxel_columns_apply_the_uniform_occupancy_scale_to_all_writes(self) -> None:
        source = TERRAIN_BUILDER.read_text(encoding="utf-8")

        self.assertIn("local occupancy = columnProfile.sparseCliffOccupancyScale", source)
        self.assertIn("occupancy = math.min(occupancy, bottomOccupancy * columnProfile.edgeOccupancyScale)", source)
        self.assertIn("occupancy = math.min(occupancy, topOccupancy * columnProfile.edgeOccupancyScale)", source)


if __name__ == "__main__":
    unittest.main()
