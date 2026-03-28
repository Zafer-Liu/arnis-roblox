from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]

DOC_EXPECTATIONS = {
    ROOT / "docs" / "vertigo-sync-boundary.md": [
        "canonical world truth, manifest semantics, and scene extraction adapters",
        "edit/full-bake orchestration and export-3d user-facing orchestration",
    ],
    ROOT / "AGENTS.md": [
        "canonical world truth, manifest semantics, and scene extraction adapters",
        "edit/full-bake orchestration and export-3d",
    ],
    ROOT / "CLAUDE.md": [
        "canonical world truth, manifest semantics, and scene extraction adapters",
        "edit/full-bake orchestration and export-3d",
    ],
}

BASELINE_STATUS_PATH = ROOT / "docs" / "superpowers" / "status" / "2026-03-28-canonical-baseline-status.md"
HISTORICAL_PLAN_EXPECTATIONS = {
    ROOT / "docs" / "superpowers" / "plans" / "2026-03-26-play-preview-convergence.md": "Status: Historical",
    ROOT
    / "docs"
    / "superpowers"
    / "plans"
    / "2026-03-26-play-preview-export-convergence-implementation.md": "Status: Historical",
}

ENTRYPOINT_RULES = {
    ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "RunAustin.lua": {
        "required": [
            "CanonicalWorldContract.resolveCanonicalManifestFamily()",
            'CanonicalWorldContract.resolveCanonicalMaterializationCandidates("play")',
            'CanonicalWorldContract.resolveCanonicalMaterializationFamily("play")',
            'CanonicalWorldContract.loadCanonicalManifestSource("play"',
        ],
        "forbidden": [
            "ManifestLoader.LoadShardedModuleHandle",
            "AustinPreviewManifestIndex",
            "AustinPreviewManifestChunks",
            "AustinPlayManifestIndex",
            "AustinPlayManifestChunks",
            "AustinFullBakeManifestIndex",
            "AustinFullBakeManifestChunks",
            "export-3d",
            "scene_ir",
            "glb",
            "fbx",
        ],
    },
    ROOT / "roblox" / "src" / "ServerScriptService" / "BootstrapAustin.server.lua": {
        "required": [
            "RunAustin.run({",
            "AustinSpawn.resolveRuntimeAnchor",
        ],
        "forbidden": [
            "ManifestLoader.LoadShardedModuleHandle",
            "AustinPreviewManifestIndex",
            "AustinPreviewManifestChunks",
            "AustinPlayManifestIndex",
            "AustinPlayManifestChunks",
            "AustinFullBakeManifestIndex",
            "AustinFullBakeManifestChunks",
            "export-3d",
            "scene_ir",
            "glb",
            "fbx",
        ],
    },
    ROOT / "roblox" / "src" / "ServerScriptService" / "ImportService" / "AustinSpawn.lua": {
        "required": [
            "function AustinSpawn.resolveAnchor",
            "function AustinSpawn.resolveRuntimeAnchor",
        ],
        "forbidden": [
            "ManifestLoader.LoadShardedModuleHandle",
            "AustinPreviewManifestIndex",
            "AustinPreviewManifestChunks",
            "AustinPlayManifestIndex",
            "AustinPlayManifestChunks",
            "AustinFullBakeManifestIndex",
            "AustinFullBakeManifestChunks",
            "export-3d",
            "scene_ir",
            "glb",
            "fbx",
        ],
    },
    ROOT / "roblox" / "src" / "ServerScriptService" / "StudioPreview" / "AustinPreviewBuilder.lua": {
        "required": [
            'CanonicalWorldContract.loadCanonicalManifestSource("preview"',
            'CanonicalWorldContract.loadCanonicalManifestSource("full_bake"',
        ],
        "forbidden": [
            "ManifestLoader.LoadShardedModuleHandle",
            "AustinPreviewManifestIndex",
            "AustinPreviewManifestChunks",
            "AustinPlayManifestIndex",
            "AustinPlayManifestChunks",
            "AustinFullBakeManifestIndex",
            "AustinFullBakeManifestChunks",
            "export-3d",
            "scene_ir",
            "glb",
            "fbx",
        ],
    },
}


class ConvergenceGuardrailTests(unittest.TestCase):
    maxDiff = None

    def test_boundary_docs_spell_out_the_ownership_split(self) -> None:
        for path, required_snippets in DOC_EXPECTATIONS.items():
            with self.subTest(path=path):
                text = path.read_text(encoding="utf-8")
                for snippet in required_snippets:
                    self.assertIn(snippet, text, f"{path} is missing required guardrail text: {snippet}")

    def test_expected_entrypoints_do_not_gain_parallel_world_definition_paths(self) -> None:
        for path, rule in ENTRYPOINT_RULES.items():
            with self.subTest(path=path):
                text = path.read_text(encoding="utf-8")
                for snippet in rule["required"]:
                    self.assertIn(snippet, text, f"{path} is missing required canonical path text: {snippet}")
                for snippet in rule["forbidden"]:
                    self.assertNotIn(
                        snippet,
                        text,
                        f"{path} must not introduce a parallel world-definition/export path: {snippet}",
                    )

    def test_baseline_status_trail_exists(self) -> None:
        self.assertTrue(
            BASELINE_STATUS_PATH.exists(),
            f"expected baseline status trail to exist at {BASELINE_STATUS_PATH}",
        )

    def test_stale_convergence_plans_are_marked_historical(self) -> None:
        for path, marker in HISTORICAL_PLAN_EXPECTATIONS.items():
            with self.subTest(path=path):
                text = path.read_text(encoding="utf-8")
                self.assertIn(marker, text, f"{path} must be marked historical to avoid stale handoff drift")


if __name__ == "__main__":
    unittest.main()
