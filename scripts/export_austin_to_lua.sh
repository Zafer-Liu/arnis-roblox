#!/usr/bin/env bash
set -euo pipefail

# Convenience wrapper to:
#  1) Fetch real OSM data for an Austin bbox via Overpass
#  2) Run the full Rust pipeline + exporter
#  3) Convert the JSON manifest into sharded Roblox Lua ModuleScripts
#
# Usage (from repo root):
#   bash scripts/export_austin_to_lua.sh
#   bash scripts/export_austin_to_lua.sh --yolo
#   bash scripts/export_austin_to_lua.sh --terrain-cell-size 2
#   bash scripts/export_austin_to_lua.sh --profile high --satellite
#   bash scripts/export_austin_to_lua.sh --emit runtime
#   bash scripts/export_austin_to_lua.sh --emit runtime,preview
#
# Default behavior:
#   Uses the default shared Austin fidelity profile from export_austin_from_osm.sh unless
#   explicit fidelity arguments are supplied on the command line. Emits all
#   downstream derivatives unless --emit is used to bound the refresh scope.
#
# Outputs:
#   rust/data/austin_overpass.json
#   rust/out/austin-manifest.sqlite
#   roblox/src/ServerStorage/SampleData/AustinManifestIndex.lua
#   roblox/src/ServerStorage/SampleData/AustinManifestChunks/
#   roblox/src/ServerScriptService/StudioPreview/AustinPreviewManifestIndex.lua
#   roblox/src/ServerScriptService/StudioPreview/AustinPreviewManifestChunks/

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
DATA_DIR="$RUST_DIR/data"
OUT_DIR="$RUST_DIR/out"
SAMPLE_DATA_DIR="$ROOT_DIR/roblox/src/ServerStorage/SampleData"
PREVIEW_DIR="$ROOT_DIR/roblox/src/ServerScriptService/StudioPreview"
RUNTIME_SHARD_MAX_BYTES="${AUSTIN_RUNTIME_SHARD_MAX_BYTES:-0}"
RUNTIME_CHUNKS_PER_SHARD="${AUSTIN_RUNTIME_CHUNKS_PER_SHARD:-8}"

mkdir -p "$DATA_DIR" "$OUT_DIR" "$SAMPLE_DATA_DIR" "$PREVIEW_DIR"

DEFAULT_FIDELITY_ARGS=()
explicit_fidelity=0
explicit_json_out=0
emit_targets="all"
compile_args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    "--emit")
      emit_targets="${2:-}"
      if [[ -z "$emit_targets" ]]; then
        echo "[export_austin_to_lua] --emit requires a comma-separated target list" >&2
        exit 1
      fi
      shift 2
      ;;
    "--profile"|"--yolo"|"--terrain-cell-size"|"--satellite")
      explicit_fidelity=1
      compile_args+=("$1")
      if [[ "$1" != "--yolo" && "$1" != "--satellite" ]]; then
        if [[ $# -lt 2 ]]; then
          echo "[export_austin_to_lua] $1 requires a value" >&2
          exit 1
        fi
        compile_args+=("$2")
        shift 2
      else
        shift 1
      fi
      ;;
    "--out")
      explicit_json_out=1
      compile_args+=("$1")
      if [[ $# -lt 2 ]]; then
        echo "[export_austin_to_lua] --out requires a value" >&2
        exit 1
      fi
      compile_args+=("$2")
      shift 2
      ;;
    *)
      compile_args+=("$1")
      shift 1
      ;;
  esac
done

emit_runtime=0
emit_preview=0
emit_harness=0
emit_verify=0

IFS=',' read -r -a emit_list <<<"$emit_targets"
for raw_target in "${emit_list[@]}"; do
  target="$(printf '%s' "$raw_target" | tr '[:upper:]' '[:lower:]' | xargs)"
  case "$target" in
    "all")
      emit_runtime=1
      emit_preview=1
      emit_harness=1
      emit_verify=1
      ;;
    "runtime"|"sample")
      emit_runtime=1
      ;;
    "preview")
      emit_preview=1
      ;;
    "harness")
      emit_harness=1
      ;;
    "verify")
      emit_verify=1
      ;;
    *)
      echo "[export_austin_to_lua] unsupported --emit target: $raw_target" >&2
      exit 1
      ;;
  esac
done

if [[ $emit_preview -eq 1 || $emit_harness -eq 1 || $emit_verify -eq 1 ]]; then
  emit_runtime=1
fi

if [[ $explicit_fidelity -eq 0 ]]; then
  DEFAULT_FIDELITY_ARGS=("--terrain-cell-size" "2")
  echo "[export_austin_to_lua] Fetching OSM + exporting manifest with the default shared Austin cell=2 path (satellite opt-in only)..."
else
  echo "[export_austin_to_lua] Fetching OSM + exporting manifest with explicit fidelity arguments..."
fi

echo "[export_austin_to_lua] Emit targets: runtime=$emit_runtime preview=$emit_preview harness=$emit_harness verify=$emit_verify"

echo "=== Fetching Overture building footprints ==="
python3 "$ROOT_DIR/scripts/fetch_overture_buildings.py" || echo "Warning: Overture fetch failed, continuing with OSM only"

if [[ $explicit_json_out -eq 0 ]]; then
  rm -f "$OUT_DIR/austin-manifest.json"
fi

export_from_osm_args=("${DEFAULT_FIDELITY_ARGS[@]}")
if [[ ${#compile_args[@]} -gt 0 ]]; then
  export_from_osm_args+=("${compile_args[@]}")
fi

bash "$ROOT_DIR/scripts/export_austin_from_osm.sh" "${export_from_osm_args[@]}"

if [[ $explicit_json_out -eq 0 ]]; then
  rm -f "$OUT_DIR/austin-manifest.json"
fi

if [[ $emit_runtime -eq 1 ]]; then
  echo "[export_austin_to_lua] Converting SQLite manifest store to sharded Lua modules..."
  runtime_args=(
    --manifest-sqlite "$OUT_DIR/austin-manifest.sqlite"
    --output-dir "$SAMPLE_DATA_DIR"
    --index-name "AustinManifestIndex"
    --shard-folder "AustinManifestChunks"
    --chunks-per-shard "$RUNTIME_CHUNKS_PER_SHARD"
  )
  if [[ "$RUNTIME_SHARD_MAX_BYTES" -gt 0 ]]; then
    runtime_args+=(--max-bytes "$RUNTIME_SHARD_MAX_BYTES")
  fi
  "$RUST_DIR/target/debug/arbx_cli" emit-runtime-lua "${runtime_args[@]}"
fi

if [[ $emit_preview -eq 1 ]]; then
  echo "[export_austin_to_lua] Refreshing Studio preview from current Austin sample-data shards..."
  python3 "$ROOT_DIR/scripts/refresh_preview_from_sample_data.py"
fi

if [[ $emit_harness -eq 1 ]]; then
  echo "[export_austin_to_lua] Refreshing bounded runtime harness sample-data from current Austin shards..."
  python3 "$ROOT_DIR/scripts/refresh_runtime_harness_from_sample_data.py"
fi

if [[ $emit_verify -eq 1 ]]; then
  echo "[export_austin_to_lua] Verifying generated Austin sample-data + preview assets..."
  python3 "$ROOT_DIR/scripts/verify_generated_austin_assets.py"
fi

echo "[export_austin_to_lua] Done. Sharded manifests written to $SAMPLE_DATA_DIR and $PREVIEW_DIR"
