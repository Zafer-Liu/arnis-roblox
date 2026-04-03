#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import selectors
import subprocess
import time
from typing import Any


class ProbeError(RuntimeError):
    pass


class ProbeTimeout(ProbeError):
    pass


class JsonRpcStdioClient:
    def __init__(self, mcp_bin: str, timeout_seconds: int, protocol_version: str, client_name: str) -> None:
        self._timeout_seconds = timeout_seconds
        self._protocol_version = protocol_version
        self._client_name = client_name
        self._request_id = 1
        self._stdout_buffer = b""
        self._stderr_buffer = b""
        self._stderr_tail: list[str] = []
        self._selector = selectors.DefaultSelector()
        self._proc = subprocess.Popen(
            [mcp_bin, "--stdio"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        if self._proc.stdin is None or self._proc.stdout is None or self._proc.stderr is None:
            raise ProbeError("rbx-studio-mcp stdio pipes are unavailable")
        self._selector.register(self._proc.stdout, selectors.EVENT_READ, data="stdout")
        self._selector.register(self._proc.stderr, selectors.EVENT_READ, data="stderr")

    def close(self) -> None:
        if self._proc.poll() is not None:
            return
        self._proc.terminate()
        try:
            self._proc.wait(timeout=1.5)
        except subprocess.TimeoutExpired:
            self._proc.kill()
            self._proc.wait(timeout=1.5)

    def _remember_stderr(self, chunk: bytes) -> None:
        self._stderr_buffer += chunk
        while b"\n" in self._stderr_buffer:
            line_raw, self._stderr_buffer = self._stderr_buffer.split(b"\n", 1)
            line = line_raw.decode("utf-8", errors="replace").strip()
            if not line:
                continue
            self._stderr_tail.append(line)
        if len(self._stderr_tail) > 40:
            self._stderr_tail = self._stderr_tail[-40:]

    def _drain_messages(self, timeout_seconds: float) -> list[dict[str, Any]]:
        deadline = time.monotonic() + max(timeout_seconds, 0)
        messages: list[dict[str, Any]] = []
        while time.monotonic() < deadline:
            if self._proc.poll() is not None and not messages:
                break
            wait_seconds = max(0.05, min(0.25, deadline - time.monotonic()))
            events = self._selector.select(wait_seconds)
            if not events:
                continue
            for key, _ in events:
                try:
                    chunk = os.read(key.fd, 4096)
                except OSError:
                    chunk = b""
                if not chunk:
                    continue
                if key.data == "stderr":
                    self._remember_stderr(chunk)
                    continue
                self._stdout_buffer += chunk
                while b"\n" in self._stdout_buffer:
                    line_raw, self._stdout_buffer = self._stdout_buffer.split(b"\n", 1)
                    line = line_raw.decode("utf-8", errors="replace").strip()
                    if not line:
                        continue
                    try:
                        message = json.loads(line)
                    except json.JSONDecodeError as exc:
                        raise ProbeError(f"invalid JSON from rbx-studio-mcp stdout: {line}") from exc
                    if isinstance(message, dict):
                        messages.append(message)
            if messages:
                break
        return messages

    def _send(self, payload: dict[str, Any]) -> None:
        if self._proc.stdin is None:
            raise ProbeError("rbx-studio-mcp stdin pipe is unavailable")
        raw = (json.dumps(payload, separators=(",", ":")) + "\n").encode("utf-8")
        try:
            self._proc.stdin.write(raw)
            self._proc.stdin.flush()
        except BrokenPipeError as exc:
            raise ProbeError("rbx-studio-mcp transport closed before request could be sent") from exc

    def _request(self, method: str, params: dict[str, Any], timeout_seconds: int | None = None) -> Any:
        timeout = timeout_seconds if timeout_seconds is not None else self._timeout_seconds
        request_id = self._request_id
        self._request_id += 1
        self._send(
            {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": method,
                "params": params,
            }
        )

        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            remaining = max(0.0, deadline - time.monotonic())
            for message in self._drain_messages(min(remaining, 0.6)):
                if message.get("id") != request_id:
                    continue
                if "error" in message:
                    raise ProbeError(f"MCP request '{method}' failed: {message['error']}")
                return message.get("result")
            if self._proc.poll() is not None:
                stderr_tail = "\n".join(self._stderr_tail[-8:])
                if stderr_tail:
                    raise ProbeError(f"rbx-studio-mcp exited while waiting for '{method}' response.\n{stderr_tail}")
                raise ProbeError(f"rbx-studio-mcp exited while waiting for '{method}' response")
        raise ProbeTimeout(f"MCP request '{method}' timed out after {timeout}s")

    def _notify(self, method: str, params: dict[str, Any]) -> None:
        self._send(
            {
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
            }
        )

    def initialize(self) -> None:
        self._request(
            "initialize",
            {
                "protocolVersion": self._protocol_version,
                "capabilities": {},
                "clientInfo": {"name": self._client_name, "version": "1.0.0"},
            },
        )
        self._notify("notifications/initialized", {})

    def call_tool(
        self,
        name: str,
        arguments: dict[str, Any] | None = None,
        *,
        allow_is_error: bool = False,
        timeout_seconds: int | None = None,
    ) -> Any:
        result = self._request(
            "tools/call",
            {"name": name, "arguments": arguments or {}},
            timeout_seconds=timeout_seconds,
        )
        if not allow_is_error and isinstance(result, dict) and result.get("isError"):
            raise ProbeError(f"MCP tool '{name}' returned isError=true: {result}")
        return result

    @property
    def stderr_tail(self) -> list[str]:
        return list(self._stderr_tail)
