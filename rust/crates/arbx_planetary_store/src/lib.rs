use std::fs;
use std::path::Path;

use arbx_roblox_export::{stream_manifest_sqlite_all, StoredChunkRecord, StoredManifestMeta};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::Value;

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

fn read_scene_meta(connection: &Connection, scene_id: &str) -> PlanetaryStoreResult<StoredManifestMeta> {
    let meta = connection
        .query_row(
            "
            SELECT
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
            FROM scenes
            WHERE scene_id = ?1
            ",
            params![scene_id],
            |row| {
                let notes_json: String = row.get(11)?;
                let notes = serde_json::from_str::<Vec<String>>(&notes_json).map_err(|err| {
                    rusqlite::Error::FromSqlConversionFailure(
                        11,
                        rusqlite::types::Type::Text,
                        Box::new(err),
                    )
                })?;
                Ok(StoredManifestMeta {
                    schema_version: row.get(0)?,
                    world_name: row.get(1)?,
                    generator: row.get(2)?,
                    source: row.get(3)?,
                    meters_per_stud: row.get(4)?,
                    chunk_size_studs: row.get(5)?,
                    bbox: arbx_geo::BoundingBox::new(row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?),
                    total_features: row.get::<_, i64>(10)? as usize,
                    notes,
                })
            },
        )
        .optional()
        .map_err(|err| -> Box<dyn std::error::Error + Send + Sync> { Box::new(err) })?;
    meta.ok_or_else(|| format!("scene {} is not present in planetary store", scene_id).into())
}

fn bbox_intersects(
    min_lat_a: f64,
    min_lon_a: f64,
    max_lat_a: f64,
    max_lon_a: f64,
    min_lat_b: f64,
    min_lon_b: f64,
    max_lat_b: f64,
    max_lon_b: f64,
) -> bool {
    min_lat_a <= max_lat_b
        && max_lat_a >= min_lat_b
        && min_lon_a <= max_lon_b
        && max_lon_a >= min_lon_b
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

fn infer_scene_id_from_world_name(
    manifest_json_path: &Path,
    world_name: &str,
    explicit: Option<&str>,
) -> String {
    if let Some(scene_id) = explicit {
        return sanitize_scene_id(scene_id);
    }
    if let Some(stem) = manifest_json_path
        .file_stem()
        .and_then(|value| value.to_str())
    {
        return sanitize_scene_id(stem);
    }
    sanitize_scene_id(world_name)
}

fn get_required_object<'a>(value: &'a Value, key: &str) -> PlanetaryStoreResult<&'a serde_json::Map<String, Value>> {
    value.get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("manifest JSON is missing object field {}", key).into())
}

fn get_required_array<'a>(value: &'a Value, key: &str) -> PlanetaryStoreResult<&'a Vec<Value>> {
    value.get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("manifest JSON is missing array field {}", key).into())
}

fn get_required_string(value: &Value, key: &str) -> PlanetaryStoreResult<String> {
    value.get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| format!("manifest JSON is missing string field {}", key).into())
}

fn get_required_f64(value: &Value, key: &str) -> PlanetaryStoreResult<f64> {
    value.get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("manifest JSON is missing numeric field {}", key).into())
}

fn get_required_usize(value: &Value, key: &str) -> PlanetaryStoreResult<usize> {
    value.get(key)
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .ok_or_else(|| format!("manifest JSON is missing unsigned integer field {}", key).into())
}

