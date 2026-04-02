use crate::{PipelineError, PipelineResult};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTruthPack {
    pub features: Vec<TruthPackFeature>,
    pub sources: Vec<TruthPackSource>,
    pub feature_sources: Vec<TruthPackFeatureSource>,
    pub retained_semantics: Vec<TruthPackSemantic>,
    pub semantic_lineage: Vec<TruthPackSemanticLineage>,
    pub collapses: Vec<TruthPackCollapse>,
    pub dropped_semantics: Vec<TruthPackDroppedSemantic>,
}

impl SourceTruthPack {
    pub fn new() -> Self {
        Self {
            features: Vec::new(),
            sources: vec![
                TruthPackSource {
                    source_name: "osm".to_string(),
                    provider: "osm".to_string(),
                    dataset: "canonical".to_string(),
                },
                TruthPackSource {
                    source_name: "overpass".to_string(),
                    provider: "osm".to_string(),
                    dataset: "overpass".to_string(),
                },
                TruthPackSource {
                    source_name: "overture".to_string(),
                    provider: "overture".to_string(),
                    dataset: "buildings".to_string(),
                },
            ],
            feature_sources: Vec::new(),
            retained_semantics: Vec::new(),
            semantic_lineage: Vec::new(),
            collapses: Vec::new(),
            dropped_semantics: Vec::new(),
        }
    }

    pub fn summary(&self, scene: impl Into<String>) -> SourceTruthPackSummary {
        let mut source_counts = BTreeMap::new();
        for source in &self.feature_sources {
            *source_counts.entry(source.source_name.clone()).or_insert(0) += 1;
        }
        SourceTruthPackSummary {
            scene: scene.into(),
            feature_count: self.features.len(),
            retained_semantic_count: self.retained_semantics.len(),
            semantic_lineage_count: self.semantic_lineage.len(),
            dropped_semantic_count: self.dropped_semantics.len(),
            collapse_count: self.collapses.len(),
            source_counts,
        }
    }
}

