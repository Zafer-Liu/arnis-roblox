#!/usr/bin/env python3
from __future__ import annotations

import argparse
from collections import Counter
from datetime import datetime, timezone
import json
from pathlib import Path
import sqlite3
from typing import Any


HOTSPOT_LIMIT = 8
OUTDOOR_FAMILIES = ("terrain", "landuse", "roads", "water", "vegetation", "structures")
OUTDOOR_FEATURE_KINDS = (
    "terrain",
    "landuse",
    "road",
    "roads",
    "water",
    "vegetation",
    "tree",
    "prop",
    "structure",
    "building",
)
FEATURE_KIND_TO_FAMILY = {
    "terrain": "terrain",
    "landuse": "landuse",
    "road": "roads",
    "roads": "roads",
    "water": "water",
    "vegetation": "vegetation",
    "tree": "vegetation",
    "prop": "vegetation",
    "structure": "structures",
    "building": "structures",
}


def _connect(db_path: Path) -> sqlite3.Connection:
    connection = sqlite3.connect(db_path)
    connection.row_factory = sqlite3.Row
    return connection


def _summary_path(db_path: Path) -> Path:
    return db_path.with_suffix(".summary.json")


def _load_summary(db_path: Path) -> dict[str, Any]:
    summary_path = _summary_path(db_path)
    if not summary_path.is_file():
        return {}
    return json.loads(summary_path.read_text(encoding="utf-8"))


def _family_for_kind(feature_kind: Any) -> str:
    normalized = str(feature_kind or "").strip().lower()
    return FEATURE_KIND_TO_FAMILY.get(normalized, normalized or "unknown")


def _coverage_with_defaults(coverage: dict[str, Any] | None) -> dict[str, dict[str, float | int]]:
    normalized: dict[str, dict[str, float | int]] = {}
    coverage = coverage or {}
    for family in OUTDOOR_FAMILIES:
        row = coverage.get(family) if isinstance(coverage.get(family), dict) else {}
        source_feature_count = int(row.get("source_feature_count", 0))
        retained_feature_count = int(row.get("retained_feature_count", 0))
        coverage_ratio = 0.0 if source_feature_count <= 0 else round(retained_feature_count / source_feature_count, 4)
        normalized[family] = {
            "source_feature_count": source_feature_count,
            "retained_feature_count": retained_feature_count,
            "coverage_ratio": coverage_ratio,
        }
    return normalized


def _top_samples(rows: list[dict[str, Any]], *, sort_keys: tuple[str, ...]) -> list[dict[str, Any]]:
    return sorted(
        rows,
        key=lambda row: tuple(str(row.get(key) or "") for key in sort_keys),
    )[:HOTSPOT_LIMIT]


def _placeholders(values: tuple[str, ...]) -> str:
    return ",".join("?" for _ in values)


def _add_finding(
    findings: list[dict[str, Any]],
    *,
    severity: str,
    code: str,
    message: str,
    metric: str,
    value: Any,
    threshold: Any,
) -> None:
    findings.append(
        {
            "severity": severity,
            "code": code,
            "message": message,
            "metric": metric,
            "value": value,
            "threshold": threshold,
        }
    )