fn build_meta_from_manifest_json(value: &Value) -> PlanetaryStoreResult<StoredManifestMeta> {
    let meta = get_required_object(value, "meta")?;
    let bbox = meta
        .get("bbox")
        .and_then(Value::as_object)
        .ok_or_else(|| "manifest JSON is missing object field meta.bbox".to_string())?;
    let notes = meta
        .get("notes")
        .and_then(Value::as_array)
        .map(|items| {
            items.iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(StoredManifestMeta {
        schema_version: get_required_string(value, "schemaVersion")?,
        world_name: meta
            .get("worldName")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .ok_or_else(|| "manifest JSON is missing string field meta.worldName".to_string())?,
        generator: meta
            .get("generator")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .ok_or_else(|| "manifest JSON is missing string field meta.generator".to_string())?,
        source: meta
            .get("source")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .ok_or_else(|| "manifest JSON is missing string field meta.source".to_string())?,
        meters_per_stud: meta
            .get("metersPerStud")
            .and_then(Value::as_f64)
            .ok_or_else(|| "manifest JSON is missing numeric field meta.metersPerStud".to_string())?,
        chunk_size_studs: meta
            .get("chunkSizeStuds")
            .and_then(Value::as_i64)
            .map(|value| value as i32)
            .ok_or_else(|| "manifest JSON is missing integer field meta.chunkSizeStuds".to_string())?,
        bbox: arbx_geo::BoundingBox::new(
            bbox.get("minLat")
                .and_then(Value::as_f64)
                .ok_or_else(|| "manifest JSON is missing numeric field meta.bbox.minLat".to_string())?,
            bbox.get("minLon")
                .and_then(Value::as_f64)
                .ok_or_else(|| "manifest JSON is missing numeric field meta.bbox.minLon".to_string())?,
            bbox.get("maxLat")
                .and_then(Value::as_f64)
                .ok_or_else(|| "manifest JSON is missing numeric field meta.bbox.maxLat".to_string())?,
            bbox.get("maxLon")
                .and_then(Value::as_f64)
                .ok_or_else(|| "manifest JSON is missing numeric field meta.bbox.maxLon".to_string())?,
        ),
        total_features: meta
            .get("totalFeatures")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .ok_or_else(|| "manifest JSON is missing unsigned integer field meta.totalFeatures".to_string())?,
        notes,
    })
}

pub fn ingest_manifest_json(
    planetary_store_path: &Path,
    manifest_json_path: &Path,
    scene_id: Option<&str>,
) -> PlanetaryStoreResult<PlanetarySceneSummary> {
    let manifest_text = fs::read_to_string(manifest_json_path)?;
    let manifest_value: Value = serde_json::from_str(&manifest_text)?;
    let meta = build_meta_from_manifest_json(&manifest_value)?;
    let resolved_scene_id = infer_scene_id_from_world_name(manifest_json_path, &meta.world_name, scene_id);
    let chunk_refs = get_required_array(&manifest_value, "chunkRefs")?;
    let chunks = get_required_array(&manifest_value, "chunks")?;

    let mut connection = open_store(planetary_store_path)?;
    let tx = connection.transaction()?;
    tx.execute(
        "DELETE FROM chunks WHERE scene_id = ?1",
        params![resolved_scene_id.as_str()],
    )?;
    tx.execute(
        "DELETE FROM scenes WHERE scene_id = ?1",
        params![resolved_scene_id.as_str()],
    )?;
    replace_scene_meta(&tx, &resolved_scene_id, manifest_json_path, &meta)?;

    let mut chunk_count = 0usize;
    let mut total_features = 0usize;
    for chunk in chunks {
        let chunk_id = get_required_string(chunk, "id")?;
        let origin = get_required_object(chunk, "originStuds")?;
        let chunk_ref = chunk_refs
            .iter()
            .find(|candidate| candidate.get("id").and_then(Value::as_str) == Some(chunk_id.as_str()))
            .ok_or_else(|| format!("manifest JSON is missing chunkRef for chunk {}", chunk_id))?;
        let subplans_json = serde_json::to_string(
            chunk_ref
                .get("subplans")
                .ok_or_else(|| format!("manifest JSON is missing subplans for chunkRef {}", chunk_id))?,
        )?;
        let chunk_json = serde_json::to_string(chunk)?;
        let record = StoredChunkRecord {
            chunk_id,
            origin_studs: arbx_geo::Vec3::new(
                origin
                    .get("x")
                    .and_then(Value::as_f64)
                    .ok_or_else(|| "manifest JSON is missing numeric field originStuds.x".to_string())?,
                origin
                    .get("y")
                    .and_then(Value::as_f64)
                    .ok_or_else(|| "manifest JSON is missing numeric field originStuds.y".to_string())?,
                origin
                    .get("z")
                    .and_then(Value::as_f64)
                    .ok_or_else(|| "manifest JSON is missing numeric field originStuds.z".to_string())?,
            ),
            feature_count: get_required_usize(chunk_ref, "featureCount")?,
            streaming_cost: get_required_f64(chunk_ref, "streamingCost")?,
            estimated_memory_cost: chunk_ref.get("estimatedMemoryCost").and_then(Value::as_f64),
            partition_version: chunk_ref
                .get("partitionVersion")
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .unwrap_or_default(),
            subplans_json,
            chunk_json,
        };
        total_features += record.feature_count;
        chunk_count += 1;
        insert_chunk(&tx, &resolved_scene_id, record)?;
    }
    tx.commit()?;

    Ok(PlanetarySceneSummary {
        scene_id: resolved_scene_id,
        world_name: meta.world_name,
        chunk_count,
        total_features,
    })
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

pub fn find_scenes_intersecting_geo_bbox(
    path: &Path,
    min_lat: f64,
    min_lon: f64,
    max_lat: f64,
    max_lon: f64,
) -> PlanetaryStoreResult<Vec<PlanetarySceneCatalogEntry>> {
    let connection = open_store(path)?;
    let mut statement = connection.prepare(
        "
        SELECT
            scenes.scene_id,
            scenes.world_name,
            scenes.chunk_size_studs,
            scenes.total_features,
            scenes.manifest_store_path,
            scenes.bbox_min_lat,
            scenes.bbox_min_lon,
            scenes.bbox_max_lat,
            scenes.bbox_max_lon,
            COUNT(chunks.chunk_id) AS chunk_count
        FROM scenes
        LEFT JOIN chunks ON chunks.scene_id = scenes.scene_id
        GROUP BY
            scenes.scene_id,
            scenes.world_name,
            scenes.chunk_size_studs,
            scenes.total_features,
            scenes.manifest_store_path,
            scenes.bbox_min_lat,
            scenes.bbox_min_lon,
            scenes.bbox_max_lat,
            scenes.bbox_max_lon
        ORDER BY scenes.scene_id ASC
        ",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((
            PlanetarySceneCatalogEntry {
                scene_id: row.get(0)?,
                world_name: row.get(1)?,
                chunk_size_studs: row.get(2)?,
                total_features: row.get::<_, i64>(3)? as usize,
                manifest_store_path: row.get(4)?,
                chunk_count: row.get::<_, i64>(9)? as usize,
            },
            (
                row.get::<_, f64>(5)?,
                row.get::<_, f64>(6)?,
                row.get::<_, f64>(7)?,
                row.get::<_, f64>(8)?,
            ),
        ))
    })?;

    let mut scenes = Vec::new();
    for row in rows {
        let (entry, (scene_min_lat, scene_min_lon, scene_max_lat, scene_max_lon)) = row?;
        if bbox_intersects(
            min_lat,
            min_lon,
            max_lat,
            max_lon,
            scene_min_lat,
            scene_min_lon,
            scene_max_lat,
            scene_max_lon,
        ) {
            scenes.push(entry);
        }
    }
    Ok(scenes)
}

