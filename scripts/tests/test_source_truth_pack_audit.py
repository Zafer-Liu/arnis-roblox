from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import sqlite3
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "source_truth_pack_audit.py"


def load_module():
    spec = importlib.util.spec_from_file_location("source_truth_pack_audit", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load module spec from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def build_truth_pack_fixture(db_path: Path) -> Path:
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
            ("dem", "elevation", "dem"),
            ("landcover", "landcover", "landuse"),
        ],
    )
    connection.executemany(
        "INSERT INTO features (feature_id, feature_kind, canonical_feature_id, is_retained) VALUES (?, ?, ?, ?)",
        [
            ("terrain_1", "terrain", "terrain_1", 1),
            ("landuse_1", "landuse", "landuse_1", 1),
            ("road_1", "road", "road_1", 1),
            ("water_1", "water", "water_1", 1),
            ("veg_1", "vegetation", "veg_1", 1),
            ("rail_1", "rail", "rail_1", 1),
            ("structure_1", "structure", "structure_1", 1),
            ("structure_overlap_1", "structure", "structure_1", 0),
            ("structure_overlap_2", "structure", "structure_1", 0),
        ],
    )
    connection.executemany(
        "INSERT INTO feature_sources (feature_id, source_name, source_feature_id, source_layer) VALUES (?, ?, ?, ?)",
        [
            ("terrain_1", "dem", "terrain-src-1", "terrain"),
            ("landuse_1", "landcover", "landuse-src-1", "landuse"),
            ("road_1", "overpass", "road-src-1", "roads"),
            ("water_1", "overpass", "water-src-1", "water"),
            ("veg_1", "overpass", "veg-src-1", "vegetation"),
            ("rail_1", "overpass", "rail-src-1", "rail"),
            ("structure_1", "overpass", "osm-1", "structures"),
            ("structure_overlap_1", "overture", "ov-1", "structures"),
            ("structure_overlap_2", "overture", "ov-2", "structures"),
        ],
    )
    connection.executemany(
        "INSERT INTO retained_semantics (feature_id, field_name, field_value) VALUES (?, ?, ?)",
        [
            ("terrain_1", "material", "Grass"),
            ("landuse_1", "landuse", "park"),
            ("road_1", "surface", "asphalt"),
            ("water_1", "kind", "river"),
            ("veg_1", "species", "oak"),
            ("veg_1", "leaf_type", "broadleaved"),
            ("rail_1", "kind", "tram"),
            ("structure_1", "usage", "school"),
            ("structure_1", "name", "Fixture Hall"),
        ],
    )
    connection.executemany(
        "INSERT INTO collapses (feature_id, retained_feature_id, collapse_kind, matched_source) VALUES (?, ?, ?, ?)",
        [
            ("rail_1", "rail_1", "same_source_overlap", "overpass->overpass"),
            ("structure_overlap_1", "structure_1", "cross_source_overlap", "overture->osm"),
            ("structure_overlap_2", "structure_1", "cross_source_overlap", "overture->osm"),
        ],
    )
    connection.executemany(
        "INSERT INTO dropped_semantics (feature_id, field_name, field_value, reason, retained_feature_id) VALUES (?, ?, ?, ?, ?)",
        [
            ("rail_1", "surface", "steel", "collapsed_into_retained_feature", "rail_1"),
            ("structure_overlap_1", "usage", "commercial", "collapsed_into_retained_feature", "structure_1"),
            ("structure_overlap_1", "height", "18", "collapsed_into_retained_feature", "structure_1"),
            ("structure_overlap_2", "material", "glass", "collapsed_into_retained_feature", "structure_1"),
        ],
    )
    connection.commit()
    connection.close()

    summary_path = db_path.with_suffix(".summary.json")
    summary_path.write_text(
        json.dumps(
            {
                "scene": "fixture",
                "feature_count": 9,
                "retained_semantic_count": 9,
                "dropped_semantic_count": 4,
                "collapse_count": 3,
                "source_counts": {
                    "dem": 1,
                    "landcover": 1,
                    "overpass": 5,
                    "overture": 2,
                },
                "outdoor_source_coverage": {
                    "terrain": {"source_feature_count": 1, "retained_feature_count": 1},
                    "landuse": {"source_feature_count": 1, "retained_feature_count": 1},
                    "roads": {"source_feature_count": 1, "retained_feature_count": 1},
                    "water": {"source_feature_count": 1, "retained_feature_count": 1},
                    "vegetation": {"source_feature_count": 1, "retained_feature_count": 1},
                    "structures": {"source_feature_count": 3, "retained_feature_count": 1},
                },
            }
        ),
        encoding="utf-8",
    )
    return summary_path


class SourceTruthPackAuditTests(unittest.TestCase):
    maxDiff = None

    def test_truth_pack_audit_reports_compact_outdoor_findings(self) -> None:
        audit = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            truth_pack_path = root / "fixture.truth-pack.sqlite"
            build_truth_pack_fixture(truth_pack_path)

            report = audit.build_report(truth_pack_path)
            codes = {finding["code"] for finding in report["findings"]}

            self.assertEqual(report["scene"], "fixture")
            self.assertEqual(report["summary"]["feature_count"], 9)
            self.assertEqual(report["summary"]["collapse_count"], 2)
            self.assertEqual(report["summary"]["dropped_semantic_count"], 3)
            self.assertEqual(report["summary"]["retained_semantic_count"], 8)
            self.assertEqual(report["summary"]["retained_semantics_by_family"]["vegetation"], 2)
            self.assertEqual(report["summary"]["dropped_semantics_by_family"]["structures"], 3)
            self.assertEqual(report["summary"]["overlap_loss_by_family"]["structures"], 2)
            self.assertEqual(
                report["summary"]["outdoor_source_coverage"]["structures"]["coverage_ratio"],
                0.3333,
            )
            self.assertEqual(
                set(report["summary"]["outdoor_source_coverage"]),
                {"terrain", "landuse", "roads", "water", "vegetation", "structures"},
            )
            self.assertIn("truth_pack_outdoor_overlap_loss", codes)
            self.assertIn("truth_pack_dropped_semantics", codes)
            self.assertIn("truth_pack_retained_semantics", codes)
            self.assertNotIn("rail", json.dumps(report["summary"]["retained_semantics_by_family"], sort_keys=True))
            self.assertEqual(len(report["samples"]["dropped_semantics"]), 3)
            self.assertEqual(len(report["samples"]["overlap_losses"]), 2)


if __name__ == "__main__":
    unittest.main()
