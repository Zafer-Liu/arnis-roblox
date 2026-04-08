from __future__ import annotations

import importlib.util
import io
import json
import os
import sys
import unittest
from pathlib import Path
from typing import Optional
from unittest import mock


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "publish_to_roblox.py"


def load_module():
    scripts_dir = str(MODULE_PATH.parent)
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
    spec = importlib.util.spec_from_file_location("publish_to_roblox", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load module spec from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules["publish_to_roblox"] = module
    spec.loader.exec_module(module)
    return module


class _FakeResponse:
    def __init__(self, payload: bytes, status: int = 200) -> None:
        self._buf = io.BytesIO(payload)
        self.status = status

    def read(self) -> bytes:
        return self._buf.read()

    def __enter__(self) -> "_FakeResponse":
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        self._buf.close()


class _CapturingOpener:
    """Minimal OpenerDirector stand-in that records the request it received."""

    def __init__(self, response_payload: bytes) -> None:
        self._response_payload = response_payload
        self.seen_request = None  # type: Optional[object]

    def open(self, request):  # noqa: A003 - matches OpenerDirector.open
        self.seen_request = request
        return _FakeResponse(self._response_payload)


class PublishToRobloxArgParsingTests(unittest.TestCase):
    def test_required_args(self) -> None:
        module = load_module()
        with self.assertRaises(SystemExit):
            module.parse_args([])

    def test_parses_all_args(self) -> None:
        module = load_module()
        args = module.parse_args(
            [
                "--place-file",
                "roblox/out/arnis-scripts-only.rbxlx",
                "--universe-id",
                "111",
                "--place-id",
                "222",
                "--version-type",
                "Saved",
                "--dry-run",
            ]
        )
        self.assertEqual(args.place_file, Path("roblox/out/arnis-scripts-only.rbxlx"))
        self.assertEqual(args.universe_id, 111)
        self.assertEqual(args.place_id, 222)
        self.assertEqual(args.version_type, "Saved")
        self.assertTrue(args.dry_run)

    def test_build_url_matches_open_cloud_endpoint(self) -> None:
        module = load_module()
        url = module.build_url(111, 222, "Published")
        self.assertEqual(
            url,
            "https://apis.roblox.com/universes/v1/111/places/222/versions?versionType=Published",
        )

    def test_infer_content_type(self) -> None:
        module = load_module()
        self.assertEqual(module.infer_content_type(Path("foo.rbxlx")), module.CONTENT_TYPE_XML)
        self.assertEqual(module.infer_content_type(Path("foo.rbxl")), module.CONTENT_TYPE_BINARY)
        self.assertEqual(module.infer_content_type(Path("foo.bin")), module.CONTENT_TYPE_BINARY)


class PublishToRobloxDryRunTests(unittest.TestCase):
    def test_dry_run_prints_preview_and_does_not_call_urllib(self) -> None:
        module = load_module()

        # Use the module file itself as a stand-in "place file" — any real
        # file on disk works because dry-run never reads its bytes.
        place_file = MODULE_PATH

        captured_stdout = io.StringIO()
        with mock.patch("urllib.request.urlopen") as fake_urlopen, mock.patch(
            "sys.stdout", captured_stdout
        ):
            exit_code = module.main(
                [
                    "--place-file",
                    str(place_file),
                    "--universe-id",
                    "111",
                    "--place-id",
                    "222",
                    "--dry-run",
                ]
            )

        self.assertEqual(exit_code, 0)
        fake_urlopen.assert_not_called()

        output = captured_stdout.getvalue()
        self.assertIn("dry-run preview", output)
        self.assertIn("universe_id     = 111", output)
        self.assertIn("place_id        = 222", output)
        self.assertIn(
            "https://apis.roblox.com/universes/v1/111/places/222/versions?versionType=Published",
            output,
        )
        self.assertIn("auth_header     = x-api-key: <redacted>", output)
        self.assertNotIn("secret", output.lower())


class PublishToRobloxMissingKeyTests(unittest.TestCase):
    def test_missing_api_key_fails_fast(self) -> None:
        module = load_module()
        place_file = MODULE_PATH  # any real file on disk
        stderr = io.StringIO()

        env = {k: v for k, v in os.environ.items() if k != module.API_KEY_ENV_VAR}
        with mock.patch.dict(os.environ, env, clear=True), mock.patch(
            "sys.stderr", stderr
        ), mock.patch("urllib.request.urlopen") as fake_urlopen:
            exit_code = module.main(
                [
                    "--place-file",
                    str(place_file),
                    "--universe-id",
                    "111",
                    "--place-id",
                    "222",
                ]
            )

        self.assertEqual(exit_code, 3)
        fake_urlopen.assert_not_called()
        self.assertIn("ROBLOX_OPEN_CLOUD_API_KEY", stderr.getvalue())

    def test_missing_place_file_exits_with_usage_error(self) -> None:
        module = load_module()
        stderr = io.StringIO()
        with mock.patch("sys.stderr", stderr):
            exit_code = module.main(
                [
                    "--place-file",
                    "/definitely/does/not/exist.rbxlx",
                    "--universe-id",
                    "111",
                    "--place-id",
                    "222",
                ]
            )
        self.assertEqual(exit_code, 2)
        self.assertIn("does not exist", stderr.getvalue())


class PublishToRobloxExecuteTests(unittest.TestCase):
    def test_execute_publish_sends_expected_request(self) -> None:
        module = load_module()

        tmp_dir = Path(self.enterContext(__import__("tempfile").TemporaryDirectory()))
        place_file = tmp_dir / "tiny.rbxlx"
        place_bytes = b"<roblox></roblox>"
        place_file.write_bytes(place_bytes)

        request = module.build_publish_request(
            place_file=place_file,
            universe_id=111,
            place_id=222,
        )
        opener = _CapturingOpener(json.dumps({"versionNumber": 42}).encode("utf-8"))
        result = module.execute_publish(request, "test-api-key", opener=opener)

        self.assertEqual(result, {"versionNumber": 42})

        seen = opener.seen_request
        self.assertIsNotNone(seen)
        self.assertEqual(seen.get_method(), "POST")
        self.assertEqual(
            seen.full_url,
            "https://apis.roblox.com/universes/v1/111/places/222/versions?versionType=Published",
        )
        # urllib normalises header names to title-case internally.
        self.assertEqual(seen.get_header("X-api-key"), "test-api-key")
        self.assertEqual(seen.get_header("Content-type"), module.CONTENT_TYPE_XML)
        self.assertEqual(seen.data, place_bytes)

    def test_execute_publish_rbxl_uses_octet_stream(self) -> None:
        module = load_module()

        tmp_dir = Path(self.enterContext(__import__("tempfile").TemporaryDirectory()))
        place_file = tmp_dir / "tiny.rbxl"
        place_file.write_bytes(b"\x00\x01\x02binary-rbxl")

        request = module.build_publish_request(
            place_file=place_file,
            universe_id=1,
            place_id=2,
        )
        self.assertEqual(request.content_type, module.CONTENT_TYPE_BINARY)

        opener = _CapturingOpener(json.dumps({"versionNumber": 7}).encode("utf-8"))
        module.execute_publish(request, "k", opener=opener)
        self.assertEqual(
            opener.seen_request.get_header("Content-type"),
            module.CONTENT_TYPE_BINARY,
        )


if __name__ == "__main__":
    unittest.main()
