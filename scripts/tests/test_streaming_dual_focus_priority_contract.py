#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
STREAMING_SERVICE_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "StreamingService.lua"
CHUNK_PRIORITY_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "ChunkPriority.lua"


class StreamingDualFocusPriorityContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.streaming_text = STREAMING_SERVICE_PATH.read_text(encoding="utf-8")
        cls.chunk_priority_text = CHUNK_PRIORITY_PATH.read_text(encoding="utf-8")

    def test_streaming_priority_preserves_near_player_sorting_during_lookahead(self) -> None:
        self.assertIn("secondaryFocusPoint", self.chunk_priority_text)
        self.assertIn("local secondaryDistSq = nil", self.chunk_priority_text)
        self.assertIn("distSq = math.min(distSq, secondaryDistSq)", self.chunk_priority_text)
        self.assertIn(
            "ChunkPriority.SortChunkEntriesByPriority(",
            self.streaming_text,
        )
        self.assertIn(
            "schedulerFocusPoint,\n            playerPos,\n            chunkSizeStuds,",
            self.streaming_text,
        )
        self.assertIn(
            "schedulerFocusPoint,\n            playerPos,\n            chunkSizeStuds,\n            forwardVector,",
            self.streaming_text,
        )

    def test_streaming_priority_uses_live_avatar_motion_when_no_previous_focal_point_exists(self) -> None:
        self.assertIn("local LIVE_PLAYER_ROOT_MOTION_THRESHOLD = 4", self.streaming_text)
        self.assertIn("resolveLivePlayerRootMotion = function()", self.streaming_text)
        self.assertIn("rootPart.AssemblyLinearVelocity", self.streaming_text)
        self.assertIn("movementDeltaStuds < 1", self.streaming_text)
        self.assertIn("movementLookaheadStuds = math.min(maxLookaheadStuds, liveMotionSpeed * lookaheadSeconds)", self.streaming_text)
        self.assertIn("movementForward = liveMotionForward", self.streaming_text)


if __name__ == "__main__":
    unittest.main()
