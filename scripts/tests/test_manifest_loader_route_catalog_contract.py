#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MANIFEST_LOADER_PATH = (
    ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "ManifestLoader.lua"
)


class ManifestLoaderRouteCatalogContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.text = MANIFEST_LOADER_PATH.read_text(encoding="utf-8")

    def test_manifest_loader_exposes_route_catalog_loader_surface(self) -> None:
        required_snippets = [
            "function ManifestLoader.ResolveModuleByPath",
            "function ManifestLoader.LoadRouteCatalogFromModule",
            "function ManifestLoader.LoadNamedRouteCatalog",
            "function ManifestLoader.LoadRouteSessionFromCatalogModule",
            "function ManifestLoader.LoadHydratedRouteFromCatalogModule",
            "function ManifestLoader.LoadRouteScheduleFromCatalogModule",
            "function ManifestLoader.LoadRouteLaneManifestFromCatalogModule",
            "function ManifestLoader.LoadRouteLaneRuntimeHandleFromCatalogModule",
        ]
        for snippet in required_snippets:
            with self.subTest(snippet=snippet):
                self.assertIn(snippet, self.text)

    def test_manifest_loader_route_catalog_consumes_runtime_native_metadata(self) -> None:
        required_snippets = [
            "route_session_module_path",
            "hydrated_route_module_path",
            "schedule_module_path",
            "manifest_module_path",
            "runtime_index_module_path",
        ]
        for snippet in required_snippets:
            with self.subTest(snippet=snippet):
                self.assertIn(snippet, self.text)


if __name__ == "__main__":
    unittest.main()
