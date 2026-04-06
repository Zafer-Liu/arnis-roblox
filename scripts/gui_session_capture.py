#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile
import time


TERMINAL_APP = "Terminal"
STUDIO_APP = "Roblox Studio"


def _build_failure_payload(
    target: Path,
    metadata_path: Path,
    *,
    capture_method: str,
    blocker_reason: str,
    error: str,
) -> dict[str, object]:
    return {
        "success": False,
        "target": str(target),
        "metadata_path": str(metadata_path),
        "capture_method": capture_method,
        "blocker_reason": blocker_reason,
        "error": error,
    }


def _annotate_metadata(metadata_path: Path, relay_payload: dict[str, object]) -> None:
    if not metadata_path.exists():
        return
    try:
        metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
    except Exception:
        return
    if not isinstance(metadata, dict):
        return
    metadata["guiSessionRelay"] = relay_payload
    metadata_path.write_text(json.dumps(metadata, indent=2, sort_keys=True), encoding="utf-8")


def _write_failure_metadata(
    metadata_path: Path,
    target: Path,
    *,
    capture_method: str,
    blocker_reason: str,
    error: str,
    relay_payload: dict[str, object],
) -> None:
    metadata = {
        "success": False,
        "target": str(target),
        "target_exists": target.exists(),
        "capture_method": capture_method,
        "blocker_reason": blocker_reason,
        "metadata_version": 1,
        "attempts": [],
        "guiSessionRelay": relay_payload,
        "error": error,
    }
    metadata_path.write_text(json.dumps(metadata, indent=2, sort_keys=True), encoding="utf-8")


def capture_via_gui_terminal(target: Path, root_dir: Path, timeout_seconds: int) -> tuple[int, dict[str, object]]:
    metadata_path = target.with_suffix(".capture.json")
    if shutil.which("open") is None:
        payload = _build_failure_payload(
            target,
            metadata_path,
            capture_method="gui_terminal_unavailable",
            blocker_reason="gui_session_unavailable",
            error="open command is unavailable",
        )
        _write_failure_metadata(
            metadata_path,
            target,
            capture_method="gui_terminal_unavailable",
            blocker_reason="gui_session_unavailable",
            error="open command is unavailable",
            relay_payload={"method": "terminal.command", "status": "open_missing", "terminalApp": TERMINAL_APP},
        )
        return 1, payload

    with tempfile.TemporaryDirectory(prefix="arnis-gui-capture-", dir="/tmp") as temp_dir:
        temp_root = Path(temp_dir)
        command_path = temp_root / "capture.command"
        stderr_path = temp_root / "stderr.log"
        exit_code_path = temp_root / "exit.code"

        command_path.write_text(
            "\n".join(
                [
                    "#!/bin/zsh",
                    "set -euo pipefail",
                    f"TARGET={shlex.quote(str(target))}",
                    f"STDERR_LOG={shlex.quote(str(stderr_path))}",
                    f"EXIT_CODE_FILE={shlex.quote(str(exit_code_path))}",
                    f"/usr/bin/osascript -e 'tell application {json.dumps(STUDIO_APP)} to activate' >/dev/null 2>&1 || true",
                    "sleep 1",
                    'if /usr/sbin/screencapture -x "$TARGET" 2>"$STDERR_LOG"; then',
                    '  printf "0\\n" > "$EXIT_CODE_FILE"',
                    "else",
                    '  status="$?"',
                    '  printf "%s\\n" "$status" > "$EXIT_CODE_FILE"',
                    "fi",
                ]
            )
            + "\n",
            encoding="utf-8",
        )
        command_path.chmod(0o755)

        launch = subprocess.run(
            ["open", "-a", TERMINAL_APP, str(command_path)],
            check=False,
            capture_output=True,
            text=True,
        )
        if launch.returncode != 0:
            payload = _build_failure_payload(
                target,
                metadata_path,
                capture_method="gui_terminal_launch_failed",
                blocker_reason="gui_session_launch_failed",
                error=(launch.stderr or launch.stdout or "").strip(),
            )
            _write_failure_metadata(
                metadata_path,
                target,
                capture_method="gui_terminal_launch_failed",
                blocker_reason="gui_session_launch_failed",
                error=(launch.stderr or launch.stdout or "").strip(),
                relay_payload={
                    "method": "terminal.command",
                    "status": "launch_failed",
                    "terminalApp": TERMINAL_APP,
                    "openReturnCode": launch.returncode,
                },
            )
            return 1, payload

        deadline = time.monotonic() + timeout_seconds
        while time.monotonic() < deadline:
            if exit_code_path.exists():
                break
            time.sleep(0.5)
        else:
            _write_failure_metadata(
                metadata_path,
                target,
                capture_method="gui_terminal_timeout",
                blocker_reason="gui_session_timeout",
                error=f"capture command did not finish within {timeout_seconds}s",
                relay_payload={
                    "method": "terminal.command",
                    "status": "timeout",
                    "timeoutSeconds": timeout_seconds,
                    "terminalApp": TERMINAL_APP,
                    "openReturnCode": launch.returncode,
                },
            )
            return 1, _build_failure_payload(
                target,
                metadata_path,
                capture_method="gui_terminal_timeout",
                blocker_reason="gui_session_timeout",
                error=f"capture command did not finish within {timeout_seconds}s",
            )

        try:
            exit_code = int(exit_code_path.read_text(encoding="utf-8").strip())
        except Exception:
            exit_code = 1

        stderr_text = stderr_path.read_text(encoding="utf-8") if stderr_path.exists() else ""
        success = exit_code == 0 and target.exists()
        payload = {
            "success": success,
            "target": str(target),
            "metadata_path": str(metadata_path),
            "capture_method": "gui_terminal_display" if success else "gui_terminal_failed",
            "blocker_reason": None if success else "gui_session_capture_failed",
            "error": stderr_text.strip() if not success else "",
        }
        metadata_payload = {
            "success": success,
            "target": str(target),
            "target_exists": target.exists(),
            "capture_method": "gui_terminal_display" if success else "gui_terminal_failed",
            "blocker_reason": None if success else "gui_session_capture_failed",
            "metadata_version": 1,
            "attempts": [
                {
                    "method": "gui_terminal_display",
                    "command": ["screencapture", "-x", str(target)],
                    "returncode": exit_code,
                    "stderr": stderr_text.strip(),
                }
            ],
        }
        metadata_path.write_text(json.dumps(metadata_payload, indent=2, sort_keys=True), encoding="utf-8")
        _annotate_metadata(
            metadata_path,
            {
                "method": "terminal.command",
                "terminalApp": TERMINAL_APP,
                "openReturnCode": launch.returncode,
                "status": "ok" if exit_code == 0 else "failed",
            },
        )
        return exit_code, payload


def main() -> int:
    parser = argparse.ArgumentParser(description="Relay screenshot capture through the logged-in GUI Terminal session.")
    parser.add_argument("--target", required=True, type=Path)
    parser.add_argument("--root-dir", required=True, type=Path)
    parser.add_argument("--timeout", type=int, default=30)
    args = parser.parse_args()

    exit_code, payload = capture_via_gui_terminal(args.target, args.root_dir, args.timeout)
    print(json.dumps(payload, separators=(",", ":")))
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
