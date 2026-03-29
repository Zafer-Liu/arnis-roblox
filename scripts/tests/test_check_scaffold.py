from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
CHECK_SCAFFOLD = ROOT / "scripts" / "check_scaffold.py"


class CheckScaffoldTests(unittest.TestCase):
    def test_generated_fixture_directory_keeps_only_a_0_4_0_example(self) -> None:
        generated_dir = ROOT / "specs" / "generated"
        json_files = sorted(path.name for path in generated_dir.glob("*.json"))

        self.assertEqual(json_files, ["sample-manifest.json"])

        manifest = json.loads((generated_dir / "sample-manifest.json").read_text(encoding="utf-8"))
        self.assertEqual(manifest.get("schemaVersion"), "0.4.0")
        self.assertIn("meta", manifest)
        self.assertIn("chunks", manifest)

    def test_sample_manifest_is_locked_to_schema_040(self) -> None:
        result = subprocess.run(
            [sys.executable, str(CHECK_SCAFFOLD)],
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(
            result.returncode,
            0,
            msg=f"check_scaffold.py failed unexpectedly\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}",
        )


if __name__ == "__main__":
    unittest.main()
