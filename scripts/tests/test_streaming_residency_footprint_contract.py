#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
STREAMING_SERVICE_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "StreamingService.lua"
CHUNK_PRIORITY_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "ChunkPriority.lua"


class StreamingResidencyFootprintContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.streaming_text = STREAMING_SERVICE_PATH.read_text(encoding="utf-8")
        cls.chunk_priority_text = CHUNK_PRIORITY_PATH.read_text(encoding="utf-8")

    def test_streaming_residency_uses_footprint_distance_for_near_player_admission(self) -> None:
        self.assertIn("local function getChunkFootprintBounds(chunkLikeOrEntry)", self.chunk_priority_text)
        self.assertIn("local function getChunkFootprintDistanceSq(chunkLikeOrEntry, point, chunkSizeStuds)", self.chunk_priority_text)
        self.assertIn("function ChunkPriority.GetChunkFootprintDistanceSq(chunkLikeOrEntry, point, chunkSizeStuds)", self.chunk_priority_text)
        self.assertIn("local chunkFootprintBounds = ChunkPriority.GetChunkFootprintBounds(chunkRef)", self.streaming_text)
        self.assertIn("local function getChunkEntryCellRange(chunkEntry, cellSize)", self.streaming_text)
        self.assertIn("local footprintBounds = ChunkPriority.GetChunkFootprintBounds(chunkRef)", self.streaming_text)
        self.assertIn("getIndexCoord(footprintBounds.minX, cellSize)", self.streaming_text)
        self.assertIn("getIndexCoord(footprintBounds.maxY, cellSize)", self.streaming_text)
        self.assertIn("for cellX = minCellX, maxCellX do", self.streaming_text)
        self.assertIn("for cellZ = minCellZ, maxCellZ do", self.streaming_text)
        self.assertIn("local seenChunkIds = {}", self.streaming_text)
        self.assertIn(
            "local chunkId = type(chunkEntry) == \"table\" and chunkEntry.ref and chunkEntry.ref.id or nil",
            self.streaming_text,
        )
        self.assertIn(
            "if type(chunkId) == \"string\" and not seenChunkIds[chunkId] then",
            self.streaming_text,
        )
        self.assertIn("ChunkPriority.GetChunkFootprintDistanceSq(chunkRef, playerPos, streamingOptions.config.ChunkSizeStuds)", self.streaming_text)
        self.assertIn("local actualDistSq = getChunkDistanceSqToPoint(chunkEntry, playerPos)", self.streaming_text)
        self.assertIn("if chunkFootprintDistanceSq > targetExitRadiusSq then", self.streaming_text)
        self.assertIn("local ringName = getChunkRingName(chunkFootprintDistanceSq, resolvedRings)", self.streaming_text)


if __name__ == "__main__":
    unittest.main()