def build_report(truth_pack: Path) -> dict[str, Any]:
    truth_pack = truth_pack.resolve()
    summary_data = _load_summary(truth_pack)
    connection = _connect(truth_pack)
    kinds_clause = _placeholders(OUTDOOR_FEATURE_KINDS)
    try:
        feature_count = int(connection.execute("SELECT COUNT(*) FROM features").fetchone()[0])
        retained_semantics_rows = [dict(row) for row in connection.execute(
            """
            SELECT f.feature_kind, COUNT(*) AS semantic_count
            FROM retained_semantics rs
            JOIN features f ON f.feature_id = rs.feature_id
            WHERE LOWER(f.feature_kind) IN ("""
            + kinds_clause
            + """)
            GROUP BY f.feature_kind
            ORDER BY f.feature_kind
            """
            ,
            OUTDOOR_FEATURE_KINDS,
        ).fetchall()]
        dropped_semantics_rows = [dict(row) for row in connection.execute(
            """
            SELECT f.feature_kind, COUNT(*) AS semantic_count
            FROM dropped_semantics ds
            JOIN features f ON f.feature_id = ds.feature_id
            WHERE LOWER(f.feature_kind) IN ("""
            + kinds_clause
            + """)
            GROUP BY f.feature_kind
            ORDER BY f.feature_kind
            """
            ,
            OUTDOOR_FEATURE_KINDS,
        ).fetchall()]
        overlap_rows = [dict(row) for row in connection.execute(
            """
            SELECT f.feature_kind, COUNT(*) AS collapse_count
            FROM collapses c
            JOIN features f ON f.feature_id = c.feature_id
            WHERE LOWER(f.feature_kind) IN ("""
            + kinds_clause
            + """)
            GROUP BY f.feature_kind
            ORDER BY f.feature_kind
            """
            ,
            OUTDOOR_FEATURE_KINDS,
        ).fetchall()]
        source_feature_counts_rows = connection.execute(
            """
            SELECT f.feature_kind, COUNT(DISTINCT fs.feature_id) AS source_feature_count
            FROM feature_sources fs
            JOIN features f ON f.feature_id = fs.feature_id
            WHERE LOWER(f.feature_kind) IN ("""
            + kinds_clause
            + """)
            GROUP BY f.feature_kind
            ORDER BY f.feature_kind
            """
            ,
            OUTDOOR_FEATURE_KINDS,
        ).fetchall()
        retained_feature_counts_rows = connection.execute(
            """
            SELECT feature_kind, COUNT(*) AS retained_feature_count
            FROM features
            WHERE is_retained = 1
              AND LOWER(feature_kind) IN ("""
            + kinds_clause
            + """)
            GROUP BY feature_kind
            ORDER BY feature_kind
            """
            ,
            OUTDOOR_FEATURE_KINDS,
        ).fetchall()
        overlap_sample_rows = [dict(row) for row in connection.execute(
            """
            SELECT c.feature_id, f.feature_kind, c.retained_feature_id, c.collapse_kind, c.matched_source
            FROM collapses c
            JOIN features f ON f.feature_id = c.feature_id
            WHERE LOWER(f.feature_kind) IN ("""
            + kinds_clause
            + """)
            ORDER BY f.feature_kind, c.feature_id, c.retained_feature_id
            LIMIT ?
            """
            ,
            (*OUTDOOR_FEATURE_KINDS, HOTSPOT_LIMIT),
        ).fetchall()]
        dropped_sample_rows = [dict(row) for row in connection.execute(
            """
            SELECT ds.feature_id, f.feature_kind, ds.field_name, ds.field_value, ds.reason, ds.retained_feature_id
            FROM dropped_semantics ds
            JOIN features f ON f.feature_id = ds.feature_id
            WHERE LOWER(f.feature_kind) IN ("""
            + kinds_clause
            + """)
            ORDER BY f.feature_kind, ds.feature_id, ds.field_name, ds.field_value
            LIMIT ?
            """
            ,
            (*OUTDOOR_FEATURE_KINDS, HOTSPOT_LIMIT),
        ).fetchall()]
    finally:
        connection.close()

    retained_semantics_by_family: Counter[str] = Counter()
    dropped_semantics_by_family: Counter[str] = Counter()
    overlap_loss_by_family: Counter[str] = Counter()
    source_feature_counts_by_family: Counter[str] = Counter()
    retained_feature_counts_by_family: Counter[str] = Counter()

    for row in retained_semantics_rows:
        retained_semantics_by_family[_family_for_kind(row.get("feature_kind"))] += int(row["semantic_count"])
    for row in dropped_semantics_rows:
        dropped_semantics_by_family[_family_for_kind(row.get("feature_kind"))] += int(row["semantic_count"])
    for row in overlap_rows:
        overlap_loss_by_family[_family_for_kind(row.get("feature_kind"))] += int(row["collapse_count"])
    for row in source_feature_counts_rows:
        source_feature_counts_by_family[_family_for_kind(row["feature_kind"])] += int(row["source_feature_count"])
    for row in retained_feature_counts_rows:
        retained_feature_counts_by_family[_family_for_kind(row["feature_kind"])] += int(row["retained_feature_count"])

    summary_coverage = summary_data.get("outdoor_source_coverage")
    if isinstance(summary_coverage, dict):
        outdoor_source_coverage = _coverage_with_defaults(summary_coverage)
    else:
        outdoor_source_coverage = _coverage_with_defaults(
            {
                family: {
                    "source_feature_count": source_feature_counts_by_family[family],
                    "retained_feature_count": retained_feature_counts_by_family[family],
                }
                for family in OUTDOOR_FAMILIES
            }
        )

    overlap_samples = [
        {
            "feature_id": str(row["feature_id"]),
            "family": _family_for_kind(row["feature_kind"]),
            "retained_feature_id": str(row["retained_feature_id"]),
            "collapse_kind": str(row["collapse_kind"]),
            "matched_source": str(row["matched_source"]),
        }
        for row in overlap_sample_rows
    ]
    dropped_samples = [
        {
            "feature_id": str(row["feature_id"]),
            "family": _family_for_kind(row["feature_kind"]),
            "field_name": str(row["field_name"]),
            "field_value": str(row["field_value"]),
            "reason": str(row["reason"]),
            "retained_feature_id": row["retained_feature_id"],
        }
        for row in dropped_sample_rows
    ]

    retained_semantic_count = sum(retained_semantics_by_family.values())
    dropped_semantic_count = sum(dropped_semantics_by_family.values())
    collapse_count = sum(overlap_loss_by_family.values())

    source_counts = summary_data.get("source_counts") if isinstance(summary_data.get("source_counts"), dict) else {}
    source_counts = {str(key): int(value) for key, value in source_counts.items()}

    summary = {
        "feature_count": feature_count,
        "retained_semantic_count": retained_semantic_count,
        "dropped_semantic_count": dropped_semantic_count,
        "collapse_count": collapse_count,
        "source_counts": source_counts,
        "retained_semantics_by_family": {
            family: int(retained_semantics_by_family.get(family, 0)) for family in OUTDOOR_FAMILIES
        },
        "dropped_semantics_by_family": {
            family: int(dropped_semantics_by_family.get(family, 0)) for family in OUTDOOR_FAMILIES
        },
        "overlap_loss_by_family": {
            family: int(overlap_loss_by_family.get(family, 0)) for family in OUTDOOR_FAMILIES
        },
        "outdoor_source_coverage": outdoor_source_coverage,
    }
    samples = {
        "overlap_losses": _top_samples(overlap_samples, sort_keys=("family", "feature_id", "retained_feature_id")),
        "dropped_semantics": _top_samples(dropped_samples, sort_keys=("family", "feature_id", "field_name")),
    }

    findings: list[dict[str, Any]] = []
    if collapse_count > 0:
        _add_finding(
            findings,
            severity="warning",
            code="truth_pack_outdoor_overlap_loss",
            message="Truth-pack records outdoor source overlaps that were collapsed before manifest emission.",
            metric="truth_pack_collapse_count",
            value=collapse_count,
            threshold="== 0",
        )
    if dropped_semantic_count > 0:
        _add_finding(
            findings,
            severity="warning",
            code="truth_pack_dropped_semantics",
            message="Truth-pack records dropped outdoor semantics that did not carry through to retained features.",
            metric="truth_pack_dropped_semantic_count",
            value=dropped_semantic_count,
            threshold="== 0",
        )
    if retained_semantic_count > 0:
        _add_finding(
            findings,
            severity="info",
            code="truth_pack_retained_semantics",
            message="Truth-pack records retained outdoor semantics by family for source coverage verification.",
            metric="truth_pack_retained_semantic_count",
            value=retained_semantic_count,
            threshold="> 0",
        )

    return {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "truth_pack_path": str(truth_pack),
        "scene": str(summary_data.get("scene") or truth_pack.name.removesuffix(".truth-pack.sqlite")),
        "summary": summary,
        "samples": samples,
        "findings": findings,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit bounded source truth-pack sqlite outputs.")
    parser.add_argument("truth_pack", type=Path, help="Path to *.truth-pack.sqlite")
    parser.add_argument("--json-out", type=Path)
    args = parser.parse_args()

    report = build_report(args.truth_pack)
    payload = json.dumps(report, indent=2, sort_keys=True)
    if args.json_out:
        args.json_out.write_text(payload, encoding="utf-8")
    else:
        print(payload)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
