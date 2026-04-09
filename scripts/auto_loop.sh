#!/usr/bin/env bash
# auto_loop.sh — one-shot build → publish → harness → telemetry → audit cycle.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Defaults
SKIP_BUILD=0
SKIP_PUBLISH=0
SKIP_HARNESS=0
DRY_RUN=0
POLL_WINDOW=600
POLL_MAX_ATTEMPTS=12
POLL_INTERVAL_SEC=5
PLACE_FILE="roblox/out/arnis-streaming.rbxl"
UNIVERSE_ID="10006866306"
PLACE_ID="108781748738397"

usage() {
  cat <<'EOF'
Usage: auto_loop.sh [options]

One-shot wrapper that runs the full build → publish → remote-harness →
telemetry-fetch → audit cycle and prints a single-line summary.

Options:
  --skip-build               Skip the vsync build phase
  --skip-publish             Skip the publish_to_roblox.py phase
  --skip-harness             Skip the remote Studio harness phase
  --poll-window SECONDS      Telemetry --since window (default 600)
  --poll-max-attempts N      Telemetry poll attempts cap (default 12)
  --poll-interval-sec N      Seconds between telemetry polls (default 5)
  --place-file PATH          Override place file path
  --universe-id N            Override universe id (default 10006866306)
  --place-id N               Override place id (default 108781748738397)
  --dry-run                  Print phases without executing network/processes
  --help                     Show this help and exit

Phases:
  1/5 build              vsync build streaming.build.project.json
  2/5 publish            publish_to_roblox.py (parses versionNumber=N)
  3/5 harness            run_studio_harness_remote.sh (failures non-fatal)
  4/5 wait-for-telemetry fetch_telemetry.py (poll loop)
  5/5 audit              live_stream_audit.py (final exit code)

Exit codes:
  0   success (audit PASS)
  1   audit FAIL
  2   audit found no records
  3   audit HTTP error
  10  build failed
  11  publish failed
  12  telemetry never landed
EOF
}

# ---------- arg parse ----------
while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-build) SKIP_BUILD=1; shift ;;
    --skip-publish) SKIP_PUBLISH=1; shift ;;
    --skip-harness) SKIP_HARNESS=1; shift ;;
    --poll-window) POLL_WINDOW="$2"; shift 2 ;;
    --poll-max-attempts) POLL_MAX_ATTEMPTS="$2"; shift 2 ;;
    --poll-interval-sec) POLL_INTERVAL_SEC="$2"; shift 2 ;;
    --place-file) PLACE_FILE="$2"; shift 2 ;;
    --universe-id) UNIVERSE_ID="$2"; shift 2 ;;
    --place-id) PLACE_ID="$2"; shift 2 ;;
    --dry-run) DRY_RUN=1; shift ;;
    --help|-h) usage; exit 0 ;;
    *) echo "unknown option: $1" >&2; usage >&2; exit 2 ;;
  esac
done

# ---------- color ----------
if [[ -t 1 ]]; then
  C_RESET=$'\033[0m'; C_BOLD=$'\033[1m'
  C_GREEN=$'\033[32m'; C_RED=$'\033[31m'; C_YELLOW=$'\033[33m'; C_CYAN=$'\033[36m'
else
  C_RESET=""; C_BOLD=""; C_GREEN=""; C_RED=""; C_YELLOW=""; C_CYAN=""
fi

banner() {
  local n="$1" name="$2"
  printf '%s==== [phase %s/5] %s ====%s\n' "${C_BOLD}${C_CYAN}" "$n" "$name" "${C_RESET}"
}

# ---------- env (publish needs ROBLOX_OPEN_CLOUD_API_KEY) ----------
# Don't `source ~/.zshrc` from bash — zsh-only syntax (autoload, functions,
# compinit, etc.) causes bash to crash silently even with `|| true`, because
# parse errors trip `set -e` before the error handler runs. Instead, shell
# out to zsh in a subshell to extract just the one env var we need.
if [[ -z "${ROBLOX_OPEN_CLOUD_API_KEY:-}" && -f "$HOME/.zshrc" && "$DRY_RUN" -eq 0 ]]; then
  if command -v zsh >/dev/null 2>&1; then
    _api_key="$(zsh -c 'source ~/.zshrc >/dev/null 2>&1; printf "%s" "${ROBLOX_OPEN_CLOUD_API_KEY:-}"' 2>/dev/null || true)"
    if [[ -n "${_api_key:-}" ]]; then
      export ROBLOX_OPEN_CLOUD_API_KEY="$_api_key"
    fi
    unset _api_key
  fi
