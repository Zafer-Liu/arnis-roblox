#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
BUILDING_BUILDER_PATH = (
    ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "BuildingBuilder.lua"
)


class BuildingShellMeshReadabilityContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.text = BUILDING_BUILDER_PATH.read_text(encoding="utf-8")

    def test_merged_shells_add_bounded_roofline_and_perimeter_cues(self) -> None:
        self.assertIn("shouldEmitMergedShellReadableCues", self.text)
        self.assertIn("buildMergedShellReadableCues", self.text)
        self.assertIn("mergedShellCueMs", self.text)
        self.assertIn("buildMergedShellRooflineCues", self.text)
        self.assertIn("buildMergedShellPerimeterCues", self.text)
        self.assertIn("ArnisMergedShellRooflineCueCount", self.text)


if __name__ == "__main__":
    unittest.main()
