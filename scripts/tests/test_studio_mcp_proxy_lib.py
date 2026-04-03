#!/usr/bin/env python3
from __future__ import annotations

import os
import unittest
from unittest import mock
import urllib.error

from scripts.studio_mcp_proxy_lib import (
    HttpProxyClient,
    ProbeError,
    build_mcp_client,
    ensure_play_mode,
    run_code_in_existing_session,
    run_code_in_play_session,
)


class FakeClient:
    def __init__(self, initial_mode: str, *, run_script_results: list[dict] | None = None) -> None:
        self.mode = initial_mode
        self.calls: list[tuple[str, dict, bool, int | None]] = []
        self.run_script_results = list(run_script_results or [])

    def call_tool(
        self,
        name: str,
        arguments: dict | None = None,
        *,
        allow_is_error: bool = False,
        timeout_seconds: int | None = None,
    ) -> dict:
        args = arguments or {}
        self.calls.append((name, args, allow_is_error, timeout_seconds))
        if name == "get_studio_mode":
            return {"content": [{"type": "text", "text": self.mode}]}
        if name == "start_stop_play":
            requested = args.get("mode", "start_play")
            if requested == "stop":
                self.mode = "stop"
            else:
                self.mode = requested
            return {"content": [{"type": "text", "text": "ok"}]}
        if name == "run_code":
            return {"content": [{"type": "text", "text": "ok"}], "isError": False}
        if name == "run_script_in_play_mode":
            if self.run_script_results:
                return self.run_script_results.pop(0)
            return {"content": [{"type": "text", "text": "ok"}], "isError": False}
        raise AssertionError(f"unexpected tool {name}")


