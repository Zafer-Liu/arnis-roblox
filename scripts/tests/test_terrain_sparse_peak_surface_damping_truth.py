from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TERRAIN_BUILDER = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "TerrainBuilder.lua"


class TerrainSparsePeakSurfaceDampingTruthTests(unittest.TestCase):
    def test_sparse_steep_peak_surface_bias_uses_bounded_coverage_damping(self) -> None:
        source = TERRAIN_BUILDER.read_text(encoding="utf-8")

        self.assertIn("local surfaceHeightCoverageDamping =", source)
        self.assertRegex(
            source,
            r"local surfaceHeightCoverageDamping = math\.clamp\(0\.5 \+ peakCoverageBias \* 2, 0\.5, 1\)",
        )
        self.assertIn(
            "local surfaceHeightBias = heightRangeFactor * peakCoverageBias",
            source,
        )
        self.assertIn(
            "surfaceHeightBias = surfaceHeightBias",
            source,
        )
        self.assertIn(
            "* surfaceHeightCoverageDamping",
            source,
        )

    def test_sparse_steep_peak_surface_bias_applies_extra_sparse_coverage_damping(self) -> None:
        source = TERRAIN_BUILDER.read_text(encoding="utf-8")

        self.assertIn("local sparsePeakCoverageDamping =", source)
        self.assertRegex(
            source,
            r"local sparsePeakCoverageDamping = math\.clamp\(peakSampleCoverage \* 8, 0\.5, 1\)",
        )
        self.assertIn(
            "surfaceHeightBias = surfaceHeightBias",
            source,
        )
        self.assertIn(
            "* surfaceHeightCoverageDamping",
            source,
        )
        self.assertIn(
            "* sparsePeakCoverageDamping",
            source,
        )

    def test_sparse_steep_peak_surface_bias_applies_isolated_peak_support_damping(self) -> None:
        source = TERRAIN_BUILDER.read_text(encoding="utf-8")

        self.assertIn("local isolatedPeakSupportDamping =", source)
        self.assertRegex(
            source,
            r"local isolatedPeakSupportDamping = math\.clamp\(0\.25 \+ normalizedPeakCoverage \* 4, 0\.25, 1\)",
        )
        self.assertIn(
            "* isolatedPeakSupportDamping",
            source,
        )


if __name__ == "__main__":
    unittest.main()
