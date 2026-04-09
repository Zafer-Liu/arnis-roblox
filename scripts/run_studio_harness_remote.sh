#!/usr/bin/env bash
set -euo pipefail

# Default the telemetry family selector the same way run_studio_harness.sh does
# so that callers who only invoke the remote wrapper don't have to pre-export
# it. Without this default, `set -u` trips the first time the variable is
# referenced downstream (line ~531).
export ARNIS_TELEMETRY_FAMILIES="${ARNIS_TELEMETRY_FAMILIES:-client_perf}"

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOCAL_ARNIS_DIR="$ROOT_DIR"
COMMON_GIT_DIR="$(git -C "$ROOT_DIR" rev-parse --git-common-dir)"
DEFAULT_VSYNC_DIR="$(cd "$(dirname "$COMMON_GIT_DIR")/.." && pwd)/vertigo-sync"
LOCAL_VSYNC_DIR="${VSYNC_REPO_DIR:-$DEFAULT_VSYNC_DIR}"
LOCAL_REMOTE_CONFIG="$ROOT_DIR/scripts/remote_studio_profiles.local.sh"
EXAMPLE_REMOTE_CONFIG="$ROOT_DIR/scripts/remote_studio_profiles.example.sh"

if [[ -f "$LOCAL_REMOTE_CONFIG" ]]; then
  # shellcheck disable=SC1090
  source "$LOCAL_REMOTE_CONFIG"
fi

REMOTE_PROFILE="${ARNIS_REMOTE_STUDIO_PROFILE:-tertiary}"
REMOTE_HOME_TOKEN="__REMOTE_HOME__"
DEFAULT_REMOTE_ROOT="$REMOTE_HOME_TOKEN/.codex-remote-studio"
DEFAULT_REMOTE_ARNIS_BASE="$REMOTE_HOME_TOKEN/Projects/arnis-roblox"
DEFAULT_REMOTE_VSYNC_BASE="$REMOTE_HOME_TOKEN/Projects/vertigo-sync"

resolve_profile_value() {
  local base_name="$1"
  local profile_name="$2"
  local fallback="$3"
  local profile_key
  profile_key="$(printf '%s' "$profile_name" | tr '[:lower:]-.' '[:upper:]__')"
  local scoped_name="${base_name}_${profile_key}"
  local resolved="${!base_name:-}"
  if [[ -z "$resolved" ]]; then
    resolved="${!scoped_name:-}"
  fi
  if [[ -z "$resolved" ]]; then
    resolved="$fallback"
  fi
  printf '%s' "$resolved"
}

