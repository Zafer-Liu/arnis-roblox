#!/usr/bin/env bash
# Capture Roblox Studio viewport via ScreenCaptureKit (macOS 14+).
# Must be run through the GUI session (Terminal.app) for screen recording permission.
# Usage: bash scripts/capture_studio_window.sh [output.png]
set -euo pipefail

OUTPUT="${1:-/tmp/studio_capture.png}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SWIFT_SRC="$ROOT_DIR/scripts/capture_studio_sck.swift"
SWIFT_BIN="/tmp/capture_studio_sck"

# Compile if needed
if [[ ! -f "$SWIFT_BIN" ]] || [[ "$SWIFT_SRC" -nt "$SWIFT_BIN" ]]; then
    swiftc "$SWIFT_SRC" -parse-as-library -o "$SWIFT_BIN" -framework Cocoa -framework ScreenCaptureKit
fi

# Activate Studio and capture
osascript -e 'tell application "Roblox Studio" to activate'
sleep 2
"$SWIFT_BIN"

# Move to requested output
if [[ -f /tmp/studio_sck.png ]]; then
    mv /tmp/studio_sck.png "$OUTPUT"
    echo "Captured to $OUTPUT"
else
    echo "Capture failed" >&2
    exit 1
fi
