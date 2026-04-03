from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TERRAIN_BUILDER = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "TerrainBuilder.lua"


class TerrainSteepMixedFillDepthTruthTests(unittest.TestCase):
    def test_steep_mixed_voxels_apply_a_secondary_shallow_fill_depth_cap(self) -> None:
        source = TERRAIN_BUILDER.read_text(encoding="utf-8")

        self.assertIn("local ridgeCoverageBias = peakCoverageBias * peakCoverageBias", source)
        self.assertRegex(
            source,
            r"local ridgeFillDepth =\s*math\.max\(\s*1,\s*TERRAIN_THICKNESS \* math\.clamp\(normalizedPeakCoverage \* 0\.5 \+ ridgeCoverageBias \* 0\.5, 0, 1\)\s*\)",
        )
        self.assertIn("surfaceFillDepth = math.min(surfaceFillDepth, ridgeFillDepth)", source)


if __name__ == "__main__":
    unittest.main()
