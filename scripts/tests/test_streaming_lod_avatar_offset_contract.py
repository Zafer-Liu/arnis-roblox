#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
STREAMING_SERVICE_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "StreamingService.lua"


class StreamingLodAvatarOffsetContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.streaming_text = STREAMING_SERVICE_PATH.read_text(encoding="utf-8")

    def test_streaming_lod_visibility_uses_camera_or_live_avatar_focus(self) -> None:
        self.assertIn("local avatarFocusPos = streamingLastFocalPoint", self.streaming_text)
        self.assertIn("local function resolveLivePlayerRootFocusPosition()", self.streaming_text)
        self.assertIn("local livePlayerRootFocusPos = resolveLivePlayerRootFocusPosition()", self.streaming_text)
        self.assertIn("if typeof(livePlayerRootFocusPos) == \"Vector3\" then", self.streaming_text)
        self.assertIn("avatarFocusPos = livePlayerRootFocusPos", self.streaming_text)
        self.assertIn("local cameraFocusPos = primaryFocusPos", self.streaming_text)
        self.assertIn("local avatarFocusPos = secondaryFocusPos", self.streaming_text)
        self.assertIn(
            "local detailVisible = isLodGroupVisibleForFocus(group, chunkCenter, cameraFocusPos, highDetailRadiusSq)",
            self.streaming_text,
        )
        self.assertIn(
            "detailVisible = isLodGroupVisibleForFocus(group, chunkCenter, avatarFocusPos, highDetailRadiusSq)",
            self.streaming_text,
        )
        self.assertIn(
            "local interiorVisible = isLodGroupVisibleForFocus(group, chunkCenter, avatarFocusPos, interiorRadiusSq)",
            self.streaming_text,
        )
        self.assertIn(
            "updateChunkEntryLodGroups(chunkEntry, immediateCameraFocusPos, playerPos, highRadius, interiorRadius)",
            self.streaming_text,
        )
        self.assertIn(
            "updateChunkEntryLodGroups(chunkEntry, camPos, avatarFocusPos, highDetailRadius, interiorRadius)",
            self.streaming_text,
        )


if __name__ == "__main__":
    unittest.main()
