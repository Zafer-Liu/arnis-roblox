#!/usr/bin/env python3
"""
Convert a large JSON manifest into a sharded Lua manifest layout suitable for
Roblox Studio sync/runtime loading.

Outputs:
  - <output-dir>/<index-name>.lua
  - <output-dir>/<shard-folder>/<index-name>_NNN.lua
  - <output-dir>/<texture-folder>/<chunk_id>.lua  (per-chunk satellite RGBA)

The index module includes lightweight chunk refs so runtime code can resolve a
chunk to its shard modules without loading the entire manifest up front.

Satellite texture embedding:
  When a chunk carries ``terrainTextureRgbaPath`` (or ``terrainTexturePath``
  with a companion ``.rgba`` file), the raw RGBA bytes are embedded in a
  separate Lua ModuleScript that returns a string literal.  Runtime Lua can
  convert this to a buffer with ``buffer.fromstring(require(module))``.
  The chunk shard data receives a ``terrainTextureModule`` key pointing to the
  module name so that TerrainBuilder can locate and load the texture lazily.
"""

import argparse
import json
import io
import sqlite3
from pathlib import Path
from typing import Any, TextIO

from chunk_fragmentation import (
    INDEX_ONLY_FIELDS,
    fragment_chunk_for_lua_shards,
)

CURRENT_SCHEMA_VERSION = "0.4.0"

# Fields that are build-time filesystem paths and must not appear in runtime Lua shards.
_BUILD_TIME_TEXTURE_FIELDS = {"terrainTexturePath", "terrainTextureRgbaPath"}

DEFAULT_TEXTURE_FOLDER = "AustinTerrainTextures"


def _rgba_bytes_to_lua_module(rgba_bytes: bytes) -> str:
    """Encode raw RGBA bytes as a Lua ModuleScript returning a string literal.

    Each byte is written as a ``\\xHH`` escape so ``buffer.fromstring()`` can
    consume the result with zero runtime decoding overhead.
    """
    escaped = "".join(f"\\x{b:02x}" for b in rgba_bytes)
    return f'return "{escaped}"\n'


def _resolve_rgba_path(chunk: dict[str, Any], manifest_dir: Path | None) -> Path | None:
    """Return the Path to the .rgba file for a chunk, or None if unavailable."""
    rgba_path_str = chunk.get("terrainTextureRgbaPath")
    if rgba_path_str:
        p = Path(rgba_path_str)
        if p.is_absolute():
            return p if p.exists() else None
        if manifest_dir:
            resolved = manifest_dir / p
            return resolved if resolved.exists() else None
        return None

    # Fallback: derive from terrainTexturePath by swapping extension
    png_path_str = chunk.get("terrainTexturePath")
    if not png_path_str:
        return None
    png_path = Path(png_path_str)
    rgba_candidate = png_path.with_suffix(".rgba")
    if rgba_candidate.is_absolute():
        return rgba_candidate if rgba_candidate.exists() else None
    if manifest_dir:
        resolved = manifest_dir / rgba_candidate
        return resolved if resolved.exists() else None
    return None


def _embed_texture_modules(
    chunks: list[dict[str, Any]],
    manifest_dir: Path | None,
    texture_dir: Path,
    texture_folder: str,
) -> int:
    """Read .rgba files for each chunk, write texture ModuleScripts.

    Mutates each chunk dict in-place: removes build-time path fields and adds
    ``terrainTextureModule`` when a texture was successfully embedded.

    Returns the number of textures embedded.
    """
    embedded = 0
    for chunk in chunks:
        rgba_path = _resolve_rgba_path(chunk, manifest_dir)
        # Always strip build-time paths regardless of whether embedding succeeds
        for field in _BUILD_TIME_TEXTURE_FIELDS:
            chunk.pop(field, None)

        if rgba_path is None:
            continue

        rgba_bytes = rgba_path.read_bytes()
        if len(rgba_bytes) == 0:
            continue

        chunk_id = chunk["id"]
        # Sanitize chunk_id for use as a module name (e.g. "-1_2" -> "neg1_2")
        safe_id = chunk_id.replace("-", "neg")
        module_name = f"Texture_{safe_id}"
        module_path = texture_dir / f"{module_name}.lua"
        module_path.parent.mkdir(parents=True, exist_ok=True)
        module_path.write_text(_rgba_bytes_to_lua_module(rgba_bytes), encoding="utf-8")

        chunk["terrainTextureModule"] = module_name
        chunk["terrainTextureFolder"] = texture_folder

        # Derive dimensions from byte count (RGBA = 4 bytes/pixel, square texture)
        pixel_count = len(rgba_bytes) // 4
        side = int(pixel_count**0.5)
        if side * side == pixel_count:
            chunk["terrainTextureWidth"] = side
            chunk["terrainTextureHeight"] = side

        embedded += 1

    return embedded


