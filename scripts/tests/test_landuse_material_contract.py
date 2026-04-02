from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
LANDUSE_BUILDER = ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "Builders" / "LanduseBuilder.lua"


class LanduseMaterialContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.text = LANDUSE_BUILDER.read_text(encoding="utf-8")

    def test_builder_carries_richer_outdoor_surface_fallbacks(self) -> None:
        self.assertIn("pitch = Enum.Material.Grass", self.text)
        self.assertIn("golf_course = Enum.Material.LeafyGrass", self.text)
        self.assertIn("education = Enum.Material.Brick", self.text)
        self.assertIn("hospital = Enum.Material.SmoothPlastic", self.text)
        self.assertIn("religious = Enum.Material.Sandstone", self.text)
        self.assertIn("retail = Enum.Material.Limestone", self.text)
        self.assertIn("railway = Enum.Material.Slate", self.text)


if __name__ == "__main__":
    unittest.main()
