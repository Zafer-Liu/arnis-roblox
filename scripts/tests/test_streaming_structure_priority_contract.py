#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
STREAMING_SERVICE_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "StreamingService.lua"
CHUNK_PRIORITY_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "ChunkPriority.lua"


class StreamingStructurePriorityContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.streaming_text = STREAMING_SERVICE_PATH.read_text(encoding="utf-8")
        cls.chunk_priority_text = CHUNK_PRIORITY_PATH.read_text(encoding="utf-8")

    def test_high_detail_building_work_items_carry_structure_priority(self) -> None:
        self.assertIn("local function shouldPrioritizeHighDetailStructures(chunkRef, targetLod)", self.streaming_text)
        self.assertIn(
            "highDetailStructurePriority = wholeChunkOptions.highDetailStructurePriority == true,",
            self.streaming_text,
        )
        self.assertIn(
            "highDetailStructurePriority = targetLod == LOD_HIGH",
            self.streaming_text,
        )
        self.assertIn(
            'and subplan.layer == "buildings",',
            self.streaming_text,
        )

    def test_chunk_priority_sorts_structural_work_ahead_of_same_band_non_structural_work(self) -> None:
        self.assertIn(
            'highDetailStructurePriority = type(workItem) == "table" and workItem.highDetailStructurePriority == true,',
            self.chunk_priority_text,
        )
        self.assertIn(
            "if leftKey.highDetailStructurePriority ~= rightKey.highDetailStructurePriority then",
            self.chunk_priority_text,
        )
        self.assertIn("return leftKey.highDetailStructurePriority == true", self.chunk_priority_text)


if __name__ == "__main__":
    unittest.main()
