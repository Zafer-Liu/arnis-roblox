from __future__ import annotations

import unittest
from pathlib import Path
import re


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
SUPERPOWERS_ROOT = ROOT / "docs" / "superpowers"
ARCHIVE_INDEX_PATH = ROOT / "docs" / "superpowers" / "archive-index.md"
ACTIVE_TRUTH_STACK = {
    "specs": ROOT / "docs" / "superpowers" / "specs" / "2026-04-06-planetary-realism-sprint-design.md",
    "plans": ROOT / "docs" / "superpowers" / "plans" / "2026-04-06-planetary-realism-sprint.md",
    "status": ROOT / "docs" / "superpowers" / "status" / "2026-04-06-planetary-realism-sprint-status.md",
}
ALLOWED_STATUSES = {"Active", "Historical", "Completed"}
RETAINED_SUPERPOWERS_DOCS = {
    ACTIVE_TRUTH_STACK["specs"],
    ACTIVE_TRUTH_STACK["plans"],
    ACTIVE_TRUTH_STACK["status"],
    BASELINE_STATUS_PATH,
    # Completed outdoor fidelity tranche (historical context)
    ROOT / "docs" / "superpowers" / "specs" / "2026-03-30-outdoor-fidelity-and-source-truth-design.md",
    ROOT / "docs" / "superpowers" / "plans" / "2026-03-30-outdoor-fidelity-and-source-truth.md",
    ROOT / "docs" / "superpowers" / "status" / "2026-03-30-outdoor-fidelity-and-source-truth-status.md",
}
ARCHIVE_INDEX_EXPECTATIONS = [
    "2026-03-28-play-fidelity-and-observability",
    "2026-03-29-breaking-compatibility-purge",
    "2026-03-26-play-preview-convergence",
]


def read_top_level_status(path: Path) -> str | None:
    lines = path.read_text(encoding="utf-8").splitlines()[:12]
    for line in lines:
        match = re.match(r"^Status:\s+(\S.+?)\s*$", line)
        if match:
            return match.group(1)
    return None

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

    def test_all_superpowers_docs_have_supported_top_level_status(self) -> None:
        for section in ("specs", "plans", "status"):
            for path in sorted((SUPERPOWERS_ROOT / section).glob("*.md")):
                with self.subTest(section=section, path=path):
                    status = read_top_level_status(path)
                    self.assertIsNotNone(status, f"{path} must declare a top-level Status marker")
                    self.assertIn(status, ALLOWED_STATUSES, f"{path} has unsupported status: {status}")

    def test_superpowers_doc_retention_set_is_compact(self) -> None:
        actual_docs: set[Path] = set()
        for section in ("specs", "plans", "status"):
            actual_docs.update((SUPERPOWERS_ROOT / section).glob("*.md"))
        self.assertEqual(actual_docs, RETAINED_SUPERPOWERS_DOCS)

    def test_superpowers_truth_stack_has_exactly_one_active_file_per_section(self) -> None:
        for section, expected_path in ACTIVE_TRUTH_STACK.items():
            active_paths: list[Path] = []
            for path in sorted((SUPERPOWERS_ROOT / section).glob("*.md")):
                if read_top_level_status(path) == "Active":
                    active_paths.append(path)
            with self.subTest(section=section):
                self.assertEqual(active_paths, [expected_path], f"{section} must have exactly one active truth surface")

    def test_active_status_file_links_only_active_plan_and_spec(self) -> None:
        status_path = ACTIVE_TRUTH_STACK["status"]
        text = status_path.read_text(encoding="utf-8")
        self.assertIn(str(ACTIVE_TRUTH_STACK["specs"].relative_to(ROOT)), text)
        self.assertIn(str(ACTIVE_TRUTH_STACK["plans"].relative_to(ROOT)), text)

    def test_archive_index_exists_and_points_back_to_deleted_workstreams(self) -> None:
        self.assertTrue(ARCHIVE_INDEX_PATH.exists(), f"expected archive index to exist at {ARCHIVE_INDEX_PATH}")
        text = ARCHIVE_INDEX_PATH.read_text(encoding="utf-8")
        for expected_snippet in ARCHIVE_INDEX_EXPECTATIONS:
            with self.subTest(snippet=expected_snippet):
                self.assertIn(expected_snippet, text)


if __name__ == "__main__":
    unittest.main()
