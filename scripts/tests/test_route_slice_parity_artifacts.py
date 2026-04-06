from __future__ import annotations

import importlib.util
import json
import sqlite3
import shutil
import sys
import tempfile
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "route_slice_parity_artifacts.py"


def load_module():
    scripts_dir = str(MODULE_PATH.parent)
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
    spec = importlib.util.spec_from_file_location("route_slice_parity_artifacts", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load module spec from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def build_manifest_store(path: Path) -> None:
    connection = sqlite3.connect(path)
    connection.executescript(
        """
        CREATE TABLE manifest_meta (
            singleton_id INTEGER PRIMARY KEY,
            schema_version TEXT NOT NULL,
            world_name TEXT NOT NULL,
            generator TEXT NOT NULL,
            source TEXT NOT NULL,
            meters_per_stud REAL NOT NULL,
            chunk_size_studs REAL NOT NULL,
            bbox_min_lat REAL,
            bbox_min_lon REAL,
            bbox_max_lat REAL,
            bbox_max_lon REAL,
            total_features INTEGER NOT NULL,
            notes_json TEXT NOT NULL
        );
        CREATE TABLE manifest_chunks (
            chunk_id TEXT PRIMARY KEY,
            origin_x REAL NOT NULL,
            origin_y REAL NOT NULL,
            origin_z REAL NOT NULL,
            feature_count INTEGER NOT NULL,
            streaming_cost REAL,
            estimated_memory_cost REAL,
            partition_version TEXT,
            subplans_json TEXT NOT NULL,
            chunk_json TEXT NOT NULL
        );
        """
    )
    connection.execute(
        """
        INSERT INTO manifest_meta (
            singleton_id, schema_version, world_name, generator, source,
            meters_per_stud, chunk_size_studs, bbox_min_lat, bbox_min_lon,
            bbox_max_lat, bbox_max_lon, total_features, notes_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            1,
            "0.4.0",
            "RouteSliceTest",
            "test",
            "test",
            1.0,
            256.0,
            0.0,
            0.0,
            1.0,
            1.0,
            3,
            json.dumps({"fixture": True}),
        ),
    )
    chunk = {
        "id": "0_0",
        "originStuds": {"x": 0, "y": 0, "z": 0},
        "terrain": {"cellSizeStuds": 4, "width": 1, "depth": 1, "material": "Grass", "heights": [0]},
        "roads": [{"id": "road_1", "kind": "secondary", "subkind": "sidewalk"}],
        "buildings": [{"id": "building_1"}],
        "props": [{"id": "prop_1", "kind": "bench"}],
    }
    connection.execute(
        """
        INSERT INTO manifest_chunks (
            chunk_id, origin_x, origin_y, origin_z, feature_count,
            streaming_cost, estimated_memory_cost, partition_version, subplans_json, chunk_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "0_0",
            0.0,
            0.0,
            0.0,
            3,
            12.0,
            32.0,
            "subplans.v1",
            json.dumps([]),
            json.dumps(chunk),
        ),
    )
    connection.commit()
    connection.close()


def write_edit_log(path: Path) -> None:
    lines = [
        "ARNIS_SCENE_EDIT_CHUNKS "
        + json.dumps(
            {
                "phase": "edit",
                "rootName": "GeneratedWorld_AustinPreview",
                "chunkIds": ["0_0"],
            },
            separators=(",", ":"),
        ),
        "ARNIS_SCENE_EDIT "
        + json.dumps(
            {
                "phase": "edit",
                "rootName": "GeneratedWorld_AustinPreview",
                "worldIdentity": "AustinManifestIndex",
                "chunkEnvelopeKind": "bounded_preview",
                "manifestSourceKind": "route_catalog",
                "manifestSourceName": "PlanetaryRouteBundle.route-catalog",
                "focus": {"x": 128, "z": 128},
                "radius": 256,
                "scene": {
                    "chunkCount": 1,
                    "buildingModelCount": 1,
                    "chunksWithRoadGeometry": 1,
                    "roadSurfacePartCount": 1,
                    "propInstanceCount": 1,
                    "roadSurfacePartCountByKind": {"secondary": {"surfacePartCount": 1, "featureCount": 1}},
                    "roadSurfacePartCountBySubkind": {"sidewalk": {"surfacePartCount": 1, "featureCount": 1}},
                    "propInstanceCountByKind": {"bench": {"instanceCount": 1, "featureCount": 1}},
                },
            },
            separators=(",", ":"),
        ),
        "ARNIS_MCP_EDIT_ACTION "
        + json.dumps(
            {
                "preview": {
                    "routeCatalogName": "PlanetaryRouteBundle.route-catalog",
                    "routeStepIndex": 1,
                    "children": 1,
                    "manifestSourceKind": "route_catalog",
                    "manifestSourceName": "PlanetaryRouteBundle.route-catalog",
                    "sceneSummary": {
                        "buildingModelCount": 1,
                        "roadSurfacePartCount": 1,
                        "propInstanceCount": 1,
                    },
                }
            },
            separators=(",", ":"),
        ),
    ]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_play_log(path: Path) -> None:
    lines = [
        "ARNIS_SCENE_PLAY "
        + json.dumps(
            {
                "phase": "play",
                "rootName": "GeneratedWorld_Austin",
                "worldIdentity": "AustinManifestIndex",
                "chunkEnvelopeKind": "runtime_resident",
                "manifestSourceKind": "route_catalog",
                "manifestSourceName": "PlanetaryRouteBundle.route-catalog",
                "focus": {"x": 128, "z": 128},
                "radius": 256,
                "scene": {
                    "chunkCount": 1,
                    "chunkIds": ["0_0"],
                    "buildingModelCount": 1,
                    "chunksWithRoadGeometry": 1,
                    "roadSurfacePartCount": 1,
                    "propInstanceCount": 1,
                    "roadSurfacePartCountByKind": {"secondary": {"surfacePartCount": 1, "featureCount": 1}},
                    "roadSurfacePartCountBySubkind": {"sidewalk": {"surfacePartCount": 1, "featureCount": 1}},
                    "propInstanceCountByKind": {"bench": {"instanceCount": 1, "featureCount": 1}},
                },
            },
            separators=(",", ":"),
        ),
        "ARNIS_CLIENT_BOOTSTRAP "
        + json.dumps(
            {
                "bootstrapState": "gameplay_ready",
                "bootstrapStateTrace": "loading_manifest,importing_startup,world_ready,streaming_ready,minimap_ready,gameplay_ready",
                "worldRootExists": True,
                "worldRootName": "GeneratedWorld_Austin",
            },
            separators=(",", ":"),
        ),
        "ARNIS_CLIENT_WORLD_COMPACT "
        + json.dumps(
            {
                "worldRootExists": True,
                "worldRootName": "GeneratedWorld_Austin",
                "supportSurfaceRole": "road",
                "groundMaterial": "Enum.Material.Asphalt",
                "nearbyBuildingModels": 1,
                "nearbyRoofParts": 1,
                "overheadRoofParts": 0,
            },
            separators=(",", ":"),
        ),
    ]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


class RouteSliceParityArtifactsTests(unittest.TestCase):
    def test_build_route_slice_artifacts_writes_subset_and_reports(self) -> None:
        module = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_sqlite = root / "austin-manifest.sqlite"
            edit_log = root / "edit.log"
            play_log = root / "play.log"
            artifact_dir = root / "artifacts"

            build_manifest_store(manifest_sqlite)
            write_edit_log(edit_log)
            write_play_log(play_log)

            summary = module.build_route_slice_artifacts(
                manifest_sqlite,
                edit_log,
                play_log,
                artifact_dir,
            )

            manifest = json.loads((artifact_dir / "route-slice-manifest.json").read_text(encoding="utf-8"))
            parity = json.loads((artifact_dir / "scene-parity.json").read_text(encoding="utf-8"))

            self.assertEqual(summary["chunkIds"], ["0_0"])
            self.assertEqual(manifest["schemaVersion"], "0.4.0")
            self.assertEqual([chunk["id"] for chunk in manifest["chunks"]], ["0_0"])
            self.assertEqual(summary["edit"]["manifestSourceKind"], "route_catalog")
            self.assertEqual(summary["play"]["manifestSourceKind"], "route_catalog")
            self.assertTrue((artifact_dir / "scene-fidelity-edit.html").exists())
            self.assertTrue((artifact_dir / "scene-fidelity-play.html").exists())
            self.assertTrue((artifact_dir / "scene-parity.html").exists())
            self.assertEqual(
                parity["comparisons"]["manifestSourceKind"]["edit"],
                "route_catalog",
            )
            self.assertEqual(
                parity["comparisons"]["manifestSourceKind"]["play"],
                "route_catalog",
            )

    def test_build_route_slice_artifacts_accepts_route_runtime_index_fallback(self) -> None:
        module = load_module()
        if shutil.which("lua") is None:
            self.skipTest("lua is not installed")

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            route_runtime_dir = root / "route-runtime" / "step-001-active"
            shard_dir = route_runtime_dir / "PlanetaryManifestChunks"
            shard_dir.mkdir(parents=True)
            index_path = route_runtime_dir / "PlanetaryManifestIndex.lua"
            edit_log = root / "edit.log"
            play_log = root / "play.log"
            artifact_dir = root / "artifacts"

            index_path.write_text(
                'return {schemaVersion="0.4.0",meta={worldName="RouteSliceTest",generator="test",source="test",metersPerStud=1,chunkSizeStuds=256,bbox={minLat=0,minLon=0,maxLat=1,maxLon=1},totalFeatures=3,notes={fixture=true}},chunkRefs={{id="0_0",originStuds={x=0,y=0,z=0},featureCount=3,streamingCost=12,estimatedMemoryCost=32,partitionVersion="subplans.v1",subplans={},shards={"PlanetaryManifestIndex_001"}}},shardFolder="PlanetaryManifestChunks",shards={"PlanetaryManifestIndex_001"}}',
                encoding="utf-8",
            )
            (shard_dir / "PlanetaryManifestIndex_001.lua").write_text(
                'return {chunks={{id="0_0",originStuds={x=0,y=0,z=0},terrain={cellSizeStuds=4,width=1,depth=1,material="Grass",heights={0}},roads={{id="road_1",kind="secondary",subkind="sidewalk"}},buildings={{id="building_1"}},props={{id="prop_1",kind="bench"}}}}}',
                encoding="utf-8",
            )
            write_edit_log(edit_log)
            write_play_log(play_log)

            summary = module.build_route_slice_artifacts(
                None,
                edit_log,
                play_log,
                artifact_dir,
                route_runtime_index=index_path,
            )

            manifest = json.loads((artifact_dir / "route-slice-manifest.json").read_text(encoding="utf-8"))
            self.assertEqual(summary["chunkIds"], ["0_0"])
            self.assertEqual(summary["routeRuntimeIndex"], str(index_path))
            self.assertEqual([chunk["id"] for chunk in manifest["chunks"]], ["0_0"])

    def test_build_route_slice_artifacts_accepts_route_runtime_index_without_lua_binary(self) -> None:
        module = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            route_runtime_dir = root / "route-runtime" / "step-001-active"
            shard_dir = route_runtime_dir / "PlanetaryManifestChunks"
            shard_dir.mkdir(parents=True)
            index_path = route_runtime_dir / "PlanetaryManifestIndex.lua"
            edit_log = root / "edit.log"
            play_log = root / "play.log"
            artifact_dir = root / "artifacts"

            index_path.write_text(
                'return {schemaVersion="0.4.0",meta={worldName="RouteSliceTest",generator="test",source="test",metersPerStud=1,chunkSizeStuds=256,bbox={minLat=0,minLon=0,maxLat=1,maxLon=1},totalFeatures=3,notes={fixture=true}},chunkRefs={{id="0_0",originStuds={x=0,y=0,z=0},featureCount=3,streamingCost=12,estimatedMemoryCost=32,partitionVersion="subplans.v1",subplans={},shards={"PlanetaryManifestIndex_001"}}},shardFolder="PlanetaryManifestChunks",shards={"PlanetaryManifestIndex_001"}}',
                encoding="utf-8",
            )
            (shard_dir / "PlanetaryManifestIndex_001.lua").write_text(
                'return {chunks={{id="0_0",originStuds={x=0,y=0,z=0},terrain={cellSizeStuds=4,width=1,depth=1,material="Grass",heights={0}},roads={{id="road_1",kind="secondary",subkind="sidewalk"}},buildings={{id="building_1"}},props={{id="prop_1",kind="bench"}}}}}',
                encoding="utf-8",
            )
            write_edit_log(edit_log)
            write_play_log(play_log)

            original_which = module.shutil.which
            module.shutil.which = lambda name: None if name == "lua" else original_which(name)
            try:
                summary = module.build_route_slice_artifacts(
                    None,
                    edit_log,
                    play_log,
                    artifact_dir,
                    route_runtime_index=index_path,
                )
            finally:
                module.shutil.which = original_which

            manifest = json.loads((artifact_dir / "route-slice-manifest.json").read_text(encoding="utf-8"))
            self.assertEqual(summary["chunkIds"], ["0_0"])
            self.assertEqual(summary["routeRuntimeIndex"], str(index_path))
            self.assertEqual([chunk["id"] for chunk in manifest["chunks"]], ["0_0"])


if __name__ == "__main__":
    unittest.main()
