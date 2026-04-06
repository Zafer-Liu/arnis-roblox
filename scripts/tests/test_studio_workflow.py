from __future__ import annotations

import importlib.util
import os
import subprocess
from pathlib import Path
import unittest
from unittest import mock


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "studio_workflow.py"


def load_module():
    spec = importlib.util.spec_from_file_location("studio_workflow", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load module spec from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class StudioWorkflowTests(unittest.TestCase):
    def test_run_control_uses_bounded_timeout_and_forwards_ui_timeout_env(self) -> None:
        module = load_module()
        with mock.patch.dict(
            os.environ,
            {"ARNIS_STUDIO_WORKFLOW_CONTROL_TIMEOUT_SECONDS": "9"},
            clear=False,
        ):
            controller = module.StudioWorkflowController()

        with mock.patch.object(module.subprocess, "run") as run_mock:
            run_mock.return_value = subprocess.CompletedProcess(["python3"], 0, "{}", "")
            controller._run_control("get-state")

        run_mock.assert_called_once()
        _, kwargs = run_mock.call_args
        self.assertEqual(kwargs["timeout"], 10)
        self.assertEqual(kwargs["env"]["ARNIS_STUDIO_UI_CONTROL_TIMEOUT_SECONDS"], "9")

    def test_run_control_converts_timeout_expired_into_completed_process(self) -> None:
        module = load_module()
        controller = module.StudioWorkflowController()

        with mock.patch.object(
            module.subprocess,
            "run",
            side_effect=subprocess.TimeoutExpired(["python3"], 9),
        ):
            result = controller._run_control("get-state")

        self.assertEqual(result.returncode, 124)
        self.assertIn("timed out", result.stderr)


if __name__ == "__main__":
    unittest.main()
