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

    def test_rate_limited_requestasync_retries_before_skipping_getasync_fallback(self) -> None:
        self.assertIn("local function isChunkFetchRateLimited(errText)", self.text)
        self.assertIn('string.find(errText, "Number of requests exceeded limit", 1, true)', self.text)
        self.assertIn('string.find(errText, "HTTP 429", 1, true)', self.text)
        self.assertIn("local REQUEST_RETRY_DELAYS_SECONDS = { 0.5, 1, 2 }", self.text)
        self.assertIn("for attempt = 1, #REQUEST_RETRY_DELAYS_SECONDS + 1 do", self.text)
        self.assertIn("if not isChunkFetchRateLimited(err) then", self.text)
        self.assertIn("task.wait(REQUEST_RETRY_DELAYS_SECONDS[attempt])", self.text)
        self.assertIn("if isChunkFetchRateLimited(err) then", self.text)
        self.assertIn('warn(("[ManifestLoader] RequestAsync rate limited for chunk %s; skipping immediate GetAsync fallback"):format(', self.text)
        self.assertIn("local fallbackData, fallbackErr = fetchChunkViaGet(chunkId)", self.text)
        rate_limit_guard_index = self.text.index("if isChunkFetchRateLimited(err) then")
        fallback_index = self.text.index("local fallbackData, fallbackErr = fetchChunkViaGet(chunkId)")
        self.assertLess(rate_limit_guard_index, fallback_index)

    def test_chunk_fetch_url_inserts_filename_before_query_suffix(self) -> None:
        self.assertIn("local function buildChunkFetchUrl(chunkBaseUrl, chunkId)", self.text)
        self.assertIn('local queryStart = string.find(chunkBaseUrl, "?", 1, true)', self.text)
        self.assertIn("local basePath = string.sub(chunkBaseUrl, 1, queryStart - 1)", self.text)
        self.assertIn("local querySuffix = string.sub(chunkBaseUrl, queryStart)", self.text)
        self.assertIn('return basePath .. chunkId .. ".json" .. querySuffix', self.text)
        self.assertIn('return chunkBaseUrl .. chunkId .. ".json"', self.text)


if __name__ == "__main__":
    unittest.main()
