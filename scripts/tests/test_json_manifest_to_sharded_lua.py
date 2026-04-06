from __future__ import annotations

import json
import sqlite3
import subprocess
import tempfile
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "json_manifest_to_sharded_lua.py"
SCHEMA = ROOT / "specs" / "chunk-manifest.schema.json"


class JsonManifestToShardedLuaTests(unittest.TestCase):
    def test_schema_allows_compile_time_chunk_refs_without_shards(self) -> None:
        schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
        chunk_ref = schema["$defs"]["chunkRef"]

        self.assertNotIn(
            "shards",
            chunk_ref.get("required", []),
            "compile-time JSON chunkRefs must remain valid without Lua index shard names",
        )
        self.assertIn(
            "shards",
            chunk_ref["properties"],
            "schema should still allow shard names for generated index artifacts",
        )

    def test_json_manifest_to_sharded_lua_rejects_non_current_json_schema(self) -> None:
        manifest = {
            "schemaVersion": "0.5.0",
            "meta": {
                "worldName": "UnsupportedSchemaTest",
                "generator": "test",
                "source": "test",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256,
                "bbox": {"minLat": 0, "minLon": 0, "maxLat": 1, "maxLon": 1},
            },
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": {"x": 0, "y": 0, "z": 0},
                    "roads": [{}],
                }
            ],
        }

        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            manifest_path = temp_root / "manifest.json"
            out_dir = temp_root / "out"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            with self.assertRaises(subprocess.CalledProcessError) as cm:
                subprocess.run(
                    [
                        "python3",
                        str(SCRIPT),
                        "--json",
                        str(manifest_path),
                        "--output-dir",
                        str(out_dir),
                        "--index-name",
                        "TestManifestIndex",
                        "--shard-folder",
                        "TestManifestChunks",
                    ],
                    check=True,
                    cwd=ROOT,
                    capture_output=True,
                    text=True,
                )

            self.assertIn("unsupported schemaVersion", cm.exception.stderr)
            self.assertIn("0.5.0", cm.exception.stderr)

    def test_chunk_refs_include_streaming_metadata(self) -> None:
        manifest = {
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "StreamingMetaTest",
                "generator": "test",
                "source": "test",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256,
                "bbox": {"minLat": 0, "minLon": 0, "maxLat": 1, "maxLon": 1},
            },
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": {"x": 0, "y": 0, "z": 0},
                    "terrain": {"cellSizeStuds": 4, "width": 1, "depth": 1, "heights": [0], "material": "Grass"},
                    "roads": [{}, {}],
                    "rails": [{}],
                    "buildings": [{}, {}],
                    "water": [{}],
                    "props": [{}, {}, {}],
                    "landuse": [{}, {}],
                    "barriers": [{}],
                }
            ],
        }

        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            manifest_path = temp_root / "manifest.json"
            out_dir = temp_root / "out"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            subprocess.run(
                [
                    "python3",
                    str(SCRIPT),
                    "--json",
                    str(manifest_path),
                    "--output-dir",
                    str(out_dir),
                    "--index-name",
                    "TestManifestIndex",
                    "--shard-folder",
                    "TestManifestChunks",
                ],
                check=True,
                cwd=ROOT,
            )

            index_text = (out_dir / "TestManifestIndex.lua").read_text(encoding="utf-8")
            self.assertIn("featureCount=13", index_text)
            self.assertIn("streamingCost=62", index_text)

    def test_runtime_shards_fragment_large_chunk_payloads_under_byte_cap(self) -> None:
        manifest = {
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "RuntimeFragmentationTest",
                "generator": "test",
                "source": "test",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256,
                "bbox": {"minLat": 0, "minLon": 0, "maxLat": 1, "maxLon": 1},
            },
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": {"x": 0, "y": 0, "z": 0},
                    "terrain": {
                        "cellSizeStuds": 2,
                        "width": 64,
                        "depth": 64,
                        "heights": list(range(512)),
                        "materials": ["Grass"] * 512,
                    },
                    "roads": [{"kind": "residential", "points": [{"x": float(i), "y": 0.0, "z": 0.0}]} for i in range(8)],
                }
            ],
        }

        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            manifest_path = temp_root / "manifest.json"
            out_dir = temp_root / "out"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            subprocess.run(
                [
                    "python3",
                    str(SCRIPT),
                    "--json",
                    str(manifest_path),
                    "--output-dir",
                    str(out_dir),
                    "--index-name",
                    "TestManifestIndex",
                    "--shard-folder",
                    "TestManifestChunks",
                    "--chunks-per-shard",
                    "1",
                    "--max-bytes",
                    "1200",
                ],
                check=True,
                cwd=ROOT,
            )

            index_text = (out_dir / "TestManifestIndex.lua").read_text(encoding="utf-8")
            shard_dir = out_dir / "TestManifestChunks"
            shard_texts = [path.read_text(encoding="utf-8") for path in sorted(shard_dir.glob("*.lua"))]

            self.assertIn("fragmentCount=", index_text)
            self.assertIn('shards={"TestManifestIndex_001","TestManifestIndex_002"', index_text)
            self.assertGreater(len(shard_texts), 2)
            self.assertTrue(
                any('terrain={cellSizeStuds=2,width=64,depth=64}' in text for text in shard_texts),
                f"expected base terrain metadata fragment, got {shard_texts}",
            )
            self.assertTrue(
                any("terrain={heights={" in text for text in shard_texts),
                f"expected terrain heights fragment, got {shard_texts}",
            )
            self.assertTrue(
                any("terrain={materials={" in text for text in shard_texts),
                f"expected terrain materials fragment, got {shard_texts}",
            )

    def test_json_manifest_to_sharded_lua_rejects_non_current_sqlite_schema(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            sqlite_path = temp_root / "manifest.sqlite"
            out_dir = temp_root / "out"

            connection = sqlite3.connect(sqlite_path)
            try:
                connection.executescript(
                    """
                    CREATE TABLE manifest_meta (
                        singleton_id INTEGER PRIMARY KEY CHECK (singleton_id = 1),
                        schema_version TEXT NOT NULL,
                        world_name TEXT NOT NULL,
                        generator TEXT NOT NULL,
                        source TEXT NOT NULL,
                        meters_per_stud REAL NOT NULL,
                        chunk_size_studs INTEGER NOT NULL,
                        bbox_min_lat REAL NOT NULL,
                        bbox_min_lon REAL NOT NULL,
                        bbox_max_lat REAL NOT NULL,
                        bbox_max_lon REAL NOT NULL,
                        total_features INTEGER NOT NULL,
                        notes_json TEXT NOT NULL
                    );
                    CREATE TABLE manifest_chunks (
                        chunk_id TEXT PRIMARY KEY,
                        origin_x REAL NOT NULL,
                        origin_y REAL NOT NULL,
                        origin_z REAL NOT NULL,
                        feature_count INTEGER NOT NULL,
                        streaming_cost REAL NOT NULL,
                        estimated_memory_cost REAL NOT NULL,
                        partition_version TEXT NOT NULL,
                        subplans_json TEXT NOT NULL,
                        chunk_json TEXT NOT NULL
                    );
                    """
                )
                connection.execute(
                    """
                    INSERT INTO manifest_meta (
                        singleton_id, schema_version, world_name, generator, source,
                        meters_per_stud, chunk_size_studs,
                        bbox_min_lat, bbox_min_lon, bbox_max_lat, bbox_max_lon,
                        total_features, notes_json
                    ) VALUES (1, '0.5.0', 'SqliteUnsupportedSchemaTest', 'test', 'test', 0.3, 256,
                              0, 0, 1, 1, 1, '[]')
                    """
                )
                connection.execute(
                    """
                    INSERT INTO manifest_chunks (
                        chunk_id, origin_x, origin_y, origin_z,
                        feature_count, streaming_cost, estimated_memory_cost,
                        partition_version, subplans_json, chunk_json
                    ) VALUES (?, 0, 0, 0, 1, 8, 32, 'subplans.v1', '[]', ?)
                    """,
                    ("0_0", json.dumps({"id": "0_0", "originStuds": {"x": 0, "y": 0, "z": 0}, "roads": []})),
                )
                connection.commit()
            finally:
                connection.close()

            with self.assertRaises(subprocess.CalledProcessError) as cm:
                subprocess.run(
                    [
                        "python3",
                        str(SCRIPT),
                        "--sqlite",
                        str(sqlite_path),
                        "--output-dir",
                        str(out_dir),
                        "--index-name",
                        "TestManifestIndex",
                        "--shard-folder",
                        "TestManifestChunks",
                    ],
                    check=True,
                    cwd=ROOT,
                    capture_output=True,
                    text=True,
                )

            self.assertIn("unsupported schemaVersion", cm.exception.stderr)
            self.assertIn("0.5.0", cm.exception.stderr)

    def test_chunk_refs_include_partition_version_and_subplans(self) -> None:
        manifest = {
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "SubplanMetaTest",
                "generator": "test",
                "source": "test",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256,
                "bbox": {"minLat": 0, "minLon": 0, "maxLat": 1, "maxLon": 1},
            },
            "chunkRefs": [
                {
                    "id": "0_0",
                    "originStuds": {"x": 0, "y": 0, "z": 0},
                    "partitionVersion": "subplans.v1",
                    "subplans": [
                        {
                            "id": "terrain",
                            "layer": "terrain",
                            "featureCount": 1,
                            "streamingCost": 40.0,
                            "bounds": {"minX": 0, "minY": 0, "maxX": 128, "maxY": 128},
                        },
                        {
                            "id": "roads",
                            "layer": "roads",
                            "featureCount": 2,
                            "streamingCost": 4.5,
                        },
                    ],
                }
            ],
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": {"x": 0, "y": 0, "z": 0},
                    "roads": [{}, {}],
                }
            ],
        }

        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            manifest_path = temp_root / "manifest.json"
            out_dir = temp_root / "out"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            subprocess.run(
                [
                    "python3",
                    str(SCRIPT),
                    "--json",
                    str(manifest_path),
                    "--output-dir",
                    str(out_dir),
                    "--index-name",
                    "TestManifestIndex",
                    "--shard-folder",
                    "TestManifestChunks",
                ],
                check=True,
                cwd=ROOT,
            )

            index_text = (out_dir / "TestManifestIndex.lua").read_text(encoding="utf-8")
            self.assertIn('partitionVersion="subplans.v1"', index_text)
            self.assertIn('subplans={{id="terrain",layer="terrain"', index_text)
            self.assertIn('featureCount=1,streamingCost=40.0,bounds={minX=0,minY=0,maxX=128,maxY=128}', index_text)
            self.assertIn('{id="roads",layer="roads",featureCount=2,streamingCost=4.5}', index_text)

    def test_chunk_refs_preserve_estimated_memory_cost_metadata(self) -> None:
        manifest = {
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "EstimatedMemoryMetaTest",
                "generator": "test",
                "source": "test",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256,
                "bbox": {"minLat": 0, "minLon": 0, "maxLat": 1, "maxLon": 1},
            },
            "chunkRefs": [
                {
                    "id": "0_0",
                    "originStuds": {"x": 0, "y": 0, "z": 0},
                    "featureCount": 3,
                    "streamingCost": 12.0,
                    "estimatedMemoryCost": 96.0,
                    "partitionVersion": "subplans.v1",
                    "subplans": [
                        {
                            "id": "terrain",
                            "layer": "terrain",
                            "featureCount": 1,
                            "streamingCost": 8.0,
                            "estimatedMemoryCost": 64.0,
                        },
                        {
                            "id": "roads",
                            "layer": "roads",
                            "featureCount": 2,
                            "streamingCost": 4.0,
                            "estimatedMemoryCost": 32.0,
                        },
                    ],
                }
            ],
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": {"x": 0, "y": 0, "z": 0},
                    "roads": [{}, {}],
                }
            ],
        }

        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            manifest_path = temp_root / "manifest.json"
            out_dir = temp_root / "out"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            subprocess.run(
                [
                    "python3",
                    str(SCRIPT),
                    "--json",
                    str(manifest_path),
                    "--output-dir",
                    str(out_dir),
                    "--index-name",
                    "TestManifestIndex",
                    "--shard-folder",
                    "TestManifestChunks",
                ],
                check=True,
                cwd=ROOT,
            )

            schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
            self.assertIn("estimatedMemoryCost", schema["$defs"]["chunkRef"]["properties"])
            self.assertIn("estimatedMemoryCost", schema["$defs"]["chunkSubplan"]["properties"])

            index_text = (out_dir / "TestManifestIndex.lua").read_text(encoding="utf-8")
            self.assertIn("estimatedMemoryCost=96.0", index_text)
            self.assertIn("streamingCost=8.0,estimatedMemoryCost=64.0", index_text)
            self.assertIn("streamingCost=4.0,estimatedMemoryCost=32.0", index_text)

    def test_chunk_refs_do_not_derive_top_level_counts_when_subplans_exist(self) -> None:
        manifest = {
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "SubplanFallbackTest",
                "generator": "test",
                "source": "test",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256,
                "bbox": {"minLat": 0, "minLon": 0, "maxLat": 1, "maxLon": 1},
            },
            "chunkRefs": [
                {
                    "id": "0_0",
                    "originStuds": {"x": 0, "y": 0, "z": 0},
                    "partitionVersion": "subplans.v1",
                    "subplans": [
                        {
                            "id": "terrain",
                            "layer": "terrain",
                            "featureCount": 7,
                            "streamingCost": 40.0,
                        },
                        {
                            "id": "roads",
                            "layer": "roads",
                            "featureCount": 8,
                            "streamingCost": 4.5,
                        },
                    ],
                }
            ],
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": {"x": 0, "y": 0, "z": 0},
                    "roads": [{}],
                    "buildings": [{}],
                }
            ],
        }

        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            manifest_path = temp_root / "manifest.json"
            out_dir = temp_root / "out"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            subprocess.run(
                [
                    "python3",
                    str(SCRIPT),
                    "--json",
                    str(manifest_path),
                    "--output-dir",
                    str(out_dir),
                    "--index-name",
                    "TestManifestIndex",
                    "--shard-folder",
                    "TestManifestChunks",
                ],
                check=True,
                cwd=ROOT,
            )

            index_text = (out_dir / "TestManifestIndex.lua").read_text(encoding="utf-8")
            self.assertIn('partitionVersion="subplans.v1"', index_text)
            self.assertIn('subplans={{id="terrain",layer="terrain",featureCount=7,streamingCost=40.0}', index_text)
            self.assertNotIn("featureCount=2", index_text)
            self.assertNotIn("streamingCost=16", index_text)

    def test_chunk_refs_preserve_aggregate_hints_when_subplans_omit_rails_and_barriers(self) -> None:
        manifest = {
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "SubplanAggregateHintTest",
                "generator": "test",
                "source": "test",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256,
                "bbox": {"minLat": 0, "minLon": 0, "maxLat": 1, "maxLon": 1},
            },
            "chunkRefs": [
                {
                    "id": "0_0",
                    "originStuds": {"x": 0, "y": 0, "z": 0},
                    "featureCount": 4,
                    "streamingCost": 17.0,
                    "partitionVersion": "subplans.v1",
                    "subplans": [
                        {
                            "id": "terrain",
                            "layer": "terrain",
                            "featureCount": 0,
                            "streamingCost": 0.0,
                        },
                        {
                            "id": "landuse",
                            "layer": "landuse",
                            "featureCount": 0,
                            "streamingCost": 0.0,
                        },
                        {
                            "id": "roads",
                            "layer": "roads",
                            "featureCount": 1,
                            "streamingCost": 4.0,
                        },
                        {
                            "id": "buildings",
                            "layer": "buildings",
                            "featureCount": 0,
                            "streamingCost": 0.0,
                        },
                        {
                            "id": "water",
                            "layer": "water",
                            "featureCount": 0,
                            "streamingCost": 0.0,
                        },
                        {
                            "id": "props",
                            "layer": "props",
                            "featureCount": 0,
                            "streamingCost": 0.0,
                        },
                    ],
                }
            ],
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": {"x": 0, "y": 0, "z": 0},
                    "roads": [{}],
                    "rails": [{}],
                    "barriers": [{}],
                }
            ],
        }

        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            manifest_path = temp_root / "manifest.json"
            out_dir = temp_root / "out"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            subprocess.run(
                [
                    "python3",
                    str(SCRIPT),
                    "--json",
                    str(manifest_path),
                    "--output-dir",
                    str(out_dir),
                    "--index-name",
                    "TestManifestIndex",
                    "--shard-folder",
                    "TestManifestChunks",
                ],
                check=True,
                cwd=ROOT,
            )

            index_text = (out_dir / "TestManifestIndex.lua").read_text(encoding="utf-8")
            self.assertIn("featureCount=4", index_text)
            self.assertIn("streamingCost=17.0", index_text)
            self.assertIn('partitionVersion="subplans.v1"', index_text)
            self.assertIn('{id="roads",layer="roads",featureCount=1,streamingCost=4.0}', index_text)

    def test_chunk_refs_keep_canonical_chunk_origin_when_index_metadata_disagrees(self) -> None:
        manifest = {
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "CanonicalOriginTest",
                "generator": "test",
                "source": "test",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256,
                "bbox": {"minLat": 0, "minLon": 0, "maxLat": 1, "maxLon": 1},
            },
            "chunkRefs": [
                {
                    "id": "0_0",
                    "originStuds": {"x": 999, "y": 888, "z": 777},
                    "partitionVersion": "subplans.v1",
                    "subplans": [
                        {
                            "id": "terrain",
                            "layer": "terrain",
                            "featureCount": 1,
                            "streamingCost": 40.0,
                        }
                    ],
                }
            ],
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": {"x": 1, "y": 2, "z": 3},
                    "roads": [{}],
                }
            ],
        }

        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            manifest_path = temp_root / "manifest.json"
            out_dir = temp_root / "out"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            subprocess.run(
                [
                    "python3",
                    str(SCRIPT),
                    "--json",
                    str(manifest_path),
                    "--output-dir",
                    str(out_dir),
                    "--index-name",
                    "TestManifestIndex",
                    "--shard-folder",
                    "TestManifestChunks",
                ],
                check=True,
                cwd=ROOT,
            )

            index_text = (out_dir / "TestManifestIndex.lua").read_text(encoding="utf-8")
            self.assertIn('originStuds={x=1,y=2,z=3}', index_text)
            self.assertNotIn('originStuds={x=999,y=888,z=777}', index_text)

    def test_chunk_level_subplan_fields_are_ignored_without_index_chunk_ref_metadata(self) -> None:
        manifest = {
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "BoundaryTest",
                "generator": "test",
                "source": "test",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256,
                "bbox": {"minLat": 0, "minLon": 0, "maxLat": 1, "maxLon": 1},
            },
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": {"x": 0, "y": 0, "z": 0},
                    "partitionVersion": "subplans.v1",
                    "subplans": [
                        {
                            "id": "roads",
                            "layer": "roads",
                            "featureCount": 2,
                            "streamingCost": 4.5,
                        }
                    ],
                    "roads": [{}, {}],
                }
            ],
        }

        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            manifest_path = temp_root / "manifest.json"
            out_dir = temp_root / "out"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            subprocess.run(
                [
                    "python3",
                    str(SCRIPT),
                    "--json",
                    str(manifest_path),
                    "--output-dir",
                    str(out_dir),
                    "--index-name",
                    "TestManifestIndex",
                    "--shard-folder",
                    "TestManifestChunks",
                ],
                check=True,
                cwd=ROOT,
            )

            index_text = (out_dir / "TestManifestIndex.lua").read_text(encoding="utf-8")
            shard_text = (out_dir / "TestManifestChunks" / "TestManifestIndex_001.lua").read_text(encoding="utf-8")
            self.assertNotIn('partitionVersion="subplans.v1"', index_text)
            self.assertNotIn("subplans={{", index_text)
            self.assertIn("featureCount=2", index_text)
            self.assertIn("streamingCost=8", index_text)
            self.assertNotIn('partitionVersion="subplans.v1"', shard_text)
            self.assertNotIn("subplans={{", shard_text)

    def test_script_accepts_sqlite_manifest_store(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            sqlite_path = temp_root / "manifest.sqlite"
            out_dir = temp_root / "out"

            connection = sqlite3.connect(sqlite_path)
            connection.executescript(
                """
                CREATE TABLE manifest_meta (
                    singleton_id INTEGER PRIMARY KEY CHECK (singleton_id = 1),
                    schema_version TEXT NOT NULL,
                    world_name TEXT NOT NULL,
                    generator TEXT NOT NULL,
                    source TEXT NOT NULL,
                    meters_per_stud REAL NOT NULL,
                    chunk_size_studs INTEGER NOT NULL,
                    bbox_min_lat REAL NOT NULL,
                    bbox_min_lon REAL NOT NULL,
                    bbox_max_lat REAL NOT NULL,
                    bbox_max_lon REAL NOT NULL,
                    total_features INTEGER NOT NULL,
                    notes_json TEXT NOT NULL
                );
                CREATE TABLE manifest_chunks (
                    chunk_id TEXT PRIMARY KEY,
                    origin_x REAL NOT NULL,
                    origin_y REAL NOT NULL,
                    origin_z REAL NOT NULL,
                    feature_count INTEGER NOT NULL,
                    streaming_cost REAL NOT NULL,
                    estimated_memory_cost REAL,
                    partition_version TEXT NOT NULL,
                    subplans_json TEXT NOT NULL,
                    chunk_json TEXT NOT NULL
                );
                """
            )
            connection.execute(
                """
                INSERT INTO manifest_meta (
                    singleton_id, schema_version, world_name, generator, source,
                    meters_per_stud, chunk_size_studs,
                    bbox_min_lat, bbox_min_lon, bbox_max_lat, bbox_max_lon,
                    total_features, notes_json
                ) VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    "0.4.0",
                    "SqliteSharder",
                    "test",
                    "test",
                    0.3,
                    256,
                    0.0,
                    0.0,
                    1.0,
                    1.0,
                    2,
                    "[]",
                ),
            )
            connection.execute(
                """
                INSERT INTO manifest_chunks (
                    chunk_id, origin_x, origin_y, origin_z,
                    feature_count, streaming_cost, estimated_memory_cost, partition_version,
                    subplans_json, chunk_json
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    "0_0",
                    0.0,
                    0.0,
                    0.0,
                    2,
                    20.0,
                    44.0,
                    "subplans.v1",
                    json.dumps([{"id": "roads", "layer": "roads", "featureCount": 2, "streamingCost": 8.0}]),
                    json.dumps(
                        {
                            "id": "0_0",
                            "originStuds": {"x": 0, "y": 0, "z": 0},
                            "roads": [{}, {}],
                            "rails": [],
                            "buildings": [],
                            "water": [],
                            "props": [],
                            "landuse": [],
                            "barriers": [],
                        }
                    ),
                ),
            )
            connection.commit()
            connection.close()

            subprocess.run(
                [
                    "python3",
                    str(SCRIPT),
                    "--sqlite",
                    str(sqlite_path),
                    "--output-dir",
                    str(out_dir),
                    "--index-name",
                    "TestManifestIndex",
                    "--shard-folder",
                    "TestManifestChunks",
                ],
                check=True,
                cwd=ROOT,
            )

            index_text = (out_dir / "TestManifestIndex.lua").read_text(encoding="utf-8")
            self.assertIn('schemaVersion="0.4.0"', index_text)
            self.assertIn("featureCount=2", index_text)
            self.assertIn("estimatedMemoryCost=44.0", index_text)
            self.assertIn('partitionVersion="subplans.v1"', index_text)


    def test_texture_embedding_creates_module_and_strips_paths(self) -> None:
        """When a chunk has terrainTexturePath with a companion .rgba file,
        the conversion script should embed the texture as a separate Lua
        ModuleScript and strip the filesystem path from the shard data."""
        manifest = {
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "TextureEmbedTest",
                "generator": "test",
                "source": "test",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256,
                "bbox": {"minLat": 0, "minLon": 0, "maxLat": 1, "maxLon": 1},
            },
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": {"x": 0, "y": 0, "z": 0},
                    "terrainTexturePath": "textures/0_0.png",
                    "roads": [{}],
                }
            ],
        }

        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            manifest_path = temp_root / "manifest.json"
            out_dir = temp_root / "out"

            # Create a companion .rgba file (2x2 RGBA = 16 bytes)
            texture_dir = temp_root / "textures"
            texture_dir.mkdir()
            rgba_bytes = bytes([255, 0, 0, 255, 0, 255, 0, 255,
                                0, 0, 255, 255, 255, 255, 255, 255])
            (texture_dir / "0_0.rgba").write_bytes(rgba_bytes)

            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            subprocess.run(
                [
                    "python3",
                    str(SCRIPT),
                    "--json",
                    str(manifest_path),
                    "--output-dir",
                    str(out_dir),
                    "--index-name",
                    "TestManifestIndex",
                    "--shard-folder",
                    "TestManifestChunks",
                    "--texture-folder",
                    "TestTextures",
                    "--manifest-dir",
                    str(temp_root),
                ],
                check=True,
                cwd=ROOT,
            )

            # Texture module should exist
            texture_module_path = out_dir / "TestTextures" / "Texture_0_0.lua"
            self.assertTrue(texture_module_path.exists(), "Texture module should be created")

            texture_src = texture_module_path.read_text(encoding="utf-8")
            self.assertTrue(texture_src.startswith('return "'), "Should return a string literal")
            # Verify it encodes the RGBA bytes as \xHH escapes
            self.assertIn("\\xff\\x00\\x00\\xff", texture_src)

            # Shard should reference the texture module, not the filesystem path
            shard_text = (out_dir / "TestManifestChunks" / "TestManifestIndex_001.lua").read_text(encoding="utf-8")
            self.assertIn('terrainTextureModule="Texture_0_0"', shard_text)
            self.assertIn('terrainTextureFolder="TestTextures"', shard_text)
            self.assertIn("terrainTextureWidth=2", shard_text)
            self.assertIn("terrainTextureHeight=2", shard_text)
            self.assertNotIn("terrainTexturePath", shard_text)
            self.assertNotIn("terrainTextureRgbaPath", shard_text)

    def test_texture_embedding_handles_missing_rgba_gracefully(self) -> None:
        """When terrainTexturePath exists but no companion .rgba file,
        the path should be stripped and no texture module created."""
        manifest = {
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "TextureMissingTest",
                "generator": "test",
                "source": "test",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256,
                "bbox": {"minLat": 0, "minLon": 0, "maxLat": 1, "maxLon": 1},
            },
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": {"x": 0, "y": 0, "z": 0},
                    "terrainTexturePath": "nonexistent/0_0.png",
                    "roads": [{}],
                }
            ],
        }

        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            manifest_path = temp_root / "manifest.json"
            out_dir = temp_root / "out"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            subprocess.run(
                [
                    "python3",
                    str(SCRIPT),
                    "--json",
                    str(manifest_path),
                    "--output-dir",
                    str(out_dir),
                    "--index-name",
                    "TestManifestIndex",
                    "--shard-folder",
                    "TestManifestChunks",
                ],
                check=True,
                cwd=ROOT,
            )

            shard_text = (out_dir / "TestManifestChunks" / "TestManifestIndex_001.lua").read_text(encoding="utf-8")
            self.assertNotIn("terrainTexturePath", shard_text)
            self.assertNotIn("terrainTextureModule", shard_text)

    def test_texture_embedding_with_negative_chunk_id(self) -> None:
        """Chunk IDs with hyphens (e.g. '-1_2') should produce valid module names."""
        manifest = {
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "NegativeIdTest",
                "generator": "test",
                "source": "test",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256,
                "bbox": {"minLat": 0, "minLon": 0, "maxLat": 1, "maxLon": 1},
            },
            "chunks": [
                {
                    "id": "-1_2",
                    "originStuds": {"x": 0, "y": 0, "z": 0},
                    "terrainTextureRgbaPath": "tiles/-1_2.rgba",
                    "roads": [{}],
                }
            ],
        }

        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            manifest_path = temp_root / "manifest.json"
            out_dir = temp_root / "out"

            # Create the .rgba file
            tiles_dir = temp_root / "tiles"
            tiles_dir.mkdir()
            rgba_bytes = bytes([128, 128, 128, 255]) * 4  # 2x2
            (tiles_dir / "-1_2.rgba").write_bytes(rgba_bytes)

            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            subprocess.run(
                [
                    "python3",
                    str(SCRIPT),
                    "--json",
                    str(manifest_path),
                    "--output-dir",
                    str(out_dir),
                    "--index-name",
                    "TestManifestIndex",
                    "--shard-folder",
                    "TestManifestChunks",
                    "--manifest-dir",
                    str(temp_root),
                ],
                check=True,
                cwd=ROOT,
            )

            # Module name should sanitize the hyphen
            texture_module_path = out_dir / "AustinTerrainTextures" / "Texture_neg1_2.lua"
            self.assertTrue(texture_module_path.exists(), "Negative-id texture module should exist")

            shard_text = (out_dir / "TestManifestChunks" / "TestManifestIndex_001.lua").read_text(encoding="utf-8")
            self.assertIn('terrainTextureModule="Texture_neg1_2"', shard_text)


if __name__ == "__main__":
    unittest.main()
