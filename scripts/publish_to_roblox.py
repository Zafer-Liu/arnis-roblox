#!/usr/bin/env python3
"""Publish a Roblox place file (.rbxlx / .rbxl) to an existing experience via
the Roblox Open Cloud REST API.

This tool is deliberately a thin wrapper around the legacy "place publish"
endpoint so the repo has a stdlib-only automation hook that matches the
two-part deployment architecture:

    small scripts-only place file  --(this script)-->  Roblox experience
    compiled manifest JSON          --(external host)-->  HttpService:GetAsync

Endpoint used (see docs/roblox-deployment.md):

    POST https://apis.roblox.com/universes/v1/{universeId}/places/{placeId}/versions
        ?versionType=Published
    Headers:
        x-api-key: <ROBLOX_OPEN_CLOUD_API_KEY>
        Content-Type: application/octet-stream
    Body: raw .rbxlx / .rbxl bytes

The API key must have the "Place Management" scope and must be scoped to the
target universe. This script never prints the key and never writes it to disk.

Usage:
    export ROBLOX_OPEN_CLOUD_API_KEY=...
    python3 scripts/publish_to_roblox.py \
        --place-file roblox/out/arnis-scripts-only.rbxlx \
        --universe-id 1234567890 \
        --place-id 9876543210

    # Preview what would be uploaded without hitting the network:
    python3 scripts/publish_to_roblox.py \
        --place-file roblox/out/arnis-scripts-only.rbxlx \
        --universe-id 1234567890 \
        --place-id 9876543210 \
        --dry-run

No third-party dependencies: uses only urllib from the standard library so
this can run in CI without a virtualenv.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Optional
from urllib import error as urllib_error
from urllib import request as urllib_request


API_KEY_ENV_VAR = "ROBLOX_OPEN_CLOUD_API_KEY"
OPEN_CLOUD_HOST = "https://apis.roblox.com"
CONTENT_TYPE_BINARY = "application/octet-stream"
CONTENT_TYPE_XML = "application/xml"
DEFAULT_VERSION_TYPE = "Published"


@dataclass(frozen=True)
class PublishRequest:
    """Fully-resolved description of a single Open Cloud publish call."""

    place_file: Path
    universe_id: int
    place_id: int
    version_type: str
    url: str
    content_type: str
    body_size_bytes: int

    def preview_lines(self) -> list[str]:
        size_mb = self.body_size_bytes / (1024 * 1024)
        return [
            "[publish_to_roblox] dry-run preview:",
            f"  place_file      = {self.place_file}",
            f"  universe_id     = {self.universe_id}",
            f"  place_id        = {self.place_id}",
            f"  version_type    = {self.version_type}",
            f"  url             = {self.url}",
            f"  content_type    = {self.content_type}",
            f"  body_size_bytes = {self.body_size_bytes} ({size_mb:.2f} MiB)",
            "  auth_header     = x-api-key: <redacted>",
        ]


def build_url(universe_id: int, place_id: int, version_type: str) -> str:
    return (
        f"{OPEN_CLOUD_HOST}/universes/v1/{universe_id}"
        f"/places/{place_id}/versions?versionType={version_type}"
    )


def infer_content_type(place_file: Path) -> str:
    # Roblox Open Cloud accepts both rbxlx (XML) and rbxl (binary). Historically
    # the docs recommend application/octet-stream for both, but application/xml
    # is also accepted for rbxlx. We keep the binary content type as the
    # default so the body is always uploaded byte-for-byte.
    suffix = place_file.suffix.lower()
    if suffix == ".rbxlx":
        return CONTENT_TYPE_XML
    return CONTENT_TYPE_BINARY


def build_publish_request(
    place_file: Path,
    universe_id: int,
    place_id: int,
    version_type: str = DEFAULT_VERSION_TYPE,
) -> PublishRequest:
    if universe_id <= 0:
        raise ValueError(f"universe_id must be positive, got {universe_id}")
    if place_id <= 0:
        raise ValueError(f"place_id must be positive, got {place_id}")
    if not place_file.exists():
        raise FileNotFoundError(f"place file does not exist: {place_file}")
    if not place_file.is_file():
        raise ValueError(f"place file is not a regular file: {place_file}")

    size = place_file.stat().st_size
    return PublishRequest(
        place_file=place_file,
        universe_id=universe_id,
        place_id=place_id,
        version_type=version_type,
        url=build_url(universe_id, place_id, version_type),
        content_type=infer_content_type(place_file),
        body_size_bytes=size,
    )


def execute_publish(
    request: PublishRequest,
    api_key: str,
    *,
    opener: Optional[urllib_request.OpenerDirector] = None,
) -> dict:
    """Perform the HTTP upload. Returns the parsed JSON response body on success.

    Raises RuntimeError with the decoded error body on any non-2xx response.
    """

    body = request.place_file.read_bytes()
    http_request = urllib_request.Request(
        request.url,
        data=body,
        method="POST",
        headers={
            "x-api-key": api_key,
            "Content-Type": request.content_type,
            "Accept": "application/json",
            "User-Agent": "arnis-roblox-publish/1.0",
        },
    )

    try:
        if opener is not None:
            response = opener.open(http_request)
        else:
            response = urllib_request.urlopen(http_request)  # noqa: S310 - trusted host
    except urllib_error.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(
            f"Roblox Open Cloud publish failed: HTTP {exc.code} {exc.reason}\n{detail}"
        ) from exc
    except urllib_error.URLError as exc:
        raise RuntimeError(f"Roblox Open Cloud publish failed: {exc.reason}") from exc

    with response:
        raw = response.read().decode("utf-8", errors="replace")

    try:
        return json.loads(raw)
    except json.JSONDecodeError as exc:
        raise RuntimeError(
            f"Roblox Open Cloud returned non-JSON body (HTTP {response.status}): {raw}"
        ) from exc


def parse_args(argv: Optional[list[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Publish a .rbxlx/.rbxl place file to Roblox via Open Cloud.",
    )
    parser.add_argument(
        "--place-file",
        type=Path,
        required=True,
        help="Path to the .rbxlx or .rbxl place file to upload.",
    )
    parser.add_argument(
        "--universe-id",
        type=int,
        required=True,
        help="Target universe (experience) ID from the creator dashboard.",
    )
    parser.add_argument(
        "--place-id",
        type=int,
        required=True,
        help="Target place ID inside the universe (usually the start place).",
    )
    parser.add_argument(
        "--version-type",
        choices=("Published", "Saved"),
        default=DEFAULT_VERSION_TYPE,
        help="Open Cloud versionType query parameter. 'Published' goes live.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the request that would be made without uploading anything.",
    )
    return parser.parse_args(argv)


def main(argv: Optional[list[str]] = None) -> int:
    args = parse_args(argv)

    try:
        request = build_publish_request(
            place_file=args.place_file,
            universe_id=args.universe_id,
            place_id=args.place_id,
            version_type=args.version_type,
        )
    except (FileNotFoundError, ValueError) as exc:
        print(f"[publish_to_roblox] {exc}", file=sys.stderr)
        return 2

    if args.dry_run:
        for line in request.preview_lines():
            print(line)
        return 0

    api_key = os.environ.get(API_KEY_ENV_VAR, "").strip()
    if not api_key:
        print(
            f"[publish_to_roblox] {API_KEY_ENV_VAR} environment variable is required "
            "(needs an Open Cloud API key with Place Management scope).",
            file=sys.stderr,
        )
        return 3

    try:
        payload = execute_publish(request, api_key)
    except RuntimeError as exc:
        print(f"[publish_to_roblox] {exc}", file=sys.stderr)
        return 4

    version_number = payload.get("versionNumber")
    if version_number is None:
        print(
            "[publish_to_roblox] upload succeeded but response did not include "
            f"versionNumber: {payload}",
        )
        return 0

    print(
        f"[publish_to_roblox] OK place_id={request.place_id} "
        f"universe_id={request.universe_id} versionNumber={version_number}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
