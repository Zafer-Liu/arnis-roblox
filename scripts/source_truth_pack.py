#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
import sqlite3
from typing import Any


def summary_path_for_db(db_path: Path) -> Path:
    return db_path.with_suffix(".summary.json")


def _connect(db_path: Path) -> sqlite3.Connection:
    connection = sqlite3.connect(db_path)
    connection.row_factory = sqlite3.Row
    return connection


def _rows_to_dicts(rows: list[sqlite3.Row]) -> list[dict[str, Any]]:
    return [dict(row) for row in rows]


def load_summary(db_path: Path | str) -> dict[str, Any]:
    db_path = Path(db_path)
    summary_path = summary_path_for_db(db_path)
    if summary_path.is_file():
        return json.loads(summary_path.read_text(encoding="utf-8"))

    connection = _connect(db_path)
    try:
        feature_count = connection.execute("SELECT COUNT(*) FROM features").fetchone()[0]
        retained_semantic_count = connection.execute(
            "SELECT COUNT(*) FROM retained_semantics"
        ).fetchone()[0]
        dropped_semantic_count = connection.execute(
            "SELECT COUNT(*) FROM dropped_semantics"
        ).fetchone()[0]
        collapse_count = connection.execute("SELECT COUNT(*) FROM collapses").fetchone()[0]
        source_counts = {
            row["source_name"]: row["count"]
            for row in connection.execute(
                """
                SELECT source_name, COUNT(*) AS count
                FROM feature_sources
                GROUP BY source_name
                ORDER BY source_name
                """
            ).fetchall()
        }
    finally:
        connection.close()

    return {
        "scene": db_path.name.removesuffix(".truth-pack.sqlite"),
        "feature_count": feature_count,
        "retained_semantic_count": retained_semantic_count,
        "dropped_semantic_count": dropped_semantic_count,
        "collapse_count": collapse_count,
        "source_counts": source_counts,
    }


def load_feature(db_path: Path | str, feature_id: str) -> dict[str, Any]:
    connection = _connect(Path(db_path))
    try:
        feature = connection.execute(
            """
            SELECT feature_id, feature_kind, canonical_feature_id, is_retained
            FROM features
            WHERE feature_id = ?
            """,
            (feature_id,),
        ).fetchone()
        if feature is None:
            raise KeyError(f"feature not found: {feature_id}")

        sources = _rows_to_dicts(
            connection.execute(
                """
                SELECT feature_id, source_name, source_feature_id, source_layer
                FROM feature_sources
                WHERE feature_id = ?
                ORDER BY source_name, source_feature_id, source_layer
                """,
                (feature_id,),
            ).fetchall()
        )
        retained_semantics = _rows_to_dicts(
            connection.execute(
                """
                SELECT feature_id, field_name, field_value
                FROM retained_semantics
                WHERE feature_id = ?
                ORDER BY field_name, field_value
                """,
                (feature_id,),
            ).fetchall()
        )
        collapses = _rows_to_dicts(
            connection.execute(
                """
                SELECT feature_id, retained_feature_id, collapse_kind, matched_source
                FROM collapses
                WHERE feature_id = ?
                ORDER BY retained_feature_id, collapse_kind, matched_source
                """,
                (feature_id,),
            ).fetchall()
        )
        dropped_semantics = _rows_to_dicts(
            connection.execute(
                """
                SELECT feature_id, field_name, field_value, reason, retained_feature_id
                FROM dropped_semantics
                WHERE feature_id = ?
                ORDER BY field_name, field_value
                """,
                (feature_id,),
            ).fetchall()
        )
    finally:
        connection.close()

    return {
        "feature": dict(feature),
        "sources": sources,
        "retained_semantics": retained_semantics,
        "collapses": collapses,
        "dropped_semantics": dropped_semantics,
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Inspect bounded source truth-pack outputs without regenerating them."
    )
    parser.add_argument("truth_pack", type=Path, help="Path to *.truth-pack.sqlite")
    parser.add_argument("--feature", help="Feature id to inspect")
    args = parser.parse_args()

    payload: dict[str, Any]
    if args.feature:
        payload = load_feature(args.truth_pack, args.feature)
    else:
        payload = load_summary(args.truth_pack)

    print(json.dumps(payload, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
