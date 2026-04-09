#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
AMBIENT_SOUNDSCAPE_PATH = (
    ROOT / "roblox" / "src" / "StarterPlayer" / "StarterPlayerScripts" / "AmbientSoundscape.client.lua"
)


class AmbientSoundscapeRuntimeContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.text = AMBIENT_SOUNDSCAPE_PATH.read_text(encoding="utf-8")

    def test_ambient_soundscape_caches_tagged_water_and_road_parts_between_updates(self) -> None:
        self.assertIn("local cachedWaterParts = {}", self.text)
        self.assertIn("local cachedRoadParts = {}", self.text)
        self.assertIn('CollectionService:GetTagged("LOD_Detail")', self.text)
        self.assertIn('CollectionService:GetTagged("Road")', self.text)
        self.assertIn('CollectionService:GetInstanceAddedSignal("LOD_Detail")', self.text)
        self.assertIn('CollectionService:GetInstanceRemovedSignal("LOD_Detail")', self.text)
        self.assertIn('CollectionService:GetInstanceAddedSignal("Road")', self.text)
        self.assertIn('CollectionService:GetInstanceRemovedSignal("Road")', self.text)
        self.assertIn("for waterPart in pairs(cachedWaterParts) do", self.text)
        self.assertIn("for part in pairs(cachedRoadParts) do", self.text)
        update_body = self.text.split("local function updateAmbience()", 1)[1].split("-- ---------------------------------------------------------------------------\n-- 3. Surface-aware footstep sounds", 1)[0]
        self.assertNotIn('CollectionService:GetTagged("LOD_Detail")', update_body)
        self.assertNotIn('CollectionService:GetTagged("Road")', update_body)

    def test_ambient_soundscape_throttles_footstep_surface_sampling(self) -> None:
        self.assertIn("local FOOTSTEP_UPDATE_INTERVAL = 0.12", self.text)
        self.assertIn("local footstepUpdateTimer = 0", self.text)
        self.assertIn("footstepUpdateTimer = footstepUpdateTimer + dt", self.text)
        self.assertIn("if footstepUpdateTimer < FOOTSTEP_UPDATE_INTERVAL and lastFootstepMaterial ~= nil then", self.text)
        self.assertIn("Keep the current surface sample until the throttle allows another raycast.", self.text)
        self.assertIn("footstepUpdateTimer = 0", self.text)
        self.assertIn('workspace:Raycast(hrp.Position, Vector3.new(0, -10, 0))', self.text)
        self.assertIn("updateFootsteps(dt)", self.text)
        self.assertIn("footstepSound.PlaybackSpeed = 0.8 + (humanoid.WalkSpeed / 16) * 0.4", self.text)
        self.assertIn("if not footstepSound then", self.text)
        self.assertIn("footstepSound:Destroy()", self.text)


if __name__ == "__main__":
    unittest.main()