fi

# ---------- timing helpers ----------
RUN_START_EPOCH=$(date +%s)
PHASE_ELAPSED_BUILD=0
PHASE_ELAPSED_PUBLISH=0
PHASE_ELAPSED_HARNESS=0
PHASE_ELAPSED_TELEMETRY=0
PHASE_ELAPSED_AUDIT=0
PHASE_RESULT_BUILD="?"
PHASE_RESULT_PUBLISH="?"
PHASE_RESULT_HARNESS="?"
PHASE_RESULT_TELEMETRY="?"
PHASE_RESULT_AUDIT="?"

now() { date +%s; }
fmt_dur() {
  local s=$1
  if (( s >= 60 )); then
    printf '%dm%02ds' $((s/60)) $((s%60))
  else
    printf '%ds' "$s"
  fi
}

# ---------- state ----------
PUBLISH_VERSION=""
HARNESS_FAILED=0
TELEMETRY_OK=0
AUDIT_EXIT=0
AUDIT_LABEL="skipped"

PUBLISH_LOG=""
trap '[[ -n "$PUBLISH_LOG" && -f "$PUBLISH_LOG" ]] && rm -f "$PUBLISH_LOG"' EXIT

cd "$REPO_ROOT"

# ============================================================
# phase 1/5 — build
# ============================================================
banner 1 "build (vsync)"
p_start=$(now)
if [[ "$SKIP_BUILD" -eq 1 ]]; then
  echo "[skip] --skip-build set"
  PHASE_RESULT_BUILD="skip"
elif [[ "$DRY_RUN" -eq 1 ]]; then
  echo "[dry-run] vsync --root $REPO_ROOT/roblox build --project streaming.build.project.json --output out/$(basename "$PLACE_FILE")"
  PHASE_RESULT_BUILD="dry"
else
  if vsync --root "$REPO_ROOT/roblox" build \
       --project streaming.build.project.json \
       --output "out/$(basename "$PLACE_FILE")"; then
    PHASE_RESULT_BUILD="ok"
  else
    PHASE_RESULT_BUILD="fail"
    PHASE_ELAPSED_BUILD=$(( $(now) - p_start ))
    printf '%s[FAIL] build phase failed%s\n' "${C_RED}" "${C_RESET}" >&2
    exit 10
  fi
fi
PHASE_ELAPSED_BUILD=$(( $(now) - p_start ))

# ============================================================
# phase 2/5 — publish
# ============================================================
banner 2 "publish (Roblox Open Cloud)"
p_start=$(now)
if [[ "$SKIP_PUBLISH" -eq 1 ]]; then
  echo "[skip] --skip-publish set"
  PHASE_RESULT_PUBLISH="skip"
elif [[ "$DRY_RUN" -eq 1 ]]; then
  echo "[dry-run] python3 scripts/publish_to_roblox.py --place-file $PLACE_FILE --universe-id $UNIVERSE_ID --place-id $PLACE_ID"
  PHASE_RESULT_PUBLISH="dry"