pub fn find_scenes_covering_geo_point(
    path: &Path,
    lat: f64,
    lon: f64,
) -> PlanetaryStoreResult<Vec<PlanetarySceneCatalogEntry>> {
    find_scenes_intersecting_geo_bbox(path, lat, lon, lat, lon)
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

pub fn read_scene_manifest_subset(
    path: &Path,
    scene_id: &str,
    min_x: f64,
    min_z: f64,
    max_x: f64,
    max_z: f64,
) -> PlanetaryStoreResult<arbx_roblox_export::StoredManifestSubset> {
    let connection = open_store(path)?;
    let meta = read_scene_meta(&connection, scene_id)?;
    let chunk_size_studs = meta.chunk_size_studs as f64;
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
        params![scene_id, max_x, chunk_size_studs, min_x, max_z, min_z],
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
    Ok(arbx_roblox_export::StoredManifestSubset { meta, chunks })
}

pub fn read_scene_manifest_subset_by_chunk_ids(
    path: &Path,
    scene_id: &str,
    chunk_ids: &[String],
) -> PlanetaryStoreResult<arbx_roblox_export::StoredManifestSubset> {
    let connection = open_store(path)?;
    let meta = read_scene_meta(&connection, scene_id)?;
    let mut chunks = Vec::new();
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
        WHERE scene_id = ?1 AND chunk_id = ?2
        ",
    )?;
    for chunk_id in chunk_ids {
        let row = statement
            .query_row(params![scene_id, chunk_id], |row| {
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
            })
            .optional()?;
        if let Some(chunk) = row {
            chunks.push(chunk);
        }
    }
    Ok(arbx_roblox_export::StoredManifestSubset { meta, chunks })
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

pub fn read_chunks_by_ids(
    path: &Path,
    scene_id: &str,
    chunk_ids: &[String],
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

    let mut chunks = Vec::new();
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
        WHERE scene_id = ?1 AND chunk_id = ?2
        ",
    )?;

    for chunk_id in chunk_ids {
        let row = statement
            .query_row(params![scene_id, chunk_id], |row| {
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
            })
            .optional()?;

        if let Some(chunk) = row {
            chunks.push(chunk);
        }
    }

    Ok(PlanetarySceneSubset {
        scene_id: scene_id.to_string(),
        world_name,
        chunk_size_studs,
        chunks,
    })
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

    #[test]
    fn planetary_store_finds_scenes_intersecting_geo_bbox() {
        let dir = tempdir().unwrap();
        let manifest_a_path = dir.path().join("sample_a.sqlite");
        let manifest_b_path = dir.path().join("sample_b.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let mut manifest_a = build_sample_multi_chunk(1, 1);
        manifest_a.meta.bbox = arbx_geo::BoundingBox::new(30.0, -98.0, 30.5, -97.5);
        let mut manifest_b = build_sample_multi_chunk(1, 1);
        manifest_b.meta.bbox = arbx_geo::BoundingBox::new(40.0, -75.0, 40.5, -74.5);
        write_manifest_sqlite(&manifest_a, &manifest_a_path).unwrap();
        write_manifest_sqlite(&manifest_b, &manifest_b_path).unwrap();

        ingest_manifest_sqlite(&store_path, &manifest_a_path, Some("austin")).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_b_path, Some("philly")).unwrap();

        let scenes = find_scenes_intersecting_geo_bbox(&store_path, 30.1, -97.9, 30.2, -97.8).unwrap();
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].scene_id, "austin");
    }

    #[test]
    fn planetary_store_finds_scenes_covering_geo_point() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let mut manifest = build_sample_multi_chunk(1, 1);
        manifest.meta.bbox = arbx_geo::BoundingBox::new(30.0, -98.0, 30.5, -97.5);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("austin")).unwrap();

        let scenes = find_scenes_covering_geo_point(&store_path, 30.2, -97.8).unwrap();
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].scene_id, "austin");
    }

    #[test]
    fn planetary_store_ingests_manifest_json() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.json");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(2, 1);
        fs::write(&manifest_path, manifest.to_json_pretty()).unwrap();

        let summary = ingest_manifest_json(&store_path, &manifest_path, Some("json_scene")).unwrap();
        assert_eq!(summary.scene_id, "json_scene");
        assert_eq!(summary.chunk_count, 2);
    }

    #[test]
    fn planetary_store_reads_chunks_by_id() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let subset = read_chunks_by_ids(
            &store_path,
            "sample_austin",
            &["0_0".to_string(), "2_0".to_string(), "missing".to_string()],
        )
        .unwrap();
        let ids: Vec<&str> = subset.chunks.iter().map(|chunk| chunk.chunk_id.as_str()).collect();
        assert_eq!(ids, vec!["0_0", "2_0"]);
    }

    #[test]
    fn planetary_store_reads_manifest_subset_by_bbox() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let subset =
            read_scene_manifest_subset(&store_path, "sample_austin", 200.0, 0.0, 500.0, 200.0).unwrap();
        assert_eq!(subset.chunks.len(), 2);
    }

    #[test]
    fn planetary_store_reads_manifest_subset_by_chunk_ids() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let subset = read_scene_manifest_subset_by_chunk_ids(
            &store_path,
            "sample_austin",
            &["0_0".to_string(), "2_0".to_string()],
        )
        .unwrap();
        assert_eq!(subset.chunks.len(), 2);
    }
}