impl Default for SourceTruthPack {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthPackFeature {
    pub feature_id: String,
    pub feature_kind: String,
    pub canonical_feature_id: Option<String>,
    pub is_retained: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthPackSource {
    pub source_name: String,
    pub provider: String,
    pub dataset: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthPackFeatureSource {
    pub feature_id: String,
    pub source_name: String,
    pub source_feature_id: String,
    pub source_layer: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthPackSemantic {
    pub feature_id: String,
    pub field_name: String,
    pub field_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthPackSemanticLineage {
    pub retained_feature_id: String,
    pub field_name: String,
    pub field_value: String,
    pub source_name: String,
    pub source_feature_id: String,
    pub resolution: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthPackCollapse {
    pub feature_id: String,
    pub retained_feature_id: String,
    pub collapse_kind: String,
    pub matched_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthPackDroppedSemantic {
    pub feature_id: String,
    pub field_name: String,
    pub field_value: String,
    pub reason: String,
    pub retained_feature_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceTruthPackSummary {
    pub scene: String,
    pub feature_count: usize,
    pub retained_semantic_count: usize,
    pub semantic_lineage_count: usize,
    pub dropped_semantic_count: usize,
    pub collapse_count: usize,
    pub source_counts: BTreeMap<String, usize>,
}

pub fn write_source_truth_pack_sqlite(pack: &SourceTruthPack, path: &Path) -> PipelineResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            PipelineError::IO(format!(
                "failed to create truth-pack parent directory: {err}"
            ))
        })?;
    }

    if path.exists() {
        fs::remove_file(path)
            .map_err(|err| PipelineError::IO(format!("failed to replace truth-pack db: {err}")))?;
    }

    let mut connection = Connection::open(path)
        .map_err(|err| PipelineError::IO(format!("failed to open truth-pack sqlite: {err}")))?;
    connection
        .execute_batch(
            "
            PRAGMA journal_mode = MEMORY;
            PRAGMA synchronous = NORMAL;
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
            CREATE TABLE semantic_lineage (
                semantic_lineage_id INTEGER PRIMARY KEY AUTOINCREMENT,
                retained_feature_id TEXT NOT NULL,
                field_name TEXT NOT NULL,
                field_value TEXT NOT NULL,
                source_name TEXT NOT NULL,
                source_feature_id TEXT NOT NULL,
                resolution TEXT NOT NULL
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
            ",
        )
        .map_err(|err| {
            PipelineError::IO(format!("failed to initialize truth-pack sqlite: {err}"))
        })?;

    let tx = connection
        .transaction()
        .map_err(|err| PipelineError::IO(format!("failed to begin truth-pack tx: {err}")))?;

    {
        let mut insert_feature = tx
            .prepare(
                "INSERT INTO features (feature_id, feature_kind, canonical_feature_id, is_retained)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .map_err(|err| PipelineError::IO(format!("prepare features insert failed: {err}")))?;
        for row in &pack.features {
            insert_feature
                .execute(params![
                    row.feature_id,
                    row.feature_kind,
                    row.canonical_feature_id,
                    if row.is_retained { 1 } else { 0 },
                ])
                .map_err(|err| PipelineError::IO(format!("insert feature failed: {err}")))?;
        }
    }

    {
        let mut insert_source = tx
            .prepare("INSERT INTO sources (source_name, provider, dataset) VALUES (?1, ?2, ?3)")
            .map_err(|err| PipelineError::IO(format!("prepare sources insert failed: {err}")))?;
        for row in &pack.sources {
            insert_source
                .execute(params![row.source_name, row.provider, row.dataset])
                .map_err(|err| PipelineError::IO(format!("insert source failed: {err}")))?;
        }
    }

    {
        let mut insert_feature_source = tx
            .prepare(
                "INSERT INTO feature_sources (feature_id, source_name, source_feature_id, source_layer)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .map_err(|err| PipelineError::IO(format!("prepare feature_sources insert failed: {err}")))?;
        for row in &pack.feature_sources {
            insert_feature_source
                .execute(params![
                    row.feature_id,
                    row.source_name,
                    row.source_feature_id,
                    row.source_layer,
                ])
                .map_err(|err| PipelineError::IO(format!("insert feature source failed: {err}")))?;
        }
    }

    {
        let mut insert_retained = tx
            .prepare(
                "INSERT INTO retained_semantics (feature_id, field_name, field_value)
                 VALUES (?1, ?2, ?3)",
            )
            .map_err(|err| {
                PipelineError::IO(format!("prepare retained_semantics insert failed: {err}"))
            })?;
        for row in &pack.retained_semantics {
            insert_retained
                .execute(params![row.feature_id, row.field_name, row.field_value])
                .map_err(|err| {
                    PipelineError::IO(format!("insert retained semantic failed: {err}"))
                })?;
        }
    }

    {
        let mut insert_lineage = tx
            .prepare(
                "INSERT INTO semantic_lineage
                 (retained_feature_id, field_name, field_value, source_name, source_feature_id, resolution)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .map_err(|err| {
                PipelineError::IO(format!("prepare semantic_lineage insert failed: {err}"))
            })?;
        for row in &pack.semantic_lineage {
            insert_lineage
                .execute(params![
                    row.retained_feature_id,
                    row.field_name,
                    row.field_value,
                    row.source_name,
                    row.source_feature_id,
                    row.resolution,
                ])
                .map_err(|err| {
                    PipelineError::IO(format!("insert semantic_lineage failed: {err}"))
                })?;
        }
    }

    {
        let mut insert_collapse = tx
            .prepare(
                "INSERT INTO collapses (feature_id, retained_feature_id, collapse_kind, matched_source)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .map_err(|err| PipelineError::IO(format!("prepare collapses insert failed: {err}")))?;
        for row in &pack.collapses {
            insert_collapse
                .execute(params![
                    row.feature_id,
                    row.retained_feature_id,
                    row.collapse_kind,
                    row.matched_source,
                ])
                .map_err(|err| PipelineError::IO(format!("insert collapse failed: {err}")))?;
        }
    }

    {
        let mut insert_dropped = tx
            .prepare(
                "INSERT INTO dropped_semantics
                 (feature_id, field_name, field_value, reason, retained_feature_id)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .map_err(|err| {
                PipelineError::IO(format!("prepare dropped_semantics insert failed: {err}"))
            })?;
        for row in &pack.dropped_semantics {
            insert_dropped
                .execute(params![
                    row.feature_id,
                    row.field_name,
                    row.field_value,
                    row.reason,
                    row.retained_feature_id,
                ])
                .map_err(|err| {
                    PipelineError::IO(format!("insert dropped semantic failed: {err}"))
                })?;
        }
    }

    tx.commit()
        .map_err(|err| PipelineError::IO(format!("commit truth-pack tx failed: {err}")))?;
    Ok(())
}

pub fn write_source_truth_pack_summary(
    summary: &SourceTruthPackSummary,
    path: &Path,
) -> PipelineResult<()> {
    let text = serde_json::to_string_pretty(summary).map_err(|err| {
        PipelineError::Serialization(format!("failed to serialize truth-pack summary: {err}"))
    })?;
    fs::write(path, text)
        .map_err(|err| PipelineError::IO(format!("failed to write truth-pack summary: {err}")))?;
    Ok(())
}