render_rsync_remote_path() {
  case "$1" in
    __REMOTE_HOME__/*)
      printf '~/%s' "${1#__REMOTE_HOME__/}"
      ;;
    *)
      printf '%s' "$1"
      ;;
  esac
}

reset_remote_stage_dir() {
  local remote_dir="$1"
  ssh "$REMOTE_HOST" 'bash -s' -- "$remote_dir" "$REMOTE_ROOT" <<'SH'
set -euo pipefail
expand_remote_path() {
  case "$1" in
    __REMOTE_HOME__/*)
      printf '%s\n' "$HOME/${1#__REMOTE_HOME__/}"
      ;;
    *)
      printf '%s\n' "$1"
      ;;
  esac
}

remote_dir="$(expand_remote_path "$1")"
remote_root="$(expand_remote_path "$2")"
case "$remote_dir" in
  "$remote_root/arnis-roblox"|"$remote_root/vertigo-sync")
    ;;
  *)
    echo "[remote-harness] refusing to reset unexpected remote stage path: $remote_dir" >&2
    exit 1
    ;;
esac
rm -rf "$remote_dir"
mkdir -p "$remote_dir"
SH
}

sync_repo_snapshot() {
  local repo_dir="$1"
  local remote_dir="$2"
  local rsync_remote_dir="$3"
  local manifest
  manifest="$(mktemp)"
  git -C "$repo_dir" ls-files -z --cached --others --exclude-standard > "$manifest"
  reset_remote_stage_dir "$remote_dir"
  rsync -a --from0 --files-from="$manifest" "$repo_dir"/ "$REMOTE_HOST:$rsync_remote_dir/"
  rm -f "$manifest"
}

ensure_remote_parent_dir() {
  local remote_path="$1"
  ssh "$REMOTE_HOST" 'bash -s' -- "$remote_path" <<'SH'
set -euo pipefail
expand_remote_path() {
  case "$1" in
    __REMOTE_HOME__/*)
      printf '%s\n' "$HOME/${1#__REMOTE_HOME__/}"
      ;;
    *)
      printf '%s\n' "$1"
      ;;
  esac
}

remote_path="$(expand_remote_path "$1")"
mkdir -p "$(dirname "$remote_path")"
SH
}

sync_optional_file() {
  local local_path="$1"
  local remote_path="$2"
  local rsync_remote_path=""
  if [[ ! -f "$local_path" ]]; then
    return 0
  fi
  ensure_remote_parent_dir "$remote_path"
  rsync_remote_path="$(render_rsync_remote_path "$remote_path")"
  rsync -a "$local_path" "$REMOTE_HOST:$rsync_remote_path"
}

sync_optional_dir() {
  local local_dir="$1"
  local remote_dir="$2"
  local rsync_remote_dir=""
  if [[ ! -d "$local_dir" ]]; then
    return 0
  fi
  ensure_remote_parent_dir "$remote_dir/.keep"
  rsync_remote_dir="$(render_rsync_remote_path "$remote_dir")"
  rsync -a --delete "$local_dir"/ "$REMOTE_HOST:$rsync_remote_dir/"
}

seed_remote_optional_file_from_base() {
  local remote_source_path="$1"
  local remote_dest_path="$2"
  ssh "$REMOTE_HOST" 'bash -s' -- "$remote_source_path" "$remote_dest_path" <<'SH'
set -euo pipefail
expand_remote_path() {
  case "$1" in
    __REMOTE_HOME__/*)
      printf '%s\n' "$HOME/${1#__REMOTE_HOME__/}"
      ;;
    *)
      printf '%s\n' "$1"
      ;;
  esac
}

remote_source_path="$(expand_remote_path "$1")"
remote_dest_path="$(expand_remote_path "$2")"
if [[ ! -f "$remote_source_path" || -f "$remote_dest_path" ]]; then
  exit 0
fi
mkdir -p "$(dirname "$remote_dest_path")"
cp "$remote_source_path" "$remote_dest_path"
SH
}

REMOTE_HOST="$(resolve_profile_value ARNIS_REMOTE_STUDIO_HOST "$REMOTE_PROFILE" "")"
REMOTE_ROOT="$(resolve_profile_value ARNIS_REMOTE_STUDIO_ROOT "$REMOTE_PROFILE" "$DEFAULT_REMOTE_ROOT")"
REMOTE_ARNIS_BASE="$(resolve_profile_value ARNIS_REMOTE_STUDIO_BASE_ARNIS "$REMOTE_PROFILE" "$DEFAULT_REMOTE_ARNIS_BASE")"
REMOTE_VSYNC_BASE="$(resolve_profile_value ARNIS_REMOTE_STUDIO_BASE_VSYNC "$REMOTE_PROFILE" "$DEFAULT_REMOTE_VSYNC_BASE")"
REMOTE_VSYNC_TARGET_DIR="$(resolve_profile_value ARNIS_REMOTE_STUDIO_VSYNC_TARGET_DIR "$REMOTE_PROFILE" "$REMOTE_VSYNC_BASE/target")"
LOCAL_ARTIFACT_DIR="${ARNIS_REMOTE_STUDIO_ARTIFACT_DIR:-/tmp/arnis-remote-studio}"
LOCAL_MANIFEST_SUMMARY_PATH="$LOCAL_ARNIS_DIR/rust/out/austin-manifest.scene-index.json"
PROOF_SYNC_TAIL_TIMEOUT_SECONDS="${ARNIS_REMOTE_STUDIO_TAIL_TIMEOUT_SECONDS:-20}"
REMOTE_SESSION_OUTPUT_LOG="$(mktemp -t arnis-remote-harness-output)"
LOCAL_REMOTE_SESSION_OUTPUT_PATH="$LOCAL_ARTIFACT_DIR/arnis-remote-harness.stdout.log"
SYNC_STAGE=1
SWIFT_SCREENSHOT=0
SWIFT_SCREENSHOT_WAIT_SECONDS=8

REMOTE_ARNIS_DIR="$REMOTE_ROOT/arnis-roblox"
REMOTE_VSYNC_DIR="$REMOTE_ROOT/vertigo-sync"
REMOTE_MANIFEST_SUMMARY_PATH="$REMOTE_ARNIS_DIR/rust/out/austin-manifest.scene-index.json"
REMOTE_HARNESS_PGID_FILE="$REMOTE_ARNIS_DIR/.arnis-remote-harness.pgid"
REMOTE_HARNESS_LOCK_DIR="$REMOTE_ARNIS_DIR/.arnis-studio-harness.lock"
REMOTE_HARNESS_STDOUT_LOG="$REMOTE_ARNIS_DIR/.arnis-remote-harness.stdout.log"
REMOTE_HARNESS_EXIT_FILE="$REMOTE_ARNIS_DIR/.arnis-remote-harness.exit"
RSYNC_REMOTE_ARNIS_DIR="$(render_rsync_remote_path "$REMOTE_ARNIS_DIR")"
RSYNC_REMOTE_VSYNC_DIR="$(render_rsync_remote_path "$REMOTE_VSYNC_DIR")"
REMOTE_HARNESS_ACTIVE=0
REMOTE_VOLATILE_ARTIFACTS=(
  /tmp/arnis-studio-harness-edit.png
  /tmp/arnis-studio-harness-edit.capture.json
  /tmp/arnis-studio-harness-play.png
  /tmp/arnis-studio-harness-play.capture.json
  /tmp/arnis-scene-fidelity-edit.json
  /tmp/arnis-scene-fidelity-edit.html
  /tmp/arnis-scene-fidelity-play.json
  /tmp/arnis-scene-fidelity-play.html
  /tmp/arnis-scene-parity.json
  /tmp/arnis-scene-parity.html
  /tmp/arnis-preview-plugin-state.json
  /tmp/arnis-preview-telemetry-summary.txt
)

stop_remote_harness_if_active() {
  if [[ $REMOTE_HARNESS_ACTIVE -ne 1 ]]; then
    return 0
  fi
  ssh "$REMOTE_HOST" 'bash -s' -- "$REMOTE_ARNIS_DIR" "$REMOTE_VSYNC_TARGET_DIR" "$REMOTE_HARNESS_PGID_FILE" "$REMOTE_HARNESS_LOCK_DIR" "$REMOTE_HARNESS_EXIT_FILE" <<'SH' >/dev/null 2>&1 || true
set -euo pipefail
expand_remote_path() {
  case "$1" in
    __REMOTE_HOME__/*)
      printf '%s\n' "$HOME/${1#__REMOTE_HOME__/}"
      ;;
    *)
      printf '%s\n' "$1"
      ;;
  esac
}

remote_arnis_dir="$(expand_remote_path "$1")"
remote_vsync_target_dir="$(expand_remote_path "$2")"
remote_harness_pgid_file="$(expand_remote_path "$3")"
remote_harness_lock_dir="$(expand_remote_path "$4")"
remote_harness_exit_file="$(expand_remote_path "$5")"
if [[ -f "$remote_harness_pgid_file" ]]; then
  remote_harness_pgid="$(tr -d '[:space:]' < "$remote_harness_pgid_file" || true)"
  if [[ -n "$remote_harness_pgid" ]]; then
    kill -TERM -- "-$remote_harness_pgid" >/dev/null 2>&1 || true
    sleep 1
    kill -KILL -- "-$remote_harness_pgid" >/dev/null 2>&1 || true
  fi
  rm -f "$remote_harness_pgid_file"
fi
rm -f "$remote_harness_exit_file"
rm -rf "$remote_harness_lock_dir"
SH
}

cleanup_remote_harness() {
  local exit_code="${1:-$?}"
  trap - EXIT INT TERM
  stop_remote_harness_if_active
  rm -f "$REMOTE_SESSION_OUTPUT_LOG"
  exit "$exit_code"
}

reset_local_artifacts() {
  local artifact_path=""
  rm -f "$LOCAL_REMOTE_SESSION_OUTPUT_PATH"
  for artifact_path in "${REMOTE_VOLATILE_ARTIFACTS[@]}"; do
    rm -f "$LOCAL_ARTIFACT_DIR/$(basename "$artifact_path")"
  done
}

reset_remote_proof_artifacts() {
  ssh "$REMOTE_HOST" 'bash -s' -- "${REMOTE_VOLATILE_ARTIFACTS[@]}" <<'SH'
set -euo pipefail
artifact_path=""
for artifact_path in "$@"; do
  rm -f "$artifact_path"
done
SH
}

sync_remote_artifacts() {
  local remote_latest_log=""
  remote_latest_log="$(ssh "$REMOTE_HOST" 'latest=$(ls -1t "$HOME"/Library/Logs/Roblox/*_Studio_*_last.log 2>/dev/null | head -n 1 || true); printf "%s" "$latest"')"
  if [[ -n "$remote_latest_log" ]]; then
    rsync -a "$REMOTE_HOST:$remote_latest_log" "$LOCAL_ARTIFACT_DIR/"
  fi

  local remote_artifact=""
  for remote_artifact in "${REMOTE_VOLATILE_ARTIFACTS[@]}"; do
    rsync -a "$REMOTE_HOST:$remote_artifact" "$LOCAL_ARTIFACT_DIR/" >/dev/null 2>&1 || true
  done
}

sync_remote_session_output() {
  rsync -a "$REMOTE_HOST:$(render_rsync_remote_path "$REMOTE_HARNESS_STDOUT_LOG")" "$REMOTE_SESSION_OUTPUT_LOG" >/dev/null 2>&1 || true
  if [[ -f "$REMOTE_SESSION_OUTPUT_LOG" ]]; then
    cp "$REMOTE_SESSION_OUTPUT_LOG" "$LOCAL_REMOTE_SESSION_OUTPUT_PATH"
  fi
}

authoritative_play_screenshot_present() {
  local capture_png="$LOCAL_ARTIFACT_DIR/arnis-studio-harness-play.png"
  local capture_metadata="$LOCAL_ARTIFACT_DIR/arnis-studio-harness-play.capture.json"
  [[ -f "$capture_png" ]] && [[ -f "$capture_metadata" ]] || return 1
  CAPTURE_METADATA_PATH="$capture_metadata" python3 - <<'PY'
import json
import os
import sys
from pathlib import Path

metadata_path = Path(os.environ["CAPTURE_METADATA_PATH"])
try:
    payload = json.loads(metadata_path.read_text(encoding="utf-8"))
except Exception:
    sys.exit(1)

success = payload.get("success") is True and payload.get("capture_method") in {"window", "rect"}
sys.exit(0 if success else 1)
PY
}

remote_harness_status() {
  ssh "$REMOTE_HOST" 'bash -s' -- "$REMOTE_HARNESS_PGID_FILE" "$REMOTE_HARNESS_EXIT_FILE" <<'SH'
set -euo pipefail
expand_remote_path() {
  case "$1" in
    __REMOTE_HOME__/*)
      printf '%s\n' "$HOME/${1#__REMOTE_HOME__/}"
      ;;
    *)
      printf '%s\n' "$1"
      ;;
  esac
}

remote_harness_pgid_file="$(expand_remote_path "$1")"
remote_harness_exit_file="$(expand_remote_path "$2")"

if [[ -f "$remote_harness_exit_file" ]]; then
  exit_code="$(tr -d '[:space:]' < "$remote_harness_exit_file" || true)"
  printf 'exit:%s\n' "${exit_code:-1}"
  exit 0
fi

if [[ -f "$remote_harness_pgid_file" ]]; then
  remote_harness_pgid="$(tr -d '[:space:]' < "$remote_harness_pgid_file" || true)"
  if [[ -n "$remote_harness_pgid" ]] && kill -0 -- "-$remote_harness_pgid" >/dev/null 2>&1; then
    printf 'running\n'
    exit 0
  fi
fi

printf 'missing\n'
SH
}

remote_proof_signal_detected() {
  if [[ ! -f "$REMOTE_SESSION_OUTPUT_LOG" ]]; then
    return 1
  fi
  rg -q 'play bootstrap trace verdict \(authoritative client bootstrap marker\): valid' "$REMOTE_SESSION_OUTPUT_LOG"
}

remote_completion_signal_detected() {
  if [[ ! -f "$REMOTE_SESSION_OUTPUT_LOG" ]]; then
    return 1
  fi
  rg -q 'main harness flow complete; exiting' "$REMOTE_SESSION_OUTPUT_LOG"
}

run_swift_screenshot_capture() {
  # Defensive wrapper: never allow swift-screenshot failure to abort the
  # overall harness run. Every step is guarded so a missing swiftc, a missing
  # GUI session, or a Screen Recording permission denial just logs a line.
  local log_prefix="[remote-harness] swift screenshot"
  local local_swift_src="$LOCAL_ARNIS_DIR/scripts/capture_studio_sck.swift"

  if [[ ! -f "$local_swift_src" ]]; then
    echo "$log_prefix: skipped (missing local source $local_swift_src)"
    return 0
  fi

  # Sync the source file to the remote so we can compile it there. The
  # persistent remote stage may already have it via sync_repo_snapshot, but
  # ensure the file exists on the remote even under --no-sync before we try
  # to compile.
  local remote_swift_src="$REMOTE_ARNIS_DIR/scripts/capture_studio_sck.swift"
  if ! ensure_remote_parent_dir "$remote_swift_src" 2>/dev/null; then
    echo "$log_prefix: ensure_remote_parent_dir failed"
    return 0
  fi
  local rsync_remote_swift_src
  rsync_remote_swift_src="$(render_rsync_remote_path "$remote_swift_src")"
  if ! rsync -a "$local_swift_src" "$REMOTE_HOST:$rsync_remote_swift_src" >/dev/null 2>&1; then
    echo "$log_prefix: rsync of capture_studio_sck.swift failed"
    return 0
  fi

  local capture_result=""
  local capture_tmp
  capture_tmp="$(mktemp -t arnis-remote-swift-capture)"
  ssh "$REMOTE_HOST" 'bash -s' -- "$remote_swift_src" "$SWIFT_SCREENSHOT_WAIT_SECONDS" >"$capture_tmp" 2>&1 <<'SH' || true
set -u
expand_remote_path() {
  case "$1" in
    __REMOTE_HOME__/*)
      printf '%s\n' "$HOME/${1#__REMOTE_HOME__/}"
      ;;
    *)
      printf '%s\n' "$1"
      ;;
  esac
}

remote_swift_src="$(expand_remote_path "$1")"
wait_seconds="$2"
bin_path="/tmp/capture_sck"
png_path="/tmp/studio_sck.png"
cmd_path="/tmp/arnis-studio-capture.command"

if [[ ! -f "$remote_swift_src" ]]; then
  printf 'error:missing-remote-source:%s\n' "$remote_swift_src"
  exit 0
fi

if ! command -v swiftc >/dev/null 2>&1; then
  printf 'error:swiftc-not-found\n'
  exit 0
fi

# Recompile if binary is missing or older than source.
needs_compile=0
if [[ ! -x "$bin_path" ]]; then
  needs_compile=1
elif [[ "$remote_swift_src" -nt "$bin_path" ]]; then
  needs_compile=1
fi

if [[ "$needs_compile" -eq 1 ]]; then
  if ! swiftc "$remote_swift_src" -parse-as-library -o "$bin_path" \
       -framework Cocoa -framework ScreenCaptureKit >/tmp/capture_sck_build.log 2>&1; then
    printf 'error:swiftc-failed (see /tmp/capture_sck_build.log)\n'
    exit 0
  fi
fi

# Refresh the PNG so we can detect that the new capture actually wrote it.
rm -f "$png_path" 2>/dev/null || true

# Wrapper .command file that activates Studio and runs the binary. Terminal.app
# has Screen Recording permission when launched through the GUI session, so we
# relay the capture through an `open`ed .command file.
cat > "$cmd_path" <<'CMD'
#!/bin/bash
osascript -e 'tell application "RobloxStudio" to activate' >/dev/null 2>&1 || \
  osascript -e 'tell application "Roblox Studio" to activate' >/dev/null 2>&1 || true
sleep 1
/tmp/capture_sck >/tmp/capture_sck_run.log 2>&1
CMD
chmod +x "$cmd_path"

if ! open "$cmd_path" >/dev/null 2>&1; then
  printf 'error:open-command-failed\n'
  exit 0
fi

# Poll for the PNG to appear, up to wait_seconds.
waited=0
while (( waited < wait_seconds )); do
  if [[ -f "$png_path" ]]; then
    size="$(stat -f%z "$png_path" 2>/dev/null || echo 0)"
    if [[ "$size" -gt 0 ]]; then
      printf 'ok:%s:%s\n' "$png_path" "$size"
      exit 0
    fi
  fi
  sleep 1
  waited=$((waited + 1))
done

printf 'error:timeout-waiting-for-png\n'
exit 0
SH
  capture_result="$(tr -d '\r' < "$capture_tmp" | tail -n 1 || true)"
  rm -f "$capture_tmp"

  echo "$log_prefix: remote=$capture_result"

  if [[ "$capture_result" != ok:* ]]; then
    return 0
  fi

  local remote_png="/tmp/studio_sck.png"
  local ts
  ts="$(date +%Y%m%d-%H%M%S)"
  local local_png="/tmp/arnis-studio-harness-swift-${ts}.png"
  if scp "$REMOTE_HOST:$remote_png" "$local_png" >/dev/null 2>&1; then
    echo "$log_prefix: saved $local_png"
  else
    echo "$log_prefix: scp of $remote_png failed"
  fi
  return 0
}

trap 'cleanup_remote_harness' EXIT INT TERM

usage() {
  cat <<EOF
Usage: $(basename "$0") [remote-runner-options] -- [run_studio_harness options]

Runs the existing Studio harness on a remote macOS host after syncing this exact
arnis-roblox worktree and adjacent vertigo-sync snapshot to a persistent remote stage.

Remote runner options:
  --remote-profile PROFILE
                      Remote profile alias. Default: ${ARNIS_REMOTE_STUDIO_PROFILE:-tertiary}
  --remote-host HOST   Remote SSH host. Overrides profile/local config.
  --remote-root PATH   Persistent remote stage root. Overrides profile/local config.
  --no-sync            Reuse the existing remote stage without rsyncing local snapshots.
  --swift-screenshot   After the remote harness session finishes, compile (if
                       needed) and run scripts/capture_studio_sck.swift on the
                       remote host via a GUI-session relay, then scp the
                       resulting /tmp/studio_sck.png back to
                       /tmp/arnis-studio-harness-swift-<timestamp>.png on this
                       machine. Failures are logged but never abort the run.
  --help               Show this help.

Local config:
  Create scripts/remote_studio_profiles.local.sh from the example template:
    $EXAMPLE_REMOTE_CONFIG

Cargo target relocation:
  Set ARNIS_REMOTE_STUDIO_VSYNC_TARGET_SSD_BASE (or its profile-scoped form,
  e.g. ARNIS_REMOTE_STUDIO_VSYNC_TARGET_SSD_BASE_TERTIARY) to relocate the
  remote vertigo-sync cargo target dir under "<ssd-base>/vertigo-sync".
  ARNIS_REMOTE_STUDIO_VSYNC_TARGET_DIR (and its scoped form) still wins if set
  explicitly. If the SSD volume is not mounted at run time, the remote falls
  back to the internal-disk default automatically.

All remaining arguments are forwarded to scripts/run_studio_harness.sh on the remote host.
Example:
  $(basename "$0") --remote-profile tertiary -- --no-play --edit-tests --spec-filter ImportManifestRegistrationChunkTruth.spec
  $(basename "$0") --remote-profile tertiary -- --play --route-catalog PlanetaryRouteBundle.route-catalog --route-lane active --route-step-index 0
EOF
}

HARNESS_ARGS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --remote-profile)
      REMOTE_PROFILE="$2"
      shift 2
      ;;
    --remote-host)
      REMOTE_HOST="$2"
      shift 2
      ;;
    --remote-root)
      REMOTE_ROOT="$2"
      REMOTE_ARNIS_DIR="$REMOTE_ROOT/arnis-roblox"
      REMOTE_VSYNC_DIR="$REMOTE_ROOT/vertigo-sync"
      shift 2
      ;;
    --no-sync)
      SYNC_STAGE=0
      shift
      ;;
    --swift-screenshot)
      SWIFT_SCREENSHOT=1
      shift
      ;;
    --help)
      usage
      exit 0
      ;;
    --)
      shift
      HARNESS_ARGS+=("$@")
      break
      ;;
    *)
      HARNESS_ARGS+=("$1")
      shift
      ;;
  esac
done

detect_requested_route_catalog_name() {
  local arg_index=0
  while [[ $arg_index -lt ${#HARNESS_ARGS[@]} ]]; do
    local arg="${HARNESS_ARGS[$arg_index]}"
    if [[ "$arg" == "--route-catalog" ]]; then
      local next_index=$((arg_index + 1))
      if [[ $next_index -lt ${#HARNESS_ARGS[@]} ]]; then
        printf '%s' "${HARNESS_ARGS[$next_index]}"
        return 0
      fi
    fi
    arg_index=$((arg_index + 1))
  done
  printf '%s' "${ARNIS_ROUTE_CATALOG_NAME:-}"
}

resolve_route_bundle_name() {
  local route_catalog_name="${1:-}"
  if [[ -n "$route_catalog_name" && "$route_catalog_name" == *.route-catalog ]]; then
    printf '%s' "${route_catalog_name%.route-catalog}"
  fi
}

resolve_local_route_bundle_dir() {
  local bundle_name="${1:-}"
  if [[ -n "${ARNIS_ROUTE_BUNDLE_DIR:-}" && -d "${ARNIS_ROUTE_BUNDLE_DIR:-}" ]]; then
    printf '%s' "${ARNIS_ROUTE_BUNDLE_DIR}"
    return 0
  fi
  if [[ "$bundle_name" == "PlanetaryRouteBundle" && -d "/tmp/arnis-local-route-bundle" ]]; then
    printf '%s' "/tmp/arnis-local-route-bundle"
    return 0
  fi
  return 1
}

REQUESTED_ROUTE_CATALOG_NAME="$(detect_requested_route_catalog_name)"
ROUTE_BUNDLE_NAME="$(resolve_route_bundle_name "$REQUESTED_ROUTE_CATALOG_NAME")"
LOCAL_ROUTE_BUNDLE_DIR=""
if [[ -n "$ROUTE_BUNDLE_NAME" ]]; then
  LOCAL_ROUTE_BUNDLE_DIR="$(resolve_local_route_bundle_dir "$ROUTE_BUNDLE_NAME" || true)"
fi

REMOTE_HOST="${REMOTE_HOST:-$(resolve_profile_value ARNIS_REMOTE_STUDIO_HOST "$REMOTE_PROFILE" "")}"
REMOTE_ROOT="${REMOTE_ROOT:-$(resolve_profile_value ARNIS_REMOTE_STUDIO_ROOT "$REMOTE_PROFILE" "$DEFAULT_REMOTE_ROOT")}"
REMOTE_ARNIS_BASE="${REMOTE_ARNIS_BASE:-$(resolve_profile_value ARNIS_REMOTE_STUDIO_BASE_ARNIS "$REMOTE_PROFILE" "$DEFAULT_REMOTE_ARNIS_BASE")}"
REMOTE_VSYNC_BASE="${REMOTE_VSYNC_BASE:-$(resolve_profile_value ARNIS_REMOTE_STUDIO_BASE_VSYNC "$REMOTE_PROFILE" "$DEFAULT_REMOTE_VSYNC_BASE")}"
REMOTE_VSYNC_TARGET_DIR="${REMOTE_VSYNC_TARGET_DIR:-$(resolve_profile_value ARNIS_REMOTE_STUDIO_VSYNC_TARGET_DIR "$REMOTE_PROFILE" "$REMOTE_VSYNC_BASE/target")}"
# Optional SSD relocation: if ARNIS_REMOTE_STUDIO_VSYNC_TARGET_SSD_BASE (or its
# profile-scoped variant) is set, derive the cargo target dir from
# "<ssd-base>/vertigo-sync" so cold builds don't fill the remote internal disk.
# The remote-side build heredoc verifies the SSD is mounted before using it and
# falls back to the internal default if not.
REMOTE_VSYNC_TARGET_SSD_BASE="$(resolve_profile_value ARNIS_REMOTE_STUDIO_VSYNC_TARGET_SSD_BASE "$REMOTE_PROFILE" "")"
REMOTE_VSYNC_TARGET_DIR_FALLBACK="$REMOTE_VSYNC_BASE/target"
if [[ -n "$REMOTE_VSYNC_TARGET_SSD_BASE" ]]; then
  REMOTE_VSYNC_TARGET_DIR="$REMOTE_VSYNC_TARGET_SSD_BASE/vertigo-sync"
fi
REMOTE_ARNIS_DIR="$REMOTE_ROOT/arnis-roblox"
REMOTE_VSYNC_DIR="$REMOTE_ROOT/vertigo-sync"
REMOTE_ROUTE_BUNDLE_DIR=""
if [[ -n "$ROUTE_BUNDLE_NAME" ]]; then
  REMOTE_ROUTE_BUNDLE_DIR="$REMOTE_ARNIS_DIR/.route-bundles/$ROUTE_BUNDLE_NAME"
fi
RSYNC_REMOTE_ARNIS_DIR="$(render_rsync_remote_path "$REMOTE_ARNIS_DIR")"
RSYNC_REMOTE_VSYNC_DIR="$(render_rsync_remote_path "$REMOTE_VSYNC_DIR")"

if [[ -z "$REMOTE_HOST" ]]; then
  PROFILE_ENV_SUFFIX="$(printf '%s' "$REMOTE_PROFILE" | tr '[:lower:]-.' '[:upper:]__')"
  echo "[remote-harness] no remote host configured for profile '$REMOTE_PROFILE'" >&2
  echo "[remote-harness] set --remote-host, export ARNIS_REMOTE_STUDIO_HOST_${PROFILE_ENV_SUFFIX}, or create $LOCAL_REMOTE_CONFIG from $EXAMPLE_REMOTE_CONFIG" >&2
  exit 1
fi

if [[ ! -d "$LOCAL_ARNIS_DIR" ]]; then
  echo "[remote-harness] missing local arnis repo: $LOCAL_ARNIS_DIR" >&2
  exit 1
fi

if [[ ! -d "$LOCAL_VSYNC_DIR" ]]; then
  echo "[remote-harness] missing local vertigo-sync repo: $LOCAL_VSYNC_DIR" >&2
  exit 1
fi

mkdir -p "$LOCAL_ARTIFACT_DIR"
reset_local_artifacts

ssh "$REMOTE_HOST" 'bash -s' -- "$REMOTE_ROOT" "$REMOTE_ARNIS_BASE" "$REMOTE_VSYNC_BASE" <<'SH'
set -euo pipefail
expand_remote_path() {
  case "$1" in
    __REMOTE_HOME__/*)
      printf '%s\n' "$HOME/${1#__REMOTE_HOME__/}"
      ;;
    *)
      printf '%s\n' "$1"
      ;;
  esac
}

remote_root="$(expand_remote_path "$1")"
remote_arnis_base="$(expand_remote_path "$2")"
remote_vsync_base="$(expand_remote_path "$3")"
remote_arnis_dir="$remote_root/arnis-roblox"
remote_vsync_dir="$remote_root/vertigo-sync"

seed_stage() {
  local source_dir="$1"
  local dest_dir="$2"
  if [[ -d "$dest_dir" ]]; then
    return 0
  fi
  if [[ ! -d "$source_dir" ]]; then
    mkdir -p "$dest_dir"
    return 0
  fi
  mkdir -p "$dest_dir"
  if git -C "$source_dir" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git -C "$source_dir" ls-files -z --cached --others --exclude-standard | \
      rsync -a --from0 --files-from=- "$source_dir"/ "$dest_dir"/
    return 0
  fi

  rsync -a --delete \
    --exclude=.git \
    --exclude=.worktrees \
    --exclude=target \
    --exclude=roblox/out \
    --exclude=.DS_Store \
    --exclude='**/__pycache__' \
    "$source_dir"/ "$dest_dir"/
}

