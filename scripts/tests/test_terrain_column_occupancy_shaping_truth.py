from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TERRAIN_BUILDER = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "TerrainBuilder.lua"


class TerrainColumnOccupancyShapingTruthTests(unittest.TestCase):
    def test_steep_mixed_voxel_columns_apply_edge_occupancy_tapering(self) -> None:
        source = TERRAIN_BUILDER.read_text(encoding="utf-8")

        self.assertIn("local edgeOccupancyScale =", source)
        self.assertIn("edgeOccupancyScale = edgeOccupancyScale", source)
        self.assertRegex(
            source,
            r"local edgeOccupancyScale = if heightRange > 0\s+then math\.clamp\(1 - heightRangeFactor \* \(1 - peakCoverageBias\) \* 0\.5, 0\.35, 1\)\s+else 1",
        )
        self.assertIn("occupancy = math.min(occupancy, bottomOccupancy * columnProfile.edgeOccupancyScale)", source)
        self.assertIn("occupancy = math.min(occupancy, topOccupancy * columnProfile.edgeOccupancyScale)", source)


if __name__ == "__main__":
    unittest.main()
