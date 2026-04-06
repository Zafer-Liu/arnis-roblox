#!/usr/bin/env python3
from __future__ import annotations

import argparse
import importlib.util
import json
import re
import shutil
import sqlite3
import subprocess
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = ROOT / "scripts"


def _load_module(module_name: str, path: Path) -> Any:
    scripts_dir = str(path.parent)
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
    spec = importlib.util.spec_from_file_location(module_name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"failed to load module spec from {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


REFRESH_PREVIEW_MODULE = _load_module(
    "refresh_preview_from_sample_data",
    SCRIPTS_DIR / "refresh_preview_from_sample_data.py",
)
FIDELITY_AUDIT_MODULE = _load_module(
    "scene_fidelity_audit",
    SCRIPTS_DIR / "scene_fidelity_audit.py",
)
PARITY_AUDIT_MODULE = _load_module(
    "scene_parity_audit",
    SCRIPTS_DIR / "scene_parity_audit.py",
)


def _load_manifest_meta_from_sqlite(path: Path) -> dict[str, Any]:
    connection = sqlite3.connect(path)
    try:
        row = connection.execute(
            """
            SELECT
                schema_version,
                world_name,
                generator,
                source,
                meters_per_stud,
                chunk_size_studs,
                bbox_min_lat,
                bbox_min_lon,
                bbox_max_lat,
                bbox_max_lon,
                total_features,
                notes_json
            FROM manifest_meta
            WHERE singleton_id = 1
            """
        ).fetchone()
    finally:
        connection.close()

    if row is None:
        raise SystemExit(f"manifest store {path} is missing manifest_meta")

    schema_version = row[0]
    REFRESH_PREVIEW_MODULE._require_current_schema_version(schema_version, str(path))
    notes = json.loads(row[11]) if isinstance(row[11], str) and row[11] else {}
    return {
        "schemaVersion": schema_version,
        "meta": {
            "worldName": row[1],
            "generator": row[2],
            "source": row[3],
            "metersPerStud": row[4],
            "chunkSizeStuds": row[5],
            "bbox": {
                "minLat": row[6],
                "minLon": row[7],
                "maxLat": row[8],
                "maxLon": row[9],
            },
            "totalFeatures": row[10],
            "notes": notes,
        },
    }


def _write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")


def _extract_chunk_ids_from_log(log_path: Path, marker: str) -> list[str]:
    payload = FIDELITY_AUDIT_MODULE._parse_latest_marker(log_path, marker)
    scene = payload.get("scene") if isinstance(payload.get("scene"), dict) else {}
    chunk_ids = scene.get("chunkIds")
    if not isinstance(chunk_ids, list) or not chunk_ids:
        raise SystemExit(f"{marker} in {log_path} did not provide scene.chunkIds")
    return [str(chunk_id) for chunk_id in chunk_ids]


def build_route_slice_manifest(
    manifest_sqlite: Path | None,
    chunk_ids: list[str],
    *,
    route_runtime_index: Path | None = None,
) -> dict[str, Any]:
    if manifest_sqlite is not None:
        manifest = _load_manifest_meta_from_sqlite(manifest_sqlite)
        schema_version, source_chunks = REFRESH_PREVIEW_MODULE.load_source_manifest_subset_from_sqlite(
            manifest_sqlite,
            chunk_ids,
        )
        manifest["schemaVersion"] = schema_version
        manifest["chunks"] = [source_chunks[chunk_id] for chunk_id in chunk_ids]
        return manifest
    if route_runtime_index is None:
        raise SystemExit("either manifest_sqlite or route_runtime_index is required")
    return _load_route_runtime_manifest(route_runtime_index, chunk_ids)


def _load_route_runtime_manifest(index_path: Path, chunk_ids: list[str]) -> dict[str, Any]:
    if shutil.which("lua") is None:
        return _load_route_runtime_manifest_without_lua(index_path, chunk_ids)

    lua_program = r"""
local index_path = arg[1]
local requested = {}
for i = 2, #arg do
  requested[arg[i]] = true
end

local function json_escape(value)
  value = value:gsub("\\", "\\\\")
  value = value:gsub("\"", "\\\"")
  value = value:gsub("\b", "\\b")
  value = value:gsub("\f", "\\f")
  value = value:gsub("\n", "\\n")
  value = value:gsub("\r", "\\r")
  value = value:gsub("\t", "\\t")
  return value
end

local function is_array(tbl)
  if type(tbl) ~= "table" then
    return false
  end
  local count = 0
  for key, _ in pairs(tbl) do
    if type(key) ~= "number" or key < 1 or key ~= math.floor(key) then
      return false
    end
    count = count + 1
  end
  for i = 1, count do
    if tbl[i] == nil then
      return false
    end
  end
  return true
end

local function encode(value)
  local value_type = type(value)
  if value_type == "nil" then
    return "null"
  end
  if value_type == "boolean" then
    return value and "true" or "false"
  end
  if value_type == "number" then
    return tostring(value)
  end
  if value_type == "string" then
    return "\"" .. json_escape(value) .. "\""
  end
  if value_type ~= "table" then
    error("unsupported json type: " .. value_type)
  end
  if is_array(value) then
    local items = {}
    for i = 1, #value do
      items[#items + 1] = encode(value[i])
    end
    return "[" .. table.concat(items, ",") .. "]"
  end
  local keys = {}
  for key, _ in pairs(value) do
    keys[#keys + 1] = key
  end
  table.sort(keys, function(a, b)
    return tostring(a) < tostring(b)
  end)
  local items = {}
  for _, key in ipairs(keys) do
    items[#items + 1] = encode(tostring(key)) .. ":" .. encode(value[key])
  end
  return "{" .. table.concat(items, ",") .. "}"
end

local function dirname(path)
  return path:match("^(.*)/[^/]+$") or "."
end

local function basename_without_ext(path)
  local name = path:match("([^/]+)$") or path
  return name:gsub("%.lua$", "")
end

local index = assert(dofile(index_path))
local index_dir = dirname(index_path)
local shard_folder = index.shardFolder or (basename_without_ext(index_path) .. "Chunks")
local shard_dir = index_dir .. "/" .. shard_folder
local chunks = {}

for _, shard_name in ipairs(index.shards or {}) do
  local shard = assert(dofile(shard_dir .. "/" .. shard_name .. ".lua"))
  for _, chunk in ipairs(shard.chunks or {}) do
    if next(requested) == nil or requested[chunk.id] then
      chunks[#chunks + 1] = chunk
    end
  end
end

local chunk_refs = {}
for _, chunk_ref in ipairs(index.chunkRefs or {}) do
  if next(requested) == nil or requested[chunk_ref.id] then
    chunk_refs[#chunk_refs + 1] = chunk_ref
  end
end

io.write(encode({
  schemaVersion = index.schemaVersion,
  meta = index.meta,
  chunkRefs = chunk_refs,
  chunks = chunks,
}))
"""
    result = subprocess.run(
        ["lua", "-", str(index_path), *chunk_ids],
        input=lua_program,
        text=True,
        capture_output=True,
        check=True,
    )
    manifest = json.loads(result.stdout)
    if not isinstance(manifest, dict):
        raise SystemExit(f"route runtime index {index_path} did not produce a manifest object")
    REFRESH_PREVIEW_MODULE._require_current_schema_version(manifest.get("schemaVersion"), str(index_path))
    return manifest


def _load_route_runtime_manifest_without_lua(index_path: Path, chunk_ids: list[str]) -> dict[str, Any]:
    requested = set(chunk_ids)
    index_text = index_path.read_text(encoding="utf-8")
    schema_match = REFRESH_PREVIEW_MODULE.SCHEMA_RE.search(index_text)
    if schema_match is None:
        schema_match = re.search(r'schemaVersion\s*=\s*"(?P<schema>[^"]+)"', index_text)
    if schema_match is None:
        raise SystemExit(f"could not parse schemaVersion from {index_path}")
    schema_version = schema_match.group("schema")
    REFRESH_PREVIEW_MODULE._require_current_schema_version(schema_version, str(index_path))

    meta_text = REFRESH_PREVIEW_MODULE._extract_lua_table(index_text, "meta")
    if meta_text is None:
        raise SystemExit(f"could not parse meta from {index_path}")
    meta = REFRESH_PREVIEW_MODULE._parse_lua_table_value(meta_text)
    if not isinstance(meta, dict):
        raise SystemExit(f"could not parse meta table from {index_path}")

    chunk_refs = []
    for chunk_ref in REFRESH_PREVIEW_MODULE._parse_chunk_ref_entries(index_text):
        chunk_id = chunk_ref.get("id")
        if not isinstance(chunk_id, str):
            continue
        if requested and chunk_id not in requested:
            continue
        chunk_refs.append(chunk_ref)

    shard_folder_match = index_text.split("shardFolder", 1)
    shard_folder = "PlanetaryManifestChunks"
    if len(shard_folder_match) > 1:
        shard_folder_text = shard_folder_match[1]
        if '"' in shard_folder_text:
            shard_folder = shard_folder_text.split('"', 2)[1]

    shard_dir = index_path.parent / shard_folder
    shards_text = REFRESH_PREVIEW_MODULE._extract_lua_table(index_text, "shards")
    if shards_text is None:
        raise SystemExit(f"could not parse shards from {index_path}")
    shard_names = REFRESH_PREVIEW_MODULE._parse_lua_table_value(shards_text)
    if not isinstance(shard_names, list):
        raise SystemExit(f"could not parse shard list from {index_path}")

    chunks = []
    for shard_name in shard_names:
        if not isinstance(shard_name, str):
            continue
        shard_path = shard_dir / f"{shard_name}.lua"
        shard_text = shard_path.read_text(encoding="utf-8")
        chunks_text = REFRESH_PREVIEW_MODULE._extract_lua_table(shard_text, "chunks")
        if chunks_text is None:
            raise SystemExit(f"could not parse chunks from {shard_path}")
        shard_chunks = REFRESH_PREVIEW_MODULE._parse_lua_table_value(chunks_text)
        if not isinstance(shard_chunks, list):
            raise SystemExit(f"could not parse chunks list from {shard_path}")
        for chunk in shard_chunks:
            if not isinstance(chunk, dict):
                continue
            chunk_id = chunk.get("id")
            if not isinstance(chunk_id, str):
                continue
            if requested and chunk_id not in requested:
                continue
            chunks.append(chunk)

    return {
        "schemaVersion": schema_version,
        "meta": meta,
        "chunkRefs": chunk_refs,
        "chunks": chunks,
    }


def build_route_slice_artifacts(
    manifest_sqlite: Path | None,
    edit_log: Path,
    play_log: Path,
    artifact_dir: Path,
    *,
    truth_pack: Path | None = None,
    preview_plugin_state: Path | None = None,
    route_runtime_index: Path | None = None,
) -> dict[str, Any]:
    chunk_ids = _extract_chunk_ids_from_log(edit_log, "ARNIS_SCENE_EDIT")
    manifest = build_route_slice_manifest(
        manifest_sqlite,
        chunk_ids,
        route_runtime_index=route_runtime_index,
    )

    manifest_json = artifact_dir / "route-slice-manifest.json"
    edit_json = artifact_dir / "scene-fidelity-edit.json"
    edit_html = artifact_dir / "scene-fidelity-edit.html"
    play_json = artifact_dir / "scene-fidelity-play.json"
    play_html = artifact_dir / "scene-fidelity-play.html"
    parity_json = artifact_dir / "scene-parity.json"
    parity_html = artifact_dir / "scene-parity.html"
    summary_json = artifact_dir / "route-slice-summary.json"

    _write_json(manifest_json, manifest)

    edit_report = FIDELITY_AUDIT_MODULE.build_report(
        manifest_json,
        edit_log,
        marker="ARNIS_SCENE_EDIT",
        truth_pack=truth_pack,
        preview_plugin_state=preview_plugin_state,
    )
    play_report = FIDELITY_AUDIT_MODULE.build_report(
        manifest_json,
        play_log,
        marker="ARNIS_SCENE_PLAY",
        truth_pack=truth_pack,
        preview_plugin_state=None,
    )
    parity_report = PARITY_AUDIT_MODULE.build_report(edit_report, play_report)

    _write_json(edit_json, edit_report)
    _write_json(play_json, play_report)
    _write_json(parity_json, parity_report)
    FIDELITY_AUDIT_MODULE.write_html_report(edit_report, edit_html)
    FIDELITY_AUDIT_MODULE.write_html_report(play_report, play_html)
    PARITY_AUDIT_MODULE.write_html_report(parity_report, parity_html)

    summary = {
        "chunkIds": chunk_ids,
        "manifestSqlite": str(manifest_sqlite) if manifest_sqlite is not None else None,
        "routeRuntimeIndex": str(route_runtime_index) if route_runtime_index is not None else None,
        "manifestJson": str(manifest_json),
        "editLog": str(edit_log),
        "playLog": str(play_log),
        "editReport": str(edit_json),
        "playReport": str(play_json),
        "parityReport": str(parity_json),
        "edit": {
            "manifestSourceKind": edit_report.get("manifestSourceKind"),
            "manifestSourceName": edit_report.get("manifestSourceName"),
            "rootName": edit_report.get("rootName"),
            "findingCount": len(edit_report.get("findings", [])),
        },
        "play": {
            "manifestSourceKind": play_report.get("manifestSourceKind"),
            "manifestSourceName": play_report.get("manifestSourceName"),
            "rootName": play_report.get("rootName"),
            "findingCount": len(play_report.get("findings", [])),
        },
        "parity": {
            "matching": parity_report.get("summary", {}).get("matching"),
            "mismatched": parity_report.get("summary", {}).get("mismatched"),
            "findingCount": len(parity_report.get("findings", [])),
        },
    }
    _write_json(summary_json, summary)
    return summary


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Build bounded edit/play route-slice fidelity and parity artifacts from authoritative Studio logs."
    )
    parser.add_argument("--manifest-sqlite", type=Path)
    parser.add_argument("--route-runtime-index", type=Path)
    parser.add_argument("--edit-log", type=Path, required=True)
    parser.add_argument("--play-log", type=Path, required=True)
    parser.add_argument("--artifact-dir", type=Path, required=True)
    parser.add_argument("--truth-pack", type=Path)
    parser.add_argument("--preview-plugin-state", type=Path)
    args = parser.parse_args(argv)

    if bool(args.manifest_sqlite) == bool(args.route_runtime_index):
        parser.error("provide exactly one of --manifest-sqlite or --route-runtime-index")

    summary = build_route_slice_artifacts(
        args.manifest_sqlite,
        args.edit_log,
        args.play_log,
        args.artifact_dir,
        truth_pack=args.truth_pack,
        preview_plugin_state=args.preview_plugin_state,
        route_runtime_index=args.route_runtime_index,
    )
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