mkdir -p "$remote_root"
seed_stage "$remote_arnis_base" "$remote_arnis_dir"
seed_stage "$remote_vsync_base" "$remote_vsync_dir"
SH

if [[ $SYNC_STAGE -eq 1 ]]; then
  sync_repo_snapshot "$LOCAL_ARNIS_DIR" "$REMOTE_ARNIS_DIR" "$RSYNC_REMOTE_ARNIS_DIR"
  sync_repo_snapshot "$LOCAL_VSYNC_DIR" "$REMOTE_VSYNC_DIR" "$RSYNC_REMOTE_VSYNC_DIR"
  sync_optional_file "$LOCAL_MANIFEST_SUMMARY_PATH" "$REMOTE_MANIFEST_SUMMARY_PATH"
  seed_remote_optional_file_from_base "$REMOTE_ARNIS_BASE/rust/out/austin-manifest.scene-index.json" "$REMOTE_MANIFEST_SUMMARY_PATH"
  if [[ -n "$LOCAL_ROUTE_BUNDLE_DIR" && -n "$REMOTE_ROUTE_BUNDLE_DIR" ]]; then
    sync_optional_dir "$LOCAL_ROUTE_BUNDLE_DIR" "$REMOTE_ROUTE_BUNDLE_DIR"
  fi
