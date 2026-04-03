#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
STREAMING_SERVICE_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "StreamingService.lua"


class StreamingImportLodRefreshContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.streaming_text = STREAMING_SERVICE_PATH.read_text(encoding="utf-8")

    def test_streaming_updates_newly_imported_chunk_lod_visibility_immediately(self) -> None:
        self.assertIn("local importedChunkEntry = ChunkLoader.GetChunkEntry(chunkRef.id, streamingOptions.worldRootName)", self.streaming_text)
        self.assertIn("local immediateCameraFocusPos = resolveCurrentCameraFocusPosition()", self.streaming_text)
        self.assertIn("updateChunkEntryLodGroups(", self.streaming_text)
        self.assertIn(
            "importedChunkEntry,",
            self.streaming_text,
        )
        self.assertIn(
            "immediateCameraFocusPos,",
            self.streaming_text,
        )
        self.assertIn(
            "playerPos,",
            self.streaming_text,
        )
        self.assertIn(
            "highRadius,",
            self.streaming_text,
        )
        self.assertIn(
            "interiorRadius",
            self.streaming_text,
        )


if __name__ == "__main__":
    unittest.main()
