#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MANIFEST_LOADER_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "ManifestLoader.lua"


class ManifestLoaderRuntimeContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.text = MANIFEST_LOADER_PATH.read_text(encoding="utf-8")

    def test_parallel_chunk_fetch_uses_bounded_worker_pool_instead_of_one_task_per_chunk(self) -> None:
        self.assertIn("local MAX_PARALLEL_CHUNK_REQUESTS = 4", self.text)
        self.assertIn("local nextIndex = 1", self.text)
        self.assertIn("local workerCount = math.min(count, MAX_PARALLEL_CHUNK_REQUESTS)", self.text)
        self.assertIn("for _ = 1, workerCount do", self.text)
        self.assertIn("task.spawn(function()", self.text)
        self.assertIn("local currentIndex = nextIndex", self.text)
        self.assertIn("nextIndex += 1", self.text)
        self.assertNotIn("for _, id in ipairs(idsToFetch) do\n                    local capturedId = id\n                    task.spawn(function()", self.text)

    def test_parallel_chunk_fetch_logs_bounded_inflight_request_count(self) -> None:
        self.assertIn('("Streaming %d chunks from Cloudflare worker..."):format(count)', self.text)
        self.assertIn('("Streaming chunks from Cloudflare (%d/%d)..."):format(completed, count)', self.text)
        self.assertIn("parallel, %d req in flight", self.text)
        self.assertIn("workerCount", self.text)


if __name__ == "__main__":
    unittest.main()
