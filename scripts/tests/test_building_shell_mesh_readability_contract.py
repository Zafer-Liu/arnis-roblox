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
        self.assertNotIn('if roofShape == "flat" or not PLAY_VISIBLE_SHELL_ROOF_SHAPES[roofShape] then', self.text)
        self.assertIn("if not PLAY_VISIBLE_SHELL_ROOF_SHAPES[roofShape] then", self.text)

    def test_merged_shells_add_bounded_street_facade_cues(self) -> None:
        self.assertIn("local function getMergedShellStreetFacadeY", self.text)
        self.assertIn("local function buildMergedShellStreetFacadeCues", self.text)
        self.assertIn("MergedShellStreetFacadeCue", self.text)
        self.assertIn("ArnisMergedShellStreetFacadeCueCount", self.text)

    def test_merged_shells_add_a_street_facing_door_cue(self) -> None:
        self.assertIn("collectSimpleShellReadableEdges", self.text)
        self.assertIn("local function buildMergedShellDoorCue", self.text)
        self.assertIn("MergedShellDoorCue", self.text)
        self.assertIn("ArnisMergedShellDoorCueCount", self.text)

    def test_merged_shells_add_bounded_window_pane_cues(self) -> None:
        self.assertIn("local function buildMergedShellWindowPaneCues", self.text)
        self.assertIn("MergedShellWindowPaneCue", self.text)
        self.assertIn("ArnisMergedShellWindowPaneCueCount", self.text)

    def test_play_visible_shell_wall_path_adds_roofline_and_beltline_readability_cues(self) -> None:
        self.assertIn("local function buildPlayVisibleShellReadableCues", self.text)
        self.assertIn("playVisibleFacadeBeltlineCount", self.text)
        self.assertIn("playVisibleRooflineCueCount", self.text)
        self.assertIn("playVisibleCornerAccentCount", self.text)
        self.assertIn("playVisibleDoorCueCount", self.text)
        self.assertIn("playVisibleWindowPaneCueCount", self.text)


if __name__ == "__main__":
    unittest.main()
