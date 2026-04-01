from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
RUST_DIR = ROOT / "rust"
DATA_DIR = RUST_DIR / "data"
MODULE_PATH = ROOT / "scripts" / "source_truth_pack.py"
AUSTIN_BBOX = "30.245,-97.765,30.305,-97.715"


def load_module():
    spec = importlib.util.spec_from_file_location("source_truth_pack", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load module spec from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def build_fixture_truth_pack(db_path: Path) -> None:
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
            feature_id TEXT NOT NULL,
            field_name TEXT NOT NULL,
            field_value TEXT NOT NULL,
            reason TEXT NOT NULL,
            retained_feature_id TEXT
        );
        """
    )
    connection.executemany(
        "INSERT INTO features (feature_id, feature_kind, canonical_feature_id, is_retained) VALUES (?, ?, ?, ?)",
        [
            ("osm_10", "building", "osm_10", 1),
            ("ov_fixture_1", "building", "osm_10", 0),
        ],
    )
    connection.executemany(
        "INSERT INTO sources (source_name, provider, dataset) VALUES (?, ?, ?)",
        [
            ("osm", "osm", "provider"),
            ("overpass", "osm", "overpass"),
            ("overture", "overture", "buildings"),
        ],
    )
    connection.executemany(
        "INSERT INTO feature_sources (feature_id, source_name, source_feature_id, source_layer) VALUES (?, ?, ?, ?)",
        [
            ("osm_10", "osm", "10", "building"),
            ("osm_10", "overpass", "10", "way"),
            ("ov_fixture_1", "overture", "fixture-1", "building"),
        ],
    )
    connection.executemany(
        "INSERT INTO retained_semantics (feature_id, field_name, field_value) VALUES (?, ?, ?)",
        [
            ("osm_10", "usage", "school"),
            ("osm_10", "name", "Fixture Hall"),
        ],
    )
    connection.execute(
        "INSERT INTO collapses (feature_id, retained_feature_id, collapse_kind, matched_source) VALUES (?, ?, ?, ?)",
        ("ov_fixture_1", "osm_10", "cross_source_overlap", "overture->osm"),
    )
    connection.executemany(
        "INSERT INTO dropped_semantics (feature_id, field_name, field_value, reason, retained_feature_id) VALUES (?, ?, ?, ?, ?)",
        [
            ("ov_fixture_1", "usage", "commercial", "collapsed_into_retained_feature", "osm_10"),
            ("ov_fixture_1", "material", "glass", "collapsed_into_retained_feature", "osm_10"),
        ],
    )
    connection.commit()
    connection.close()


class SourceTruthPackHelperTests(unittest.TestCase):
    maxDiff = None

    def test_helper_reads_existing_truth_pack_outputs(self) -> None:
        module = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            db_path = root / "fixture.truth-pack.sqlite"
            summary_path = root / "fixture.truth-pack.summary.json"
            build_fixture_truth_pack(db_path)
            summary_path.write_text(
                json.dumps(
                    {
                        "scene": "fixture",
                        "feature_count": 2,
                        "retained_semantic_count": 2,
                        "dropped_semantic_count": 2,
                        "collapse_count": 1,
                        "source_counts": {"osm": 1, "overpass": 1, "overture": 1},
                    }
                ),
                encoding="utf-8",
            )

            summary = module.load_summary(db_path)
            feature = module.load_feature(db_path, "osm_10")

            self.assertEqual(summary["scene"], "fixture")
            self.assertEqual(feature["feature"]["feature_id"], "osm_10")
            self.assertEqual(
                [row["source_name"] for row in feature["sources"]],
                ["osm", "overpass"],
            )
            self.assertEqual(feature["retained_semantics"][0]["field_name"], "name")
            self.assertEqual(feature["retained_semantics"][1]["field_name"], "usage")
            self.assertEqual(feature["collapses"], [])


class SourceTruthPackCompileTests(unittest.TestCase):
    maxDiff = None

    def run_compile(self, output_root: Path) -> tuple[Path, Path, Path, Path]:
        if not (DATA_DIR / "austin_overpass.json").is_file():
            self.skipTest("requires rust/data/austin_overpass.json")
        if not (DATA_DIR / "overture_buildings.geojson").is_file():
            self.skipTest("requires rust/data/overture_buildings.geojson")

        manifest_path = output_root / "austin-manifest.json"
        manifest_sqlite_path = output_root / "austin-manifest.sqlite"
        truth_pack_path = output_root / "austin.truth-pack.sqlite"
        summary_path = output_root / "austin.truth-pack.summary.json"
        result = subprocess.run(
            [
                "cargo",
                "run",
                "-p",
                "arbx_cli",
                "--",
                "compile",
                "--source",
                "data/austin_overpass.json",
                "--bbox",
                AUSTIN_BBOX,
                "--profile",
                "fast",
                "--out",
                str(manifest_path),
                "--sqlite-out",
                str(manifest_sqlite_path),
                "--truth-pack-out",
                str(truth_pack_path),
                "--truth-pack-summary-out",
                str(summary_path),
            ],
            cwd=RUST_DIR,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(
            result.returncode,
            0,
            msg=f"compile failed\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}",
        )
        return manifest_path, manifest_sqlite_path, truth_pack_path, summary_path

    def test_compile_writes_truth_pack_and_records_source_union(self) -> None:
        module = load_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            _, _, truth_pack_path, summary_path = self.run_compile(temp_root)

            self.assertTrue(truth_pack_path.is_file(), f"missing truth-pack sqlite at {truth_pack_path}")
            self.assertTrue(summary_path.is_file(), f"missing truth-pack summary at {summary_path}")

            connection = sqlite3.connect(truth_pack_path)
            tables = {
                row[0]
                for row in connection.execute(
                    "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name"
                )
            }
            self.assertTrue(
                {
                    "features",
                    "sources",
                    "feature_sources",
                    "retained_semantics",
                    "collapses",
                    "dropped_semantics",
                }.issubset(tables)
            )

            source_names = {
                row[0]
                for row in connection.execute("SELECT source_name FROM sources ORDER BY source_name")
            }
            self.assertEqual(source_names, {"osm", "overpass", "overture"})

            multi_source_provenance = connection.execute(
                """
                SELECT COUNT(*)
                FROM (
                    SELECT feature_id
                    FROM feature_sources
                    GROUP BY feature_id
                    HAVING COUNT(DISTINCT source_name) >= 2
                )
                """
            ).fetchone()[0]
            collapse_count = connection.execute("SELECT COUNT(*) FROM collapses").fetchone()[0]
            retained_count = connection.execute("SELECT COUNT(*) FROM retained_semantics").fetchone()[0]
            dropped_count = connection.execute("SELECT COUNT(*) FROM dropped_semantics").fetchone()[0]
            feature_count = connection.execute("SELECT COUNT(*) FROM features").fetchone()[0]
            connection.close()

            self.assertGreater(multi_source_provenance, 0)
            self.assertGreater(collapse_count, 0)
            self.assertGreater(retained_count, 0)
            self.assertGreater(dropped_count, 0)
            self.assertGreater(feature_count, 0)

            summary = module.load_summary(truth_pack_path)
            self.assertEqual(summary["scene"], "austin")
            self.assertEqual(summary["feature_count"], feature_count)
            self.assertEqual(summary["collapse_count"], collapse_count)
            self.assertEqual(summary["retained_semantic_count"], retained_count)
            self.assertEqual(summary["dropped_semantic_count"], dropped_count)
            self.assertEqual(set(summary["source_counts"]), {"osm", "overpass", "overture"})


if __name__ == "__main__":
    unittest.main()
