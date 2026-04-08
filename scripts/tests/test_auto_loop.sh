#!/usr/bin/env bash
# Smoke tests for auto_loop.sh — no network, no shellspec.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AUTO_LOOP="$SCRIPT_DIR/../auto_loop.sh"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[[ -x "$AUTO_LOOP" ]] || fail "auto_loop.sh not executable: $AUTO_LOOP"

# 1) syntax check
bash -n "$AUTO_LOOP" || fail "bash -n failed"
pass "bash -n auto_loop.sh"

# 2) --help prints Usage and exits 0
help_out="$(bash "$AUTO_LOOP" --help)"
echo "$help_out" | grep -q "Usage" || fail "--help missing 'Usage'"
pass "--help prints Usage"

# 3) full --dry-run lists all 5 phases and exits 0
dry_out="$(bash "$AUTO_LOOP" --dry-run 2>&1)"
for label in \
  "phase 1/5" "phase 2/5" "phase 3/5" "phase 4/5" "phase 5/5" \
  "build" "publish" "harness" "wait-for-telemetry" "audit"
do
  echo "$dry_out" | grep -q "$label" || fail "--dry-run missing '$label'"
done
echo "$dry_out" | grep -q "auto-loop finished:" || fail "--dry-run missing summary"
pass "--dry-run shows all 5 phases and summary"

# 4) skip-build/publish/harness + dry-run only executes telemetry + audit work
skip_out="$(bash "$AUTO_LOOP" --skip-build --skip-publish --skip-harness --dry-run 2>&1)"
echo "$skip_out" | grep -q "wait-for-telemetry" || fail "skipped run missing telemetry phase"
echo "$skip_out" | grep -q "live stream audit" || fail "skipped run missing audit phase"
echo "$skip_out" | grep -q "fetch_telemetry.py" || fail "skipped run did not announce telemetry call"
echo "$skip_out" | grep -q "live_stream_audit.py" || fail "skipped run did not announce audit call"
# Make sure the build/publish/harness phases were skipped (no dry-run command lines)
if echo "$skip_out" | grep -q "vsync .* build"; then
  fail "skipped run unexpectedly announced vsync build"
fi
if echo "$skip_out" | grep -q "publish_to_roblox.py"; then
  fail "skipped run unexpectedly announced publish_to_roblox.py"
fi
if echo "$skip_out" | grep -q "run_studio_harness_remote.sh"; then
  fail "skipped run unexpectedly announced harness"
fi
pass "skip-* + dry-run scopes to telemetry + audit"

echo "all auto_loop.sh smoke tests passed"
