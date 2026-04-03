from __future__ import annotations

import importlib.util
import io
import json
import subprocess
import tempfile
from pathlib import Path
from contextlib import redirect_stdout
import unittest
from unittest import mock


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "studio_ui_control.py"


def load_module():
    spec = importlib.util.spec_from_file_location("studio_ui_control", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load module spec from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class StudioUiControlTests(unittest.TestCase):
    def test_capture_screenshot_prefers_window_capture_then_falls_back_to_display(self) -> None:
        mod = load_module()
        with tempfile.TemporaryDirectory() as tmpdir:
            target_path = Path(tmpdir) / "studio.png"
            commands: list[list[str]] = []

            def fake_run_capture_command(command: list[str]) -> subprocess.CompletedProcess[str]:
                commands.append(command)
                if "-l" in command:
                    return subprocess.CompletedProcess(command, 1, "", "could not create image from display")
                target_path.write_bytes(b"png")
                return subprocess.CompletedProcess(command, 0, "", "")

            with (
                mock.patch.object(
                    mod,
                    "capture_session_snapshot",
                    return_value=(0, {"state": "playing", "front_window": "place.rbxlx - Roblox Studio", "window_count": 1}),
                ),
                mock.patch.object(mod, "capture_process_count", return_value=1),
                mock.patch.object(
                    mod,
                    "resolve_front_window_capture_target",
                    return_value=(
                        0,
                        {
                            "window_id": 321,
                            "owner_name": "RobloxStudio",
                            "window_name": "place.rbxlx - Roblox Studio",
                            "bounds": {"X": 10, "Y": 20, "Width": 1440, "Height": 900},
                        },
                        "",
                    ),
                ),
                mock.patch.object(mod, "run_capture_command", side_effect=fake_run_capture_command),
            ):
                exit_code = mod.capture_screenshot(str(target_path))

            self.assertEqual(exit_code, 0)
            self.assertEqual(commands[0], ["screencapture", "-x", "-l", "321", str(target_path)])
            self.assertEqual(commands[1], ["screencapture", "-x", str(target_path)])
            metadata = json.loads(target_path.with_suffix(".capture.json").read_text(encoding="utf-8"))
            self.assertTrue(metadata["success"])
            self.assertEqual(metadata["capture_method"], "display")
            self.assertEqual(metadata["attempts"][0]["method"], "window")
            self.assertEqual(metadata["attempts"][0]["stderr"], "could not create image from display")
            self.assertEqual(metadata["attempts"][1]["method"], "display")
            self.assertEqual(metadata["session_status"]["status"], "ready_play")
            self.assertEqual(metadata["window_lookup_error"], "")

    def test_capture_screenshot_records_failure_metadata_when_display_capture_fails(self) -> None:
        mod = load_module()
        with tempfile.TemporaryDirectory() as tmpdir:
            target_path = Path(tmpdir) / "studio.png"
            command = ["screencapture", "-x", str(target_path)]
            with (
                mock.patch.object(
                    mod,
                    "capture_session_snapshot",
                    return_value=(0, {"state": "editor_ready", "front_window": "place.rbxlx - Roblox Studio", "window_count": 1}),
                ),
                mock.patch.object(mod, "capture_process_count", return_value=1),
                mock.patch.object(
                    mod,
                    "resolve_front_window_capture_target",
                    return_value=(1, {}, "Execution error: Error: JXA lookup failed"),
                ),
                mock.patch.object(
                    mod,
                    "run_capture_command",
                    return_value=subprocess.CompletedProcess(command, 1, "", "could not create image from display"),
                ),
            ):
                exit_code = mod.capture_screenshot(str(target_path))

            self.assertEqual(exit_code, 1)
            metadata = json.loads(target_path.with_suffix(".capture.json").read_text(encoding="utf-8"))
            self.assertFalse(metadata["success"])
            self.assertEqual(metadata["capture_method"], "failed")
            self.assertEqual(metadata["attempts"], [{"method": "display", "command": command, "returncode": 1, "stderr": "could not create image from display"}])
            self.assertEqual(metadata["session_status"]["status"], "ready_edit")
            self.assertEqual(metadata["window_lookup_error"], "Execution error: Error: JXA lookup failed")

    def test_infer_state_label_uses_enabled_stop_item_for_playing(self) -> None:
        mod = load_module()
        state = mod.infer_state_label(
            {
                "front_window": "place.rbxlx - Roblox Studio",
                "has_test_menu": True,
                "has_stop_menu_item": True,
                "has_stop_menu_item_enabled": True,
                "button_names": [],
                "window_count": 1,
                "has_file_menu": True,
            }
        )
        self.assertEqual(state, "playing")

    def test_infer_state_label_treats_disabled_stop_item_as_editor_ready(self) -> None:
        mod = load_module()
        state = mod.infer_state_label(
            {
                "front_window": "place.rbxlx - Roblox Studio",
                "has_test_menu": True,
                "has_stop_menu_item": True,
                "has_stop_menu_item_enabled": False,
                "button_names": [],
                "window_count": 1,
                "has_file_menu": True,
            }
        )
        self.assertEqual(state, "editor_ready")

    def test_classify_session_status_not_running(self) -> None:
        mod = load_module()
        status = mod.classify_session_status(
            {
                "state": "not_running",
                "front_window": "",
                "window_count": 0,
            },
            0,
        )
        self.assertEqual(status["status"], "not_running")
        self.assertTrue(status["safe_to_open"])
        self.assertFalse(status["safe_to_quit"])

    def test_classify_session_status_editor_ready(self) -> None:
        mod = load_module()
        status = mod.classify_session_status(
            {
                "state": "editor_ready",
                "front_window": "place.rbxlx - Roblox Studio",
                "window_count": 1,
            },
            1,
        )
        self.assertEqual(status["status"], "ready_edit")
        self.assertTrue(status["ready_for_menu"])
        self.assertTrue(status["ready_for_harness"])

    def test_classify_session_status_playing(self) -> None:
        mod = load_module()
        status = mod.classify_session_status(
            {
                "state": "playing",
                "front_window": "place.rbxlx - Roblox Studio",
                "window_count": 1,
            },
            1,
        )
        self.assertEqual(status["status"], "ready_play")
        self.assertTrue(status["ready_play"])
        self.assertTrue(status["safe_to_open"])

    def test_classify_session_status_dialog_blocked(self) -> None:
        mod = load_module()
        status = mod.classify_session_status(
            {
                "state": "save_prompt",
                "front_window": "Do you want to save",
                "window_count": 1,
            },
            1,
        )
        self.assertEqual(status["status"], "blocked_dialog")
        self.assertTrue(status["blocked_dialog"])
        self.assertFalse(status["ready_for_harness"])

    def test_classify_session_status_file_panel_blocked(self) -> None:
        mod = load_module()
        status = mod.classify_session_status(
            {
                "state": "menu_ready",
                "front_window": "Open Roblox File",
                "window_count": 3,
            },
            1,
        )
        self.assertEqual(status["status"], "blocked_dialog")
        self.assertTrue(status["blocked_dialog"])
        self.assertFalse(status["ready_for_harness"])

    def test_classify_session_status_lighting_migration_blocked(self) -> None:
        mod = load_module()
        status = mod.classify_session_status(
            {
                "state": "menu_ready",
                "front_window": "Lighting Technology Migration",
                "window_count": 2,
            },
            1,
        )
        self.assertEqual(status["status"], "blocked_dialog")
        self.assertTrue(status["blocked_dialog"])
        self.assertFalse(status["ready_for_harness"])

    def test_classify_session_status_save_close_blocked(self) -> None:
        mod = load_module()
        status = mod.classify_session_status(
            {
                "state": "menu_ready",
                "front_window": "Do you want to save changes before closing?",
                "window_count": 2,
            },
            1,
        )
        self.assertEqual(status["status"], "blocked_dialog")
        self.assertTrue(status["blocked_dialog"])
        self.assertFalse(status["ready_for_harness"])

    def test_classify_session_status_transitioning(self) -> None:
        mod = load_module()
        status = mod.classify_session_status(
            {
                "state": "window_open",
                "front_window": "Roblox Studio",
                "window_count": 1,
            },
            1,
        )
        self.assertEqual(status["status"], "transitioning")
        self.assertTrue(status["transitioning"])
        self.assertFalse(status["ready_for_harness"])

    def test_run_osascript_returns_timeout_exit_code(self) -> None:
        mod = load_module()
        with mock.patch.object(
            mod.subprocess,
            "run",
            side_effect=subprocess.TimeoutExpired(cmd=["osascript"], timeout=5),
        ) as run_mock:
            exit_code = mod.run_osascript('tell application "System Events" to return')
        self.assertEqual(exit_code, 124)
        self.assertEqual(run_mock.call_args.kwargs["timeout"], mod.OSASCRIPT_TIMEOUT_SECONDS)

    def test_capture_osascript_returns_timeout_exit_code_and_empty_output(self) -> None:
        mod = load_module()
        with mock.patch.object(
            mod.subprocess,
            "run",
            side_effect=subprocess.TimeoutExpired(cmd=["osascript"], timeout=5),
        ) as run_mock:
            exit_code, output = mod.capture_osascript('tell application "System Events" to return')
        self.assertEqual(exit_code, 124)
        self.assertEqual(output, "")
        self.assertEqual(run_mock.call_args.kwargs["timeout"], mod.OSASCRIPT_TIMEOUT_SECONDS)

    def test_capture_session_snapshot_falls_back_to_fast_probe_after_full_timeout(self) -> None:
        mod = load_module()
        fast_payload = {
            "front_window": "Roblox Studio",
            "window_count": 1,
            "menu_count": 0,
            "has_file_menu": False,
            "has_plugins_menu": False,
            "has_test_menu": False,
            "has_start_test_session_menu_item": False,
            "has_start_test_session_menu_item_enabled": False,
            "has_play_menu_item": False,
            "has_play_menu_item_enabled": False,
            "has_stop_menu_item": False,
            "has_stop_menu_item_enabled": False,
            "window_names": ["Roblox Studio"],
            "button_names": [],
            "state": "start_page",
        }
        with (
            mock.patch.object(mod, "capture_state_snapshot", return_value=(124, {})),
            mock.patch.object(mod, "capture_fast_session_snapshot", return_value=(0, fast_payload), create=True),
        ):
            exit_code, payload = mod.capture_session_snapshot()
        self.assertEqual(exit_code, 0)
        self.assertEqual(payload["state"], "start_page")
        self.assertEqual(payload["front_window"], "Roblox Studio")

    def test_get_session_status_value_uses_fast_probe_when_full_snapshot_times_out(self) -> None:
        mod = load_module()
        fast_payload = {
            "front_window": "Roblox Studio",
            "window_count": 1,
            "menu_count": 0,
            "has_file_menu": False,
            "has_plugins_menu": False,
            "has_test_menu": False,
            "has_start_test_session_menu_item": False,
            "has_start_test_session_menu_item_enabled": False,
            "has_play_menu_item": False,
            "has_play_menu_item_enabled": False,
            "has_stop_menu_item": False,
            "has_stop_menu_item_enabled": False,
            "window_names": ["Roblox Studio"],
            "button_names": [],
            "state": "start_page",
        }
        with (
            mock.patch.object(mod, "capture_state_snapshot", return_value=(124, {})),
            mock.patch.object(mod, "capture_fast_session_snapshot", return_value=(0, fast_payload), create=True),
            mock.patch.object(mod, "capture_process_count", return_value=1),
            io.StringIO() as buffer,
            redirect_stdout(buffer),
        ):
            exit_code = mod.get_session_status_value("ready_for_harness")
            output = buffer.getvalue().strip()
        self.assertEqual(exit_code, 0)
        self.assertEqual(output, "true")

    def test_dismiss_startup_dialogs_checks_continue_for_lighting_migration(self) -> None:
        mod = load_module()
        with mock.patch.object(mod, "run_osascript", return_value=0) as run_mock:
            exit_code = mod.dismiss_startup_dialogs()
        self.assertEqual(exit_code, 0)
        script = run_mock.call_args.args[0]
        self.assertIn('click button "Continue" of w', script)
        self.assertIn('click button "Continue" of sheet 1 of w', script)

    def test_get_session_status_value_prints_booleans_lowercase(self) -> None:
        mod = load_module()
        with (
            mock.patch.object(
                mod,
                "capture_state_snapshot",
                return_value=(0, {"state": "editor_ready", "front_window": "Place1 - Roblox Studio", "window_count": 1}),
            ),
            mock.patch.object(mod, "capture_process_count", return_value=1),
            io.StringIO() as buffer,
            redirect_stdout(buffer),
        ):
            exit_code = mod.get_session_status_value("can_start_test_session")
            output = buffer.getvalue().strip()
        self.assertEqual(exit_code, 0)
        self.assertEqual(output, "false")


if __name__ == "__main__":
    unittest.main()