fi
reset_remote_proof_artifacts

REMOTE_HARNESS_ACTIVE=1
# NOTE: `ssh host bash -s -- "" "foo"` collapses empty-string positional
# args on the remote side (OpenSSH builds the remote command as a
# word-split string, not a quoted argv array), so any empty arg in this
# list would shift every downstream positional by -1 and corrupt the
# argument order. We substitute `__EMPTY__` as a sentinel for any arg
# that may legitimately be empty (REMOTE_ROUTE_BUNDLE_DIR) and decode it
# back to "" on the remote side. This matches how ARNIS_TELEMETRY_FAMILIES
# and other optional args have to be passed over ssh.
POSITIONAL_EMPTY="__EMPTY__"
sanitize_positional() {
  if [[ -z "$1" ]]; then
    printf '%s' "$POSITIONAL_EMPTY"
  else
    printf '%s' "$1"
  fi
}
ssh "$REMOTE_HOST" 'bash -s' -- \
  "$(sanitize_positional "$SYNC_STAGE")" \
  "$(sanitize_positional "$REMOTE_ARNIS_DIR")" \
  "$(sanitize_positional "$REMOTE_VSYNC_DIR")" \
  "$(sanitize_positional "$REMOTE_VSYNC_TARGET_DIR")" \
  "$(sanitize_positional "$REMOTE_VSYNC_TARGET_DIR_FALLBACK")" \
  "$(sanitize_positional "$REMOTE_ROUTE_BUNDLE_DIR")" \
  "$(sanitize_positional "$ARNIS_TELEMETRY_FAMILIES")" \
  "$(sanitize_positional "$REMOTE_HARNESS_PGID_FILE")" \
  "$(sanitize_positional "$REMOTE_HARNESS_STDOUT_LOG")" \
  "$(sanitize_positional "$REMOTE_HARNESS_EXIT_FILE")" \
  "$(sanitize_positional "$REMOTE_HARNESS_LOCK_DIR")" \
  "${HARNESS_ARGS[@]}" <<'SH'
