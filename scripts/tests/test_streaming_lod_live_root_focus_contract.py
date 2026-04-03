#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
STREAMING_SERVICE_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "StreamingService.lua"


class StreamingLodLiveRootFocusContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.streaming_text = STREAMING_SERVICE_PATH.read_text(encoding="utf-8")

    def test_lod_update_uses_live_player_root_focus_before_last_scheduler_focal_point(self) -> None:
        self.assertIn("local function resolveLivePlayerRootFocusPosition()", self.streaming_text)
        self.assertIn('local rootPart = character:FindFirstChild("HumanoidRootPart")', self.streaming_text)
        self.assertIn("return rootPart.Position", self.streaming_text)
        self.assertIn("local avatarFocusPos = streamingLastFocalPoint", self.streaming_text)
        self.assertIn(
            "local livePlayerRootFocusPos = resolveLivePlayerRootFocusPosition()",
            self.streaming_text,
        )
        self.assertIn(
            'if typeof(livePlayerRootFocusPos) == "Vector3" then',
            self.streaming_text,
        )
        self.assertIn(
            "avatarFocusPos = livePlayerRootFocusPos",
            self.streaming_text,
        )
        self.assertIn(
            "updateChunkEntryLodGroups(chunkEntry, camPos, avatarFocusPos, highDetailRadius, interiorRadius)",
            self.streaming_text,
        )


if __name__ == "__main__":
    unittest.main()
