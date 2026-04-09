from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "gui_session_capture.py"


def load_module():
    spec = importlib.util.spec_from_file_location("gui_session_capture", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load module spec from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class GuiSessionCaptureTests(unittest.TestCase):
    def test_returns_failure_when_open_command_missing(self) -> None:
        mod = load_module()
        with tempfile.TemporaryDirectory() as temp_dir:
            target = Path(temp_dir) / "shot.png"
            metadata_path = target.with_suffix(".capture.json")
            root = Path(temp_dir)
            with mock.patch.object(mod.shutil, "which", return_value=None):
                exit_code, payload = mod.capture_via_gui_terminal(target, root, 5)
            self.assertEqual(exit_code, 1)
            self.assertEqual(payload["blocker_reason"], "gui_session_unavailable")
            metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
            self.assertEqual(metadata["guiSessionRelay"]["status"], "open_missing")

    def test_launches_terminal_command_and_returns_capture_payload(self) -> None:
        mod = load_module()
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            target = root / "shot.png"
            metadata_path = target.with_suffix(".capture.json")
            helper_path = root / "scripts" / "studio_ui_control.py"
            helper_path.parent.mkdir(parents=True, exist_ok=True)
            helper_path.write_text("#!/usr/bin/env python3\n", encoding="utf-8")
            target.write_bytes(b"png")
            metadata_path.write_text(
                json.dumps(
                    {
                        "success": True,
                        "target": str(target),
                        "target_exists": True,
                        "capture_method": "window",
                        "blocker_reason": None,
                    }
                ),
                encoding="utf-8",
            )

            original_exists = Path.exists
            original_read_text = Path.read_text

            def fake_exists(path_obj: Path):
                if path_obj.name == "exit.code":
                    return True
                return original_exists(path_obj)

            def fake_read_text(path_obj: Path, *args, **kwargs):
                if path_obj.name == "exit.code":
                    return "0\n"
                return original_read_text(path_obj, *args, **kwargs)

            with (
                mock.patch.object(mod.shutil, "which", return_value="/usr/bin/open"),
                mock.patch.object(
                    mod.subprocess,
                    "run",
                    return_value=subprocess.CompletedProcess(["open"], 0, "", ""),
                ),
                mock.patch.object(Path, "exists", fake_exists),
                mock.patch.object(Path, "read_text", fake_read_text),
            ):
                exit_code, payload = mod.capture_via_gui_terminal(target, root, 5)

            self.assertEqual(exit_code, 0)
            self.assertTrue(payload["success"])
            self.assertEqual(payload["capture_method"], "window")
            metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
            self.assertEqual(metadata["guiSessionRelay"]["status"], "ok")
            self.assertEqual(metadata["guiSessionRelay"]["method"], "terminal.command")

    def test_records_launch_failure_in_metadata(self) -> None:
        mod = load_module()
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            target = root / "shot.png"
            metadata_path = target.with_suffix(".capture.json")
            helper_path = root / "scripts" / "studio_ui_control.py"
            helper_path.parent.mkdir(parents=True, exist_ok=True)
            helper_path.write_text("#!/usr/bin/env python3\n", encoding="utf-8")
            with (
                mock.patch.object(mod.shutil, "which", return_value="/usr/bin/open"),
                mock.patch.object(
                    mod.subprocess,
                    "run",
                    return_value=subprocess.CompletedProcess(["open"], 1, "", "launch failed"),
                ),
            ):
                exit_code, payload = mod.capture_via_gui_terminal(target, root, 5)
            self.assertEqual(exit_code, 1)
            self.assertEqual(payload["blocker_reason"], "gui_session_launch_failed")
            metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
            self.assertEqual(metadata["guiSessionRelay"]["status"], "launch_failed")

    def test_terminal_command_tries_both_studio_app_names_before_capture(self) -> None:
        mod = load_module()
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            target = root / "shot.png"
            helper_path = root / "scripts" / "studio_ui_control.py"
            helper_path.parent.mkdir(parents=True, exist_ok=True)
            helper_path.write_text("#!/usr/bin/env python3\n", encoding="utf-8")
            target.write_bytes(b"png")
            target.with_suffix(".capture.json").write_text(
                json.dumps(
                    {
                        "success": True,
                        "target": str(target),
                        "target_exists": True,
                        "capture_method": "rect",
                        "blocker_reason": None,
                    }
                ),
                encoding="utf-8",
            )
            written_command = {"text": ""}

            original_write_text = Path.write_text

            def fake_write_text(path_obj: Path, text: str, *args, **kwargs):
                if path_obj.name == "capture.command":
                    written_command["text"] = text
                return original_write_text(path_obj, text, *args, **kwargs)

            original_exists = Path.exists
            original_read_text = Path.read_text

            def fake_exists(path_obj: Path):
                if path_obj == target:
                    return True
                if path_obj.name == "exit.code":
                    return True
                return original_exists(path_obj)

            def fake_read_text(path_obj: Path, *args, **kwargs):
                if path_obj.name == "exit.code":
                    return "0\n"
                return original_read_text(path_obj, *args, **kwargs)

            with (
                mock.patch.object(mod.shutil, "which", return_value="/usr/bin/open"),
                mock.patch.object(
                    mod.subprocess,
                    "run",
                    return_value=subprocess.CompletedProcess(["open"], 0, "", ""),
                ),
                mock.patch.object(Path, "write_text", fake_write_text),
                mock.patch.object(Path, "exists", fake_exists),
                mock.patch.object(Path, "read_text", fake_read_text),
            ):
                exit_code, payload = mod.capture_via_gui_terminal(target, root, 5)

            self.assertEqual(exit_code, 0)
            self.assertTrue(payload["success"])
            self.assertIn('tell application "RobloxStudio" to activate', written_command["text"])
            self.assertIn('tell application "Roblox Studio" to activate', written_command["text"])
            self.assertIn('STUDIO_UI_CONTROL=', written_command["text"])
            self.assertIn('capture-screenshot --target "$TARGET"', written_command["text"])


if __name__ == "__main__":
    unittest.main()