set -euo pipefail
POSITIONAL_EMPTY="__EMPTY__"
decode_positional() {
  if [[ "$1" == "$POSITIONAL_EMPTY" ]]; then
    printf ''
  else
    printf '%s' "$1"
  fi
}
expand_remote_path() {
  case "$1" in
    __REMOTE_HOME__/*)
      printf '%s\n' "$HOME/${1#__REMOTE_HOME__/}"
      ;;
    *)
      printf '%s\n' "$1"
      ;;
  esac
}

sync_stage="$(decode_positional "$1")"
shift
remote_arnis_dir="$(expand_remote_path "$(decode_positional "$1")")"
shift
remote_vsync_dir="$(expand_remote_path "$(decode_positional "$1")")"
shift
remote_vsync_target_dir="$(expand_remote_path "$(decode_positional "$1")")"
shift
remote_vsync_target_dir_fallback="$(expand_remote_path "$(decode_positional "$1")")"
shift
remote_route_bundle_dir="$(expand_remote_path "$(decode_positional "$1")")"
shift
remote_telemetry_families="$(decode_positional "$1")"
shift
remote_harness_pgid_file="$(expand_remote_path "$(decode_positional "$1")")"
shift
remote_harness_stdout_log="$(expand_remote_path "$(decode_positional "$1")")"
shift
remote_harness_exit_file="$(expand_remote_path "$(decode_positional "$1")")"
shift
remote_harness_lock_dir="$(expand_remote_path "$(decode_positional "$1")")"
shift

ensure_remote_stage_ready() {
  local arnis_dir="$1"
  local vsync_dir="$2"
  local hint=""
  if [[ "$sync_stage" -eq 0 ]]; then
    hint="; re-run without --no-sync to seed the remote stage from the current worktree"
  fi
  if [[ ! -f "$arnis_dir/scripts/run_studio_harness.sh" ]]; then
    echo "[remote-harness] missing remote arnis stage at $arnis_dir$hint" >&2
    exit 1
  fi
  if [[ ! -f "$vsync_dir/Cargo.toml" ]]; then
    echo "[remote-harness] missing remote vertigo-sync stage at $vsync_dir$hint" >&2
    exit 1
  fi
}

needs_vsync_build() {
  local repo_dir="$1"
  local target_dir="$2"
  local binary="$target_dir/debug/vsync"
  if [[ ! -x "$binary" ]]; then
    return 0
  fi

  local source_path=""
  for source_path in \
    "$repo_dir/Cargo.toml" \
    "$repo_dir/Cargo.lock" \
    "$repo_dir/build.rs" \
    "$repo_dir/src" \
    "$repo_dir/assets"; do
    if [[ -e "$source_path" ]] && find "$source_path" -type f -newer "$binary" -print -quit | grep -q .; then
      return 0
    fi
  done

  return 1
}

ensure_remote_stage_ready "$remote_arnis_dir" "$remote_vsync_dir"

# If the cargo target dir was relocated to an SSD (e.g. /Volumes/<name>/...),
# verify the volume is mounted before trusting it. Otherwise fall back to the
# internal-disk default so a missing SSD never wedges the build.
if [[ -n "$remote_vsync_target_dir_fallback" \
      && "$remote_vsync_target_dir" != "$remote_vsync_target_dir_fallback" ]]; then
  ssd_ok=1
  case "$remote_vsync_target_dir" in
    /Volumes/*)
      ssd_volume_root="/Volumes/$(printf '%s' "${remote_vsync_target_dir#/Volumes/}" | cut -d/ -f1)"
      if [[ ! -d "$ssd_volume_root" ]] || ! mount | grep -q " on $ssd_volume_root "; then
        ssd_ok=0
      fi
      ;;
  esac
  if [[ $ssd_ok -eq 0 ]]; then
    echo "[remote-harness] SSD path not available, falling back to internal disk" >&2
    remote_vsync_target_dir="$remote_vsync_target_dir_fallback"
  fi
fi
mkdir -p "$remote_vsync_target_dir"

if needs_vsync_build "$remote_vsync_dir" "$remote_vsync_target_dir"; then
CARGO_TARGET_DIR="$remote_vsync_target_dir" \
  cargo build --manifest-path "$remote_vsync_dir/Cargo.toml" --bin vsync >/dev/null
fi

cd "$remote_arnis_dir"
rm -f "$remote_harness_pgid_file" "$remote_harness_stdout_log" "$remote_harness_exit_file"
cat > "$remote_arnis_dir/.arnis-remote-harness-launch.sh" <<EOF
#!/usr/bin/env bash
set -u
cd "$remote_arnis_dir"
status=0
HARNESS_LOCK_DIR="$remote_harness_lock_dir" \\
VSYNC_REPO_DIR="$remote_vsync_dir" \\
VSYNC_BIN="$remote_vsync_target_dir/debug/vsync" \\
ARNIS_ROUTE_BUNDLE_DIR="$remote_route_bundle_dir" \\
ARNIS_TELEMETRY_FAMILIES="$remote_telemetry_families" \\
ARNIS_GUI_SESSION_CAPTURE="\${ARNIS_GUI_SESSION_CAPTURE:-1}" \\
ARNIS_PARENT_WATCHDOG=0 \\
bash "$remote_arnis_dir/scripts/run_studio_harness.sh" "\$@" || status=\$?
printf '%s\n' "\$status" > "$remote_harness_exit_file"
rm -f "$remote_harness_pgid_file"
exit "\$status"
EOF
chmod +x "$remote_arnis_dir/.arnis-remote-harness-launch.sh"
nohup "$remote_arnis_dir/.arnis-remote-harness-launch.sh" "$@" >"$remote_harness_stdout_log" 2>&1 </dev/null &
remote_harness_pid=$!
remote_harness_pgid="$(ps -o pgid= "$remote_harness_pid" | tr -d '[:space:]')"
printf '%s\n' "$remote_harness_pgid" > "$remote_harness_pgid_file"
printf 'launched:%s\n' "$remote_harness_pid"
SH
proof_signal_seen=0
proof_signal_seen_at=0
play_screenshot_fired=0
# Delay between proof_signal detection (Studio rendered world in Play mode)
# and firing the ScreenCaptureKit relay. Gives the world a few extra seconds
# to actually import visible geometry before we snap, so the artifact shows
# buildings/terrain instead of a blank sky.
PLAY_SCREENSHOT_DELAY_SECONDS=8
completion_signal_seen_at=0
wrapper_wait_bounded=0

while true; do
  sync_remote_session_output || true
  if [[ $proof_signal_seen -eq 0 ]] && remote_proof_signal_detected; then
    proof_signal_seen=1
    proof_signal_seen_at="$(date +%s)"
    sync_remote_artifacts || true
  fi

  # Fire the Swift/ScreenCaptureKit relay mid-Play instead of after the
  # harness shuts down. Previously we ran it post-completion, by which
  # point Studio had already exited Play and the artifact was an empty
  # Edit viewport. We only do this when --swift-screenshot was requested.
  if [[ $SWIFT_SCREENSHOT -eq 1 && $proof_signal_seen -eq 1 && $play_screenshot_fired -eq 0 && $completion_signal_seen_at -eq 0 ]]; then
    now_epoch="$(date +%s)"
    if (( now_epoch - proof_signal_seen_at >= PLAY_SCREENSHOT_DELAY_SECONDS )); then
      echo "[remote-harness] firing play-mode screenshot relay (proof+${PLAY_SCREENSHOT_DELAY_SECONDS}s)"
      run_swift_screenshot_capture || true
      play_screenshot_fired=1
    fi
  fi

  if [[ $completion_signal_seen_at -eq 0 ]] && remote_completion_signal_detected; then
    completion_signal_seen_at="$(date +%s)"
    sync_remote_artifacts || true
  fi

  if [[ $completion_signal_seen_at -ne 0 ]]; then
    now_epoch="$(date +%s)"
    if (( now_epoch - completion_signal_seen_at >= PROOF_SYNC_TAIL_TIMEOUT_SECONDS )); then
      echo "[remote-harness] bounded remote cleanup tail exceeded ${PROOF_SYNC_TAIL_TIMEOUT_SECONDS}s after proof completion; stopping wrapper wait" >&2
      wrapper_wait_bounded=1
      break
    fi
  fi

  remote_state="$(remote_harness_status)"
  if [[ "$remote_state" == exit:* || "$remote_state" == "missing" ]]; then
    break
  fi

  sleep 1
done

remote_exit_code=0
sync_remote_session_output || true
remote_state="$(remote_harness_status)"
if [[ "$remote_state" == exit:* ]]; then
  remote_exit_code="${remote_state#exit:}"
elif [[ "$remote_state" == "missing" ]]; then
  remote_exit_code=1
fi

if [[ $wrapper_wait_bounded -eq 1 ]]; then
  stop_remote_harness_if_active
  remote_exit_code=0
fi

REMOTE_HARNESS_ACTIVE=0
sync_remote_artifacts

if [[ $SWIFT_SCREENSHOT -eq 1 && $play_screenshot_fired -eq 0 ]] && ! authoritative_play_screenshot_present; then
  # Fallback: Play-mode capture never fired (e.g. proof_signal was never
  # detected, or --swift-screenshot came through a path that never hit the
  # in-loop branch). Capture post-harness so we at least get the Edit
  # viewport rather than nothing at all.
  run_swift_screenshot_capture || true
fi

if [[ $remote_exit_code -ne 0 ]]; then
  exit "$remote_exit_code"
fi

echo "[remote-harness] remote host: $REMOTE_HOST"
echo "[remote-harness] remote profile: $REMOTE_PROFILE"
echo "[remote-harness] remote arnis dir: $REMOTE_ARNIS_DIR"
echo "[remote-harness] remote vsync dir: $REMOTE_VSYNC_DIR"
echo "[remote-harness] local artifacts: $LOCAL_ARTIFACT_DIR"
