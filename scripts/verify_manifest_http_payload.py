#!/usr/bin/env python3
"""Verify that an arbx_cli compiled manifest JSON file is suitable for delivery
via Roblox `HttpService:GetAsync` + `HttpService:JSONDecode`.

This is the runtime path used by `WorldConfig.ManifestSource.mode = "external_url"`
(see `roblox/src/ServerScriptService/ImportService/ManifestLoader.lua` —
`loadFromExternalSource`). Because Roblox's JSONDecode is strict UTF-8 JSON, we
re-encode the parsed payload and confirm it round-trips and matches the schema
shape that `ChunkSchema.validateManifest` expects.

Usage:
    python3 scripts/verify_manifest_http_payload.py path/to/austin.json

Exits non-zero on any failure so this can run in CI alongside compile output.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


REQUIRED_TOP_KEYS = ("schemaVersion", "chunks")


def verify_manifest_payload(manifest_path: Path) -> int:
    if not manifest_path.exists():
        print(f"[verify_manifest_http_payload] missing file: {manifest_path}", file=sys.stderr)
        return 2

    raw_bytes = manifest_path.read_bytes()
    size_mb = len(raw_bytes) / (1024 * 1024)

    try:
        raw_text = raw_bytes.decode("utf-8")
    except UnicodeDecodeError as exc:
        print(
            f"[verify_manifest_http_payload] {manifest_path} is not valid UTF-8: {exc}",
            file=sys.stderr,
        )
        return 3

    try:
        parsed = json.loads(raw_text)
    except json.JSONDecodeError as exc:
        print(
            f"[verify_manifest_http_payload] JSON parse failed for {manifest_path}: {exc}",
            file=sys.stderr,
        )
        return 4

    if not isinstance(parsed, dict):
        print(
            f"[verify_manifest_http_payload] top-level JSON value must be an object, got {type(parsed).__name__}",
            file=sys.stderr,
        )
        return 5

    missing = [key for key in REQUIRED_TOP_KEYS if key not in parsed]
    if missing:
        print(
            f"[verify_manifest_http_payload] manifest is missing required top-level keys: {missing}",
            file=sys.stderr,
        )
        return 6

    chunks = parsed.get("chunks")
    if not isinstance(chunks, list) or not chunks:
        print(
            "[verify_manifest_http_payload] manifest.chunks must be a non-empty array",
            file=sys.stderr,
        )
        return 7

    for index, chunk in enumerate(chunks):
        if not isinstance(chunk, dict):
            print(
                f"[verify_manifest_http_payload] chunks[{index}] is not an object",
                file=sys.stderr,
            )
            return 8
        if "id" not in chunk:
            print(
                f"[verify_manifest_http_payload] chunks[{index}] is missing 'id'",
                file=sys.stderr,
            )
            return 9

    # Round-trip to confirm encode-equivalence; Roblox JSONDecode rejects values
    # that contain NaN/Inf, so reject them here too.
    try:
        round_tripped = json.dumps(parsed, allow_nan=False)
    except ValueError as exc:
        print(
            f"[verify_manifest_http_payload] manifest contains non-finite values rejected by JSONDecode: {exc}",
            file=sys.stderr,
        )
        return 10

    print(
        f"[verify_manifest_http_payload] OK schemaVersion={parsed.get('schemaVersion')!r} "
        f"chunks={len(chunks)} size_mb={size_mb:.2f} round_trip_bytes={len(round_tripped)}"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path, help="Path to compiled manifest JSON")
    args = parser.parse_args()
    return verify_manifest_payload(args.manifest)


if __name__ == "__main__":
    raise SystemExit(main())