else
  PUBLISH_LOG="$(mktemp -t auto_loop_publish.XXXXXX)"
  if python3 scripts/publish_to_roblox.py \
       --place-file "$PLACE_FILE" \
       --universe-id "$UNIVERSE_ID" \
       --place-id "$PLACE_ID" 2>&1 | tee "$PUBLISH_LOG"; then
    # `set -euo pipefail` + pipefail + a failing grep in a command
    # substitution silently kills the script at the assignment, even
    # though the assignment is followed by a second fallback. Each grep
    # gets its own `|| true` so an empty match only leaves the variable
    # empty instead of aborting the whole auto-loop.
    PUBLISH_VERSION="$(grep -E '^versionNumber=' "$PUBLISH_LOG" 2>/dev/null | tail -n1 | cut -d= -f2 | tr -d '[:space:]' || true)"
    if [[ -z "$PUBLISH_VERSION" ]]; then
      PUBLISH_VERSION="$(grep -Eo 'versionNumber=[0-9]+' "$PUBLISH_LOG" 2>/dev/null | tail -n1 | cut -d= -f2 || true)"
    fi
    PHASE_RESULT_PUBLISH="ok"
    [[ -n "$PUBLISH_VERSION" ]] && echo "[publish] versionNumber=$PUBLISH_VERSION"
  else
    PHASE_RESULT_PUBLISH="fail"
    PHASE_ELAPSED_PUBLISH=$(( $(now) - p_start ))
    printf '%s[FAIL] publish phase failed%s\n' "${C_RED}" "${C_RESET}" >&2
    exit 11
  fi
fi
PHASE_ELAPSED_PUBLISH=$(( $(now) - p_start ))

# ============================================================
# phase 3/5 — harness
# ============================================================
banner 3 "remote Studio harness"
p_start=$(now)
if [[ "$SKIP_HARNESS" -eq 1 ]]; then
  echo "[skip] --skip-harness set"
  PHASE_RESULT_HARNESS="skip"
elif [[ "$DRY_RUN" -eq 1 ]]; then
  echo "[dry-run] bash scripts/run_studio_harness_remote.sh --swift-screenshot -- --small-place --play-wait 180 --takeover"
  PHASE_RESULT_HARNESS="dry"
else
  if bash scripts/run_studio_harness_remote.sh --swift-screenshot -- --small-place --play-wait 180 --takeover; then
    PHASE_RESULT_HARNESS="ok"
  else
    HARNESS_FAILED=1
    PHASE_RESULT_HARNESS="fail"
    printf '%s[WARN] harness exited non-zero; continuing to telemetry%s\n' "${C_YELLOW}" "${C_RESET}" >&2
  fi
fi
PHASE_ELAPSED_HARNESS=$(( $(now) - p_start ))

# ============================================================
# phase 4/5 — wait-for-telemetry
# ============================================================
banner 4 "wait-for-telemetry"
p_start=$(now)
if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "[dry-run] python3 scripts/fetch_telemetry.py --limit 5 --since $POLL_WINDOW (poll up to $POLL_MAX_ATTEMPTS x ${POLL_INTERVAL_SEC}s)"
  PHASE_RESULT_TELEMETRY="dry"
else
  attempt=0
  while (( attempt < POLL_MAX_ATTEMPTS )); do
    attempt=$((attempt + 1))
    printf '[telemetry] attempt %d/%d (since=%ss)\n' "$attempt" "$POLL_MAX_ATTEMPTS" "$POLL_WINDOW"
    # Run fetch_telemetry.py and capture both output AND exit code. The python
    # tool exits 0 when at least one record matches --since, 2 when no records
    # match, and 1 on transport/JSON errors. The previous grep-based heuristic
    # looked for literal tokens ("recordId", "placeVersion", etc) that never
    # appear in the rendered table (its headers read "runId" / "place v"),
    # so even a successful fetch with fresh rows was reported as "no telemetry
    # landed". Anchor detection on the exit code plus presence of at least
    # one data row (a non-header, non-bar line that contains "|").
    set +e
    fetch_out="$(python3 scripts/fetch_telemetry.py --limit 5 --since "$POLL_WINDOW" 2>&1)"
    fetch_rc=$?
    set -e
    printf '%s\n' "$fetch_out"
    if [[ "$fetch_rc" -eq 0 ]]; then
      # A populated table has at least one data row with "|" that is not
      # the header ("runId") or a bar line ("====").
      data_rows="$(printf '%s\n' "$fetch_out" \
        | grep -F '|' \
        | grep -Ev 'runId|=====' \
        || true)"
      if [[ -n "$data_rows" ]]; then
        TELEMETRY_OK=1
        PHASE_RESULT_TELEMETRY="ok"
        break
      fi
    fi
    if (( attempt < POLL_MAX_ATTEMPTS )); then
      sleep "$POLL_INTERVAL_SEC"
    fi
  done
  if [[ "$TELEMETRY_OK" -ne 1 ]]; then
    PHASE_RESULT_TELEMETRY="fail"
    PHASE_ELAPSED_TELEMETRY=$(( $(now) - p_start ))
    printf '%s[FAIL] no telemetry landed within %d attempts%s\n' "${C_RED}" "$POLL_MAX_ATTEMPTS" "${C_RESET}" >&2
    exit 12
  fi
