from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TERRAIN_BUILDER = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "TerrainBuilder.lua"


class TerrainChunkEdgeTruthTests(unittest.TestCase):
    def test_subsampled_edge_voxels_use_unclamped_neighbor_aware_cell_coordinates(self) -> None:
        source = TERRAIN_BUILDER.read_text(encoding="utf-8")

        self.assertIn("local rawSampleCellX = math.floor((sampleWorldX - plan.origin.x) / plan.cellSize)", source)
        self.assertIn("local rawSampleCellZ = math.floor((sampleWorldZ - plan.origin.z) / plan.cellSize)", source)
        self.assertIn("local sampleCellX = rawSampleCellX", source)
        self.assertIn("local sampleCellZ = rawSampleCellZ", source)
        self.assertIn("local function resolveOffsetNeighborIndex(cellIndex, localCount, neighborCount, isPositiveDirection)", source)
        self.assertIn("resolveOffsetNeighborIndex(cellX, plan.gridW, neighborTerrain.width or 0, true)", source)
        self.assertIn("resolveOffsetNeighborIndex(cellX, plan.gridW, neighborTerrain.width or 0, false)", source)
        self.assertNotIn(
            "math.max(0, math.min(plan.gridW - 1, math.floor((sampleWorldX - plan.origin.x) / plan.cellSize)))",
            source,
        )
        self.assertNotIn(
            "math.max(0, math.min(plan.gridD - 1, math.floor((sampleWorldZ - plan.origin.z) / plan.cellSize)))",
            source,
        )

    def test_peak_surface_bias_scales_with_peak_coverage_instead_of_snapping_every_steep_voxel_to_the_max(self) -> None:
        source = TERRAIN_BUILDER.read_text(encoding="utf-8")

        self.assertIn("local peakSampleCount = 0", source)
        self.assertIn("local peakSampleCoverage = peakSampleCount / sampleCount", source)
        self.assertIn("local heightRangeFactor = math.clamp(heightRange / TERRAIN_WRITE_RESOLUTION, 0, 1)", source)
        self.assertIn("local peakCoverageBias = math.max(peakSampleCoverage, normalizedPeakCoverage)", source)
        self.assertIn(
            "local surfaceHeightBias = heightRangeFactor * peakCoverageBias",
            source,
        )
        self.assertRegex(
            source,
            r"surfaceFillDepth = if heightRange > 0\s+then math\.max\(1, TERRAIN_THICKNESS \* math\.clamp\(normalizedPeakCoverage \+ peakCoverageBias \* 0\.25, 0, 1\)\)",
        )


if __name__ == "__main__":
    unittest.main()
