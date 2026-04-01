from __future__ import annotations

import importlib.util
import json
import sqlite3
import tempfile
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "scene_fidelity_audit.py"


def load_module():
    spec = importlib.util.spec_from_file_location("scene_fidelity_audit", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load module spec from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def build_truth_pack_fixture(db_path: Path) -> None:
    connection = sqlite3.connect(db_path)
    connection.executescript(
        """
        CREATE TABLE features (
            feature_id TEXT PRIMARY KEY,
            feature_kind TEXT NOT NULL,
            canonical_feature_id TEXT,
            is_retained INTEGER NOT NULL
        );
        CREATE TABLE sources (
            source_name TEXT PRIMARY KEY,
            provider TEXT NOT NULL,
            dataset TEXT NOT NULL
        );
        CREATE TABLE feature_sources (
            feature_id TEXT NOT NULL,
            source_name TEXT NOT NULL,
            source_feature_id TEXT NOT NULL,
            source_layer TEXT NOT NULL
        );
        CREATE TABLE retained_semantics (
            feature_id TEXT NOT NULL,
            field_name TEXT NOT NULL,
            field_value TEXT NOT NULL
        );
        CREATE TABLE collapses (
            collapse_id INTEGER PRIMARY KEY AUTOINCREMENT,
            feature_id TEXT NOT NULL,
            retained_feature_id TEXT NOT NULL,
            collapse_kind TEXT NOT NULL,
            matched_source TEXT NOT NULL
        );
        CREATE TABLE dropped_semantics (
            dropped_semantic_id INTEGER PRIMARY KEY AUTOINCREMENT,
            feature_id TEXT NOT NULL,
            field_name TEXT NOT NULL,
            field_value TEXT NOT NULL,
            reason TEXT NOT NULL,
            retained_feature_id TEXT
        );
        """
    )
    connection.executemany(
        "INSERT INTO sources (source_name, provider, dataset) VALUES (?, ?, ?)",
        [
            ("overpass", "osm", "overpass"),
            ("overture", "overture", "buildings"),
        ],
    )
    connection.executemany(
        "INSERT INTO features (feature_id, feature_kind, canonical_feature_id, is_retained) VALUES (?, ?, ?, ?)",
        [
            ("road_1", "road", "road_1", 1),
            ("structure_1", "structure", "structure_1", 1),
            ("structure_overlap_1", "structure", "structure_1", 0),
            ("structure_overlap_2", "structure", "structure_1", 0),
        ],
    )
    connection.executemany(
        "INSERT INTO feature_sources (feature_id, source_name, source_feature_id, source_layer) VALUES (?, ?, ?, ?)",
        [
            ("road_1", "overpass", "road-src-1", "roads"),
            ("structure_1", "overpass", "osm-1", "structures"),
            ("structure_overlap_1", "overture", "ov-1", "structures"),
            ("structure_overlap_2", "overture", "ov-2", "structures"),
        ],
    )
    connection.executemany(
        "INSERT INTO retained_semantics (feature_id, field_name, field_value) VALUES (?, ?, ?)",
        [
            ("road_1", "surface", "asphalt"),
            ("structure_1", "usage", "school"),
        ],
    )
    connection.executemany(
        "INSERT INTO collapses (feature_id, retained_feature_id, collapse_kind, matched_source) VALUES (?, ?, ?, ?)",
        [
            ("structure_overlap_1", "structure_1", "cross_source_overlap", "overture->osm"),
            ("structure_overlap_2", "structure_1", "cross_source_overlap", "overture->osm"),
        ],
    )
    connection.executemany(
        "INSERT INTO dropped_semantics (feature_id, field_name, field_value, reason, retained_feature_id) VALUES (?, ?, ?, ?, ?)",
        [
            ("structure_overlap_1", "usage", "commercial", "collapsed_into_retained_feature", "structure_1"),
            ("structure_overlap_2", "material", "glass", "collapsed_into_retained_feature", "structure_1"),
        ],
    )
    connection.commit()
    connection.close()

    db_path.with_suffix(".summary.json").write_text(
        json.dumps(
            {
                "scene": "fixture",
                "feature_count": 4,
                "retained_semantic_count": 2,
                "dropped_semantic_count": 2,
                "collapse_count": 2,
                "source_counts": {"overpass": 2, "overture": 2},
                "outdoor_source_coverage": {
                    "terrain": {"source_feature_count": 0, "retained_feature_count": 0},
                    "landuse": {"source_feature_count": 0, "retained_feature_count": 0},
                    "roads": {"source_feature_count": 1, "retained_feature_count": 1},
                    "water": {"source_feature_count": 0, "retained_feature_count": 0},
                    "vegetation": {"source_feature_count": 0, "retained_feature_count": 0},
                    "structures": {"source_feature_count": 3, "retained_feature_count": 1},
                },
            }
        ),
        encoding="utf-8",
    )


class SceneFidelityAuditTests(unittest.TestCase):
    maxDiff = None

    def test_report_rejects_missing_or_non_current_schema_version(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"

            log_path.write_text(
                "ARNIS_SCENE_PLAY "
                + json.dumps(
                    {
                        "phase": "play",
                        "focus": {"x": 0.0, "z": 0.0},
                        "radius": 512.0,
                        "rootName": "GeneratedWorld_Austin",
                        "worldIdentity": "AustinManifestIndex",
                        "chunkEnvelopeKind": "runtime_resident",
                        "scene": {"chunkCount": 0, "buildingModelCount": 0},
                    },
                    separators=(",", ":"),
                ),
                encoding="utf-8",
            )

            manifest_path.write_text(
                json.dumps(
                    {
                        "meta": {
                            "worldName": "SchemaAuditTown",
                            "metersPerStud": 1.0,
                            "chunkSizeStuds": 256,
                        },
                        "chunks": [],
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaises(SystemExit) as missing_schema_error:
                audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_PLAY")
            self.assertIn("schemaVersion", str(missing_schema_error.exception))

            manifest_path.write_text(
                json.dumps(
                    {
                        "schemaVersion": "0.3.0",
                        "meta": {
                            "worldName": "SchemaAuditTown",
                            "metersPerStud": 1.0,
                            "chunkSizeStuds": 256,
                        },
                        "chunks": [],
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaises(SystemExit) as unsupported_schema_error:
                audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_PLAY")
            self.assertIn("unsupported schemaVersion", str(unsupported_schema_error.exception))

    def test_report_parses_latest_scene_marker_and_flags_missing_geometry(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"
            html_path = root / "scene-report.html"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "SceneAuditTown",
                    "metersPerStud": 0.3,
                    "chunkSizeStuds": 256,
                },
                "chunks": [
                    {
                        "id": "0_0",
                        "originStuds": {"x": 0, "y": 0, "z": 0},
                        "roads": [
                            {"id": "road_1", "kind": "secondary", "subkind": "sidewalk"},
                            {"id": "road_2", "kind": "secondary", "subkind": "none"},
                        ],
                        "buildings": [
                            {
                                "id": "bldg_1",
                                "usage": "office",
                                "roof": "flat",
                                "material": "Concrete",
                                "roofMaterial": "Slate",
                            },
                            {
                                "id": "bldg_2",
                                "usage": "office",
                                "roof": "flat",
                                "material": "stone",
                                "roofMaterial": "copper",
                            },
                        ],
                        "water": [
                            {
                                "id": "water_poly_1",
                                "kind": "pond",
                                "material": "Water",
                                "type": "polygon",
                                "footprint": [
                                    {"x": 16, "z": 16},
                                    {"x": 48, "z": 16},
                                    {"x": 48, "z": 48},
                                    {"x": 16, "z": 48},
                                ],
                            },
                            {
                                "id": "water_ribbon_1",
                                "kind": "stream",
                                "material": "Water",
                                "points": [{"x": 96, "y": 0, "z": 32}, {"x": 144, "y": 0, "z": 48}],
                                "widthStuds": 8,
                            },
                        ],
                        "props": [
                            {
                                "id": "prop_tree_1",
                                "kind": "tree",
                                "species": "oak",
                                "position": {"x": 60, "y": 0, "z": 60},
                            },
                            {
                                "id": "prop_fountain_1",
                                "kind": "fountain",
                                "position": {"x": 84, "y": 0, "z": 72},
                            },
                        ],
                        "landuse": [],
                        "barriers": [],
                        "rails": [],
                    },
                    {
                        "id": "1_0",
                        "originStuds": {"x": 256, "y": 0, "z": 0},
                        "roads": [{"id": "road_3", "kind": "residential", "subkind": "sidewalk"}],
                        "buildings": [{"id": "bldg_3", "usage": "government", "roof": "gabled"}],
                        "water": [],
                        "props": [],
                        "landuse": [],
                        "barriers": [],
                        "rails": [],
                    },
                ],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            stale_payload = {
                "phase": "play",
                "focus": {"x": 64.0, "z": 64.0},
                "radius": 350.0,
                "rootName": "GeneratedWorld_Austin",
                "worldIdentity": "StaleWorldIdentity",
                "chunkEnvelopeKind": "runtime_resident",
                "scene": {
                    "chunkCount": 1,
                    "buildingModelCount": 1,
                    "chunksWithBuildingModels": 1,
                    "roadTaggedPartCount": 1,
                    "chunksWithRoadGeometry": 1,
                },
            }
            live_payload = {
                "phase": "play",
                "focus": {"x": 128.0, "z": 128.0},
                "radius": 400.0,
                "rootName": "GeneratedWorld_Austin",
                "worldIdentity": "AustinManifestIndex",
                "chunkEnvelopeKind": "runtime_resident",
                "scene": {
                    "chunkCount": 2,
                    "buildingModelCount": 1,
                    "buildingDetailPartCount": 0,
                    "buildingModelsWithRoof": 0,
                    "buildingModelsWithoutRoof": 1,
                    "buildingModelsWithDirectRoof": 0,
                    "buildingModelsWithMergedRoofOnly": 0,
                    "buildingModelsWithNoRoofEvidence": 1,
                    "buildingShellMeshPartCount": 0,
                    "chunksWithBuildingModels": 1,
                    "roadTaggedPartCount": 0,
                    "chunksWithRoadGeometry": 0,
                    "roadSurfacePartCountByKind": {
                        "secondary": {
                            "surfacePartCount": 1,
                            "featureCount": 1,
                            "sourceIds": ["road_1"],
                        }
                    },
                    "roadSurfacePartCountBySubkind": {
                        "sidewalk": {
                            "surfacePartCount": 1,
                            "featureCount": 1,
                            "sourceIds": ["road_1"],
                        }
                    },
                    "buildingRoofCoverageByUsage": {
                        "office": {
                            "buildingModelCount": 1,
                            "withRoofCount": 0,
                            "withoutRoofCount": 1,
                            "directRoofCount": 0,
                            "mergedRoofOnlyCount": 0,
                            "noRoofEvidenceCount": 1,
                        }
                    },
                    "buildingRoofCoverageByShape": {
                        "flat": {
                            "buildingModelCount": 1,
                            "withRoofCount": 0,
                            "withoutRoofCount": 1,
                            "directRoofCount": 0,
                            "mergedRoofOnlyCount": 0,
                            "noRoofEvidenceCount": 1,
                        }
                    },
                    "buildingModelCountByWallMaterial": {
                        "concrete": {"buildingModelCount": 1, "sourceIds": ["bldg_1"]},
                    },
                    "buildingModelCountByRoofMaterial": {
                        "slate": {"buildingModelCount": 1, "sourceIds": ["bldg_1"]},
                    },
                    "waterSurfacePartCount": 1,
                    "waterSurfacePartCountByType": {
                        "polygon": {"surfacePartCount": 1, "sourceIds": ["water_poly_1"]},
                    },
                    "waterSurfacePartCountByKind": {
                        "pond": {"surfacePartCount": 1, "sourceIds": ["water_poly_1"]},
                    },
                    "propInstanceCount": 1,
                    "propInstanceCountByKind": {
                        "tree": {"instanceCount": 1},
                    },
                    "ambientPropInstanceCount": 2,
                    "ambientPropInstanceCountByKind": {
                        "unknown": {"instanceCount": 2},
                    },
                    "treeInstanceCount": 1,
                    "treeInstanceCountBySpecies": {
                        "oak": {"instanceCount": 1},
                    },
                    "vegetationInstanceCount": 1,
                    "vegetationInstanceCountByKind": {
                        "tree": {"instanceCount": 1},
                    },
                    "chunksWithProps": 1,
                    "chunksWithVegetation": 1,
                    "chunksWithAmbientProps": 1,
                    "chunksWithWaterGeometry": 0,
                    "meshPartCount": 0,
                    "basePartCount": 0,
                },
            }
            live_chunks = {
                "phase": "play",
                "rootName": "GeneratedWorld_Austin",
                "chunkIds": ["0_0", "1_0"],
            }
            live_roof_usage = {
                "phase": "play",
                "rootName": "GeneratedWorld_Austin",
                "bucket": "office",
                "stats": live_payload["scene"]["buildingRoofCoverageByUsage"]["office"],
            }
            live_roof_shapes = {
                "phase": "play",
                "rootName": "GeneratedWorld_Austin",
                "buildingRoofCoverageByShape": live_payload["scene"]["buildingRoofCoverageByShape"],
            }
            trailing_scalar = {
                "phase": "play",
                "rootName": "GeneratedWorld_Austin",
                "key": "proceduralTreeInstanceCount",
                "value": 3,
            }
            log_path.write_text(
                "\n".join(
                    [
                        'ARNIS_SCENE_PLAY {"phase":"play","scene":{"buildingModelCount":1,"broken":"unterminated}',
                        "ARNIS_SCENE_PLAY " + json.dumps(stale_payload, separators=(",", ":")),
                        'ARNIS_SCENE_PLAY_ROOF_USAGE_BUCKET {"phase":"play","bucket":"broken"',
                        "other log noise",
                        "ARNIS_SCENE_PLAY_CHUNKS " + json.dumps(live_chunks, separators=(",", ":")),
                        "ARNIS_SCENE_PLAY_ROOF_USAGE_BUCKET "
                        + json.dumps(live_roof_usage, separators=(",", ":")),
                        "ARNIS_SCENE_PLAY_ROOF_SHAPES " + json.dumps(live_roof_shapes, separators=(",", ":")),
                        "ARNIS_SCENE_PLAY " + json.dumps(live_payload, separators=(",", ":")),
                        "ARNIS_SCENE_PLAY_SCALAR " + json.dumps(trailing_scalar, separators=(",", ":")),
                    ]
                ),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_PLAY")
            codes = {finding["code"] for finding in report["findings"]}

            self.assertEqual(report["rootName"], "GeneratedWorld_Austin")
            self.assertEqual(report["worldIdentity"], "AustinManifestIndex")
            self.assertEqual(report["chunkEnvelopeKind"], "runtime_resident")
            self.assertEqual(report["scene"]["chunkCount"], 2)
            self.assertEqual(report["scene"]["chunkIds"], ["0_0", "1_0"])
            self.assertEqual(report["scene"]["buildingModelCount"], 1)
            self.assertEqual(report["scene"]["buildingModelsWithRoof"], 0)
            self.assertEqual(report["scene"]["buildingModelsWithoutRoof"], 1)
            self.assertEqual(report["scene"]["buildingModelsWithDirectRoof"], 0)
            self.assertEqual(report["scene"]["buildingModelsWithMergedRoofOnly"], 0)
            self.assertEqual(report["scene"]["buildingModelsWithNoRoofEvidence"], 1)
            self.assertEqual(report["scene"]["roadSurfacePartCountByKind"]["secondary"]["surfacePartCount"], 1)
            self.assertEqual(report["scene"]["roadSurfacePartCountBySubkind"]["sidewalk"]["surfacePartCount"], 1)
            self.assertEqual(report["scene"]["buildingRoofCoverageByUsage"]["office"]["withoutRoofCount"], 1)
            self.assertEqual(report["scene"]["buildingRoofCoverageByUsage"]["office"]["noRoofEvidenceCount"], 1)
            self.assertEqual(report["scene"]["buildingRoofCoverageByShape"]["flat"]["withoutRoofCount"], 1)
            self.assertEqual(report["scene"]["buildingRoofCoverageByShape"]["flat"]["noRoofEvidenceCount"], 1)
            self.assertEqual(report["scene"]["waterSurfacePartCount"], 1)
            self.assertEqual(report["scene"]["waterSurfacePartCountByType"]["polygon"]["surfacePartCount"], 1)
            self.assertEqual(report["scene"]["waterSurfacePartCountByKind"]["pond"]["surfacePartCount"], 1)
            self.assertEqual(report["scene"]["propInstanceCount"], 1)
            self.assertEqual(report["scene"]["propInstanceCountByKind"]["tree"]["instanceCount"], 1)
            self.assertEqual(report["scene"]["ambientPropInstanceCount"], 2)
            self.assertEqual(report["scene"]["ambientPropInstanceCountByKind"]["unknown"]["instanceCount"], 2)
            self.assertEqual(report["scene"]["treeInstanceCount"], 1)
            self.assertEqual(report["scene"]["treeInstanceCountBySpecies"]["oak"]["instanceCount"], 1)
            self.assertEqual(report["scene"]["proceduralTreeInstanceCount"], 3)
            self.assertEqual(report["scene"]["vegetationInstanceCount"], 1)
            self.assertEqual(report["scene"]["vegetationInstanceCountByKind"]["tree"]["instanceCount"], 1)
            self.assertEqual(report["manifest"]["buildingCount"], 3)
            self.assertEqual(report["manifest"]["buildingCountByUsage"]["office"], 2)
            self.assertEqual(report["manifest"]["buildingCountByUsage"]["government"], 1)
            self.assertEqual(report["manifest"]["buildingCountByRoofShape"]["flat"], 2)
            self.assertEqual(report["manifest"]["buildingCountByRoofShape"]["gabled"], 1)
            self.assertEqual(report["manifest"]["buildingCountByExplicitWallMaterial"]["concrete"], 1)
            self.assertEqual(report["manifest"]["buildingCountByExplicitWallMaterial"]["cobblestone"], 1)
            self.assertEqual(report["manifest"]["buildingCountByExplicitRoofMaterial"]["slate"], 1)
            self.assertEqual(report["manifest"]["buildingCountByExplicitRoofMaterial"]["metal"], 1)
            self.assertEqual(report["manifest"]["roadCount"], 3)
            self.assertEqual(report["manifest"]["roadCountByKind"]["secondary"], 2)
            self.assertEqual(report["manifest"]["roadCountByKind"]["residential"], 1)
            self.assertEqual(report["manifest"]["roadCountBySubkind"]["sidewalk"], 2)
            self.assertEqual(report["manifest"]["roadCountBySubkind"]["none"], 1)

    def test_report_does_not_translate_legacy_building_kind_into_usage(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "LegacyUsageTown",
                    "metersPerStud": 1.0,
                    "chunkSizeStuds": 256,
                },
                "chunks": [
                    {
                        "id": "0_0",
                        "originStuds": {"x": 0, "y": 0, "z": 0},
                        "roads": [],
                        "buildings": [
                            {
                                "id": "legacy_building_1",
                                "kind": "office",
                                "roof": "flat",
                            }
                        ],
                        "water": [],
                        "props": [],
                        "landuse": [],
                        "barriers": [],
                        "rails": [],
                    }
                ],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            log_path.write_text(
                "ARNIS_SCENE_PLAY "
                + json.dumps(
                    {
                        "phase": "play",
                        "focus": {"x": 0.0, "z": 0.0},
                        "radius": 512.0,
                        "rootName": "GeneratedWorld_Austin",
                        "worldIdentity": "AustinManifestIndex",
                        "chunkEnvelopeKind": "runtime_resident",
                        "scene": {"chunkCount": 0, "buildingModelCount": 0},
                    },
                    separators=(",", ":"),
                ),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_PLAY")

            self.assertEqual(report["manifest"]["buildingCount"], 1)
            self.assertEqual(report["manifest"]["buildingCountByUsage"]["unknown"], 1)
            self.assertNotIn("office", report["manifest"]["buildingCountByUsage"])

    def test_report_parses_latest_client_world_compact_marker_without_deriving_extra_assumptions(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "SceneAuditTown",
                    "metersPerStud": 0.3,
                    "chunkSizeStuds": 256,
                },
                "chunks": [],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            scene_payload = {
                "phase": "play",
                "focus": {"x": 0.0, "z": 0.0},
                "radius": 64.0,
                "rootName": "GeneratedWorld_Austin",
                "worldIdentity": "AustinManifestIndex",
                "chunkEnvelopeKind": "runtime_resident",
                "scene": {"chunkCount": 0, "buildingModelCount": 0},
            }
            stale_client_world = {
                "worldRootName": "GeneratedWorld_Stale",
                "worldRootExists": True,
                "nearbyBuildingModels": 99,
                "nearbyMergedBuildingMeshParts": 12,
                "nearbyRoofParts": 34,
                "overheadRoofParts": 7,
                "groundMaterial": "Enum.Material.Grass",
                "bootstrapState": "stale",
            }
            live_client_world = {
                "worldRootName": "GeneratedWorld_Austin",
                "worldRootExists": True,
                "nearbyBuildingModels": 7,
                "nearbyMergedBuildingMeshParts": 0,
                "nearbyRoofParts": 14,
                "overheadRoofParts": 4,
                "groundMaterial": "Enum.Material.Concrete",
                "bootstrapState": "gameplay_ready",
                "bootstrapStateTrace": "loading_manifest,importing_startup,world_ready,streaming_ready,minimap_ready,gameplay_ready",
                "nearestBuildingSourceIds": ["osm_952130555", "osm_93135618"],
                "overheadRoofSourceIds": ["osm_952130555", "osm_269078411"],
            }
            log_path.write_text(
                "\n".join(
                    [
                        "ARNIS_CLIENT_WORLD_COMPACT " + json.dumps(stale_client_world, separators=(",", ":")),
                        "ARNIS_SCENE_PLAY " + json.dumps(scene_payload, separators=(",", ":")),
                        "ARNIS_CLIENT_WORLD_COMPACT " + json.dumps(live_client_world, separators=(",", ":")),
                    ]
                ),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_PLAY")

            self.assertEqual(report["clientWorld"]["worldRootName"], "GeneratedWorld_Austin")
            self.assertEqual(report["clientWorld"]["groundMaterial"], "Enum.Material.Concrete")
            self.assertEqual(report["clientWorld"]["nearbyBuildingModels"], 7)
            self.assertEqual(report["clientWorld"]["nearbyRoofParts"], 14)
            self.assertEqual(report["clientWorld"]["overheadRoofParts"], 4)
            self.assertEqual(report["clientWorld"]["bootstrapState"], "gameplay_ready")
            self.assertEqual(report["clientWorld"]["nearestBuildingSourceIds"], ["osm_952130555", "osm_93135618"])

    def test_report_preserves_structured_local_support_and_enclosure_client_world_fields(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "SupportTown",
                    "metersPerStud": 0.3,
                    "chunkSizeStuds": 256,
                },
                "chunks": [],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            scene_payload = {
                "phase": "play",
                "focus": {"x": 0.0, "z": 0.0},
                "radius": 64.0,
                "rootName": "GeneratedWorld_Austin",
                "worldIdentity": "AustinManifestIndex",
                "chunkEnvelopeKind": "runtime_resident",
                "scene": {"chunkCount": 0, "buildingModelCount": 0},
            }
            client_world = {
                "worldRootName": "GeneratedWorld_Austin",
                "worldRootExists": True,
                "nearbyBuildingModels": 4,
                "nearbyMergedBuildingMeshParts": 1,
                "nearbyRoofParts": 6,
                "overheadRoofParts": 2,
                "groundMaterial": "Enum.Material.Concrete",
                "supportSurfaceRole": "road",
                "supportY": 12.5,
                "terrainY": 11.0,
                "supportMinusTerrainYStuds": 1.5,
                "nearbyWallParts": 8,
                "collidableWallPartsNearby": 7,
                "nearestWallDistanceStuds": 4.25,
                "overheadRoofMinClearanceStuds": 13.5,
                "localSupport": {
                    "surfaceRole": "road",
                    "supportY": 12.5,
                    "terrainY": 11.0,
                    "supportMinusTerrainYStuds": 1.5,
                },
                "localEnclosure": {
                    "nearbyWallParts": 8,
                    "collidableWallPartsNearby": 7,
                    "nearestWallDistanceStuds": 4.25,
                },
                "localRoofCover": {
                    "nearbyRoofParts": 6,
                    "overheadRoofParts": 2,
                    "overheadRoofMinClearanceStuds": 13.5,
                },
            }
            log_path.write_text(
                "\n".join(
                    [
                        "ARNIS_SCENE_PLAY " + json.dumps(scene_payload, separators=(",", ":")),
                        "ARNIS_CLIENT_WORLD_COMPACT " + json.dumps(client_world, separators=(",", ":")),
                    ]
                ),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_PLAY")

            self.assertEqual(report["clientWorld"]["supportSurfaceRole"], "road")
            self.assertEqual(report["clientWorld"]["supportMinusTerrainYStuds"], 1.5)
            self.assertEqual(report["clientWorld"]["localSupport"]["surfaceRole"], "road")
            self.assertEqual(report["clientWorld"]["localEnclosure"]["nearbyWallParts"], 8)
            self.assertEqual(report["clientWorld"]["localRoofCover"]["overheadRoofMinClearanceStuds"], 13.5)

    def test_report_preserves_structured_local_terrain_client_world_fields(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"
            html_path = root / "report.html"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "TerrainProbeTown",
                    "metersPerStud": 0.3,
                    "chunkSizeStuds": 256,
                },
                "chunks": [],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            scene_payload = {
                "phase": "play",
                "focus": {"x": 0.0, "z": 0.0},
                "radius": 64.0,
                "rootName": "GeneratedWorld_Austin",
                "worldIdentity": "AustinManifestIndex",
                "chunkEnvelopeKind": "runtime_resident",
                "scene": {"chunkCount": 0, "buildingModelCount": 0},
            }
            client_world = {
                "worldRootName": "GeneratedWorld_Austin",
                "worldRootExists": True,
                "supportSurfaceRole": "terrain",
                "localSupport": {"surfaceRole": "terrain"},
                "localTerrain": {
                    "status": "ok",
                    "samplePattern": "cross_5",
                    "sampleRadiusStuds": 12,
                    "sampleCount": 5,
                    "missingSampleCount": 0,
                    "centerTerrainY": 10.0,
                    "minTerrainY": 8.0,
                    "maxTerrainY": 14.0,
                    "heightRangeStuds": 6.0,
                    "maxStepStuds": 4.0,
                    "meanAbsStepStuds": 2.5,
                },
            }
            log_path.write_text(
                "\n".join(
                    [
                        "ARNIS_SCENE_PLAY " + json.dumps(scene_payload, separators=(",", ":")),
                        "ARNIS_CLIENT_WORLD_COMPACT " + json.dumps(client_world, separators=(",", ":")),
                    ]
                ),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_PLAY")
            audit.write_html_report(report, html_path)

            self.assertEqual(report["clientWorld"]["localTerrain"]["status"], "ok")
            self.assertEqual(report["clientWorld"]["localTerrain"]["sampleCount"], 5)
            self.assertEqual(report["clientWorld"]["localTerrain"]["heightRangeStuds"], 6.0)
            self.assertEqual(report["summary"]["clientLocalTerrainStatus"], "ok")
            self.assertEqual(report["summary"]["clientLocalTerrainMaxStepStuds"], 4.0)
            self.assertEqual(report["summary"]["clientLocalTerrainMeanAbsStepStuds"], 2.5)

            html = html_path.read_text(encoding="utf-8")
            self.assertIn("client_local_terrain_status", html)
            self.assertIn("client_local_terrain_height_range_studs", html)
            self.assertIn("client_local_terrain_max_step_studs", html)

    def test_report_merges_dedicated_local_experience_marker_when_compact_marker_is_truncated(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {"worldName": "SceneAuditTown", "chunkSizeStuds": 256},
                "chunks": [],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            payload = {
                "phase": "play",
                "focus": {"x": 0.0, "z": 0.0},
                "radius": 128.0,
                "rootName": "GeneratedWorld_Austin",
                "worldIdentity": "AustinManifestIndex",
                "chunkEnvelopeKind": "runtime_resident",
                "scene": {
                    "chunkCount": 0,
                    "buildingModelCount": 0,
                    "chunksWithRoadGeometry": 0,
                },
            }
            local_experience = {
                "worldRootName": "GeneratedWorld_Austin",
                "worldRootExists": True,
                "bootstrapAttemptId": "attempt-7",
                "bootstrapState": "gameplay_ready",
                "localSupport": {
                    "surfaceRole": "terrain",
                    "supportY": 5.1,
                    "terrainY": 5.1,
                    "supportMinusTerrainYStuds": 0.0,
                    "sourceIds": [],
                },
                "localTerrain": {
                    "status": "ok",
                    "samplePattern": "cross_5",
                    "sampleRadiusStuds": 12.0,
                    "sampleCount": 5,
                    "missingSampleCount": 0,
                    "centerTerrainY": 5.1,
                    "minTerrainY": 4.8,
                    "maxTerrainY": 7.2,
                    "heightRangeStuds": 2.4,
                    "maxStepStuds": 1.7,
                    "meanAbsStepStuds": 0.9,
                },
                "localEnclosure": {
                    "nearbyWallParts": 4,
                    "collidableWallPartsNearby": 4,
                    "nearestWallDistanceStuds": 2.2,
                },
                "localRoofCover": {
                    "nearbyRoofParts": 8,
                    "overheadRoofParts": 2,
                    "overheadRoofMinClearanceStuds": 12.4,
                    "overheadRoofSourceIds": ["osm_1"],
                },
            }
            log_path.write_text(
                "\n".join(
                    [
                        "ARNIS_SCENE_PLAY " + json.dumps(payload, separators=(",", ":")),
                        'ARNIS_CLIENT_WORLD_COMPACT {"worldRootName":"GeneratedWorld_Austin","localTerrain"',
                        "ARNIS_CLIENT_LOCAL_EXPERIENCE " + json.dumps(local_experience, separators=(",", ":")),
                    ]
                ),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_PLAY")

            self.assertEqual(report["clientWorld"]["worldRootName"], "GeneratedWorld_Austin")
            self.assertEqual(report["clientWorld"]["bootstrapAttemptId"], "attempt-7")
            self.assertEqual(report["clientWorld"]["localTerrain"]["status"], "ok")
            self.assertEqual(report["clientWorld"]["localTerrain"]["maxStepStuds"], 1.7)
            self.assertEqual(report["summary"]["clientLocalTerrainStatus"], "ok")
            self.assertEqual(report["summary"]["clientLocalTerrainMaxStepStuds"], 1.7)

    def test_report_flags_missing_local_terrain_roughness_when_support_is_terrain(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "TerrainFindingTown",
                    "metersPerStud": 1.0,
                    "chunkSizeStuds": 256,
                },
                "chunks": [],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            scene_payload = {
                "phase": "play",
                "focus": {"x": 0.0, "z": 0.0},
                "radius": 64.0,
                "rootName": "GeneratedWorld_Austin",
                "worldIdentity": "AustinManifestIndex",
                "chunkEnvelopeKind": "runtime_resident",
                "scene": {"chunkCount": 0, "buildingModelCount": 0},
            }
            client_world = {
                "worldRootName": "GeneratedWorld_Austin",
                "worldRootExists": True,
                "supportSurfaceRole": "terrain",
                "localSupport": {"surfaceRole": "terrain"},
                "localTerrain": {
                    "status": "insufficient_samples",
                    "samplePattern": "cross_5",
                    "sampleRadiusStuds": 12,
                    "sampleCount": 1,
                    "missingSampleCount": 4,
                    "centerTerrainY": 10.0,
                    "minTerrainY": 10.0,
                    "maxTerrainY": 10.0,
                    "heightRangeStuds": None,
                    "maxStepStuds": None,
                    "meanAbsStepStuds": None,
                },
            }
            log_path.write_text(
                "\n".join(
                    [
                        "ARNIS_SCENE_PLAY " + json.dumps(scene_payload, separators=(",", ":")),
                        "ARNIS_CLIENT_WORLD_COMPACT " + json.dumps(client_world, separators=(",", ":")),
                    ]
                ),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_PLAY")
            codes = {finding["code"] for finding in report["findings"]}

            self.assertIn("client_local_terrain_roughness_missing", codes)

    def test_report_surfaces_player_local_exposure_findings_and_html_metrics(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"
            html_path = root / "report.html"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "ExposureTown",
                    "metersPerStud": 1.0,
                    "chunkSizeStuds": 256,
                },
                "chunks": [],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            scene_payload = {
                "phase": "play",
                "focus": {"x": 0.0, "z": 0.0},
                "radius": 64.0,
                "rootName": "GeneratedWorld_Austin",
                "worldIdentity": "AustinManifestIndex",
                "chunkEnvelopeKind": "runtime_resident",
                "scene": {"chunkCount": 0, "buildingModelCount": 0},
            }
            client_world = {
                "worldRootName": "GeneratedWorld_Austin",
                "worldRootExists": True,
                "nearbyBuildingModels": 3,
                "nearbyRoofParts": 4,
                "supportSurfaceRole": "unknown",
                "localSupport": {"surfaceRole": "unknown"},
                "localEnclosure": {
                    "nearbyWallParts": 0,
                    "collidableWallPartsNearby": 0,
                    "nearestWallDistanceStuds": None,
                },
                "localRoofCover": {
                    "nearbyRoofParts": 4,
                    "overheadRoofParts": 0,
                    "overheadRoofMinClearanceStuds": None,
                },
            }
            log_path.write_text(
                "\n".join(
                    [
                        "ARNIS_SCENE_PLAY " + json.dumps(scene_payload, separators=(",", ":")),
                        "ARNIS_CLIENT_WORLD_COMPACT " + json.dumps(client_world, separators=(",", ":")),
                    ]
                ),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_PLAY")
            audit.write_html_report(report, html_path)

            codes = {finding["code"] for finding in report["findings"]}
            self.assertIn("client_local_support_unknown", codes)
            self.assertIn("client_local_enclosure_gap", codes)
            self.assertIn("client_local_roof_cover_gap", codes)

            html = html_path.read_text(encoding="utf-8")
            self.assertIn("client_local_support_surface_role", html)
            self.assertIn("client_local_enclosure_nearby_wall_parts", html)
            self.assertIn("client_local_roof_cover_overhead_roof_parts", html)

    def test_report_carries_manifest_terrain_granularity_context(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"
            html_path = root / "report.html"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "TerrainSceneTown",
                    "metersPerStud": 1.0,
                    "chunkSizeStuds": 256,
                },
                "chunks": [
                    {
                        "id": "0_0",
                        "originStuds": {"x": 0, "y": 0, "z": 0},
                        "terrain": {
                            "cellSizeStuds": 4,
                            "width": 4,
                            "depth": 4,
                            "heights": [0] * 16,
                            "materials": ["Grass"] * 12 + ["Sand"] * 4,
                            "material": "Grass",
                        },
                        "roads": [],
                        "rails": [],
                        "buildings": [],
                        "props": [],
                        "water": [],
                    },
                    {
                        "id": "1_0",
                        "originStuds": {"x": 256, "y": 0, "z": 0},
                        "terrain": {
                            "cellSizeStuds": 8,
                            "width": 2,
                            "depth": 2,
                            "heights": [0] * 4,
                            "materials": ["Grass"] * 4,
                            "material": "Grass",
                        },
                        "roads": [],
                        "rails": [],
                        "buildings": [],
                        "props": [],
                        "water": [],
                    },
                ],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            scene_payload = {
                "phase": "play",
                "focus": {"x": 0.0, "z": 0.0},
                "radius": 1024.0,
                "rootName": "GeneratedWorld_Austin",
                "worldIdentity": "AustinManifestIndex",
                "chunkEnvelopeKind": "runtime_resident",
                "scene": {"chunkCount": 2, "buildingModelCount": 0},
            }
            log_path.write_text(
                "ARNIS_SCENE_PLAY " + json.dumps(scene_payload, separators=(",", ":")),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_PLAY")

            self.assertEqual(report["manifest"]["terrainCellSizeDistribution"], {"4": 1, "8": 1})
            self.assertEqual(report["manifest"]["terrainAreaByCellSizeStuds"], {"4": 256.0, "8": 256.0})
            self.assertEqual(report["manifest"]["terrainMaterialAreaDistribution"]["Grass"], 448.0)
            self.assertEqual(report["manifest"]["terrainMaterialAreaDistribution"]["Sand"], 64.0)
            self.assertEqual(report["manifest"]["terrainDominantMaterial"], "Grass")
            self.assertAlmostEqual(report["manifest"]["terrainDominantMaterialRatio"], 448.0 / 512.0, places=4)

            audit.write_html_report(report, html_path)
            html = html_path.read_text(encoding="utf-8")
            self.assertIn("manifest_terrain_dominant_material_ratio", html)
            self.assertIn("manifest_terrain_cell_size_distribution", html)

    def test_main_can_render_html_from_precomputed_report_json(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = root / "report.json"
            html_path = root / "report.html"
            report = {
                "summary": {
                    "marker": "ARNIS_SCENE_EDIT",
                    "building_model_ratio": 0.5,
                    "road_geometry_ratio": 1.0,
                    "chunk_ratio": 1.0,
                    "water_geometry_ratio": 0.5,
                },
                "scene": {
                    "chunkCount": 1,
                    "buildingModelsWithDirectShell": 7,
                    "buildingModelsMissingDirectShell": 2,
                    "buildingModelsWithRoof": 5,
                    "buildingModelsWithoutRoof": 2,
                    "buildingModelsWithDirectRoof": 3,
                    "buildingModelsWithMergedRoofOnly": 2,
                    "buildingModelsWithNoRoofEvidence": 2,
                    "buildingRoofCoverageByUsage": {
                        "office": {
                            "buildingModelCount": 4,
                            "withRoofCount": 3,
                            "withoutRoofCount": 1,
                            "directRoofCount": 2,
                            "mergedRoofOnlyCount": 1,
                            "noRoofEvidenceCount": 1,
                        }
                    },
                    "buildingRoofCoverageByShape": {
                        "flat": {
                            "buildingModelCount": 4,
                            "withRoofCount": 3,
                            "withoutRoofCount": 1,
                            "directRoofCount": 2,
                            "mergedRoofOnlyCount": 1,
                            "noRoofEvidenceCount": 1,
                        }
                    },
                    "waterSurfacePartCount": 3,
                    "waterSurfacePartCountByType": {
                        "polygon": {"surfacePartCount": 2},
                        "ribbon": {"surfacePartCount": 1},
                    },
                    "waterSurfacePartCountByKind": {
                        "pond": {"surfacePartCount": 2},
                        "stream": {"surfacePartCount": 1},
                    },
                    "propInstanceCount": 5,
                    "propInstanceCountByKind": {
                        "tree": {"instanceCount": 3},
                        "fountain": {"instanceCount": 2},
                    },
                    "ambientPropInstanceCount": 4,
                    "ambientPropInstanceCountByKind": {
                        "unknown": {"instanceCount": 4},
                    },
                    "treeInstanceCount": 3,
                    "treeInstanceCountBySpecies": {
                        "oak": {"instanceCount": 2},
                        "elm": {"instanceCount": 1},
                    },
                    "vegetationInstanceCount": 3,
                    "vegetationInstanceCountByKind": {
                        "tree": {"instanceCount": 3},
                    },
                    "chunksWithProps": 1,
                    "chunksWithVegetation": 1,
                    "chunksWithAmbientProps": 1,
                    "chunksWithWaterGeometry": 1,
                    "mergedBuildingMeshPartCount": 5,
                    "roadCrosswalkStripeCount": 9,
                },
                "manifest": {
                    "chunkCount": 2,
                    "buildingCountByUsage": {"office": 4},
                    "buildingCountByRoofShape": {"flat": 4},
                    "roadCountByKind": {"secondary": 2, "residential": 1},
                    "roadCountBySubkind": {"sidewalk": 2, "none": 1},
                    "propCount": 5,
                    "propCountByKind": {"tree": 3, "fountain": 2},
                    "treeCount": 3,
                    "treeCountBySpecies": {"oak": 2, "elm": 1},
                    "vegetationCount": 3,
                    "vegetationCountByKind": {"tree": 3},
                    "waterCountByKind": {"pond": 1, "stream": 1},
                    "waterCountByType": {"polygon": 1, "ribbon": 1},
                },
                "findings": [
                    {
                        "severity": "high",
                        "code": "missing_building_models",
                        "message": "scene built 1 building models but manifest expected 2",
                    }
                ],
            }
            report_path.write_text(json.dumps(report), encoding="utf-8")

            exit_code = audit.main(["--report-json", str(report_path), "--html-out", str(html_path)])

            self.assertEqual(exit_code, 0)
            html = html_path.read_text(encoding="utf-8")
            self.assertIn("ARNIS_SCENE_EDIT", html)
            self.assertIn("missing_building_models", html)
            self.assertIn("building_model_ratio", html)
            self.assertIn("building_models_with_direct_shell", html)
            self.assertIn("building_models_missing_direct_shell", html)
            self.assertIn("building_models_with_roof", html)
            self.assertIn("building_models_without_roof", html)
            self.assertIn("building_models_with_direct_roof", html)
            self.assertIn("building_models_with_merged_roof_only", html)
            self.assertIn("building_models_with_no_roof_evidence", html)
            self.assertIn("water_surface_part_count", html)
            self.assertIn("prop_instance_count", html)
            self.assertIn("ambient_prop_instance_count", html)
            self.assertIn("tree_instance_count", html)
            self.assertIn("vegetation_instance_count", html)
            self.assertIn("chunks_with_water_geometry", html)
            self.assertIn("Roof Coverage By Usage", html)
            self.assertIn("Scene Roof Coverage By Shape", html)
            self.assertIn("Manifest Roof Expectations By Usage", html)
            self.assertIn("Manifest Roof Expectations By Shape", html)
            self.assertIn("Water Surface Breakdown", html)
            self.assertIn("Water Surface By Kind", html)
            self.assertIn("Prop Breakdown", html)
            self.assertIn("Ambient Props", html)
            self.assertIn("Tree Species", html)
            self.assertIn("Vegetation Breakdown", html)
            self.assertIn("office", html)
            self.assertIn("flat", html)
            self.assertIn("polygon", html)
            self.assertIn("ribbon", html)
            self.assertIn("pond", html)
            self.assertIn("stream", html)
            self.assertIn("oak", html)
            self.assertIn("merged_building_mesh_part_count", html)
            self.assertIn("road_crosswalk_stripe_count", html)

    def test_main_enriches_loaded_report_from_raw_log_and_rewrites_json(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = root / "report.json"
            log_path = root / "studio.log"
            out_path = root / "report.enriched.json"
            html_path = root / "report.html"

            report = {
                "phase": "play",
                "rootName": "GeneratedWorld_Austin",
                "scene": {
                    "buildingModelCount": 1,
                },
                "clientWorld": {
                    "nearbyBuildingModels": 1,
                    "nearbyRoofParts": 1,
                },
                "manifest": {
                    "chunkCount": 1,
                    "buildingCount": 1,
                    "chunksWithRoads": 0,
                    "chunksWithRails": 0,
                    "railCount": 0,
                    "propCount": 0,
                    "vegetationCount": 0,
                    "chunksWithWater": 0,
                    "roadCountByKind": {},
                    "roadCountBySubkind": {},
                    "railCountByKind": {},
                    "waterCountByKind": {},
                    "waterCountByType": {},
                    "propCountByKind": {},
                    "treeCountBySpecies": {},
                    "vegetationCountByKind": {},
                    "buildingCountByExplicitWallMaterial": {},
                    "buildingCountByExplicitRoofMaterial": {},
                },
                "summary": {
                    "marker": "ARNIS_SCENE_PLAY",
                    "building_model_ratio": 1.0,
                },
                "findings": [],
            }
            local_experience = {
                "worldRootName": "GeneratedWorld_Austin",
                "localSupport": {"surfaceRole": "terrain", "supportMinusTerrainYStuds": 0.0},
                "localTerrain": {
                    "status": "ok",
                    "sampleCount": 5,
                    "missingSampleCount": 0,
                    "maxStepStuds": 1.25,
                    "meanAbsStepStuds": 0.5,
                },
                "localEnclosure": {"nearbyWallParts": 2, "collidableWallPartsNearby": 2},
                "localRoofCover": {"nearbyRoofParts": 1, "overheadRoofParts": 1},
            }
            report_path.write_text(json.dumps(report), encoding="utf-8")
            log_path.write_text(
                "ARNIS_CLIENT_LOCAL_EXPERIENCE " + json.dumps(local_experience, separators=(",", ":")) + "\n",
                encoding="utf-8",
            )

            exit_code = audit.main(
                [
                    "--report-json",
                    str(report_path),
                    "--log",
                    str(log_path),
                    "--json-out",
                    str(out_path),
                    "--html-out",
                    str(html_path),
                ]
            )

            self.assertEqual(exit_code, 0)
            enriched = json.loads(out_path.read_text(encoding="utf-8"))
            self.assertEqual(enriched["clientWorld"]["localTerrain"]["status"], "ok")
            self.assertEqual(enriched["summary"]["clientLocalTerrainStatus"], "ok")
            self.assertEqual(enriched["summary"]["clientLocalTerrainMaxStepStuds"], 1.25)
            self.assertFalse(any(item["code"] == "client_local_enclosure_gap" for item in enriched["findings"]))
            html = html_path.read_text(encoding="utf-8")
            self.assertIn("client_local_terrain_status", html)
            self.assertIn("localTerrain", html)

    def test_report_reassembles_split_prop_and_vegetation_buckets(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "SceneAuditTown",
                    "metersPerStud": 0.3,
                    "chunkSizeStuds": 256,
                },
                "chunks": [
                    {
                        "id": "0_0",
                        "originStuds": {"x": 0, "y": 0, "z": 0},
                        "roads": [],
                        "buildings": [],
                        "water": [],
                        "props": [
                            {"id": "tree_1", "kind": "tree", "species": "oak", "position": {"x": 0, "y": 0, "z": 0}},
                            {
                                "id": "fountain_1",
                                "kind": "fountain",
                                "position": {"x": 4, "y": 0, "z": 4},
                            },
                        ],
                        "landuse": [],
                        "barriers": [],
                        "rails": [],
                    }
                ],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            scene_payload = {
                "phase": "edit",
                "focus": {"x": 32.0, "z": 32.0},
                "radius": 256.0,
                "rootName": "GeneratedWorld_AustinPreview",
                "scene": {
                    "chunkCount": 1,
                    "propInstanceCount": 2,
                    "ambientPropInstanceCount": 3,
                    "treeInstanceCount": 1,
                    "vegetationInstanceCount": 1,
                    "chunksWithProps": 1,
                    "chunksWithAmbientProps": 1,
                    "chunksWithVegetation": 1,
                    "chunksWithRoadGeometry": 0,
                    "chunksWithWaterGeometry": 0,
                    "buildingModelCount": 0,
                },
            }
            log_path.write_text(
                "\n".join(
                    [
                        "ARNIS_SCENE_EDIT_CHUNKS "
                        + json.dumps(
                            {"phase": "edit", "rootName": "GeneratedWorld_AustinPreview", "chunkIds": ["0_0"]},
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_PROP_KIND_BUCKET "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "tree",
                                "stats": {"instanceCount": 1},
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_PROP_KIND_IDS_BATCH "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "tree",
                                "sourceIds": ["tree_1"],
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_PROP_KIND_BUCKET "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "fountain",
                                "stats": {"instanceCount": 1},
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_PROP_KIND_IDS_BATCH "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "fountain",
                                "sourceIds": ["fountain_1"],
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_AMBIENT_PROP_KIND_BUCKET "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "unknown",
                                "stats": {"instanceCount": 3},
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_TREE_SPECIES_BUCKET "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "oak",
                                "stats": {"instanceCount": 1},
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_TREE_SPECIES_IDS_BATCH "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "oak",
                                "sourceIds": ["tree_1"],
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_VEGETATION_KIND_BUCKET "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "tree",
                                "stats": {"instanceCount": 1},
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_VEGETATION_KIND_IDS_BATCH "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "tree",
                                "sourceIds": ["tree_1"],
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT " + json.dumps(scene_payload, separators=(",", ":")),
                    ]
                ),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_EDIT")

            self.assertEqual(report["scene"]["propInstanceCountByKind"]["tree"]["instanceCount"], 1)
            self.assertEqual(report["scene"]["propInstanceCountByKind"]["tree"]["sourceIds"], ["tree_1"])
            self.assertEqual(report["scene"]["propInstanceCountByKind"]["fountain"]["instanceCount"], 1)
            self.assertEqual(report["scene"]["propInstanceCountByKind"]["fountain"]["sourceIds"], ["fountain_1"])
            self.assertEqual(report["scene"]["ambientPropInstanceCountByKind"]["unknown"]["instanceCount"], 3)
            self.assertEqual(report["scene"]["treeInstanceCountBySpecies"]["oak"]["instanceCount"], 1)
            self.assertEqual(report["scene"]["treeInstanceCountBySpecies"]["oak"]["sourceIds"], ["tree_1"])
            self.assertEqual(report["scene"]["vegetationInstanceCountByKind"]["tree"]["instanceCount"], 1)
            self.assertEqual(report["scene"]["vegetationInstanceCountByKind"]["tree"]["sourceIds"], ["tree_1"])

    def test_report_reassembles_split_scalar_and_water_buckets(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "SceneAuditTown",
                    "metersPerStud": 0.3,
                    "chunkSizeStuds": 256,
                },
                "chunks": [
                    {
                        "id": "0_0",
                        "originStuds": {"x": 0, "y": 0, "z": 0},
                        "roads": [],
                        "buildings": [],
                        "water": [
                            {
                                "id": "water_1",
                                "kind": "pond",
                                "material": "Water",
                                "type": "polygon",
                                "footprint": [
                                    {"x": 16, "z": 16},
                                    {"x": 48, "z": 16},
                                    {"x": 48, "z": 48},
                                    {"x": 16, "z": 48},
                                ],
                            }
                        ],
                        "props": [],
                        "landuse": [],
                        "barriers": [],
                        "rails": [],
                    }
                ],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            base_payload = {
                "phase": "edit",
                "focus": {"x": 32.0, "z": 32.0},
                "radius": 256.0,
                "rootName": "GeneratedWorld_AustinPreview",
                "scene": {},
            }
            log_path.write_text(
                "\n".join(
                    [
                        "ARNIS_SCENE_EDIT_SCALAR "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "key": "buildingModelCount",
                                "value": 2,
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_SCALAR "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "key": "waterSurfacePartCount",
                                "value": 1,
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_SCALAR "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "key": "chunksWithWaterGeometry",
                                "value": 1,
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_WATER_TYPE_BUCKET "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "polygon",
                                "stats": {"surfacePartCount": 1},
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_WATER_KIND_BUCKET "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "pond",
                                "stats": {"surfacePartCount": 1},
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_WATER_KIND_IDS_BATCH "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "pond",
                                "sourceIds": ["water_1"],
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_ROAD_KIND_BUCKET "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "footway",
                                "stats": {"surfacePartCount": 2, "featureCount": 2},
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_ROAD_KIND_IDS_BATCH "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "footway",
                                "sourceIds": ["road_a", "road_b"],
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_ROAD_SUBKIND_BUCKET "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "sidewalk",
                                "stats": {"surfacePartCount": 1, "featureCount": 1},
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_ROAD_SUBKIND_IDS_BATCH "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "sidewalk",
                                "sourceIds": ["road_a"],
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT " + json.dumps(base_payload, separators=(",", ":")),
                    ]
                ),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_EDIT")

            self.assertEqual(report["scene"]["buildingModelCount"], 2)
            self.assertEqual(report["scene"]["waterSurfacePartCount"], 1)
            self.assertEqual(report["scene"]["chunksWithWaterGeometry"], 1)
            self.assertEqual(report["scene"]["waterSurfacePartCountByType"]["polygon"]["surfacePartCount"], 1)
            self.assertEqual(report["scene"]["waterSurfacePartCountByKind"]["pond"]["surfacePartCount"], 1)
            self.assertEqual(report["scene"]["waterSurfacePartCountByKind"]["pond"]["sourceIds"], ["water_1"])
            self.assertEqual(report["scene"]["roadSurfacePartCountByKind"]["footway"]["surfacePartCount"], 2)
            self.assertEqual(report["scene"]["roadSurfacePartCountByKind"]["footway"]["sourceIds"], ["road_a", "road_b"])
            self.assertEqual(report["scene"]["roadSurfacePartCountBySubkind"]["sidewalk"]["surfacePartCount"], 1)
            self.assertEqual(report["scene"]["roadSurfacePartCountBySubkind"]["sidewalk"]["sourceIds"], ["road_a"])

    def test_merged_roof_finding_accepts_shell_mesh_support(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "SceneAuditTown",
                    "metersPerStud": 0.3,
                    "chunkSizeStuds": 256,
                },
                "chunks": [
                    {
                        "id": "0_0",
                        "originStuds": {"x": 0, "y": 0, "z": 0},
                        "roads": [],
                        "buildings": [{"id": "b1"}],
                        "water": [],
                        "props": [],
                        "landuse": [],
                        "barriers": [],
                        "rails": [],
                    }
                ],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            payload = {
                "phase": "edit",
                "focus": {"x": 32.0, "z": 32.0},
                "radius": 256.0,
                "rootName": "GeneratedWorld_AustinPreview",
                "scene": {
                    "chunkCount": 1,
                    "buildingModelCount": 1,
                    "buildingModelsWithRoof": 1,
                    "buildingModelsWithoutRoof": 0,
                    "buildingModelsWithDirectRoof": 0,
                    "buildingModelsWithMergedRoofOnly": 1,
                    "buildingModelsWithNoRoofEvidence": 0,
                    "buildingShellMeshPartCount": 1,
                    "mergedBuildingMeshPartCount": 0,
                    "chunksWithBuildingModels": 1,
                },
            }
            log_path.write_text(
                "ARNIS_SCENE_EDIT " + json.dumps(payload, separators=(",", ":")),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_EDIT")
            codes = {finding["code"] for finding in report["findings"]}

            self.assertIn("merged_roof_only_coverage", codes)
            self.assertNotIn("merged_roof_claim_without_mesh_support", codes)

    def test_shaped_roof_closure_gap_is_reported(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "SceneAuditTown",
                    "metersPerStud": 0.3,
                    "chunkSizeStuds": 256,
                },
                "chunks": [
                    {
                        "id": "0_0",
                        "originStuds": {"x": 0, "y": 0, "z": 0},
                        "roads": [],
                        "buildings": [{"id": "b1"}],
                        "water": [],
                        "props": [],
                        "landuse": [],
                        "barriers": [],
                        "rails": [],
                    }
                ],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            payload = {
                "phase": "edit",
                "focus": {"x": 32.0, "z": 32.0},
                "radius": 256.0,
                "rootName": "GeneratedWorld_AustinPreview",
                "scene": {
                    "chunkCount": 1,
                    "buildingModelCount": 1,
                    "buildingModelsWithRoof": 1,
                    "buildingModelsWithoutRoof": 0,
                    "buildingModelsWithDirectRoof": 1,
                    "buildingModelsWithMergedRoofOnly": 0,
                    "buildingModelsWithNoRoofEvidence": 0,
                    "buildingModelsWithRoofClosureDeck": 0,
                    "chunksWithBuildingModels": 1,
                    "buildingRoofCoverageByShape": {
                        "gabled": {
                            "buildingModelCount": 1,
                            "withRoofCount": 1,
                            "withoutRoofCount": 0,
                            "directRoofCount": 1,
                            "mergedRoofOnlyCount": 0,
                            "noRoofEvidenceCount": 0,
                            "closureDeckCount": 0,
                        }
                    },
                },
            }
            log_path.write_text(
                "ARNIS_SCENE_EDIT " + json.dumps(payload, separators=(",", ":")),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_EDIT")
            codes = {finding["code"] for finding in report["findings"]}

            self.assertIn("shaped_roof_closure_gap", codes)

    def test_roof_shape_gap_traces_direct_geometry_not_merged_only_evidence(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "SceneAuditTown",
                    "metersPerStud": 0.3,
                    "chunkSizeStuds": 256,
                },
                "chunks": [
                    {
                        "id": "0_0",
                        "originStuds": {"x": 0, "y": 0, "z": 0},
                        "roads": [],
                        "buildings": [{"id": "b1", "roof": "gabled"}],
                        "water": [],
                        "props": [],
                        "landuse": [],
                        "barriers": [],
                        "rails": [],
                    }
                ],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            payload = {
                "phase": "edit",
                "focus": {"x": 32.0, "z": 32.0},
                "radius": 256.0,
                "rootName": "GeneratedWorld_AustinPreview",
                "scene": {
                    "chunkCount": 1,
                    "buildingModelCount": 1,
                    "buildingModelsWithRoof": 1,
                    "buildingModelsWithoutRoof": 0,
                    "buildingModelsWithDirectRoof": 0,
                    "buildingModelsWithMergedRoofOnly": 1,
                    "buildingModelsWithNoRoofEvidence": 0,
                    "chunksWithBuildingModels": 1,
                    "buildingRoofCoverageByShape": {
                        "gabled": {
                            "buildingModelCount": 1,
                            "withRoofCount": 1,
                            "withoutRoofCount": 0,
                            "directRoofCount": 0,
                            "mergedRoofOnlyCount": 1,
                            "noRoofEvidenceCount": 0,
                        }
                    },
                },
            }
            log_path.write_text(
                "ARNIS_SCENE_EDIT " + json.dumps(payload, separators=(",", ":")),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_EDIT")
            roof_shape_findings = [finding for finding in report["findings"] if finding["code"] == "roof_shape_scene_gap"]
            codes = {finding["code"] for finding in report["findings"]}

            self.assertIn("merged_roof_only_coverage", codes)
            self.assertIn("roof_shape_scene_gap", codes)
            self.assertNotIn("missing_roof_evidence", codes)
            self.assertEqual(len(roof_shape_findings), 1)
            self.assertIn("direct roof geometries", roof_shape_findings[0]["message"])

    def test_road_gap_uses_unique_source_ids_when_available(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "SceneAuditTown",
                    "metersPerStud": 0.3,
                    "chunkSizeStuds": 256,
                },
                "chunks": [
                    {
                        "id": "0_0",
                        "originStuds": {"x": 0, "y": 0, "z": 0},
                        "roads": [
                            {"id": "road_a", "kind": "footway", "subkind": "none"},
                            {"id": "road_b", "kind": "footway", "subkind": "none"},
                        ],
                        "buildings": [],
                        "water": [],
                        "props": [],
                        "landuse": [],
                        "barriers": [],
                        "rails": [],
                    }
                ],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            payload = {
                "phase": "edit",
                "focus": {"x": 32.0, "z": 32.0},
                "radius": 350.0,
                "rootName": "GeneratedWorld_AustinPreview",
                "scene": {
                    "chunkCount": 1,
                    "buildingModelCount": 0,
                    "chunksWithBuildingModels": 0,
                    "chunksWithRoadGeometry": 1,
                    "roadSurfacePartCountByKind": {
                        "footway": {
                            "surfacePartCount": 20,
                            "featureCount": 1,
                            "sourceIds": ["road_a", "road_b"],
                        }
                    },
                    "roadSurfacePartCountBySubkind": {
                        "none": {
                            "surfacePartCount": 20,
                            "featureCount": 1,
                            "sourceIds": ["road_a", "road_b"],
                        }
                    },
                },
            }
            chunks_payload = {
                "phase": "edit",
                "rootName": "GeneratedWorld_AustinPreview",
                "chunkIds": ["0_0"],
            }
            log_path.write_text(
                "\n".join(
                    [
                        "ARNIS_SCENE_EDIT_CHUNKS " + json.dumps(chunks_payload, separators=(",", ":")),
                        "ARNIS_SCENE_EDIT " + json.dumps(payload, separators=(",", ":")),
                    ]
                ),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_EDIT")

            self.assertEqual(report["summary"]["roadKindGaps"], [])
            self.assertEqual(report["summary"]["roadSubkindGaps"], [])
            codes = {finding["code"] for finding in report["findings"]}
            self.assertNotIn("road_kind_scene_gap", codes)
            self.assertNotIn("road_subkind_scene_gap", codes)

    def test_attached_sidewalk_and_curb_gaps_use_manifest_sidewalk_truth(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"
            html_path = root / "scene-report.html"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "SceneAuditTown",
                    "metersPerStud": 0.3,
                    "chunkSizeStuds": 256,
                },
                "chunks": [
                    {
                        "id": "0_0",
                        "originStuds": {"x": 0, "y": 0, "z": 0},
                        "roads": [
                            {
                                "id": "road_attached_a",
                                "kind": "residential",
                                "subkind": "none",
                                "hasSidewalk": True,
                                "sidewalk": "both",
                            },
                            {
                                "id": "road_attached_b",
                                "kind": "secondary",
                                "subkind": "none",
                                "hasSidewalk": True,
                                "sidewalk": "left",
                            },
                            {
                                "id": "road_separate",
                                "kind": "secondary",
                                "subkind": "none",
                                "hasSidewalk": False,
                                "sidewalk": "separate",
                            },
                        ],
                        "buildings": [],
                        "water": [],
                        "props": [],
                        "landuse": [],
                        "barriers": [],
                        "rails": [],
                    }
                ],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            payload = {
                "phase": "edit",
                "focus": {"x": 32.0, "z": 32.0},
                "radius": 350.0,
                "rootName": "GeneratedWorld_AustinPreview",
                "scene": {
                    "chunkCount": 1,
                    "buildingModelCount": 0,
                    "chunksWithBuildingModels": 0,
                    "chunksWithRoadGeometry": 1,
                    "roadSurfacePartCountBySubkind": {
                        "sidewalk": {
                            "surfacePartCount": 1,
                            "featureCount": 1,
                            "sourceIds": ["road_attached_a"],
                        },
                        "curb": {
                            "surfacePartCount": 0,
                            "featureCount": 0,
                            "sourceIds": [],
                        },
                    },
                },
            }
            chunks_payload = {
                "phase": "edit",
                "rootName": "GeneratedWorld_AustinPreview",
                "chunkIds": ["0_0"],
            }
            log_path.write_text(
                "\n".join(
                    [
                        "ARNIS_SCENE_EDIT_CHUNKS " + json.dumps(chunks_payload, separators=(",", ":")),
                        "ARNIS_SCENE_EDIT " + json.dumps(payload, separators=(",", ":")),
                    ]
                ),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_EDIT")
            codes = {finding["code"] for finding in report["findings"]}

            self.assertIn("attached_sidewalk_scene_gap", codes)
            self.assertIn("curb_scene_gap", codes)
            self.assertEqual(
                report["summary"]["attachedSidewalkGaps"],
                [
                    {
                        "bucket": "attached_sidewalk",
                        "manifestCount": 2,
                        "sceneCount": 1,
                        "missingIds": ["road_attached_b"],
                    }
                ],
            )
            self.assertEqual(
                report["summary"]["curbGaps"],
                [
                    {
                        "bucket": "curb",
                        "manifestCount": 2,
                        "sceneCount": 0,
                        "missingIds": ["road_attached_a", "road_attached_b"],
                    }
                ],
            )
            audit.write_html_report(report, html_path)
            html = html_path.read_text(encoding="utf-8")
            self.assertIn("Attached Sidewalk Gaps", html)
            self.assertIn("Curb Gaps", html)
            self.assertIn("road_attached_b", html)

    def test_rail_kind_gaps_use_manifest_rail_truth_and_render_html_sections(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"
            html_path = root / "scene-report.html"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "SceneAuditTown",
                    "metersPerStud": 0.3,
                    "chunkSizeStuds": 256,
                },
                "chunks": [
                    {
                        "id": "0_0",
                        "originStuds": {"x": 0, "y": 0, "z": 0},
                        "roads": [],
                        "buildings": [],
                        "water": [],
                        "props": [],
                        "landuse": [],
                        "barriers": [],
                        "rails": [
                            {
                                "id": "osm_rail_1",
                                "kind": "rail",
                                "points": [{"x": 0, "y": 0, "z": 0}, {"x": 64, "y": 0, "z": 0}],
                            },
                            {
                                "id": "osm_tram_1",
                                "kind": "tram",
                                "points": [{"x": 0, "y": 0, "z": 8}, {"x": 64, "y": 0, "z": 8}],
                            },
                        ],
                    }
                ],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            chunks_payload = {
                "phase": "edit",
                "rootName": "GeneratedWorld_AustinPreview",
                "chunkIds": ["0_0"],
            }
            scalar_payload = {
                "phase": "edit",
                "rootName": "GeneratedWorld_AustinPreview",
                "key": "chunksWithRailGeometry",
                "value": 1,
            }
            rail_bucket_payload = {
                "phase": "edit",
                "rootName": "GeneratedWorld_AustinPreview",
                "bucket": "rail",
                "stats": {"instanceCount": 1},
            }
            rail_ids_payload = {
                "phase": "edit",
                "rootName": "GeneratedWorld_AustinPreview",
                "bucket": "rail",
                "sourceIds": ["osm_rail_1"],
            }
            scene_payload = {
                "phase": "edit",
                "focus": {"x": 32.0, "z": 32.0},
                "radius": 350.0,
                "rootName": "GeneratedWorld_AustinPreview",
                "scene": {
                    "chunkCount": 1,
                },
            }
            log_path.write_text(
                "\n".join(
                    [
                        "ARNIS_SCENE_EDIT_CHUNKS " + json.dumps(chunks_payload, separators=(",", ":")),
                        "ARNIS_SCENE_EDIT_SCALAR " + json.dumps(scalar_payload, separators=(",", ":")),
                        "ARNIS_SCENE_EDIT_RAIL_KIND_BUCKET " + json.dumps(rail_bucket_payload, separators=(",", ":")),
                        "ARNIS_SCENE_EDIT_RAIL_KIND_IDS_BATCH " + json.dumps(rail_ids_payload, separators=(",", ":")),
                        "ARNIS_SCENE_EDIT " + json.dumps(scene_payload, separators=(",", ":")),
                    ]
                ),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_EDIT")
            codes = {finding["code"] for finding in report["findings"]}

            self.assertIn("rail_kind_scene_gap", codes)
            self.assertEqual(
                report["summary"]["railKindGaps"],
                [
                    {
                        "bucket": "tram",
                        "manifestCount": 1,
                        "sceneCount": 0,
                        "missingIds": ["osm_tram_1"],
                    }
                ],
            )
            audit.write_html_report(report, html_path)
            html = html_path.read_text(encoding="utf-8")
            self.assertIn("Manifest Rail Kinds", html)
            self.assertIn("Rail Receipts By Kind", html)
            self.assertIn("Rail Kind Gaps", html)
            self.assertIn("osm_tram_1", html)

    def test_prop_and_vegetation_gaps_use_unique_source_ids_when_available(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "SceneAuditTown",
                    "metersPerStud": 0.3,
                    "chunkSizeStuds": 256,
                },
                "chunks": [
                    {
                        "id": "0_0",
                        "originStuds": {"x": 0, "y": 0, "z": 0},
                        "roads": [],
                        "buildings": [],
                        "water": [],
                        "props": [
                            {"id": "tree_a", "kind": "tree", "species": "oak", "position": {"x": 0, "y": 0, "z": 0}},
                            {"id": "tree_b", "kind": "tree", "species": "oak", "position": {"x": 4, "y": 0, "z": 4}},
                        ],
                        "landuse": [],
                        "barriers": [],
                        "rails": [],
                    }
                ],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            payload = {
                "phase": "edit",
                "focus": {"x": 32.0, "z": 32.0},
                "radius": 350.0,
                "rootName": "GeneratedWorld_AustinPreview",
                "scene": {
                    "chunkCount": 1,
                    "propInstanceCountByKind": {
                        "tree": {"instanceCount": 1, "sourceIds": ["tree_a", "tree_b"]},
                    },
                    "treeInstanceCountBySpecies": {
                        "oak": {"instanceCount": 1, "sourceIds": ["tree_a", "tree_b"]},
                    },
                    "vegetationInstanceCountByKind": {
                        "tree": {"instanceCount": 1, "sourceIds": ["tree_a", "tree_b"]},
                    },
                },
            }
            chunks_payload = {
                "phase": "edit",
                "rootName": "GeneratedWorld_AustinPreview",
                "chunkIds": ["0_0"],
            }
            log_path.write_text(
                "\n".join(
                    [
                        "ARNIS_SCENE_EDIT_CHUNKS " + json.dumps(chunks_payload, separators=(",", ":")),
                        "ARNIS_SCENE_EDIT " + json.dumps(payload, separators=(",", ":")),
                    ]
                ),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_EDIT")

            self.assertEqual(report["summary"]["propKindGaps"], [])
            self.assertEqual(report["summary"]["treeSpeciesGaps"], [])
            self.assertEqual(report["summary"]["vegetationKindGaps"], [])
            codes = {finding["code"] for finding in report["findings"]}
            self.assertNotIn("prop_kind_scene_gap", codes)
            self.assertNotIn("tree_species_scene_gap", codes)
            self.assertNotIn("vegetation_kind_scene_gap", codes)

    def test_tree_connectivity_fields_and_finding(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "SceneAuditTown",
                    "metersPerStud": 0.3,
                    "chunkSizeStuds": 256,
                },
                "chunks": [
                    {
                        "id": "0_0",
                        "originStuds": {"x": 0, "y": 0, "z": 0},
                        "roads": [],
                        "buildings": [],
                        "water": [],
                        "props": [
                            {"id": "tree_1", "kind": "tree", "species": "oak", "position": {"x": 0, "y": 0, "z": 0}},
                            {"id": "tree_2", "kind": "tree", "species": "elm", "position": {"x": 12, "y": 0, "z": 0}},
                            {"id": "tree_3", "kind": "tree", "species": "cedar", "position": {"x": 24, "y": 0, "z": 0}},
                        ],
                        "landuse": [],
                        "barriers": [],
                        "rails": [],
                    }
                ],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            payload = {
                "phase": "edit",
                "focus": {"x": 32.0, "z": 32.0},
                "radius": 256.0,
                "rootName": "GeneratedWorld_AustinPreview",
                "scene": {
                    "chunkCount": 1,
                    "propInstanceCount": 3,
                    "propInstanceCountByKind": {"tree": {"instanceCount": 3}},
                    "treeInstanceCount": 3,
                    "treeInstanceCountBySpecies": {
                        "oak": {"instanceCount": 1},
                        "elm": {"instanceCount": 1},
                        "cedar": {"instanceCount": 1},
                    },
                    "vegetationInstanceCount": 3,
                    "vegetationInstanceCountByKind": {"tree": {"instanceCount": 3}},
                    "treeModelsWithConnectedTrunkCanopy": 1,
                    "treeModelsMissingTrunk": 1,
                    "treeModelsMissingCanopy": 0,
                    "treeModelsWithDetachedCanopy": 1,
                    "treeConnectivityBySpecies": {
                        "oak": {
                            "treeInstanceCount": 1,
                            "connectedCount": 0,
                            "missingTrunkCount": 1,
                            "missingCanopyCount": 0,
                            "detachedCanopyCount": 0,
                        },
                        "elm": {
                            "treeInstanceCount": 1,
                            "connectedCount": 1,
                            "missingTrunkCount": 0,
                            "missingCanopyCount": 0,
                            "detachedCanopyCount": 0,
                        },
                        "cedar": {
                            "treeInstanceCount": 1,
                            "connectedCount": 0,
                            "missingTrunkCount": 0,
                            "missingCanopyCount": 0,
                            "detachedCanopyCount": 1,
                        },
                    },
                },
            }
            log_path.write_text("ARNIS_SCENE_EDIT " + json.dumps(payload, separators=(",", ":")), encoding="utf-8")

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_EDIT")
            codes = {finding["code"] for finding in report["findings"]}

            self.assertEqual(report["scene"]["treeModelsWithConnectedTrunkCanopy"], 1)
            self.assertEqual(report["scene"]["treeModelsMissingTrunk"], 1)
            self.assertEqual(report["scene"]["treeModelsWithDetachedCanopy"], 1)
            self.assertEqual(report["scene"]["treeConnectivityBySpecies"]["oak"]["missingTrunkCount"], 1)
            self.assertEqual(report["scene"]["treeConnectivityBySpecies"]["elm"]["connectedCount"], 1)
            self.assertEqual(report["scene"]["treeConnectivityBySpecies"]["cedar"]["detachedCanopyCount"], 1)
            self.assertIn("tree_connectivity_gaps", codes)

    def test_report_reassembles_split_building_material_buckets(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {
                    "worldName": "SceneAuditTown",
                    "metersPerStud": 0.3,
                    "chunkSizeStuds": 256,
                },
                "chunks": [
                    {
                        "id": "0_0",
                        "originStuds": {"x": 0, "y": 0, "z": 0},
                        "roads": [],
                        "buildings": [
                            {
                                "id": "bldg_1",
                                "usage": "office",
                                "roof": "flat",
                                "material": "Concrete",
                                "roofMaterial": "Slate",
                            }
                        ],
                        "water": [],
                        "props": [],
                        "landuse": [],
                        "barriers": [],
                        "rails": [],
                    }
                ],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            payload = {
                "phase": "edit",
                "focus": {"x": 32.0, "z": 32.0},
                "radius": 256.0,
                "rootName": "GeneratedWorld_AustinPreview",
                "scene": {
                    "chunkCount": 1,
                    "buildingModelCount": 1,
                    "chunksWithBuildingModels": 1,
                    "buildingModelsWithRoof": 1,
                    "buildingRoofCoverageByUsage": {
                        "office": {
                            "buildingModelCount": 1,
                            "withRoofCount": 1,
                            "withoutRoofCount": 0,
                            "directRoofCount": 1,
                            "mergedRoofOnlyCount": 0,
                            "noRoofEvidenceCount": 0,
                        }
                    },
                    "buildingRoofCoverageByShape": {
                        "flat": {
                            "buildingModelCount": 1,
                            "withRoofCount": 1,
                            "withoutRoofCount": 0,
                            "directRoofCount": 1,
                            "mergedRoofOnlyCount": 0,
                            "noRoofEvidenceCount": 0,
                        }
                    },
                },
            }
            log_path.write_text(
                "\n".join(
                    [
                        "ARNIS_SCENE_EDIT_BUILDING_WALL_MATERIAL_BUCKET "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "concrete",
                                "stats": {"buildingModelCount": 1},
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_BUILDING_WALL_MATERIAL_IDS_BATCH "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "concrete",
                                "sourceIds": ["bldg_1"],
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_BUILDING_ROOF_MATERIAL_BUCKET "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "slate",
                                "stats": {"buildingModelCount": 1},
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT_BUILDING_ROOF_MATERIAL_IDS_BATCH "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_AustinPreview",
                                "bucket": "slate",
                                "sourceIds": ["bldg_1"],
                            },
                            separators=(",", ":"),
                        ),
                        "ARNIS_SCENE_EDIT " + json.dumps(payload, separators=(",", ":")),
                    ]
                ),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_EDIT")
            codes = {finding["code"] for finding in report["findings"]}

            self.assertEqual(report["scene"]["buildingModelCountByWallMaterial"]["concrete"]["buildingModelCount"], 1)
            self.assertEqual(report["scene"]["buildingModelCountByWallMaterial"]["concrete"]["sourceIds"], ["bldg_1"])
            self.assertEqual(report["scene"]["buildingModelCountByRoofMaterial"]["slate"]["buildingModelCount"], 1)
            self.assertEqual(report["scene"]["buildingModelCountByRoofMaterial"]["slate"]["sourceIds"], ["bldg_1"])
            self.assertEqual(report["summary"]["explicitWallMaterialGaps"], [])
            self.assertEqual(report["summary"]["explicitRoofMaterialGaps"], [])
            self.assertNotIn("explicit_wall_material_scene_gap", codes)
            self.assertNotIn("explicit_roof_material_scene_gap", codes)

    def test_report_ignores_trailing_fragments_from_other_run_key(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {"worldName": "SceneAuditTown", "metersPerStud": 0.3, "chunkSizeStuds": 256},
                "chunks": [{"id": "0_0", "originStuds": {"x": 0, "y": 0, "z": 0}, "roads": [], "buildings": [], "water": [], "props": [], "landuse": [], "barriers": [], "rails": []}],
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            payload = {
                "phase": "edit",
                "focus": {"x": 32.0, "z": 32.0},
                "radius": 256.0,
                "rootName": "GeneratedWorld_AustinPreview",
                "scene": {"chunkCount": 1, "buildingModelCount": 0},
            }
            log_path.write_text(
                "\n".join(
                    [
                        "ARNIS_SCENE_EDIT " + json.dumps(payload, separators=(",", ":")),
                        "ARNIS_SCENE_EDIT_BUILDING_WALL_MATERIAL_BUCKET "
                        + json.dumps(
                            {
                                "phase": "edit",
                                "rootName": "GeneratedWorld_Stale",
                                "bucket": "glass",
                                "stats": {"buildingModelCount": 99},
                            },
                            separators=(",", ":"),
                        ),
                    ]
                ),
                encoding="utf-8",
            )

            report = audit.build_report(manifest_path, log_path, marker="ARNIS_SCENE_EDIT")
            self.assertNotIn("buildingModelCountByWallMaterial", report["scene"])

    def test_truth_pack_carries_through_compact_summary_into_json_and_html(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"
            html_path = root / "report.html"
            truth_pack_path = root / "fixture.truth-pack.sqlite"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {"worldName": "SceneAuditTown", "metersPerStud": 1.0, "chunkSizeStuds": 256},
                "chunks": [
                    {
                        "id": "0_0",
                        "originStuds": {"x": 0, "y": 0, "z": 0},
                        "roads": [{"id": "road_1", "kind": "secondary", "subkind": "none"}],
                        "buildings": [{"id": "structure_1", "usage": "school", "roof": "flat"}],
                        "water": [],
                        "props": [],
                        "landuse": [],
                        "barriers": [],
                        "rails": [],
                    }
                ],
            }
            payload = {
                "phase": "play",
                "focus": {"x": 64.0, "z": 64.0},
                "radius": 256.0,
                "rootName": "GeneratedWorld_Austin",
                "worldIdentity": "AustinManifestIndex",
                "chunkEnvelopeKind": "runtime_resident",
                "scene": {
                    "chunkCount": 1,
                    "chunkIds": ["0_0"],
                    "buildingModelCount": 1,
                    "chunksWithBuildingModels": 1,
                    "roadTaggedPartCount": 1,
                    "chunksWithRoadGeometry": 1,
                },
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            log_path.write_text("ARNIS_SCENE_PLAY " + json.dumps(payload, separators=(",", ":")), encoding="utf-8")
            build_truth_pack_fixture(truth_pack_path)

            report = audit.build_report(
                manifest_path,
                log_path,
                marker="ARNIS_SCENE_PLAY",
                truth_pack=truth_pack_path,
            )
            audit.write_html_report(report, html_path)
            html = html_path.read_text(encoding="utf-8")

            self.assertIn("truthPack", report["summary"])
            self.assertEqual(report["summary"]["truthPack"]["scene"], "fixture")
            self.assertEqual(report["summary"]["truthPack"]["collapseCount"], 2)
            self.assertEqual(report["summary"]["truthPack"]["droppedSemanticCount"], 2)
            self.assertEqual(report["summary"]["truthPack"]["outdoorSourceCoverage"]["structures"]["retained_feature_count"], 1)
            self.assertEqual(report["summary"]["truthPack"]["droppedSemanticsBreakdown"]["structures"]["usage"], 1)
            self.assertEqual(
                report["summary"]["truthPack"]["collapseBreakdown"]["structures"]["overture->osm|cross_source_overlap"],
                2,
            )
            self.assertEqual(len(report["summary"]["truthPack"]["samples"]["overlap_losses"]), 2)
            self.assertEqual(len(report["summary"]["truthPack"]["samples"]["dropped_semantics"]), 2)
            self.assertNotIn("truth_pack_path", report["summary"]["truthPack"])
            self.assertIn("Truth Pack", html)
            self.assertIn("truth_pack_outdoor_overlap_loss", html)
            self.assertIn("structure_overlap_1", html)
            self.assertNotIn(str(truth_pack_path), html)

    def test_preview_plugin_state_carries_compact_hotspot_summary_into_json_and_html(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_path = root / "manifest.json"
            log_path = root / "studio.log"
            plugin_state_path = root / "plugin-state.json"
            html_path = root / "report.html"

            manifest = {
                "schemaVersion": "0.4.0",
                "meta": {"worldName": "SceneAuditTown", "metersPerStud": 1.0, "chunkSizeStuds": 256},
                "chunks": [
                    {
                        "id": "0_0",
                        "originStuds": {"x": 0, "y": 0, "z": 0},
                        "roads": [],
                        "buildings": [],
                        "water": [],
                        "props": [],
                        "landuse": [],
                        "barriers": [],
                        "rails": [],
                    }
                ],
            }
            payload = {
                "phase": "edit",
                "focus": {"x": 64.0, "z": 64.0},
                "radius": 256.0,
                "rootName": "GeneratedWorld_AustinPreview",
                "worldIdentity": "AustinPreviewBuilder",
                "chunkEnvelopeKind": "bounded_preview",
                "scene": {
                    "chunkCount": 1,
                    "chunkIds": ["0_0"],
                },
            }
            plugin_state = {
                "preview_runtime": {
                    "studio_connected": True,
                    "plugin_attached": True,
                    "project_loaded": True,
                    "sync_status": "connected",
                    "connection": {"ws_connected": True},
                },
                "preview_project": {
                    "preview": {
                        "build_active": False,
                        "state_apply_pending": False,
                        "sync_state": "idle",
                    },
                    "full_bake": {"active": False, "last_result": "success"},
                },
                "preview_project_snapshot": {
                    "counters": {
                        "build_scheduled": 1,
                        "sync_complete": 2,
                        "sync_cancelled": 0,
                        "state_apply_succeeded": 1,
                        "state_apply_failed": 0,
                    },
                    "chunkTotals": {"imported": 80, "skipped": 0, "unloaded": 0},
                    "lastSync": {"elapsedMs": 17384},
                    "lastSlowChunk": {
                        "chunkId": "7_5",
                        "phase": "preview",
                        "totalMs": 166,
                        "buildingsMs": 121,
                        "terrainMs": 18,
                        "terrainMaterialKindCount": 3,
                        "terrainDominantMaterial": "Grass",
                        "terrainDominantMaterialCellCount": 64,
                        "terrainNonGrassCellCount": 12,
                        "roadsMs": 9,
                        "landuseTerrainFillMs": 6,
                        "artifactCount": 2,
                    },
                },
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            log_path.write_text("ARNIS_SCENE_EDIT " + json.dumps(payload, separators=(",", ":")), encoding="utf-8")
            plugin_state_path.write_text(json.dumps(plugin_state), encoding="utf-8")

            report = audit.build_report(
                manifest_path,
                log_path,
                marker="ARNIS_SCENE_EDIT",
                preview_plugin_state=plugin_state_path,
            )
            audit.write_html_report(report, html_path)
            html = html_path.read_text(encoding="utf-8")

            self.assertIn("previewTelemetry", report["summary"])
            self.assertEqual(report["summary"]["previewTelemetry"]["hotspot"]["status"], "present")
            self.assertEqual(report["summary"]["previewTelemetry"]["hotspot"]["slowChunk"]["chunkId"], "7_5")
            self.assertEqual(report["summary"]["previewTelemetry"]["hotspot"]["slowChunk"]["terrainMaterialKindCount"], 3)
            self.assertNotIn(str(plugin_state_path), json.dumps(report, sort_keys=True))
            self.assertIn("Preview Hotspot", html)
            self.assertIn("7_5", html)
            self.assertNotIn(str(plugin_state_path), html)


if __name__ == "__main__":
    unittest.main()
