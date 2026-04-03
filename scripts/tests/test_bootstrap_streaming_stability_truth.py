#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
BOOTSTRAP_PATH = ROOT / "roblox" / "src" / "ServerScriptService" / "BootstrapAustin.server.lua"


class BootstrapStreamingStabilityTruthTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.bootstrap_text = BOOTSTRAP_PATH.read_text(encoding="utf-8")

    def test_bootstrap_requires_multiple_ready_polls_before_streaming_ready(self) -> None:
        self.assertIn("local STARTUP_STREAMING_REQUIRED_READY_POLLS =", self.bootstrap_text)
        self.assertIn("local readyPollCount = 0", self.bootstrap_text)
        self.assertIn("if startupResidency.ready then", self.bootstrap_text)
        self.assertIn("readyPollCount += 1", self.bootstrap_text)
        self.assertIn("if readyPollCount >= STARTUP_STREAMING_REQUIRED_READY_POLLS then", self.bootstrap_text)
        self.assertIn("readyPollCount = 0", self.bootstrap_text)


if __name__ == "__main__":
    unittest.main()
