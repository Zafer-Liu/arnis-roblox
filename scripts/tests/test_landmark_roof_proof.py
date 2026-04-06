from __future__ import annotations

import importlib.util
import json
import shutil
import sys
import tempfile
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "landmark_roof_proof.py"


def load_module():
    scripts_dir = str(MODULE_PATH.parent)
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
    spec = importlib.util.spec_from_file_location("landmark_roof_proof", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load module spec from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


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
                    "buildingModelCount": 2,
                    "roadSurfacePartCount": 1,
                    "propInstanceCount": 0,
                },
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
                "nearbyBuildingModels": 2,
                "nearbyRoofParts": 4,
                "overheadRoofParts": 2,
                "nearbyReadableFacadeCueParts": 6,
                "nearbyWallParts": 8,
                "collidableWallPartsNearby": 8,
                "nearestBuildingSourceIds": ["building_capitol", "building_office"],
            },
            separators=(",", ":"),
        ),
    ]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_truncated_play_log(path: Path) -> None:
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
                    "buildingModelCount": 2,
                    "roadSurfacePartCount": 1,
                    "propInstanceCount": 0,
                },
            },
            separators=(",", ":"),
        ),
        'ARNIS_CLIENT_WORLD_COMPACT {"groundMaterial":"Enum.Material.Asphalt","nearbyReadableFacadeCueParts":6,"nearestBuildingSourceIds":["building_capitol","building_office"],"nearbyBuildingModels":2,"nearbyRoofParts":4,"supportSurfaceRole":"road"',
        'ARNIS_CLIENT_LOCAL_EXPERIENCE {"localRoofCover":{"overheadRoofParts":2},"localEnclosure":{"nearbyWallParts":8,"collidableWallPartsNearby":8}',
    ]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


class LandmarkRoofProofTests(unittest.TestCase):
    def test_build_landmark_roof_proof_accepts_route_runtime_index(self) -> None:
        module = load_module()
        if shutil.which("lua") is None:
            self.skipTest("lua is not installed")

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            route_runtime_dir = root / "route-runtime" / "step-000-active"
            shard_dir = route_runtime_dir / "PlanetaryManifestChunks"
            shard_dir.mkdir(parents=True)
            index_path = route_runtime_dir / "PlanetaryManifestIndex.lua"
            log_path = root / "play.log"
            artifact_dir = root / "artifacts"

            index_path.write_text(
                'return {schemaVersion="0.4.0",meta={worldName="RouteSliceTest",generator="test",source="test",metersPerStud=1,chunkSizeStuds=256,bbox={minLat=0,minLon=0,maxLat=1,maxLon=1},totalFeatures=2,notes={fixture=true}},chunkRefs={{id="0_0",originStuds={x=0,y=0,z=0},featureCount=2,streamingCost=12,estimatedMemoryCost=32,partitionVersion="subplans.v1",subplans={},shards={"PlanetaryManifestIndex_001"}}},shardFolder="PlanetaryManifestChunks",shards={"PlanetaryManifestIndex_001"}}',
                encoding="utf-8",
            )
            (shard_dir / "PlanetaryManifestIndex_001.lua").write_text(
                'return {chunks={{id="0_0",originStuds={x=0,y=0,z=0},terrain={cellSizeStuds=4,width=1,depth=1,material="Grass",heights={0}},roads={{id="road_1",kind="secondary",subkind="sidewalk"}},buildings={{id="building_capitol",name="Texas State Capitol",usage="government",roof="flat",roofShape="flat",roofMaterial="copper",material="Limestone"},{id="building_office",usage="office",roof="dome",roofShape="dome",material="Concrete"}},props={}}}}',
                encoding="utf-8",
            )
            write_play_log(log_path)

            proof = module.build_landmark_roof_proof(
                None,
                log_path,
                artifact_dir,
                route_runtime_index=index_path,
            )

            self.assertEqual(proof["chunkIds"], ["0_0"])
            self.assertEqual(proof["manifestSourceKind"], "route_catalog")
            self.assertEqual(proof["playerNearbyBuildings"][0]["id"], "building_capitol")
            self.assertEqual(proof["playerNearbyBuildings"][1]["id"], "building_office")
            self.assertEqual(proof["namedBuildingsInSlice"][0]["name"], "Texas State Capitol")
            self.assertEqual(proof["iconicRoofBuildingsInSlice"][0]["id"], "building_office")
            self.assertEqual(proof["clientWorldSummary"]["overheadRoofParts"], 2)
            self.assertTrue((artifact_dir / "landmark-roof-proof.json").exists())
            self.assertTrue((artifact_dir / "landmark-roof-proof.md").exists())

    def test_build_landmark_roof_proof_salvages_truncated_client_marker_fields(self) -> None:
        module = load_module()
        if shutil.which("lua") is None:
            self.skipTest("lua is not installed")

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            route_runtime_dir = root / "route-runtime" / "step-000-active"
            shard_dir = route_runtime_dir / "PlanetaryManifestChunks"
            shard_dir.mkdir(parents=True)
            index_path = route_runtime_dir / "PlanetaryManifestIndex.lua"
            log_path = root / "play.log"
            artifact_dir = root / "artifacts"

            index_path.write_text(
                'return {schemaVersion="0.4.0",meta={worldName="RouteSliceTest",generator="test",source="test",metersPerStud=1,chunkSizeStuds=256,bbox={minLat=0,minLon=0,maxLat=1,maxLon=1},totalFeatures=2,notes={fixture=true}},chunkRefs={{id="0_0",originStuds={x=0,y=0,z=0},featureCount=2,streamingCost=12,estimatedMemoryCost=32,partitionVersion="subplans.v1",subplans={},shards={"PlanetaryManifestIndex_001"}}},shardFolder="PlanetaryManifestChunks",shards={"PlanetaryManifestIndex_001"}}',
                encoding="utf-8",
            )
            (shard_dir / "PlanetaryManifestIndex_001.lua").write_text(
                'return {chunks={{id="0_0",originStuds={x=0,y=0,z=0},terrain={cellSizeStuds=4,width=1,depth=1,material="Grass",heights={0}},roads={{id="road_1",kind="secondary",subkind="sidewalk"}},buildings={{id="building_capitol",name="Texas State Capitol",usage="government",roof="flat",roofShape="flat",roofMaterial="copper",material="Limestone"},{id="building_office",usage="office",roof="dome",roofShape="dome",material="Concrete"}},props={}}}}',
                encoding="utf-8",
            )
            write_truncated_play_log(log_path)

            proof = module.build_landmark_roof_proof(
                None,
                log_path,
                artifact_dir,
                route_runtime_index=index_path,
            )

            self.assertEqual(proof["clientWorldSummary"]["nearbyBuildingModels"], 2)
            self.assertEqual(proof["clientWorldSummary"]["nearbyRoofParts"], 4)
            self.assertEqual(proof["clientWorldSummary"]["overheadRoofParts"], 2)
            self.assertEqual(proof["clientWorldSummary"]["supportSurfaceRole"], "road")
            self.assertEqual(proof["clientWorldSummary"]["nearbyWallParts"], 8)
            self.assertEqual(proof["clientWorldSummary"]["collidableWallPartsNearby"], 8)
            self.assertEqual(proof["clientWorldSummary"]["nearbyReadableFacadeCueParts"], 6)
            self.assertEqual(proof["playerNearbyBuildings"][0]["id"], "building_capitol")
            self.assertEqual(proof["playerNearbyBuildings"][1]["id"], "building_office")


if __name__ == "__main__":
    unittest.main()