def _require_current_schema_version(schema_version: str, source_label: str) -> None:
    if schema_version != CURRENT_SCHEMA_VERSION:
        raise SystemExit(
            f"unsupported schemaVersion {schema_version!r} in {source_label}; expected {CURRENT_SCHEMA_VERSION!r}"
        )


def to_lua(value: Any, out: TextIO, indent: int = 0) -> None:
    if isinstance(value, dict):
        out.write("{")
        items = list(value.items())
        for i, (k, v) in enumerate(items):
            out.write(f"{k}=")
            to_lua(v, out, indent + 2)
            if i + 1 != len(items):
                out.write(",")
        out.write("}")
    elif isinstance(value, list):
        out.write("{")
        for i, v in enumerate(value):
            to_lua(v, out, indent + 2)
            if i + 1 != len(value):
                out.write(",")
        out.write("}")
    elif isinstance(value, str):
        escaped = (
            value.replace("\\", "\\\\")
            .replace('"', '\\"')
            .replace("\n", "\\n")
            .replace("\r", "\\r")
            .replace("\t", "\\t")
        )
        out.write(f'"{escaped}"')
    elif isinstance(value, bool):
        out.write("true" if value else "false")
    elif value is None:
        out.write("nil")
    else:
        out.write(str(value))


def write_lua_module(path: Path, data: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as out:
        out.write("return ")
        to_lua(data, out, indent=0)
        out.write("\n")


def lua_len(value: Any) -> int:
    out = io.StringIO()
    out.write("return ")
    to_lua(value, out, indent=0)
    out.write("\n")
    return len(out.getvalue().encode("utf-8"))


def lua_value_len(value: Any) -> int:
    out = io.StringIO()
    to_lua(value, out, indent=0)
    return len(out.getvalue().encode("utf-8"))


def clear_existing_shards(shard_dir: Path, index_name: str) -> None:
    if not shard_dir.exists():
        return
    pattern = f"{index_name}_*.lua"
    for existing in shard_dir.glob(pattern):
        existing.unlink()

def strip_index_only_fields(chunk: dict[str, Any]) -> dict[str, Any]:
    return {key: value for key, value in chunk.items() if key not in INDEX_ONLY_FIELDS}


def fragment_chunk(chunk: dict[str, Any], max_bytes: int | None) -> list[dict[str, Any]]:
    return fragment_chunk_for_lua_shards(
        strip_index_only_fields(chunk),
        max_bytes,
        lua_len_fn=lua_len,
        lua_value_len_fn=lua_value_len,
        chunk_label="runtime chunk",
    )


def chunk_feature_count(chunk: dict[str, Any]) -> int:
    total = 0
    for key in ("roads", "rails", "buildings", "water", "props", "landuse", "barriers"):
        value = chunk.get(key)
        if isinstance(value, list):
            total += len(value)
    if chunk.get("terrain") is not None:
        total += 1
    return total


def chunk_streaming_cost(chunk: dict[str, Any]) -> int:
    weights = {
        "roads": 4,
        "rails": 3,
        "buildings": 12,
        "water": 2,
        "props": 1,
        "landuse": 6,
        "barriers": 2,
    }
    total = 0
    for key, weight in weights.items():
        value = chunk.get(key)
        if isinstance(value, list):
            total += len(value) * weight
    if chunk.get("terrain") is not None:
        total += 8
    return total


def chunk_ref_metadata(chunk: dict[str, Any], metadata: dict[str, Any] | None = None) -> dict[str, Any]:
    metadata = metadata or {}
    has_subplans = metadata.get("subplans") is not None
    chunk_ref = {
        "id": chunk["id"],
        "originStuds": chunk.get("originStuds", {"x": 0, "y": 0, "z": 0}),
        "shards": [],
    }
    feature_count = metadata.get("featureCount")
    if feature_count is None and not has_subplans:
        feature_count = chunk_feature_count(chunk)
    if feature_count is not None:
        chunk_ref["featureCount"] = feature_count

    streaming_cost = metadata.get("streamingCost")
    if streaming_cost is None and not has_subplans:
        streaming_cost = chunk_streaming_cost(chunk)
    if streaming_cost is not None:
        chunk_ref["streamingCost"] = streaming_cost
    estimated_memory_cost = metadata.get("estimatedMemoryCost")
    if estimated_memory_cost is not None:
        chunk_ref["estimatedMemoryCost"] = estimated_memory_cost

    if metadata.get("partitionVersion") is not None:
        chunk_ref["partitionVersion"] = metadata["partitionVersion"]
    if metadata.get("subplans") is not None:
        chunk_ref["subplans"] = metadata["subplans"]
    return chunk_ref


def load_manifest_from_sqlite(path: Path) -> dict[str, Any]:
    connection = sqlite3.connect(path)
    try:
        meta_row = connection.execute(
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
        if meta_row is None:
            raise SystemExit(f"manifest store {path} is missing manifest_meta")

        _require_current_schema_version(meta_row[0], str(path))
        notes = json.loads(meta_row[11])
        chunk_rows = connection.execute(
            """
            SELECT
                chunk_id,
                origin_x,
                origin_y,
                origin_z,
                feature_count,
                streaming_cost,
                estimated_memory_cost,
                partition_version,
                subplans_json,
                chunk_json
            FROM manifest_chunks
            ORDER BY chunk_id
            """
        ).fetchall()
    finally:
        connection.close()

    chunks: list[dict[str, Any]] = []
    chunk_refs: list[dict[str, Any]] = []
    for row in chunk_rows:
        chunk_json = json.loads(row[9])
        if not isinstance(chunk_json, dict):
            raise SystemExit(f"manifest store {path} contains malformed chunk JSON for {row[0]}")
        chunks.append(chunk_json)
        chunk_refs.append(
            {
                "id": row[0],
                "originStuds": {"x": row[1], "y": row[2], "z": row[3]},
                "featureCount": row[4],
                "streamingCost": row[5],
                "estimatedMemoryCost": row[6],
                "partitionVersion": row[7],
                "subplans": json.loads(row[8]),
            }
        )

    return {
        "schemaVersion": meta_row[0],
        "meta": {
            "worldName": meta_row[1],
            "generator": meta_row[2],
            "source": meta_row[3],
            "metersPerStud": meta_row[4],
            "chunkSizeStuds": meta_row[5],
            "bbox": {
                "minLat": meta_row[6],
                "minLon": meta_row[7],
                "maxLat": meta_row[8],
                "maxLon": meta_row[9],
            },
            "totalFeatures": meta_row[10],
            "notes": notes,
        },
        "chunks": chunks,
        "chunkRefs": chunk_refs,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Convert JSON manifest to sharded Lua modules")
    parser.add_argument("--json", help="Input JSON manifest path")
    parser.add_argument("--sqlite", help="Input SQLite manifest store path")
    parser.add_argument("--output-dir", required=True, help="Output Lua directory")
    parser.add_argument("--index-name", default="AustinManifestIndex", help="Index module name")
    parser.add_argument("--shard-folder", default="AustinManifestChunks", help="Shard folder name")
    parser.add_argument("--chunks-per-shard", type=int, default=32, help="Chunks per shard module")
    parser.add_argument("--max-bytes", type=int, default=None, help="Maximum Lua module size in bytes")
    parser.add_argument(
        "--manifest-dir",
        default=None,
        help="Directory to resolve relative texture paths against (defaults to manifest file parent)",
    )
    parser.add_argument(
        "--texture-folder",
        default=DEFAULT_TEXTURE_FOLDER,
        help="Folder name for per-chunk texture ModuleScripts",
    )
    args = parser.parse_args()

    if bool(args.json) == bool(args.sqlite):
        raise SystemExit("provide exactly one of --json or --sqlite")

    manifest_dir: Path | None = None
    if args.manifest_dir is not None:
        manifest_dir = Path(args.manifest_dir)
    elif args.json is not None:
        manifest_dir = Path(args.json).resolve().parent
    elif args.sqlite is not None:
        manifest_dir = Path(args.sqlite).resolve().parent

    if args.sqlite is not None:
        data = load_manifest_from_sqlite(Path(args.sqlite))
    else:
        with open(args.json, "r", encoding="utf-8") as f:
            data = json.load(f)

    schema_version = data.get("schemaVersion")
    if not isinstance(schema_version, str) or not schema_version:
        raise SystemExit("manifest must contain a schemaVersion string")
    _require_current_schema_version(schema_version, args.sqlite or args.json or "<unknown manifest>")

    source_chunks = data.get("chunks", [])
    if not isinstance(source_chunks, list) or not source_chunks:
        raise SystemExit("manifest must contain a non-empty chunks array")

    source_chunk_refs = data.get("chunkRefs", [])
    chunk_ref_metadata_by_id: dict[str, dict[str, Any]] = {}
    if source_chunk_refs is not None:
        if not isinstance(source_chunk_refs, list):
            raise SystemExit("chunkRefs must be an array when present")
        for chunk_ref in source_chunk_refs:
            if not isinstance(chunk_ref, dict) or "id" not in chunk_ref:
                raise SystemExit("chunkRefs entries must be objects with an id")
            chunk_ref_metadata_by_id[chunk_ref["id"]] = chunk_ref

    output_dir = Path(args.output_dir)
    texture_dir = output_dir / args.texture_folder

    # Embed satellite texture RGBA data as separate Lua ModuleScripts.
    # This mutates source_chunks in-place: strips build-time paths and adds
    # terrainTextureModule / terrainTextureFolder / dimension fields.
    texture_count = _embed_texture_modules(
        source_chunks, manifest_dir, texture_dir, args.texture_folder,
    )

    chunks: list[dict[str, Any]] = []
    chunk_ref_by_id: dict[str, dict[str, Any]] = {}
    for chunk in source_chunks:
        chunk_id = chunk["id"]
        chunk_ref_by_id[chunk_id] = chunk_ref_metadata(chunk, chunk_ref_metadata_by_id.get(chunk_id))
        chunks.extend(fragment_chunk(chunk, args.max_bytes))

    shard_dir = output_dir / args.shard_folder
    clear_existing_shards(shard_dir, args.index_name)
    shard_names = []

    shard_count = (len(chunks) + args.chunks_per_shard - 1) // args.chunks_per_shard
    for shard_index in range(shard_count):
        start = shard_index * args.chunks_per_shard
        end = start + args.chunks_per_shard
        shard_chunks = chunks[start:end]
        shard_name = f"{args.index_name}_{shard_index + 1:03d}"
        shard_names.append(shard_name)
        for shard_chunk in shard_chunks:
            chunk_ref_by_id[shard_chunk["id"]]["shards"].append(shard_name)
        write_lua_module(shard_dir / f"{shard_name}.lua", {"chunks": shard_chunks})

    index_module = {
        "schemaVersion": data["schemaVersion"],
        "meta": data["meta"],
        "shardFolder": args.shard_folder,
        "shards": shard_names,
        "chunkCount": len(source_chunks),
        "fragmentCount": len(chunks),
        "chunksPerShard": args.chunks_per_shard,
        "chunkRefs": [chunk_ref_by_id[chunk["id"]] for chunk in source_chunks],
    }
    write_lua_module(output_dir / f"{args.index_name}.lua", index_module)

    print(f"Wrote index module to {output_dir / f'{args.index_name}.lua'}")
    print(f"Wrote {shard_count} shard modules to {shard_dir}")
    if texture_count > 0:
        print(f"Wrote {texture_count} texture modules to {texture_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
