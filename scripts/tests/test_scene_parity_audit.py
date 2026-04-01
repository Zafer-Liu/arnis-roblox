from __future__ import annotations

import importlib.util
import json
import tempfile
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "scene_parity_audit.py"


def load_module():
    spec = importlib.util.spec_from_file_location("scene_parity_audit", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load module spec from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class SceneParityAuditTests(unittest.TestCase):
    maxDiff = None

    def test_build_report_compares_client_world_observability_when_present(self) -> None:
        audit = load_module()
        edit_report = {
            "rootName": "GeneratedWorld_AustinPreview",
            "worldIdentity": "AustinManifestIndex",
            "chunkEnvelopeKind": "bounded_preview",
            "focus": {"x": 0, "z": 0},
            "radius": 256,
            "summary": {"marker": "ARNIS_SCENE_EDIT"},
            "scene": {
                "chunkIds": ["0_0"],
                "buildingModelCount": 1,
            },
            "clientWorld": {
                "worldRootName": "GeneratedWorld_AustinPreview",
                "worldRootExists": True,
                "nearbyBuildingModels": 7,
                "nearbyMergedBuildingMeshParts": 0,
                "nearbyRoofParts": 14,
                "overheadRoofParts": 4,
                "groundMaterial": "Enum.Material.Concrete",
                "bootstrapState": "gameplay_ready",
                "supportSurfaceRole": "road",
                "supportY": 12.5,
                "terrainY": 11.0,
                "supportMinusTerrainYStuds": 1.5,
                "nearbyWallParts": 10,
                "collidableWallPartsNearby": 10,
                "nearestWallDistanceStuds": 4.0,
                "overheadRoofMinClearanceStuds": 12.0,
            },
        }
        play_report = {
            "rootName": "GeneratedWorld_Austin",
            "worldIdentity": "AustinManifestIndex",
            "chunkEnvelopeKind": "runtime_resident",
            "focus": {"x": 0, "z": 0},
            "radius": 256,
            "summary": {"marker": "ARNIS_SCENE_PLAY"},
            "scene": {
                "chunkIds": ["0_0", "1_0"],
                "buildingModelCount": 1,
            },
            "clientWorld": {
                "worldRootName": "GeneratedWorld_Austin",
                "worldRootExists": True,
                "nearbyBuildingModels": 7,
                "nearbyMergedBuildingMeshParts": 0,
                "nearbyRoofParts": 14,
                "overheadRoofParts": 2,
                "groundMaterial": "Enum.Material.Asphalt",
                "bootstrapState": "gameplay_ready",
                "supportSurfaceRole": "terrain",
                "supportY": 10.0,
                "terrainY": 10.0,
                "supportMinusTerrainYStuds": 0.0,
                "nearbyWallParts": 2,
                "collidableWallPartsNearby": 2,
                "nearestWallDistanceStuds": 18.0,
                "overheadRoofMinClearanceStuds": 24.0,
            },
        }

        report = audit.build_report(edit_report, play_report)
        codes = {finding["code"] for finding in report["findings"]}

        self.assertIn("client_world_mismatch", codes)
        self.assertEqual(report["comparisons"]["clientWorld"]["edit"]["groundMaterial"], "Enum.Material.Concrete")
        self.assertEqual(report["comparisons"]["clientWorld"]["play"]["groundMaterial"], "Enum.Material.Asphalt")
        self.assertEqual(report["comparisons"]["clientWorld"]["edit"]["nearbyRoofParts"], 14)
        self.assertEqual(report["comparisons"]["clientWorld"]["play"]["overheadRoofParts"], 2)
        self.assertEqual(report["comparisons"]["clientWorld"]["edit"]["supportSurfaceRole"], "road")
        self.assertEqual(report["comparisons"]["clientWorld"]["play"]["nearestWallDistanceStuds"], 18.0)

    def test_build_report_accepts_contract_aligned_preview_subset_and_world_identity(self) -> None:
        audit = load_module()
        edit_report = {
            "rootName": "GeneratedWorld_AustinPreview",
            "worldIdentity": "AustinManifestIndex",
            "chunkEnvelopeKind": "bounded_preview",
            "focus": {"x": 0, "z": 0},
            "radius": 256,
            "summary": {"marker": "ARNIS_SCENE_EDIT"},
            "scene": {
                "chunkIds": ["0_0"],
                "buildingModelCount": 1,
                "buildingModelsWithDirectShell": 1,
                "buildingModelsWithRoofClosureDeck": 1,
                "roadSurfacePartCount": 2,
                "waterSurfacePartCount": 0,
                "propInstanceCount": 3,
                "roadSurfacePartCountBySubkind": {"sidewalk": {"surfacePartCount": 1, "featureCount": 1}},
                "waterSurfacePartCountByType": {},
                "waterSurfacePartCountByKind": {},
                "railReceiptCountByKind": {},
                "vegetationInstanceCountByKind": {"tree": {"instanceCount": 1, "featureCount": 1}},
                "treeInstanceCountBySpecies": {"oak": {"instanceCount": 1, "featureCount": 1}},
                "buildingRoofCoverageByUsage": {"office": {"buildingModelCount": 1, "withRoofCount": 1}},
                "buildingRoofCoverageByShape": {"flat": {"buildingModelCount": 1, "withRoofCount": 1}},
                "buildingModelCountByWallMaterial": {"concrete": {"buildingModelCount": 1, "sourceIds": ["a"]}},
                "buildingModelCountByRoofMaterial": {"slate": {"buildingModelCount": 1, "sourceIds": ["a"]}},
                "roadSurfacePartCountByKind": {"secondary": {"surfacePartCount": 2, "featureCount": 1}},
            },
        }
        play_report = {
            "rootName": "GeneratedWorld_Austin",
            "worldIdentity": "AustinManifestIndex",
            "chunkEnvelopeKind": "runtime_resident",
            "focus": {"x": 0, "z": 0},
            "radius": 256,
            "summary": {"marker": "ARNIS_SCENE_PLAY"},
            "scene": {
                "chunkIds": ["0_0", "1_0"],
                "buildingModelCount": 1,
                "buildingModelsWithDirectShell": 1,
                "buildingModelsWithRoofClosureDeck": 1,
                "roadSurfacePartCount": 2,
                "waterSurfacePartCount": 0,
                "propInstanceCount": 3,
                "roadSurfacePartCountBySubkind": {"sidewalk": {"surfacePartCount": 1, "featureCount": 1}},
                "waterSurfacePartCountByType": {},
                "waterSurfacePartCountByKind": {},
                "railReceiptCountByKind": {},
                "vegetationInstanceCountByKind": {"tree": {"instanceCount": 1, "featureCount": 1}},
                "treeInstanceCountBySpecies": {"oak": {"instanceCount": 1, "featureCount": 1}},
                "buildingRoofCoverageByUsage": {"office": {"buildingModelCount": 1, "withRoofCount": 1}},
                "buildingRoofCoverageByShape": {"flat": {"buildingModelCount": 1, "withRoofCount": 1}},
                "buildingModelCountByWallMaterial": {"concrete": {"buildingModelCount": 1, "sourceIds": ["a"]}},
                "buildingModelCountByRoofMaterial": {"slate": {"buildingModelCount": 1, "sourceIds": ["a"]}},
                "roadSurfacePartCountByKind": {"secondary": {"surfacePartCount": 2, "featureCount": 1}},
            },
        }

        report = audit.build_report(edit_report, play_report)
        codes = {finding["code"] for finding in report["findings"]}

        self.assertNotIn("world_identity_mismatch", codes)
        self.assertNotIn("chunk_ids_mismatch", codes)
        self.assertEqual(report["summary"]["matching"], 21)
        self.assertEqual(report["summary"]["mismatched"], 0)
        self.assertEqual(report["comparisons"]["worldIdentity"]["edit"], "AustinManifestIndex")
        self.assertEqual(report["comparisons"]["worldIdentity"]["play"], "AustinManifestIndex")

    def test_build_report_quantizes_local_terrain_metrics_in_client_world_comparison(self) -> None:
        audit = load_module()
        edit_report = {
            "rootName": "GeneratedWorld_Austin",
            "worldIdentity": "AustinManifestIndex",
            "chunkEnvelopeKind": "runtime_resident",
            "focus": {"x": 0, "z": 0},
            "radius": 256,
            "summary": {"marker": "ARNIS_SCENE_EDIT"},
            "scene": {
                "chunkIds": ["0_0"],
                "buildingModelCount": 1,
            },
            "clientWorld": {
                "worldRootName": "GeneratedWorld_Austin",
                "worldRootExists": True,
                "supportSurfaceRole": "terrain",
                "localSupport": {"surfaceRole": "terrain"},
                "localTerrain": {
                    "status": "ok",
                    "samplePattern": "cross_5",
                    "sampleRadiusStuds": 12.04,
                    "sampleCount": 5,
                    "missingSampleCount": 0,
                    "centerTerrainY": 10.04,
                    "minTerrainY": 8.04,
                    "maxTerrainY": 14.04,
                    "heightRangeStuds": 6.04,
                    "maxStepStuds": 4.04,
                    "meanAbsStepStuds": 2.04,
                },
            },
        }
        play_report = {
            "rootName": "GeneratedWorld_Austin",
            "worldIdentity": "AustinManifestIndex",
            "chunkEnvelopeKind": "runtime_resident",
            "focus": {"x": 0, "z": 0},
            "radius": 256,
            "summary": {"marker": "ARNIS_SCENE_PLAY"},
            "scene": {
                "chunkIds": ["0_0"],
                "buildingModelCount": 1,
            },
            "clientWorld": {
                "worldRootName": "GeneratedWorld_Austin",
                "worldRootExists": True,
                "supportSurfaceRole": "terrain",
                "localSupport": {"surfaceRole": "terrain"},
                "localTerrain": {
                    "status": "ok",
                    "samplePattern": "cross_5",
                    "sampleRadiusStuds": 12.01,
                    "sampleCount": 5,
                    "missingSampleCount": 0,
                    "centerTerrainY": 10.01,
                    "minTerrainY": 8.01,
                    "maxTerrainY": 14.01,
                    "heightRangeStuds": 6.01,
                    "maxStepStuds": 4.01,
                    "meanAbsStepStuds": 2.01,
                },
            },
        }

        report = audit.build_report(edit_report, play_report)
        codes = {finding["code"] for finding in report["findings"]}

        self.assertNotIn("client_world_mismatch", codes)
        self.assertEqual(report["comparisons"]["clientWorld"]["edit"]["localTerrain"]["maxStepStuds"], 4.0)
        self.assertEqual(report["comparisons"]["clientWorld"]["play"]["localTerrain"]["heightRangeStuds"], 6.0)

    def test_build_report_flags_scalar_and_bucket_parity_gaps(self) -> None:
        audit = load_module()
        edit_report = {
            "rootName": "GeneratedWorld_Austin",
            "worldIdentity": "AustinManifestIndex",
            "chunkEnvelopeKind": "bounded_preview",
            "focus": {"x": 0, "z": 0},
            "radius": 256,
            "summary": {"marker": "ARNIS_SCENE_EDIT"},
            "scene": {
                "chunkIds": ["0_0", "1_0"],
                "buildingModelCount": 7,
                "buildingModelsWithDirectShell": 7,
                "buildingModelsWithRoofClosureDeck": 5,
                "roadSurfacePartCount": 15,
                "waterSurfacePartCount": 3,
                "propInstanceCount": 9,
                "roadSurfacePartCountBySubkind": {
                    "sidewalk": {"surfacePartCount": 4, "featureCount": 2},
                },
                "waterSurfacePartCountByType": {
                    "polygon": {"surfacePartCount": 3, "featureCount": 2},
                },
                "waterSurfacePartCountByKind": {
                    "pond": {"surfacePartCount": 2, "featureCount": 1},
                },
                "railReceiptCountByKind": {
                    "rail": {"receiptCount": 1, "featureCount": 1},
                },
                "vegetationInstanceCountByKind": {
                    "tree": {"instanceCount": 5, "featureCount": 5},
                },
                "treeInstanceCountBySpecies": {
                    "oak": {"instanceCount": 5, "featureCount": 5},
                },
                "buildingRoofCoverageByUsage": {
                    "office": {"buildingModelCount": 4, "withRoofCount": 4, "withoutRoofCount": 0},
                },
                "buildingRoofCoverageByShape": {
                    "flat": {"buildingModelCount": 4, "withRoofCount": 4, "withoutRoofCount": 0},
                    "gabled": {"buildingModelCount": 3, "withRoofCount": 2, "withoutRoofCount": 1},
                },
                "buildingModelCountByWallMaterial": {
                    "concrete": {"buildingModelCount": 4, "sourceIds": ["a", "b"]},
                },
                "buildingModelCountByRoofMaterial": {
                    "slate": {"buildingModelCount": 2, "sourceIds": ["a"]},
                },
                "roadSurfacePartCountByKind": {
                    "secondary": {"surfacePartCount": 9, "featureCount": 5},
                    "residential": {"surfacePartCount": 6, "featureCount": 4},
                },
            },
        }
        play_report = {
            "rootName": "GeneratedWorld_Austin_Play",
            "worldIdentity": "AustinRuntimeManifest",
            "chunkEnvelopeKind": "runtime_resident",
            "focus": {"x": 64, "z": 0},
            "radius": 512,
            "summary": {"marker": "ARNIS_SCENE_PLAY"},
            "scene": {
                "chunkIds": ["0_0", "2_0"],
                "buildingModelCount": 6,
                "buildingModelsWithDirectShell": 6,
                "buildingModelsWithRoofClosureDeck": 3,
                "roadSurfacePartCount": 15,
                "waterSurfacePartCount": 1,
                "propInstanceCount": 8,
                "roadSurfacePartCountBySubkind": {
                    "sidewalk": {"surfacePartCount": 2, "featureCount": 1},
                },
                "waterSurfacePartCountByType": {
                    "ribbon": {"surfacePartCount": 1, "featureCount": 1},
                },
                "waterSurfacePartCountByKind": {
                    "stream": {"surfacePartCount": 1, "featureCount": 1},
                },
                "railReceiptCountByKind": {
                    "rail": {"receiptCount": 0, "featureCount": 1},
                },
                "vegetationInstanceCountByKind": {
                    "tree": {"instanceCount": 3, "featureCount": 3},
                },
                "treeInstanceCountBySpecies": {
                    "oak": {"instanceCount": 3, "featureCount": 3},
                },
                "buildingRoofCoverageByUsage": {
                    "office": {"buildingModelCount": 3, "withRoofCount": 2, "withoutRoofCount": 1},
                },
                "buildingRoofCoverageByShape": {
                    "flat": {"buildingModelCount": 4, "withRoofCount": 3, "withoutRoofCount": 1},
                    "gabled": {"buildingModelCount": 2, "withRoofCount": 2, "withoutRoofCount": 0},
                },
                "buildingModelCountByWallMaterial": {
                    "concrete": {"buildingModelCount": 3, "sourceIds": ["a"]},
                },
                "buildingModelCountByRoofMaterial": {
                    "slate": {"buildingModelCount": 1, "sourceIds": ["a"]},
                },
                "roadSurfacePartCountByKind": {
                    "secondary": {"surfacePartCount": 7, "featureCount": 4},
                    "residential": {"surfacePartCount": 6, "featureCount": 4},
                },
            },
        }

        report = audit.build_report(edit_report, play_report)
        codes = {finding["code"] for finding in report["findings"]}

        self.assertIn("chunk_ids_mismatch", codes)
        self.assertIn("building_model_count_mismatch", codes)
        self.assertIn("water_surface_part_count_mismatch", codes)
        self.assertIn("prop_instance_count_mismatch", codes)
        self.assertIn("world_identity_mismatch", codes)
        self.assertIn("focus_mismatch", codes)
        self.assertIn("radius_mismatch", codes)
        self.assertIn("building_direct_shell_count_mismatch", codes)
        self.assertIn("building_roof_closure_deck_count_mismatch", codes)
        self.assertIn("road_subkind_surface_mismatch", codes)
        self.assertIn("water_type_surface_mismatch", codes)
        self.assertIn("water_kind_surface_mismatch", codes)
        self.assertIn("rail_kind_receipt_mismatch", codes)
        self.assertIn("vegetation_kind_instance_mismatch", codes)
        self.assertIn("tree_species_instance_mismatch", codes)
        self.assertIn("roof_usage_coverage_mismatch", codes)
        self.assertIn("roof_shape_coverage_mismatch", codes)
        self.assertIn("wall_material_count_mismatch", codes)
        self.assertIn("roof_material_count_mismatch", codes)
        self.assertIn("road_kind_surface_mismatch", codes)
        self.assertEqual(report["summary"]["matching"], 1)
        self.assertEqual(report["summary"]["mismatched"], 20)
        self.assertEqual(report["summary"]["totalChecks"], 21)
        self.assertEqual(report["comparisons"]["chunkIds"]["edit"], ["0_0", "1_0"])
        self.assertEqual(report["comparisons"]["chunkIds"]["play"], ["0_0", "2_0"])
        self.assertEqual(report["comparisons"]["worldIdentity"]["edit"], "AustinManifestIndex")
        self.assertEqual(report["comparisons"]["worldIdentity"]["play"], "AustinRuntimeManifest")

    def test_build_report_accepts_source_id_subset_buckets_for_bounded_preview(self) -> None:
        audit = load_module()
        edit_report = {
            "rootName": "GeneratedWorld_AustinPreview",
            "worldIdentity": "AustinManifestIndex",
            "chunkEnvelopeKind": "bounded_preview",
            "focus": {"x": 0, "z": 0},
            "radius": 256,
            "summary": {"marker": "ARNIS_SCENE_EDIT"},
            "scene": {
                "chunkIds": ["0_0"],
                "buildingModelCount": 1,
                "buildingModelsWithDirectShell": 1,
                "buildingModelsWithRoofClosureDeck": 1,
                "roadSurfacePartCount": 2,
                "waterSurfacePartCount": 1,
                "propInstanceCount": 1,
                "roadSurfacePartCountBySubkind": {
                    "sidewalk": {"surfacePartCount": 2, "featureCount": 1, "sourceIds": ["road_a"]}
                },
                "waterSurfacePartCountByType": {
                    "polygon": {"surfacePartCount": 1, "featureCount": 1, "sourceIds": ["water_a"]}
                },
                "waterSurfacePartCountByKind": {
                    "pond": {"surfacePartCount": 1, "featureCount": 1, "sourceIds": ["water_a"]}
                },
                "railReceiptCountByKind": {
                    "rail": {"receiptCount": 1, "featureCount": 1, "sourceIds": ["rail_a"]}
                },
                "vegetationInstanceCountByKind": {
                    "tree": {"instanceCount": 1, "featureCount": 1, "sourceIds": ["tree_a"]}
                },
                "treeInstanceCountBySpecies": {
                    "oak": {"instanceCount": 1, "featureCount": 1, "sourceIds": ["tree_a"]}
                },
                "buildingRoofCoverageByUsage": {
                    "office": {"buildingModelCount": 1, "withRoofCount": 1, "withoutRoofCount": 0}
                },
                "buildingRoofCoverageByShape": {
                    "flat": {"buildingModelCount": 1, "withRoofCount": 1, "withoutRoofCount": 0}
                },
                "buildingModelCountByWallMaterial": {
                    "concrete": {"buildingModelCount": 1, "sourceIds": ["bldg_a"]}
                },
                "buildingModelCountByRoofMaterial": {
                    "slate": {"buildingModelCount": 1, "sourceIds": ["bldg_a"]}
                },
                "roadSurfacePartCountByKind": {
                    "secondary": {"surfacePartCount": 2, "featureCount": 1, "sourceIds": ["road_a"]}
                },
            },
        }
        play_report = {
            "rootName": "GeneratedWorld_Austin",
            "worldIdentity": "AustinManifestIndex",
            "chunkEnvelopeKind": "runtime_resident",
            "focus": {"x": 0, "z": 0},
            "radius": 256,
            "summary": {"marker": "ARNIS_SCENE_PLAY"},
            "scene": {
                "chunkIds": ["0_0", "1_0"],
                "buildingModelCount": 3,
                "buildingModelsWithDirectShell": 2,
                "buildingModelsWithRoofClosureDeck": 2,
                "roadSurfacePartCount": 5,
                "waterSurfacePartCount": 2,
                "propInstanceCount": 3,
                "roadSurfacePartCountBySubkind": {
                    "sidewalk": {
                        "surfacePartCount": 4,
                        "featureCount": 3,
                        "sourceIds": ["road_a", "road_b", "road_c"],
                    }
                },
                "waterSurfacePartCountByType": {
                    "polygon": {
                        "surfacePartCount": 2,
                        "featureCount": 2,
                        "sourceIds": ["water_a", "water_b"],
                    }
                },
                "waterSurfacePartCountByKind": {
                    "pond": {
                        "surfacePartCount": 2,
                        "featureCount": 2,
                        "sourceIds": ["water_a", "water_b"],
                    }
                },
                "railReceiptCountByKind": {
                    "rail": {
                        "receiptCount": 3,
                        "featureCount": 3,
                        "sourceIds": ["rail_a", "rail_b", "rail_c"],
                    }
                },
                "vegetationInstanceCountByKind": {
                    "tree": {
                        "instanceCount": 2,
                        "featureCount": 2,
                        "sourceIds": ["tree_a", "tree_b"],
                    }
                },
                "treeInstanceCountBySpecies": {
                    "oak": {
                        "instanceCount": 2,
                        "featureCount": 2,
                        "sourceIds": ["tree_a", "tree_b"],
                    }
                },
                "buildingRoofCoverageByUsage": {
                    "office": {"buildingModelCount": 3, "withRoofCount": 3, "withoutRoofCount": 0}
                },
                "buildingRoofCoverageByShape": {
                    "flat": {"buildingModelCount": 3, "withRoofCount": 3, "withoutRoofCount": 0}
                },
                "buildingModelCountByWallMaterial": {
                    "concrete": {"buildingModelCount": 3, "sourceIds": ["bldg_a", "bldg_b", "bldg_c"]}
                },
                "buildingModelCountByRoofMaterial": {
                    "slate": {"buildingModelCount": 2, "sourceIds": ["bldg_a", "bldg_b"]}
                },
                "roadSurfacePartCountByKind": {
                    "secondary": {
                        "surfacePartCount": 5,
                        "featureCount": 3,
                        "sourceIds": ["road_a", "road_b", "road_c"],
                    }
                },
            },
        }

        report = audit.build_report(edit_report, play_report)
        codes = {finding["code"] for finding in report["findings"]}

        self.assertNotIn("road_subkind_surface_mismatch", codes)
        self.assertNotIn("water_type_surface_mismatch", codes)
        self.assertNotIn("water_kind_surface_mismatch", codes)
        self.assertNotIn("rail_kind_receipt_mismatch", codes)
        self.assertNotIn("vegetation_kind_instance_mismatch", codes)
        self.assertNotIn("tree_species_instance_mismatch", codes)
        self.assertNotIn("wall_material_count_mismatch", codes)
        self.assertNotIn("roof_material_count_mismatch", codes)
        self.assertNotIn("road_kind_surface_mismatch", codes)
        self.assertNotIn("building_model_count_mismatch", codes)
        self.assertNotIn("building_direct_shell_count_mismatch", codes)
        self.assertNotIn("building_roof_closure_deck_count_mismatch", codes)
        self.assertNotIn("road_surface_part_count_mismatch", codes)
        self.assertNotIn("water_surface_part_count_mismatch", codes)
        self.assertNotIn("prop_instance_count_mismatch", codes)
        self.assertNotIn("roof_usage_coverage_mismatch", codes)
        self.assertNotIn("roof_shape_coverage_mismatch", codes)

    def test_build_report_rejects_preview_bucket_with_non_subset_source_ids(self) -> None:
        audit = load_module()
        edit_report = {
            "rootName": "GeneratedWorld_AustinPreview",
            "worldIdentity": "AustinManifestIndex",
            "chunkEnvelopeKind": "bounded_preview",
            "focus": {"x": 0, "z": 0},
            "radius": 256,
            "summary": {"marker": "ARNIS_SCENE_EDIT"},
            "scene": {
                "chunkIds": ["0_0"],
                "buildingModelCount": 1,
                "buildingModelsWithDirectShell": 1,
                "buildingModelsWithRoofClosureDeck": 1,
                "roadSurfacePartCount": 2,
                "waterSurfacePartCount": 0,
                "propInstanceCount": 0,
                "roadSurfacePartCountBySubkind": {
                    "sidewalk": {"surfacePartCount": 2, "featureCount": 1, "sourceIds": ["road_extra"]}
                },
                "waterSurfacePartCountByType": {},
                "waterSurfacePartCountByKind": {},
                "railReceiptCountByKind": {},
                "vegetationInstanceCountByKind": {},
                "treeInstanceCountBySpecies": {},
                "buildingRoofCoverageByUsage": {},
                "buildingRoofCoverageByShape": {},
                "buildingModelCountByWallMaterial": {},
                "buildingModelCountByRoofMaterial": {},
                "roadSurfacePartCountByKind": {
                    "secondary": {"surfacePartCount": 2, "featureCount": 1, "sourceIds": ["road_extra"]}
                },
            },
        }
        play_report = {
            "rootName": "GeneratedWorld_Austin",
            "worldIdentity": "AustinManifestIndex",
            "chunkEnvelopeKind": "runtime_resident",
            "focus": {"x": 0, "z": 0},
            "radius": 256,
            "summary": {"marker": "ARNIS_SCENE_PLAY"},
            "scene": {
                "chunkIds": ["0_0", "1_0"],
                "buildingModelCount": 2,
                "buildingModelsWithDirectShell": 2,
                "buildingModelsWithRoofClosureDeck": 2,
                "roadSurfacePartCount": 4,
                "waterSurfacePartCount": 0,
                "propInstanceCount": 0,
                "roadSurfacePartCountBySubkind": {
                    "sidewalk": {"surfacePartCount": 4, "featureCount": 2, "sourceIds": ["road_a", "road_b"]}
                },
                "waterSurfacePartCountByType": {},
                "waterSurfacePartCountByKind": {},
                "railReceiptCountByKind": {},
                "vegetationInstanceCountByKind": {},
                "treeInstanceCountBySpecies": {},
                "buildingRoofCoverageByUsage": {},
                "buildingRoofCoverageByShape": {},
                "buildingModelCountByWallMaterial": {},
                "buildingModelCountByRoofMaterial": {},
                "roadSurfacePartCountByKind": {
                    "secondary": {"surfacePartCount": 4, "featureCount": 2, "sourceIds": ["road_a", "road_b"]}
                },
            },
        }

        report = audit.build_report(edit_report, play_report)
        codes = {finding["code"] for finding in report["findings"]}

        self.assertIn("road_subkind_surface_mismatch", codes)
        self.assertIn("road_kind_surface_mismatch", codes)

    def test_build_report_rejects_non_monotonic_subset_counts(self) -> None:
        audit = load_module()
        edit_report = {
            "rootName": "GeneratedWorld_AustinPreview",
            "worldIdentity": "AustinManifestIndex",
            "chunkEnvelopeKind": "bounded_preview",
            "focus": {"x": 0, "z": 0},
            "radius": 256,
            "summary": {"marker": "ARNIS_SCENE_EDIT"},
            "scene": {
                "chunkIds": ["0_0"],
                "buildingModelCount": 4,
                "buildingModelsWithDirectShell": 3,
                "buildingModelsWithRoofClosureDeck": 3,
                "roadSurfacePartCount": 6,
                "waterSurfacePartCount": 2,
                "propInstanceCount": 4,
                "roadSurfacePartCountBySubkind": {},
                "waterSurfacePartCountByType": {},
                "waterSurfacePartCountByKind": {},
                "railReceiptCountByKind": {},
                "vegetationInstanceCountByKind": {},
                "treeInstanceCountBySpecies": {},
                "buildingRoofCoverageByUsage": {
                    "office": {"buildingModelCount": 4, "withRoofCount": 4, "withoutRoofCount": 0}
                },
                "buildingRoofCoverageByShape": {
                    "flat": {"buildingModelCount": 4, "withRoofCount": 4, "withoutRoofCount": 0}
                },
                "buildingModelCountByWallMaterial": {},
                "buildingModelCountByRoofMaterial": {},
                "roadSurfacePartCountByKind": {},
            },
        }
        play_report = {
            "rootName": "GeneratedWorld_Austin",
            "worldIdentity": "AustinManifestIndex",
            "chunkEnvelopeKind": "runtime_resident",
            "focus": {"x": 0, "z": 0},
            "radius": 256,
            "summary": {"marker": "ARNIS_SCENE_PLAY"},
            "scene": {
                "chunkIds": ["0_0", "1_0"],
                "buildingModelCount": 3,
                "buildingModelsWithDirectShell": 2,
                "buildingModelsWithRoofClosureDeck": 2,
                "roadSurfacePartCount": 5,
                "waterSurfacePartCount": 1,
                "propInstanceCount": 3,
                "roadSurfacePartCountBySubkind": {},
                "waterSurfacePartCountByType": {},
                "waterSurfacePartCountByKind": {},
                "railReceiptCountByKind": {},
                "vegetationInstanceCountByKind": {},
                "treeInstanceCountBySpecies": {},
                "buildingRoofCoverageByUsage": {
                    "office": {"buildingModelCount": 3, "withRoofCount": 3, "withoutRoofCount": 0}
                },
                "buildingRoofCoverageByShape": {
                    "flat": {"buildingModelCount": 3, "withRoofCount": 3, "withoutRoofCount": 0}
                },
                "buildingModelCountByWallMaterial": {},
                "buildingModelCountByRoofMaterial": {},
                "roadSurfacePartCountByKind": {},
            },
        }

        report = audit.build_report(edit_report, play_report)
        codes = {finding["code"] for finding in report["findings"]}

        self.assertIn("building_model_count_mismatch", codes)
        self.assertIn("building_direct_shell_count_mismatch", codes)
        self.assertIn("building_roof_closure_deck_count_mismatch", codes)
        self.assertIn("road_surface_part_count_mismatch", codes)
        self.assertIn("water_surface_part_count_mismatch", codes)
        self.assertIn("prop_instance_count_mismatch", codes)
        self.assertIn("roof_usage_coverage_mismatch", codes)
        self.assertIn("roof_shape_coverage_mismatch", codes)

    def test_main_renders_json_and_html_from_reports(self) -> None:
        audit = load_module()
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            edit_path = root / "edit.json"
            play_path = root / "play.json"
            parity_path = root / "parity.json"
            html_path = root / "parity.html"

            report_payload = {
                "rootName": "GeneratedWorld_Austin",
                "worldIdentity": "AustinManifestIndex",
                "chunkEnvelopeKind": "runtime_resident",
                "focus": {"x": 0, "z": 0},
                "radius": 256,
                "summary": {"marker": "ARNIS_SCENE_EDIT"},
                "scene": {
                    "chunkIds": ["0_0"],
                    "buildingModelCount": 1,
                    "buildingModelsWithDirectShell": 1,
                    "buildingModelsWithRoofClosureDeck": 1,
                    "roadSurfacePartCount": 2,
                    "waterSurfacePartCount": 0,
                    "propInstanceCount": 3,
                    "roadSurfacePartCountBySubkind": {"sidewalk": {"surfacePartCount": 1, "featureCount": 1}},
                    "waterSurfacePartCountByType": {},
                    "waterSurfacePartCountByKind": {},
                    "railReceiptCountByKind": {},
                    "vegetationInstanceCountByKind": {"tree": {"instanceCount": 1, "featureCount": 1}},
                    "treeInstanceCountBySpecies": {"oak": {"instanceCount": 1, "featureCount": 1}},
                    "buildingRoofCoverageByUsage": {"office": {"buildingModelCount": 1, "withRoofCount": 1}},
                    "buildingRoofCoverageByShape": {"flat": {"buildingModelCount": 1, "withRoofCount": 1}},
                    "buildingModelCountByWallMaterial": {"concrete": {"buildingModelCount": 1, "sourceIds": ["a"]}},
                    "buildingModelCountByRoofMaterial": {"slate": {"buildingModelCount": 1, "sourceIds": ["a"]}},
                    "roadSurfacePartCountByKind": {"secondary": {"surfacePartCount": 2, "featureCount": 1}},
                },
            }
            edit_path.write_text(json.dumps(report_payload), encoding="utf-8")
            play_path.write_text(json.dumps(report_payload | {"summary": {"marker": "ARNIS_SCENE_PLAY"}}), encoding="utf-8")

            exit_code = audit.main(
                [
                    "--edit-report",
                    str(edit_path),
                    "--play-report",
                    str(play_path),
                    "--json-out",
                    str(parity_path),
                    "--html-out",
                    str(html_path),
                ]
            )

            self.assertEqual(exit_code, 0)
            written = json.loads(parity_path.read_text(encoding="utf-8"))
            html = html_path.read_text(encoding="utf-8")
            self.assertEqual(written["summary"]["matching"], 21)
            self.assertEqual(written["summary"]["mismatched"], 0)
            self.assertIn("Scene Parity Audit", html)
            self.assertIn("chunkIds", html)
            self.assertIn("buildingRoofCoverageByShape", html)
            self.assertIn("roadSurfacePartCountBySubkind", html)

    def test_truth_pack_mismatch_is_not_subset_allowed(self) -> None:
        audit = load_module()
        edit_report = {
            "rootName": "GeneratedWorld_AustinPreview",
            "worldIdentity": "AustinManifestIndex",
            "chunkEnvelopeKind": "bounded_preview",
            "focus": {"x": 0, "z": 0},
            "radius": 256,
            "summary": {
                "marker": "ARNIS_SCENE_EDIT",
                "truthPack": {
                    "scene": "fixture",
                    "collapseCount": 2,
                    "droppedSemanticCount": 2,
                    "retainedSemanticCount": 2,
                    "droppedSemanticsBreakdown": {"structures": {"usage": 1}},
                    "collapseBreakdown": {"structures": {"overture->osm|cross_source_overlap": 2}},
                    "outdoorSourceCoverage": {
                        "structures": {
                            "source_feature_count": 3,
                            "retained_feature_count": 1,
                            "coverage_ratio": 0.3333,
                        }
                    },
                },
            },
            "scene": {
                "chunkIds": ["0_0"],
                "buildingModelCount": 1,
                "buildingModelsWithDirectShell": 1,
                "buildingModelsWithRoofClosureDeck": 1,
                "roadSurfacePartCount": 1,
                "waterSurfacePartCount": 0,
                "propInstanceCount": 0,
                "roadSurfacePartCountBySubkind": {},
                "waterSurfacePartCountByType": {},
                "waterSurfacePartCountByKind": {},
                "railReceiptCountByKind": {},
                "vegetationInstanceCountByKind": {},
                "treeInstanceCountBySpecies": {},
                "buildingRoofCoverageByUsage": {"school": {"buildingModelCount": 1, "withRoofCount": 1, "withoutRoofCount": 0}},
                "buildingRoofCoverageByShape": {"flat": {"buildingModelCount": 1, "withRoofCount": 1, "withoutRoofCount": 0}},
                "buildingModelCountByWallMaterial": {},
                "buildingModelCountByRoofMaterial": {},
                "roadSurfacePartCountByKind": {},
            },
        }
        play_report = {
            "rootName": "GeneratedWorld_Austin",
            "worldIdentity": "AustinManifestIndex",
            "chunkEnvelopeKind": "runtime_resident",
            "focus": {"x": 0, "z": 0},
            "radius": 256,
            "summary": {
                "marker": "ARNIS_SCENE_PLAY",
                "truthPack": {
                    "scene": "fixture",
                    "collapseCount": 1,
                    "droppedSemanticCount": 2,
                    "retainedSemanticCount": 2,
                    "droppedSemanticsBreakdown": {"structures": {"usage": 1}},
                    "collapseBreakdown": {"structures": {"overture->osm|cross_source_overlap": 1}},
                    "outdoorSourceCoverage": {
                        "structures": {
                            "source_feature_count": 3,
                            "retained_feature_count": 1,
                            "coverage_ratio": 0.3333,
                        }
                    },
                },
            },
            "scene": {
                "chunkIds": ["0_0", "1_0"],
                "buildingModelCount": 2,
                "buildingModelsWithDirectShell": 2,
                "buildingModelsWithRoofClosureDeck": 2,
                "roadSurfacePartCount": 2,
                "waterSurfacePartCount": 0,
                "propInstanceCount": 0,
                "roadSurfacePartCountBySubkind": {},
                "waterSurfacePartCountByType": {},
                "waterSurfacePartCountByKind": {},
                "railReceiptCountByKind": {},
                "vegetationInstanceCountByKind": {},
                "treeInstanceCountBySpecies": {},
                "buildingRoofCoverageByUsage": {"school": {"buildingModelCount": 2, "withRoofCount": 2, "withoutRoofCount": 0}},
                "buildingRoofCoverageByShape": {"flat": {"buildingModelCount": 2, "withRoofCount": 2, "withoutRoofCount": 0}},
                "buildingModelCountByWallMaterial": {},
                "buildingModelCountByRoofMaterial": {},
                "roadSurfacePartCountByKind": {},
            },
        }

        report = audit.build_report(edit_report, play_report)
        codes = {finding["code"] for finding in report["findings"]}

        self.assertIn("truth_pack_mismatch", codes)
        self.assertNotIn("building_model_count_mismatch", codes)
        self.assertNotIn("building_direct_shell_count_mismatch", codes)
        self.assertNotIn("building_roof_closure_deck_count_mismatch", codes)
        self.assertNotIn("road_surface_part_count_mismatch", codes)
        self.assertEqual(report["comparisons"]["truthPack"]["edit"]["collapseCount"], 2)
        self.assertEqual(report["comparisons"]["truthPack"]["play"]["collapseCount"], 1)
        self.assertEqual(
            report["comparisons"]["truthPack"]["edit"]["collapseBreakdown"]["structures"]["overture->osm|cross_source_overlap"],
            2,
        )


if __name__ == "__main__":
    unittest.main()