class StudioMcpProxyLibTests(unittest.TestCase):
    def test_http_proxy_client_retries_locked_response_then_succeeds(self) -> None:
        client = HttpProxyClient("http://127.0.0.1:44755/request", timeout_seconds=12)
        success_response = mock.MagicMock()
        success_response.read.return_value = b'{"success":true,"response":"start_play"}'
        success_context = mock.MagicMock()
        success_context.__enter__.return_value = success_response
        success_context.__exit__.return_value = False

        with mock.patch(
            "urllib.request.urlopen",
            side_effect=[
                urllib.error.HTTPError(client._proxy_url, 423, "Locked", hdrs=None, fp=None),
                success_context,
            ],
        ) as urlopen_mock, mock.patch("time.sleep") as sleep_mock:
            result = client.call_tool("get_studio_mode", {})

        self.assertEqual(result["content"][0]["text"], "start_play")
        self.assertEqual(urlopen_mock.call_count, 2)
        sleep_mock.assert_called_once()

    def test_http_proxy_client_raises_after_locked_retry_budget_is_exhausted(self) -> None:
        client = HttpProxyClient("http://127.0.0.1:44755/request", timeout_seconds=12)
        locked_error = urllib.error.HTTPError(client._proxy_url, 423, "Locked", hdrs=None, fp=None)

        with mock.patch("urllib.request.urlopen", side_effect=[locked_error] * 4), mock.patch("time.sleep") as sleep_mock:
            with self.assertRaises(ProbeError):
                client.call_tool("get_studio_mode", {})

        self.assertEqual(sleep_mock.call_count, 3)

    def test_build_mcp_client_prefers_direct_stdio_by_default_even_with_proxy_url(self) -> None:
        direct_calls: list[tuple[str, int, str, str]] = []

        class FakeDirectClient:
            def __init__(
                self,
                mcp_bin: str,
                *,
                timeout_seconds: int,
                protocol_version: str,
                client_name: str,
            ) -> None:
                direct_calls.append((mcp_bin, timeout_seconds, protocol_version, client_name))

        with mock.patch.dict(
            os.environ,
            {"MCP_PROXY_URL": "http://127.0.0.1:44755/proxy"},
            clear=False,
        ):
            client = build_mcp_client(
                FakeDirectClient,
                mcp_bin="/tmp/rbx-studio-mcp",
                timeout_seconds=22,
                protocol_version="2025-11-25",
                client_name="unit-test",
            )

        self.assertIsInstance(client, FakeDirectClient)
        self.assertEqual(
            direct_calls,
            [("/tmp/rbx-studio-mcp", 22, "2025-11-25", "unit-test")],
        )

    def test_build_mcp_client_uses_proxy_only_when_explicitly_enabled(self) -> None:
        class FakeDirectClient:
            def __init__(self, *args, **kwargs) -> None:
                raise AssertionError("direct stdio client should not be constructed")

        with mock.patch.dict(
            os.environ,
            {
                "MCP_PROXY_URL": "http://127.0.0.1:44755/proxy",
                "HARNESS_USE_MCP_PROXY": "1",
            },
            clear=False,
        ):
            client = build_mcp_client(
                FakeDirectClient,
                mcp_bin="/tmp/rbx-studio-mcp",
                timeout_seconds=22,
                protocol_version="2025-11-25",
                client_name="unit-test",
            )

        self.assertIsInstance(client, HttpProxyClient)

    def test_ensure_play_mode_starts_play_when_stopped(self) -> None:
        client = FakeClient(initial_mode="stop")

        ensure_play_mode(client, requested_mode="start_play")

        self.assertEqual(client.mode, "start_play")
        self.assertEqual(client.calls[0][0], "get_studio_mode")
        self.assertEqual(client.calls[1][0], "start_stop_play")
        self.assertEqual(client.calls[1][1], {"mode": "start_play"})

    def test_run_code_in_play_session_uses_run_script_in_play_mode(self) -> None:
        client = FakeClient(initial_mode="start_play")

        result = run_code_in_play_session(
            client,
            "print('hello')",
            requested_mode="start_play",
            timeout_seconds=77,
        )

        self.assertEqual(result["content"][0]["text"], "ok")
        tool_names = [call[0] for call in client.calls]
        self.assertEqual(tool_names, ["get_studio_mode", "run_script_in_play_mode"])
        self.assertEqual(client.calls[-1][1], {"code": "print('hello')", "mode": "start_play", "timeout": 77})
        self.assertEqual(client.calls[-1][3], 77)

    def test_run_code_in_existing_session_uses_run_code_without_session_transition(self) -> None:
        client = FakeClient(initial_mode="start_play")

        result = run_code_in_existing_session(
            client,
            "print('hello')",
            timeout_seconds=77,
        )

        self.assertEqual(result["content"][0]["text"], "ok")
        self.assertEqual(
            client.calls,
            [("run_code", {"command": "print('hello')"}, True, 77)],
        )

    def test_run_code_in_play_session_stops_and_retries_when_previous_play_session_is_stuck(self) -> None:
        client = FakeClient(
            initial_mode="start_play",
            run_script_results=[
                {
                    "content": [
                        {
                            "type": "text",
                            "text": "StudioTestService: Previous call to start play session has not been completed",
                        }
                    ],
                    "isError": True,
                },
                {"content": [{"type": "text", "text": "ok"}], "isError": False},
            ],
        )

        result = run_code_in_play_session(
            client,
            "print('hello')",
            requested_mode="start_play",
            timeout_seconds=77,
        )

        self.assertEqual(result["content"][0]["text"], "ok")
        tool_names = [call[0] for call in client.calls]
        self.assertEqual(
            tool_names,
            [
                "get_studio_mode",
                "run_script_in_play_mode",
                "start_stop_play",
                "get_studio_mode",
                "start_stop_play",
                "run_script_in_play_mode",
            ],
        )
        self.assertEqual(client.calls[2][1], {"mode": "stop"})
        self.assertEqual(client.calls[4][1], {"mode": "start_play"})


if __name__ == "__main__":
    unittest.main()
