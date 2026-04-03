#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
STREAMING_SERVICE_PATH = (
    ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "StreamingService.lua"
)


class StreamingLodFootprintContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.streaming_text = STREAMING_SERVICE_PATH.read_text(encoding="utf-8")

    def test_lod_visibility_uses_footprint_distance_instead_of_anchor_distance(self) -> None:
        self.assertIn("local function getLodGroupFootprintBounds(group, fallbackPosition)", self.streaming_text)
        self.assertIn(
            "local function getLodGroupFootprintDistanceSq(group, fallbackPosition, point)",
            self.streaming_text,
        )
        self.assertIn("local highDetailRadiusSq = highDetailRadius * highDetailRadius", self.streaming_text)
        self.assertIn("local interiorRadiusSq = interiorRadius * interiorRadius", self.streaming_text)
        self.assertRegex(
            self.streaming_text,
            r"local detailVisible = getLodGroupFootprintDistanceSq\(group, chunkCenter, camPos\) <= highDetailRadiusSq",
        )
        self.assertRegex(
            self.streaming_text,
            r"detailVisible = getLodGroupFootprintDistanceSq\(group, chunkCenter, secondaryFocusPos\)\s*<= highDetailRadiusSq",
        )
        self.assertRegex(
            self.streaming_text,
            r"local interiorVisible = getLodGroupFootprintDistanceSq\(group, chunkCenter, camPos\) <= interiorRadiusSq",
        )
        self.assertRegex(
            self.streaming_text,
            r"interiorVisible = getLodGroupFootprintDistanceSq\(group, chunkCenter, secondaryFocusPos\)\s*<= interiorRadiusSq",
        )
        self.assertNotIn(
            "local detailVisible = (getLodGroupAnchor(group, chunkCenter) - camPos).Magnitude <= highDetailRadius",
            self.streaming_text,
        )
        self.assertNotIn(
            "local interiorVisible = (getLodGroupAnchor(group, chunkCenter) - camPos).Magnitude <= interiorRadius",
            self.streaming_text,
        )


if __name__ == "__main__":
    unittest.main()
