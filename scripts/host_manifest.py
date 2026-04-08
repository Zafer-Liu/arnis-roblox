#!/usr/bin/env python3
"""Helper for hosting a compiled Arnis manifest JSON so the Roblox runtime
`ManifestLoader` can fetch it through `HttpService:GetAsync` +
`HttpService:JSONDecode`.

This script does three things, in order:

  1. Re-runs the existing `verify_manifest_http_payload.py` sanity check so we
     never host a manifest that would fail `HttpService:JSONDecode`.
  2. Prints concrete hosting instructions for the supported lanes
     (S3 / Cloudflare R2 / GitHub Pages / plain HTTPS bucket) plus the
     exact `WorldConfig.ManifestSource` snippet to wire it up.
  3. Optionally serves the manifest locally on a configurable port via
     `http.server` for smoke-testing from Studio without a public host.

Usage:

    # Verify + print hosting instructions
    python3 scripts/host_manifest.py roblox/out/austin.json

    # Verify + serve locally on http://0.0.0.0:8787/austin.json
    python3 scripts/host_manifest.py roblox/out/austin.json --serve --port 8787

The local server binds to all interfaces by default so a Roblox Studio client
on another LAN machine can hit it; override with `--host 127.0.0.1` if you
only want localhost.
"""

from __future__ import annotations

import argparse
import http.server
import importlib.util
import socketserver
import sys
from pathlib import Path
from typing import Optional


REPO_ROOT = Path(__file__).resolve().parents[1]
VERIFY_SCRIPT = REPO_ROOT / "scripts" / "verify_manifest_http_payload.py"


def _load_verifier():
    spec = importlib.util.spec_from_file_location(
        "verify_manifest_http_payload", VERIFY_SCRIPT
    )
    if spec is None or spec.loader is None:
        raise RuntimeError(f"failed to load verifier module from {VERIFY_SCRIPT}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def verify_manifest(manifest_path: Path) -> int:
    verifier = _load_verifier()
    return verifier.verify_manifest_payload(manifest_path)


def print_hosting_instructions(manifest_path: Path) -> None:
    file_name = manifest_path.name
    size_mb = manifest_path.stat().st_size / (1024 * 1024)
    lines = [
        "",
        f"[host_manifest] manifest {manifest_path} is {size_mb:.2f} MiB and"
        " passed the HttpService decodability check.",
        "",
        "Pick one of the following hosting lanes. Each one is free-tier",
        "friendly and works with Roblox HttpService GET requests:",
        "",
        "  1. Amazon S3 (public-read object):",
        f"       aws s3 cp {manifest_path} s3://<bucket>/{file_name} \\",
        "              --acl public-read --content-type application/json",
        f"     URL: https://<bucket>.s3.amazonaws.com/{file_name}",
        "",
        "  2. Cloudflare R2 (via a public bucket or Worker route):",
        f"       wrangler r2 object put <bucket>/{file_name} \\",
        f"              --file {manifest_path} \\",
        "              --content-type application/json",
        f"     URL: https://<your-r2-domain>/{file_name}",
        "",
        "  3. GitHub Pages (simplest free option):",
        f"       cp {manifest_path} docs/manifests/{file_name}",
        "       git add docs/manifests && git commit -m 'publish manifest' && git push",
        f"     URL: https://<user>.github.io/<repo>/manifests/{file_name}",
        "",
        "  4. Any plain HTTPS static host (Netlify, Fly volumes, etc.) — just",
        "     serve the file with Content-Type: application/json.",
        "",
        "Once the file is reachable, configure the runtime manifest source in",
        "roblox/src/ReplicatedStorage/Shared/WorldConfig.lua:",
        "",
        "    WorldConfig.ManifestSource = {",
        '        mode = "external_url",',
        f'        url = "https://<your-host>/{file_name}",',
        "        timeoutSeconds = 20,",
        "    }",
        "",
        "The ManifestLoader will then call HttpService:GetAsync(url) followed",
        "by HttpService:JSONDecode and validate against ChunkSchema.",
        "",
    ]
    print("\n".join(lines))


def serve_locally(manifest_path: Path, host: str, port: int) -> int:
    directory = str(manifest_path.parent.resolve())
    file_name = manifest_path.name

    class ManifestHandler(http.server.SimpleHTTPRequestHandler):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, directory=directory, **kwargs)

        def end_headers(self):  # noqa: D401 - override stdlib hook
            # HttpService on Roblox servers prefers explicit application/json
            # and is happy with permissive CORS, which helps when testing from
            # Studio plugins or the browser dev panel.
            if self.path.endswith(".json"):
                self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            super().end_headers()

        def log_message(self, format: str, *args) -> None:  # noqa: A002 - stdlib signature
            sys.stderr.write(
                f"[host_manifest] {self.address_string()} - {format % args}\n"
            )

    with socketserver.TCPServer((host, port), ManifestHandler) as httpd:
        url = f"http://{host}:{port}/{file_name}"
        print(f"[host_manifest] serving {manifest_path} at {url}")
        print("[host_manifest] press Ctrl+C to stop.")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n[host_manifest] stopped.")
    return 0


def parse_args(argv: Optional[list[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path, help="Path to compiled manifest JSON")
    parser.add_argument(
        "--serve",
        action="store_true",
        help="After verification, serve the manifest on a local HTTP port.",
    )
    parser.add_argument(
        "--host",
        default="0.0.0.0",
        help="Bind host for --serve (default: 0.0.0.0, i.e. all interfaces).",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=8787,
        help="Bind port for --serve (default: 8787).",
    )
    return parser.parse_args(argv)


def main(argv: Optional[list[str]] = None) -> int:
    args = parse_args(argv)
    manifest_path = args.manifest

    if not manifest_path.exists():
        print(f"[host_manifest] missing manifest file: {manifest_path}", file=sys.stderr)
        return 2

    verify_status = verify_manifest(manifest_path)
    if verify_status != 0:
        print(
            f"[host_manifest] verify_manifest_http_payload failed with exit {verify_status};"
            " refusing to host a manifest the runtime cannot decode.",
            file=sys.stderr,
        )
        return verify_status

    print_hosting_instructions(manifest_path)

    if args.serve:
        return serve_locally(manifest_path, args.host, args.port)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
