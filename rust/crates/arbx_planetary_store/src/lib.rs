use std::fs;
use std::path::Path;

use arbx_roblox_export::{stream_manifest_sqlite_all, StoredChunkRecord, StoredManifestMeta};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

pub type PlanetaryStoreResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanetarySceneSummary {
    pub scene_id: String,
    pub world_name: String,
    pub chunk_count: usize,
    pub total_features: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanetarySceneCatalogEntry {
    pub scene_id: String,
    pub world_name: String,
    pub chunk_size_studs: i32,
    pub chunk_count: usize,
    pub total_features: usize,
    pub manifest_store_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanetaryStoreSummary {
    pub scene_count: usize,
    pub chunk_count: usize,
    pub total_features: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlanetarySceneSubset {
    pub scene_id: String,
    pub world_name: String,
    pub chunk_size_studs: i32,
    pub chunks: Vec<StoredChunkRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlanetaryChunkSummary {
    pub chunk_id: String,
    pub origin_x: f64,
    pub origin_z: f64,
    pub feature_count: usize,
    pub streaming_cost: f64,
    pub estimated_memory_cost: Option<f64>,
    pub partition_version: String,
}

fn ensure_parent_dir(path: &Path) -> PlanetaryStoreResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn open_store(path: &Path) -> PlanetaryStoreResult<Connection> {
    ensure_parent_dir(path)?;
    let connection = Connection::open(path)?;
    connection.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        CREATE TABLE IF NOT EXISTS scenes (
            scene_id TEXT PRIMARY KEY,
            manifest_store_path TEXT NOT NULL,
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
        CREATE TABLE IF NOT EXISTS chunks (
            scene_id TEXT NOT NULL,
            chunk_id TEXT NOT NULL,
            origin_x REAL NOT NULL,
            origin_y REAL NOT NULL,
            origin_z REAL NOT NULL,
            feature_count INTEGER NOT NULL,
            streaming_cost REAL NOT NULL,
            estimated_memory_cost REAL,
            partition_version TEXT NOT NULL,
            subplans_json TEXT NOT NULL,
            chunk_json TEXT NOT NULL,
            PRIMARY KEY (scene_id, chunk_id),
            FOREIGN KEY (scene_id) REFERENCES scenes(scene_id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_chunks_scene_origin
            ON chunks(scene_id, origin_x, origin_z);
        ",
    )?;
    Ok(connection)
}

pub fn init_planetary_store(path: &Path) -> PlanetaryStoreResult<()> {
    let _ = open_store(path)?;
    Ok(())
}

fn sanitize_scene_id(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "scene".to_string()
    } else {
        trimmed.to_string()
    }
}

fn infer_scene_id(
    manifest_store_path: &Path,
    meta: &StoredManifestMeta,
    explicit: Option<&str>,
) -> String {
    if let Some(scene_id) = explicit {
        return sanitize_scene_id(scene_id);
    }
    if let Some(stem) = manifest_store_path
        .file_stem()
        .and_then(|value| value.to_str())
    {
        return sanitize_scene_id(stem);
    }
    sanitize_scene_id(&meta.world_name)
}

fn replace_scene_meta(
    connection: &Connection,
    scene_id: &str,
    manifest_store_path: &Path,
    meta: &StoredManifestMeta,
) -> PlanetaryStoreResult<()> {
    let notes_json = serde_json::to_string(&meta.notes)?;
    connection.execute(
        "
        INSERT INTO scenes (
            scene_id,
            manifest_store_path,
            schema_version,
            world_name,
            generator,
            source,
            meters_per_stud,
            chunk_size_studs,
            bbox_min_lat,
            bbox_min_lon,
            bbox_max_lat,
            bbox_max_lon,
            total_features,
            notes_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
        ",
        params![
            scene_id,
            manifest_store_path.display().to_string(),
            meta.schema_version,
            meta.world_name,
            meta.generator,
            meta.source,
            meta.meters_per_stud,
            meta.chunk_size_studs,
            meta.bbox.min.lat,
            meta.bbox.min.lon,
            meta.bbox.max.lat,
            meta.bbox.max.lon,
            meta.total_features as i64,
            notes_json,
        ],
    )?;
    Ok(())
}

fn insert_chunk(
    connection: &Connection,
    scene_id: &str,
    chunk: StoredChunkRecord,
) -> PlanetaryStoreResult<()> {
    connection.execute(
        "
        INSERT INTO chunks (
            scene_id,
            chunk_id,
            origin_x,
            origin_y,
            origin_z,
            feature_count,
            streaming_cost,
            estimated_memory_cost,
            partition_version,
            subplans_json,
            chunk_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ",
        params![
            scene_id,
            chunk.chunk_id,
            chunk.origin_studs.x,
            chunk.origin_studs.y,
            chunk.origin_studs.z,
            chunk.feature_count as i64,
            chunk.streaming_cost,
            chunk.estimated_memory_cost,
            chunk.partition_version,
            chunk.subplans_json,
            chunk.chunk_json,
        ],
    )?;
    Ok(())
}

pub fn ingest_manifest_sqlite(
    planetary_store_path: &Path,
    manifest_store_path: &Path,
    scene_id: Option<&str>,
) -> PlanetaryStoreResult<PlanetarySceneSummary> {
    let mut connection = open_store(planetary_store_path)?;
    let meta = stream_manifest_sqlite_all(manifest_store_path, |_| Ok(()))?;
    let resolved_scene_id = infer_scene_id(manifest_store_path, &meta, scene_id);
    let tx = connection.transaction()?;

    tx.execute(
        "DELETE FROM chunks WHERE scene_id = ?1",
        params![resolved_scene_id.as_str()],
    )?;
    tx.execute(
        "DELETE FROM scenes WHERE scene_id = ?1",
        params![resolved_scene_id.as_str()],
    )?;
    replace_scene_meta(&tx, &resolved_scene_id, manifest_store_path, &meta)?;

    let mut chunk_count = 0usize;
    let mut total_features = 0usize;
    stream_manifest_sqlite_all(manifest_store_path, |chunk| {
        total_features += chunk.feature_count;
        chunk_count += 1;
        insert_chunk(&tx, &resolved_scene_id, chunk)
    })?;

    tx.commit()?;

    Ok(PlanetarySceneSummary {
        scene_id: resolved_scene_id,
        world_name: meta.world_name,
        chunk_count,
        total_features,
    })
}

pub fn summarize_planetary_store(path: &Path) -> PlanetaryStoreResult<PlanetaryStoreSummary> {
    let connection = open_store(path)?;
    let scene_count = connection.query_row("SELECT COUNT(*) FROM scenes", [], |row| {
        row.get::<_, i64>(0)
    })? as usize;
    let (chunk_count, total_features) = connection.query_row(
        "SELECT COUNT(*), COALESCE(SUM(feature_count), 0) FROM chunks",
        [],
        |row| {
            Ok((
                row.get::<_, i64>(0)? as usize,
                row.get::<_, i64>(1)? as usize,
            ))
        },
    )?;
    Ok(PlanetaryStoreSummary {
        scene_count,
        chunk_count,
        total_features,
    })
}

pub fn list_scenes(path: &Path) -> PlanetaryStoreResult<Vec<PlanetarySceneCatalogEntry>> {
    let connection = open_store(path)?;
    let mut statement = connection.prepare(
        "
        SELECT
            scenes.scene_id,
            scenes.world_name,
            scenes.chunk_size_studs,
            scenes.total_features,
            scenes.manifest_store_path,
            COUNT(chunks.chunk_id) AS chunk_count
        FROM scenes
        LEFT JOIN chunks ON chunks.scene_id = scenes.scene_id
        GROUP BY
            scenes.scene_id,
            scenes.world_name,
            scenes.chunk_size_studs,
            scenes.total_features,
            scenes.manifest_store_path
        ORDER BY scenes.scene_id ASC
        ",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(PlanetarySceneCatalogEntry {
            scene_id: row.get(0)?,
            world_name: row.get(1)?,
            chunk_size_studs: row.get(2)?,
            total_features: row.get::<_, i64>(3)? as usize,
            manifest_store_path: row.get(4)?,
            chunk_count: row.get::<_, i64>(5)? as usize,
        })
    })?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    Ok(entries)
}

pub fn read_scene_catalog_entry(
    path: &Path,
    scene_id: &str,
) -> PlanetaryStoreResult<Option<PlanetarySceneCatalogEntry>> {
    let connection = open_store(path)?;
    connection
        .query_row(
            "
            SELECT
                scenes.scene_id,
                scenes.world_name,
                scenes.chunk_size_studs,
                scenes.total_features,
                scenes.manifest_store_path,
                COUNT(chunks.chunk_id) AS chunk_count
            FROM scenes
            LEFT JOIN chunks ON chunks.scene_id = scenes.scene_id
            WHERE scenes.scene_id = ?1
            GROUP BY
                scenes.scene_id,
                scenes.world_name,
                scenes.chunk_size_studs,
                scenes.total_features,
                scenes.manifest_store_path
            ",
            params![scene_id],
            |row| {
                Ok(PlanetarySceneCatalogEntry {
                    scene_id: row.get(0)?,
                    world_name: row.get(1)?,
                    chunk_size_studs: row.get(2)?,
                    total_features: row.get::<_, i64>(3)? as usize,
                    manifest_store_path: row.get(4)?,
                    chunk_count: row.get::<_, i64>(5)? as usize,
                })
            },
        )
        .optional()
        .map_err(Into::into)
}

pub fn scene_exists(path: &Path, scene_id: &str) -> PlanetaryStoreResult<bool> {
    let connection = open_store(path)?;
    let exists = connection
        .query_row(
            "SELECT 1 FROM scenes WHERE scene_id = ?1 LIMIT 1",
            params![scene_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    Ok(exists)
}

pub fn read_scene_chunk_subset(
    path: &Path,
    scene_id: &str,
    min_x: f64,
    min_z: f64,
    max_x: f64,
    max_z: f64,
) -> PlanetaryStoreResult<PlanetarySceneSubset> {
    let connection = open_store(path)?;
    let (world_name, chunk_size_studs) = connection
        .query_row(
            "SELECT world_name, chunk_size_studs FROM scenes WHERE scene_id = ?1",
            params![scene_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
        )
        .optional()?
        .ok_or_else(|| format!("scene {} is not present in planetary store", scene_id))?;

    let mut statement = connection.prepare(
        "
        SELECT
            chunk_id,
            origin_x,
            origin_y,
            origin_z,
            feature_count,
            streaming_cost,
            estimated_memory_cost,
            partition_version,
            subplans_json,
            chunk_json
        FROM chunks
        WHERE scene_id = ?1
          AND origin_x <= ?2
          AND origin_x + ?3 >= ?4
          AND origin_z <= ?5
          AND origin_z + ?3 >= ?6
        ORDER BY chunk_id ASC
        ",
    )?;
    let rows = statement.query_map(
        params![
            scene_id,
            max_x,
            chunk_size_studs as f64,
            min_x,
            max_z,
            min_z
        ],
        |row| {
            Ok(StoredChunkRecord {
                chunk_id: row.get(0)?,
                origin_studs: arbx_geo::Vec3::new(row.get(1)?, row.get(2)?, row.get(3)?),
                feature_count: row.get::<_, i64>(4)? as usize,
                streaming_cost: row.get(5)?,
                estimated_memory_cost: row.get(6)?,
                partition_version: row.get(7)?,
                subplans_json: row.get(8)?,
                chunk_json: row.get(9)?,
            })
        },
    )?;

    let mut chunks = Vec::new();
    for row in rows {
        chunks.push(row?);
    }

    Ok(PlanetarySceneSubset {
        scene_id: scene_id.to_string(),
        world_name,
        chunk_size_studs,
        chunks,
    })
}

pub fn read_scene_chunk_summary_subset(
    path: &Path,
    scene_id: &str,
    min_x: f64,
    min_z: f64,
    max_x: f64,
    max_z: f64,
    limit: Option<usize>,
) -> PlanetaryStoreResult<Vec<PlanetaryChunkSummary>> {
    let connection = open_store(path)?;
    let chunk_size_studs: i32 = connection
        .query_row(
            "SELECT chunk_size_studs FROM scenes WHERE scene_id = ?1",
            params![scene_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| format!("scene {} is not present in planetary store", scene_id))?;

    let mut sql = String::from(
        "
        SELECT
            chunk_id,
            origin_x,
            origin_z,
            feature_count,
            streaming_cost,
            estimated_memory_cost,
            partition_version
        FROM chunks
        WHERE scene_id = ?1
          AND origin_x <= ?2
          AND origin_x + ?3 >= ?4
          AND origin_z <= ?5
          AND origin_z + ?3 >= ?6
        ORDER BY origin_z ASC, origin_x ASC
        ",
    );
    if let Some(limit) = limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }

    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map(
        params![scene_id, max_x, chunk_size_studs as f64, min_x, max_z, min_z],
        |row| {
            Ok(PlanetaryChunkSummary {
                chunk_id: row.get(0)?,
                origin_x: row.get(1)?,
                origin_z: row.get(2)?,
                feature_count: row.get::<_, i64>(3)? as usize,
                streaming_cost: row.get(4)?,
                estimated_memory_cost: row.get(5)?,
                partition_version: row.get(6)?,
            })
        },
    )?;

    let mut chunks = Vec::new();
    for row in rows {
        chunks.push(row?);
    }
    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arbx_roblox_export::{build_sample_multi_chunk, write_manifest_sqlite};
    use tempfile::tempdir;

    #[test]
    fn planetary_store_initializes_and_summarizes_empty_store() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("planetary.sqlite");

        init_planetary_store(&store_path).unwrap();
        let summary = summarize_planetary_store(&store_path).unwrap();

        assert_eq!(
            summary,
            PlanetaryStoreSummary {
                scene_count: 0,
                chunk_count: 0,
                total_features: 0,
            }
        );
    }

    #[test]
    fn planetary_store_ingests_manifest_sqlite() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(2, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        let ingested =
            ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();
        let summary = summarize_planetary_store(&store_path).unwrap();

        assert_eq!(ingested.scene_id, "sample_austin");
        assert_eq!(ingested.chunk_count, 2);
        assert!(scene_exists(&store_path, "sample_austin").unwrap());
        assert_eq!(summary.scene_count, 1);
        assert_eq!(summary.chunk_count, 2);
        assert_eq!(summary.total_features, ingested.total_features);
    }

    #[test]
    fn planetary_store_replaces_existing_scene_on_reingest() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(1, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let summary = summarize_planetary_store(&store_path).unwrap();
        assert_eq!(summary.scene_count, 1);
        assert_eq!(summary.chunk_count, 1);
    }

    #[test]
    fn planetary_store_reads_scene_chunk_subset_by_stud_bbox() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let subset =
            read_scene_chunk_subset(&store_path, "sample_austin", 200.0, 0.0, 500.0, 200.0)
                .unwrap();
        let ids: Vec<&str> = subset
            .chunks
            .iter()
            .map(|chunk| chunk.chunk_id.as_str())
            .collect();

        assert_eq!(subset.chunk_size_studs, 256);
        assert_eq!(ids, vec!["0_0", "1_0"]);
    }

    #[test]
    fn planetary_store_lists_scenes() {
        let dir = tempdir().unwrap();
        let manifest_a_path = dir.path().join("sample_a.sqlite");
        let manifest_b_path = dir.path().join("sample_b.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        write_manifest_sqlite(&build_sample_multi_chunk(1, 1), &manifest_a_path).unwrap();
        write_manifest_sqlite(&build_sample_multi_chunk(2, 1), &manifest_b_path).unwrap();

        ingest_manifest_sqlite(&store_path, &manifest_a_path, Some("austin_a")).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_b_path, Some("austin_b")).unwrap();

        let scenes = list_scenes(&store_path).unwrap();
        assert_eq!(scenes.len(), 2);
        assert_eq!(scenes[0].scene_id, "austin_a");
        assert_eq!(scenes[1].scene_id, "austin_b");
        assert_eq!(scenes[0].chunk_count, 1);
        assert_eq!(scenes[1].chunk_count, 2);
    }

    #[test]
    fn planetary_store_reads_chunk_summary_subset_by_stud_bbox() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let chunks = read_scene_chunk_summary_subset(
            &store_path,
            "sample_austin",
            200.0,
            0.0,
            500.0,
            200.0,
            Some(1),
        )
        .unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_id, "0_0");
    }
}
