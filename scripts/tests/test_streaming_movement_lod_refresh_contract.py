#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
STREAMING_SERVICE_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "StreamingService.lua"


class StreamingMovementLodRefreshContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.streaming_text = STREAMING_SERVICE_PATH.read_text(encoding="utf-8")

    def test_streaming_forces_faster_lod_refresh_when_live_player_root_moves_meaningfully(self) -> None:
        self.assertIn("local LOD_MOVEMENT_REFRESH_THRESHOLD_STUDS =", self.streaming_text)
        self.assertIn("local function shouldForceMovementLodRefresh()", self.streaming_text)
        self.assertIn("local livePlayerRootFocusPos = resolveLivePlayerRootFocusPosition()", self.streaming_text)
        self.assertIn("if movementDelta.Magnitude < LOD_MOVEMENT_REFRESH_THRESHOLD_STUDS then", self.streaming_text)
        self.assertIn("if shouldForceMovementLodRefresh() then", self.streaming_text)
        self.assertIn("lastLODUpdate = LOD_UPDATE_INTERVAL", self.streaming_text)


if __name__ == "__main__":
    unittest.main()
