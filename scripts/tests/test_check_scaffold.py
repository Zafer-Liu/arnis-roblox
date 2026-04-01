from __future__ import annotations

import subprocess
import sys
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
CHECK_SCAFFOLD = ROOT / "scripts" / "check_scaffold.py"


class CheckScaffoldTests(unittest.TestCase):
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
