#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
STREAMING_SERVICE_PATH = (
    ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "StreamingService.lua"
)


class StreamingStartupPrimeContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.streaming_text = STREAMING_SERVICE_PATH.read_text(encoding="utf-8")

    def test_start_primes_initial_update_before_heartbeat_loop(self) -> None:
        start_index = self.streaming_text.index("function StreamingService.Start(manifest, options)")
        stop_index = self.streaming_text.index("function StreamingService.Stop()")
        start_block = self.streaming_text[start_index:stop_index]

        update_index = start_block.index("StreamingService.Update()")
        heartbeat_index = start_block.index("heartbeatConn = RunService.Heartbeat:Connect")

        self.assertLess(
            update_index,
            heartbeat_index,
            "expected Start() to run an initial streaming update before the heartbeat scheduler takes over",
        )


if __name__ == "__main__":
    unittest.main()