fi
PHASE_ELAPSED_TELEMETRY=$(( $(now) - p_start ))

# ============================================================
# phase 5/5 — audit
# ============================================================
banner 5 "live stream audit"
p_start=$(now)
if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "[dry-run] python3 scripts/live_stream_audit.py --since-seconds $POLL_WINDOW --allow-failures"
  PHASE_RESULT_AUDIT="dry"
  AUDIT_LABEL="dry"
else
  set +e
  python3 scripts/live_stream_audit.py --since-seconds "$POLL_WINDOW" --allow-failures
  AUDIT_EXIT=$?
  set -e
  case "$AUDIT_EXIT" in
    0) AUDIT_LABEL="PASS"; PHASE_RESULT_AUDIT="ok" ;;
    1) AUDIT_LABEL="FAIL"; PHASE_RESULT_AUDIT="fail" ;;
    2) AUDIT_LABEL="NO-RECORDS"; PHASE_RESULT_AUDIT="fail" ;;
    3) AUDIT_LABEL="HTTP-ERROR"; PHASE_RESULT_AUDIT="fail" ;;
    *) AUDIT_LABEL="exit=$AUDIT_EXIT"; PHASE_RESULT_AUDIT="fail" ;;
  esac
fi
PHASE_ELAPSED_AUDIT=$(( $(now) - p_start ))

# ============================================================
# summary
# ============================================================
TOTAL=$(( $(now) - RUN_START_EPOCH ))
echo
printf '%s---- per-phase elapsed ----%s\n' "${C_BOLD}" "${C_RESET}"
printf '  %-10s %-6s %s\n' "build"     "$PHASE_RESULT_BUILD"     "$(fmt_dur "$PHASE_ELAPSED_BUILD")"
printf '  %-10s %-6s %s\n' "publish"   "$PHASE_RESULT_PUBLISH"   "$(fmt_dur "$PHASE_ELAPSED_PUBLISH")"
printf '  %-10s %-6s %s\n' "harness"   "$PHASE_RESULT_HARNESS"   "$(fmt_dur "$PHASE_ELAPSED_HARNESS")"
printf '  %-10s %-6s %s\n' "telemetry" "$PHASE_RESULT_TELEMETRY" "$(fmt_dur "$PHASE_ELAPSED_TELEMETRY")"
printf '  %-10s %-6s %s\n' "audit"     "$PHASE_RESULT_AUDIT"     "$(fmt_dur "$PHASE_ELAPSED_AUDIT")"

build_s="$PHASE_RESULT_BUILD"
publish_s="$PHASE_RESULT_PUBLISH"
[[ -n "$PUBLISH_VERSION" ]] && publish_s="v${PUBLISH_VERSION}"
harness_s="$PHASE_RESULT_HARNESS"
telemetry_s="$PHASE_RESULT_TELEMETRY"

summary_color="${C_GREEN}"
final_exit="$AUDIT_EXIT"
if [[ "$DRY_RUN" -eq 1 ]]; then
  final_exit=0
  summary_color="${C_CYAN}"
elif [[ "$AUDIT_EXIT" -ne 0 ]]; then
  summary_color="${C_RED}"
fi

printf '%sauto-loop finished: build=%s publish=%s harness=%s telemetry=%s audit=%s total=%s%s\n' \
  "${summary_color}" \
  "$build_s" "$publish_s" "$harness_s" "$telemetry_s" "$AUDIT_LABEL" \
  "$(fmt_dur "$TOTAL")" \
  "${C_RESET}"

if [[ "$HARNESS_FAILED" -eq 1 && "$final_exit" -eq 0 ]]; then
  printf '%s[WARN] harness exited non-zero earlier but audit passed; exiting 0%s\n' \
    "${C_YELLOW}" "${C_RESET}"
fi

exit "$final_exit"
