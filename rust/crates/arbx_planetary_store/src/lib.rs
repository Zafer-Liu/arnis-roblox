use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use arbx_geo::{LatLon, Mercator};
use arbx_pipeline::SourceTruthPackSummary;
use arbx_roblox_export::{stream_manifest_sqlite_all, StoredChunkRecord, StoredManifestMeta};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
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
    pub truth_pack_scene: Option<String>,
    pub truth_pack_feature_count: Option<usize>,
    pub truth_pack_retained_semantic_count: Option<usize>,
    pub truth_pack_semantic_lineage_count: Option<usize>,
    pub truth_pack_dropped_semantic_count: Option<usize>,
    pub truth_pack_collapse_count: Option<usize>,
    pub truth_pack_source_counts: Option<BTreeMap<String, usize>>,
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
    pub has_terrain: bool,
    pub road_count: usize,
    pub rail_count: usize,
    pub building_count: usize,
    pub water_count: usize,
    pub prop_count: usize,
    pub landuse_count: usize,
    pub barrier_count: usize,
    pub center_distance_sq: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlanetaryDeliveryWindow {
    pub scene: PlanetarySceneCatalogEntry,
    pub focus_lat: f64,
    pub focus_lon: f64,
    pub focus_x: f64,
    pub focus_z: f64,
    pub radius_studs: f64,
    pub chunk_count: usize,
    pub total_feature_count: usize,
    pub total_streaming_cost: f64,
    pub total_estimated_memory_cost: f64,
    pub chunks: Vec<PlanetaryChunkSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanetaryDeliveryPlan {
    pub scene_id: String,
    pub world_name: String,
    pub manifest_store_path: String,
    pub selection_mode: String,
    pub focus_lat: Option<f64>,
    pub focus_lon: Option<f64>,
    pub focus_x: Option<f64>,
    pub focus_z: Option<f64>,
    pub radius_studs: Option<f64>,
    pub chunk_count: usize,
    pub total_feature_count: usize,
    pub total_streaming_cost: f64,
    pub total_estimated_memory_cost: f64,
    pub chunk_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct SceneGeoCandidate {
    entry: PlanetarySceneCatalogEntry,
    bbox: arbx_geo::BoundingBox,
    meters_per_stud: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct ChunkPayloadSummary {
    has_terrain: bool,
    road_count: usize,
    rail_count: usize,
    building_count: usize,
    water_count: usize,
    prop_count: usize,
    landuse_count: usize,
    barrier_count: usize,
}

fn read_scene_meta(
    connection: &Connection,
    scene_id: &str,
) -> PlanetaryStoreResult<StoredManifestMeta> {
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
                    bbox: arbx_geo::BoundingBox::new(
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                    ),
                    total_features: row.get::<_, i64>(10)? as usize,
                    notes,
                })
            },
        )
        .optional()
        .map_err(|err| -> Box<dyn std::error::Error + Send + Sync> { Box::new(err) })?;
    meta.ok_or_else(|| format!("scene {} is not present in planetary store", scene_id).into())
}

fn decode_truth_pack_source_counts(
    value: Option<String>,
) -> PlanetaryStoreResult<Option<BTreeMap<String, usize>>> {
    match value {
        Some(text) => Ok(Some(serde_json::from_str::<BTreeMap<String, usize>>(
            &text,
        )?)),
        None => Ok(None),
    }
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

fn bbox_area_degrees(bbox: arbx_geo::BoundingBox) -> f64 {
    bbox.width_degrees() * bbox.height_degrees()
}

fn scene_truth_score(entry: &PlanetarySceneCatalogEntry) -> usize {
    entry.truth_pack_feature_count.unwrap_or(0)
        + entry.truth_pack_retained_semantic_count.unwrap_or(0)
        + entry.truth_pack_semantic_lineage_count.unwrap_or(0)
}

fn summarize_delivery_chunks(chunks: &[PlanetaryChunkSummary]) -> (usize, usize, f64, f64) {
    let mut total_feature_count = 0usize;
    let mut total_streaming_cost = 0.0f64;
    let mut total_estimated_memory_cost = 0.0f64;
    for chunk in chunks {
        total_feature_count += chunk.feature_count;
        total_streaming_cost += chunk.streaming_cost;
        total_estimated_memory_cost += chunk.estimated_memory_cost.unwrap_or(chunk.streaming_cost);
    }
    (
        chunks.len(),
        total_feature_count,
        total_streaming_cost,
        total_estimated_memory_cost,
    )
}

fn delivery_plan_from_window(
    selection_mode: &str,
    window: &PlanetaryDeliveryWindow,
) -> PlanetaryDeliveryPlan {
    PlanetaryDeliveryPlan {
        scene_id: window.scene.scene_id.clone(),
        world_name: window.scene.world_name.clone(),
        manifest_store_path: window.scene.manifest_store_path.clone(),
        selection_mode: selection_mode.to_string(),
        focus_lat: Some(window.focus_lat),
        focus_lon: Some(window.focus_lon),
        focus_x: Some(window.focus_x),
        focus_z: Some(window.focus_z),
        radius_studs: Some(window.radius_studs),
        chunk_count: window.chunk_count,
        total_feature_count: window.total_feature_count,
        total_streaming_cost: window.total_streaming_cost,
        total_estimated_memory_cost: window.total_estimated_memory_cost,
        chunk_ids: window
            .chunks
            .iter()
            .map(|chunk| chunk.chunk_id.clone())
            .collect(),
    }
}

fn apply_chunk_summary_constraints(
    chunks: Vec<PlanetaryChunkSummary>,
    limit: Option<usize>,
    max_streaming_cost: Option<f64>,
    max_estimated_memory_cost: Option<f64>,
) -> Vec<PlanetaryChunkSummary> {
    let mut selected = Vec::new();
    let mut running_streaming_cost = 0.0f64;
    let mut running_estimated_memory_cost = 0.0f64;
    for chunk in chunks {
        if let Some(limit) = limit {
            if selected.len() >= limit {
                break;
            }
        }
        let next_streaming_cost = running_streaming_cost + chunk.streaming_cost;
        if let Some(max_streaming_cost) = max_streaming_cost {
            if next_streaming_cost > max_streaming_cost {
                break;
            }
        }
        let next_estimated_memory_cost = running_estimated_memory_cost
            + chunk.estimated_memory_cost.unwrap_or(chunk.streaming_cost);
        if let Some(max_estimated_memory_cost) = max_estimated_memory_cost {
            if next_estimated_memory_cost > max_estimated_memory_cost {
                break;
            }
        }
        running_streaming_cost = next_streaming_cost;
        running_estimated_memory_cost = next_estimated_memory_cost;
        selected.push(chunk);
    }
    selected
}

fn compare_scene_priority(
    left_entry: &PlanetarySceneCatalogEntry,
    left_bbox: arbx_geo::BoundingBox,
    right_entry: &PlanetarySceneCatalogEntry,
    right_bbox: arbx_geo::BoundingBox,
) -> std::cmp::Ordering {
    scene_truth_score(right_entry)
        .cmp(&scene_truth_score(left_entry))
        .then_with(|| {
            bbox_area_degrees(left_bbox)
                .partial_cmp(&bbox_area_degrees(right_bbox))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .then_with(|| left_entry.scene_id.cmp(&right_entry.scene_id))
}

fn tile_x_to_lon(x: u32, zoom: u8) -> f64 {
    x as f64 / 2f64.powi(zoom as i32) * 360.0 - 180.0
}

fn tile_y_to_lat(y: u32, zoom: u8) -> f64 {
    let n = std::f64::consts::PI - (2.0 * std::f64::consts::PI * y as f64) / 2f64.powi(zoom as i32);
    n.sinh().atan().to_degrees()
}

fn slippy_tile_bbox(zoom: u8, x: u32, y: u32) -> arbx_geo::BoundingBox {
    let min_lon = tile_x_to_lon(x, zoom);
    let max_lon = tile_x_to_lon(x + 1, zoom);
    let max_lat = tile_y_to_lat(y, zoom);
    let min_lat = tile_y_to_lat(y + 1, zoom);
    arbx_geo::BoundingBox::new(min_lat, min_lon, max_lat, max_lon)
}

fn project_geo_bbox_to_local_bounds(
    bbox: arbx_geo::BoundingBox,
    center: LatLon,
    meters_per_stud: f64,
) -> (f64, f64, f64, f64) {
    let corners = [
        Mercator::project(
            LatLon::new(bbox.min.lat, bbox.min.lon),
            center,
            meters_per_stud,
        ),
        Mercator::project(
            LatLon::new(bbox.min.lat, bbox.max.lon),
            center,
            meters_per_stud,
        ),
        Mercator::project(
            LatLon::new(bbox.max.lat, bbox.min.lon),
            center,
            meters_per_stud,
        ),
        Mercator::project(
            LatLon::new(bbox.max.lat, bbox.max.lon),
            center,
            meters_per_stud,
        ),
    ];
    let mut min_x = f64::INFINITY;
    let mut min_z = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_z = f64::NEG_INFINITY;
    for corner in corners {
        min_x = min_x.min(corner.x);
        min_z = min_z.min(corner.z);
        max_x = max_x.max(corner.x);
        max_z = max_z.max(corner.z);
    }
    (min_x, min_z, max_x, max_z)
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
            notes_json TEXT NOT NULL,
            truth_pack_scene TEXT,
            truth_pack_feature_count INTEGER,
            truth_pack_retained_semantic_count INTEGER,
            truth_pack_semantic_lineage_count INTEGER,
            truth_pack_dropped_semantic_count INTEGER,
            truth_pack_collapse_count INTEGER,
            truth_pack_source_counts_json TEXT
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
            has_terrain INTEGER NOT NULL DEFAULT 0,
            road_count INTEGER NOT NULL DEFAULT 0,
            rail_count INTEGER NOT NULL DEFAULT 0,
            building_count INTEGER NOT NULL DEFAULT 0,
            water_count INTEGER NOT NULL DEFAULT 0,
            prop_count INTEGER NOT NULL DEFAULT 0,
            landuse_count INTEGER NOT NULL DEFAULT 0,
            barrier_count INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (scene_id, chunk_id),
            FOREIGN KEY (scene_id) REFERENCES scenes(scene_id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_chunks_scene_origin
            ON chunks(scene_id, origin_x, origin_z);
        CREATE INDEX IF NOT EXISTS idx_scenes_geo_bbox
            ON scenes(bbox_min_lat, bbox_min_lon, bbox_max_lat, bbox_max_lon);
        CREATE INDEX IF NOT EXISTS idx_chunks_scene_building_origin
            ON chunks(scene_id, building_count, origin_x, origin_z);
        CREATE INDEX IF NOT EXISTS idx_chunks_scene_terrain_origin
            ON chunks(scene_id, has_terrain, origin_x, origin_z);
        ",
    )?;
    ensure_scene_truth_pack_columns(&connection)?;
    ensure_chunk_payload_columns(&connection)?;
    Ok(connection)
}

fn ensure_scene_truth_pack_columns(connection: &Connection) -> PlanetaryStoreResult<()> {
    let mut existing = std::collections::BTreeSet::new();
    let mut statement = connection.prepare("PRAGMA table_info(scenes)")?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    for row in rows {
        existing.insert(row?);
    }

    for (column, sql_type) in [
        ("truth_pack_scene", "TEXT"),
        ("truth_pack_feature_count", "INTEGER"),
        ("truth_pack_retained_semantic_count", "INTEGER"),
        ("truth_pack_semantic_lineage_count", "INTEGER"),
        ("truth_pack_dropped_semantic_count", "INTEGER"),
        ("truth_pack_collapse_count", "INTEGER"),
        ("truth_pack_source_counts_json", "TEXT"),
    ] {
        if !existing.contains(column) {
            connection.execute(
                &format!("ALTER TABLE scenes ADD COLUMN {column} {sql_type}"),
                [],
            )?;
        }
    }
    Ok(())
}

fn ensure_chunk_payload_columns(connection: &Connection) -> PlanetaryStoreResult<()> {
    let mut existing = std::collections::BTreeSet::new();
    let mut statement = connection.prepare("PRAGMA table_info(chunks)")?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    for row in rows {
        existing.insert(row?);
    }

    for (column, sql_type) in [
        ("has_terrain", "INTEGER NOT NULL DEFAULT 0"),
        ("road_count", "INTEGER NOT NULL DEFAULT 0"),
        ("rail_count", "INTEGER NOT NULL DEFAULT 0"),
        ("building_count", "INTEGER NOT NULL DEFAULT 0"),
        ("water_count", "INTEGER NOT NULL DEFAULT 0"),
        ("prop_count", "INTEGER NOT NULL DEFAULT 0"),
        ("landuse_count", "INTEGER NOT NULL DEFAULT 0"),
        ("barrier_count", "INTEGER NOT NULL DEFAULT 0"),
    ] {
        if !existing.contains(column) {
            connection.execute(
                &format!("ALTER TABLE chunks ADD COLUMN {column} {sql_type}"),
                [],
            )?;
        }
    }

    Ok(())
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

fn get_required_object<'a>(
    value: &'a Value,
    key: &str,
) -> PlanetaryStoreResult<&'a serde_json::Map<String, Value>> {
    value
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("manifest JSON is missing object field {}", key).into())
}

fn get_required_array<'a>(value: &'a Value, key: &str) -> PlanetaryStoreResult<&'a Vec<Value>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("manifest JSON is missing array field {}", key).into())
}

fn get_required_string(value: &Value, key: &str) -> PlanetaryStoreResult<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| format!("manifest JSON is missing string field {}", key).into())
}

fn get_required_f64(value: &Value, key: &str) -> PlanetaryStoreResult<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("manifest JSON is missing numeric field {}", key).into())
}

fn get_required_usize(value: &Value, key: &str) -> PlanetaryStoreResult<usize> {
    value
        .get(key)
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
            items
                .iter()
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
            .ok_or_else(|| {
                "manifest JSON is missing numeric field meta.metersPerStud".to_string()
            })?,
        chunk_size_studs: meta
            .get("chunkSizeStuds")
            .and_then(Value::as_i64)
            .map(|value| value as i32)
            .ok_or_else(|| {
                "manifest JSON is missing integer field meta.chunkSizeStuds".to_string()
            })?,
        bbox: arbx_geo::BoundingBox::new(
            bbox.get("minLat").and_then(Value::as_f64).ok_or_else(|| {
                "manifest JSON is missing numeric field meta.bbox.minLat".to_string()
            })?,
            bbox.get("minLon").and_then(Value::as_f64).ok_or_else(|| {
                "manifest JSON is missing numeric field meta.bbox.minLon".to_string()
            })?,
            bbox.get("maxLat").and_then(Value::as_f64).ok_or_else(|| {
                "manifest JSON is missing numeric field meta.bbox.maxLat".to_string()
            })?,
            bbox.get("maxLon").and_then(Value::as_f64).ok_or_else(|| {
                "manifest JSON is missing numeric field meta.bbox.maxLon".to_string()
            })?,
        ),
        total_features: meta
            .get("totalFeatures")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .ok_or_else(|| {
                "manifest JSON is missing unsigned integer field meta.totalFeatures".to_string()
            })?,
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
    let resolved_scene_id =
        infer_scene_id_from_world_name(manifest_json_path, &meta.world_name, scene_id);
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
            .find(|candidate| {
                candidate.get("id").and_then(Value::as_str) == Some(chunk_id.as_str())
            })
            .ok_or_else(|| format!("manifest JSON is missing chunkRef for chunk {}", chunk_id))?;
        let subplans_json = serde_json::to_string(chunk_ref.get("subplans").ok_or_else(|| {
            format!(
                "manifest JSON is missing subplans for chunkRef {}",
                chunk_id
            )
        })?)?;
        let chunk_json = serde_json::to_string(chunk)?;
        let record = StoredChunkRecord {
            chunk_id,
            origin_studs: arbx_geo::Vec3::new(
                origin.get("x").and_then(Value::as_f64).ok_or_else(|| {
                    "manifest JSON is missing numeric field originStuds.x".to_string()
                })?,
                origin.get("y").and_then(Value::as_f64).ok_or_else(|| {
                    "manifest JSON is missing numeric field originStuds.y".to_string()
                })?,
                origin.get("z").and_then(Value::as_f64).ok_or_else(|| {
                    "manifest JSON is missing numeric field originStuds.z".to_string()
                })?,
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
    let payload = summarize_chunk_payload(&chunk.chunk_json)?;
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
            chunk_json,
            has_terrain,
            road_count,
            rail_count,
            building_count,
            water_count,
            prop_count,
            landuse_count,
            barrier_count
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
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
            if payload.has_terrain { 1 } else { 0 },
            payload.road_count as i64,
            payload.rail_count as i64,
            payload.building_count as i64,
            payload.water_count as i64,
            payload.prop_count as i64,
            payload.landuse_count as i64,
            payload.barrier_count as i64,
        ],
    )?;
    Ok(())
}

fn summarize_chunk_payload(chunk_json: &str) -> PlanetaryStoreResult<ChunkPayloadSummary> {
    let value: Value = serde_json::from_str(chunk_json)?;
    let object = value
        .as_object()
        .ok_or_else(|| "chunk payload is not a JSON object".to_string())?;
    let count = |key: &str| {
        object
            .get(key)
            .and_then(Value::as_array)
            .map_or(0usize, Vec::len)
    };
    Ok(ChunkPayloadSummary {
        has_terrain: object.get("terrain").is_some() && !object.get("terrain").unwrap().is_null(),
        road_count: count("roads"),
        rail_count: count("rails"),
        building_count: count("buildings"),
        water_count: count("water"),
        prop_count: count("props"),
        landuse_count: count("landuse"),
        barrier_count: count("barriers"),
    })
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
            scenes.truth_pack_scene,
            scenes.truth_pack_feature_count,
            scenes.truth_pack_retained_semantic_count,
            scenes.truth_pack_semantic_lineage_count,
            scenes.truth_pack_dropped_semantic_count,
            scenes.truth_pack_collapse_count,
            scenes.truth_pack_source_counts_json,
            COUNT(chunks.chunk_id) AS chunk_count
        FROM scenes
        LEFT JOIN chunks ON chunks.scene_id = scenes.scene_id
        GROUP BY
            scenes.scene_id,
            scenes.world_name,
            scenes.chunk_size_studs,
            scenes.total_features,
            scenes.manifest_store_path,
            scenes.truth_pack_scene,
            scenes.truth_pack_feature_count,
            scenes.truth_pack_retained_semantic_count,
            scenes.truth_pack_semantic_lineage_count,
            scenes.truth_pack_dropped_semantic_count,
            scenes.truth_pack_collapse_count,
            scenes.truth_pack_source_counts_json
        ORDER BY scenes.scene_id ASC
        ",
    )?;
    let rows = statement.query_map([], |row| {
        let source_counts_json: Option<String> = row.get(11)?;
        Ok(PlanetarySceneCatalogEntry {
            scene_id: row.get(0)?,
            world_name: row.get(1)?,
            chunk_size_studs: row.get(2)?,
            total_features: row.get::<_, i64>(3)? as usize,
            manifest_store_path: row.get(4)?,
            truth_pack_scene: row.get(5)?,
            truth_pack_feature_count: row.get::<_, Option<i64>>(6)?.map(|value| value as usize),
            truth_pack_retained_semantic_count: row
                .get::<_, Option<i64>>(7)?
                .map(|value| value as usize),
            truth_pack_semantic_lineage_count: row
                .get::<_, Option<i64>>(8)?
                .map(|value| value as usize),
            truth_pack_dropped_semantic_count: row
                .get::<_, Option<i64>>(9)?
                .map(|value| value as usize),
            truth_pack_collapse_count: row.get::<_, Option<i64>>(10)?.map(|value| value as usize),
            truth_pack_source_counts: decode_truth_pack_source_counts(source_counts_json).map_err(
                |err| {
                    rusqlite::Error::FromSqlConversionFailure(11, rusqlite::types::Type::Text, err)
                },
            )?,
            chunk_count: row.get::<_, i64>(12)? as usize,
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
                scenes.truth_pack_scene,
                scenes.truth_pack_feature_count,
                scenes.truth_pack_retained_semantic_count,
                scenes.truth_pack_semantic_lineage_count,
                scenes.truth_pack_dropped_semantic_count,
                scenes.truth_pack_collapse_count,
                scenes.truth_pack_source_counts_json,
                COUNT(chunks.chunk_id) AS chunk_count
            FROM scenes
            LEFT JOIN chunks ON chunks.scene_id = scenes.scene_id
            WHERE scenes.scene_id = ?1
            GROUP BY
                scenes.scene_id,
                scenes.world_name,
                scenes.chunk_size_studs,
                scenes.total_features,
                scenes.manifest_store_path,
                scenes.truth_pack_scene,
                scenes.truth_pack_feature_count,
                scenes.truth_pack_retained_semantic_count,
                scenes.truth_pack_semantic_lineage_count,
                scenes.truth_pack_dropped_semantic_count,
                scenes.truth_pack_collapse_count,
                scenes.truth_pack_source_counts_json
            ",
            params![scene_id],
            |row| {
                let source_counts_json: Option<String> = row.get(11)?;
                Ok(PlanetarySceneCatalogEntry {
                    scene_id: row.get(0)?,
                    world_name: row.get(1)?,
                    chunk_size_studs: row.get(2)?,
                    total_features: row.get::<_, i64>(3)? as usize,
                    manifest_store_path: row.get(4)?,
                    truth_pack_scene: row.get(5)?,
                    truth_pack_feature_count: row
                        .get::<_, Option<i64>>(6)?
                        .map(|value| value as usize),
                    truth_pack_retained_semantic_count: row
                        .get::<_, Option<i64>>(7)?
                        .map(|value| value as usize),
                    truth_pack_semantic_lineage_count: row
                        .get::<_, Option<i64>>(8)?
                        .map(|value| value as usize),
                    truth_pack_dropped_semantic_count: row
                        .get::<_, Option<i64>>(9)?
                        .map(|value| value as usize),
                    truth_pack_collapse_count: row
                        .get::<_, Option<i64>>(10)?
                        .map(|value| value as usize),
                    truth_pack_source_counts: decode_truth_pack_source_counts(source_counts_json)
                        .map_err(|err| {
                        rusqlite::Error::FromSqlConversionFailure(
                            11,
                            rusqlite::types::Type::Text,
                            err,
                        )
                    })?,
                    chunk_count: row.get::<_, i64>(12)? as usize,
                })
            },
        )
        .optional()
        .map_err(Into::into)
}

pub fn attach_truth_pack_summary(
    path: &Path,
    scene_id: &str,
    summary: &SourceTruthPackSummary,
) -> PlanetaryStoreResult<()> {
    let connection = open_store(path)?;
    let source_counts_json = serde_json::to_string(&summary.source_counts)?;
    let updated = connection.execute(
        "
        UPDATE scenes
        SET
            truth_pack_scene = ?2,
            truth_pack_feature_count = ?3,
            truth_pack_retained_semantic_count = ?4,
            truth_pack_semantic_lineage_count = ?5,
            truth_pack_dropped_semantic_count = ?6,
            truth_pack_collapse_count = ?7,
            truth_pack_source_counts_json = ?8
        WHERE scene_id = ?1
        ",
        params![
            scene_id,
            summary.scene,
            summary.feature_count as i64,
            summary.retained_semantic_count as i64,
            summary.semantic_lineage_count as i64,
            summary.dropped_semantic_count as i64,
            summary.collapse_count as i64,
            source_counts_json,
        ],
    )?;
    if updated == 0 {
        return Err(format!("scene {} is not present in planetary store", scene_id).into());
    }
    Ok(())
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
            scenes.truth_pack_scene,
            scenes.truth_pack_feature_count,
            scenes.truth_pack_retained_semantic_count,
            scenes.truth_pack_semantic_lineage_count,
            scenes.truth_pack_dropped_semantic_count,
            scenes.truth_pack_collapse_count,
            scenes.truth_pack_source_counts_json,
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
            scenes.truth_pack_scene,
            scenes.truth_pack_feature_count,
            scenes.truth_pack_retained_semantic_count,
            scenes.truth_pack_semantic_lineage_count,
            scenes.truth_pack_dropped_semantic_count,
            scenes.truth_pack_collapse_count,
            scenes.truth_pack_source_counts_json,
            scenes.bbox_min_lat,
            scenes.bbox_min_lon,
            scenes.bbox_max_lat,
            scenes.bbox_max_lon
        ORDER BY scenes.scene_id ASC
        ",
    )?;
    let rows = statement.query_map([], |row| {
        let source_counts_json: Option<String> = row.get(11)?;
        Ok((
            PlanetarySceneCatalogEntry {
                scene_id: row.get(0)?,
                world_name: row.get(1)?,
                chunk_size_studs: row.get(2)?,
                total_features: row.get::<_, i64>(3)? as usize,
                manifest_store_path: row.get(4)?,
                truth_pack_scene: row.get(5)?,
                truth_pack_feature_count: row.get::<_, Option<i64>>(6)?.map(|value| value as usize),
                truth_pack_retained_semantic_count: row
                    .get::<_, Option<i64>>(7)?
                    .map(|value| value as usize),
                truth_pack_semantic_lineage_count: row
                    .get::<_, Option<i64>>(8)?
                    .map(|value| value as usize),
                truth_pack_dropped_semantic_count: row
                    .get::<_, Option<i64>>(9)?
                    .map(|value| value as usize),
                truth_pack_collapse_count: row
                    .get::<_, Option<i64>>(10)?
                    .map(|value| value as usize),
                truth_pack_source_counts: decode_truth_pack_source_counts(source_counts_json)
                    .map_err(|err| {
                        rusqlite::Error::FromSqlConversionFailure(
                            11,
                            rusqlite::types::Type::Text,
                            err,
                        )
                    })?,
                chunk_count: row.get::<_, i64>(16)? as usize,
            },
            (
                row.get::<_, f64>(12)?,
                row.get::<_, f64>(13)?,
                row.get::<_, f64>(14)?,
                row.get::<_, f64>(15)?,
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
            scenes.push((
                entry,
                arbx_geo::BoundingBox::new(
                    scene_min_lat,
                    scene_min_lon,
                    scene_max_lat,
                    scene_max_lon,
                ),
            ));
        }
    }
    scenes.sort_by(|left, right| compare_scene_priority(&left.0, left.1, &right.0, right.1));
    Ok(scenes.into_iter().map(|(entry, _)| entry).collect())
}

fn find_scene_geo_candidates_covering_point(
    path: &Path,
    lat: f64,
    lon: f64,
) -> PlanetaryStoreResult<Vec<SceneGeoCandidate>> {
    let connection = open_store(path)?;
    let mut statement = connection.prepare(
        "
        SELECT
            scenes.scene_id,
            scenes.world_name,
            scenes.chunk_size_studs,
            scenes.total_features,
            scenes.manifest_store_path,
            scenes.truth_pack_scene,
            scenes.truth_pack_feature_count,
            scenes.truth_pack_retained_semantic_count,
            scenes.truth_pack_semantic_lineage_count,
            scenes.truth_pack_dropped_semantic_count,
            scenes.truth_pack_collapse_count,
            scenes.truth_pack_source_counts_json,
            scenes.bbox_min_lat,
            scenes.bbox_min_lon,
            scenes.bbox_max_lat,
            scenes.bbox_max_lon,
            scenes.meters_per_stud,
            COUNT(chunks.chunk_id) AS chunk_count
        FROM scenes
        LEFT JOIN chunks ON chunks.scene_id = scenes.scene_id
        GROUP BY
            scenes.scene_id,
            scenes.world_name,
            scenes.chunk_size_studs,
            scenes.total_features,
            scenes.manifest_store_path,
            scenes.truth_pack_scene,
            scenes.truth_pack_feature_count,
            scenes.truth_pack_retained_semantic_count,
            scenes.truth_pack_semantic_lineage_count,
            scenes.truth_pack_dropped_semantic_count,
            scenes.truth_pack_collapse_count,
            scenes.truth_pack_source_counts_json,
            scenes.bbox_min_lat,
            scenes.bbox_min_lon,
            scenes.bbox_max_lat,
            scenes.bbox_max_lon,
            scenes.meters_per_stud
        ORDER BY scenes.scene_id ASC
        ",
    )?;
    let rows = statement.query_map([], |row| {
        let source_counts_json: Option<String> = row.get(11)?;
        Ok(SceneGeoCandidate {
            entry: PlanetarySceneCatalogEntry {
                scene_id: row.get(0)?,
                world_name: row.get(1)?,
                chunk_size_studs: row.get(2)?,
                total_features: row.get::<_, i64>(3)? as usize,
                manifest_store_path: row.get(4)?,
                truth_pack_scene: row.get(5)?,
                truth_pack_feature_count: row.get::<_, Option<i64>>(6)?.map(|value| value as usize),
                truth_pack_retained_semantic_count: row
                    .get::<_, Option<i64>>(7)?
                    .map(|value| value as usize),
                truth_pack_semantic_lineage_count: row
                    .get::<_, Option<i64>>(8)?
                    .map(|value| value as usize),
                truth_pack_dropped_semantic_count: row
                    .get::<_, Option<i64>>(9)?
                    .map(|value| value as usize),
                truth_pack_collapse_count: row
                    .get::<_, Option<i64>>(10)?
                    .map(|value| value as usize),
                truth_pack_source_counts: decode_truth_pack_source_counts(source_counts_json)
                    .map_err(|err| {
                        rusqlite::Error::FromSqlConversionFailure(
                            11,
                            rusqlite::types::Type::Text,
                            err,
                        )
                    })?,
                chunk_count: row.get::<_, i64>(17)? as usize,
            },
            bbox: arbx_geo::BoundingBox::new(
                row.get(12)?,
                row.get(13)?,
                row.get(14)?,
                row.get(15)?,
            ),
            meters_per_stud: row.get(16)?,
        })
    })?;

    let focus = LatLon::new(lat, lon);
    let mut scenes = Vec::new();
    for row in rows {
        let row = row?;
        if row.bbox.contains(focus) {
            scenes.push(row);
        }
    }
    scenes.sort_by(|left, right| {
        compare_scene_priority(&left.entry, left.bbox, &right.entry, right.bbox)
    });
    Ok(scenes)
}

pub fn find_best_scene_covering_geo_point(
    path: &Path,
    lat: f64,
    lon: f64,
) -> PlanetaryStoreResult<Option<PlanetarySceneCatalogEntry>> {
    Ok(find_scene_geo_candidates_covering_point(path, lat, lon)?
        .into_iter()
        .next()
        .map(|candidate| candidate.entry))
}

pub fn find_scenes_covering_geo_point(
    path: &Path,
    lat: f64,
    lon: f64,
) -> PlanetaryStoreResult<Vec<PlanetarySceneCatalogEntry>> {
    Ok(find_scene_geo_candidates_covering_point(path, lat, lon)?
        .into_iter()
        .map(|candidate| candidate.entry)
        .collect())
}

pub fn find_best_scene_covering_tile(
    path: &Path,
    zoom: u8,
    x: u32,
    y: u32,
) -> PlanetaryStoreResult<Option<PlanetarySceneCatalogEntry>> {
    Ok(find_scenes_covering_tile(path, zoom, x, y)?
        .into_iter()
        .next())
}

pub fn find_scenes_covering_tile(
    path: &Path,
    zoom: u8,
    x: u32,
    y: u32,
) -> PlanetaryStoreResult<Vec<PlanetarySceneCatalogEntry>> {
    let bbox = slippy_tile_bbox(zoom, x, y);
    find_scenes_intersecting_geo_bbox(path, bbox.min.lat, bbox.min.lon, bbox.max.lat, bbox.max.lon)
}

pub fn build_delivery_window_around_geo_point(
    path: &Path,
    lat: f64,
    lon: f64,
    radius_studs: f64,
    limit: Option<usize>,
    require_buildings: bool,
    require_terrain: bool,
    max_streaming_cost: Option<f64>,
    max_estimated_memory_cost: Option<f64>,
) -> PlanetaryStoreResult<Option<PlanetaryDeliveryWindow>> {
    let Some(scene) = find_scene_geo_candidates_covering_point(path, lat, lon)?
        .into_iter()
        .next()
    else {
        return Ok(None);
    };
    let focus = Mercator::project(
        LatLon::new(lat, lon),
        scene.bbox.center(),
        scene.meters_per_stud,
    );
    let chunks = read_scene_chunk_summary_around_point(
        path,
        &scene.entry.scene_id,
        focus.x,
        focus.z,
        radius_studs,
        limit,
        require_buildings,
        require_terrain,
        max_streaming_cost,
        max_estimated_memory_cost,
    )?;
    let (chunk_count, total_feature_count, total_streaming_cost, total_estimated_memory_cost) =
        summarize_delivery_chunks(&chunks);
    Ok(Some(PlanetaryDeliveryWindow {
        scene: scene.entry,
        focus_lat: lat,
        focus_lon: lon,
        focus_x: focus.x,
        focus_z: focus.z,
        radius_studs,
        chunk_count,
        total_feature_count,
        total_streaming_cost,
        total_estimated_memory_cost,
        chunks,
    }))
}

pub fn build_delivery_plan_around_geo_point(
    path: &Path,
    lat: f64,
    lon: f64,
    radius_studs: f64,
    limit: Option<usize>,
    require_buildings: bool,
    require_terrain: bool,
    max_streaming_cost: Option<f64>,
    max_estimated_memory_cost: Option<f64>,
) -> PlanetaryStoreResult<Option<PlanetaryDeliveryPlan>> {
    Ok(build_delivery_window_around_geo_point(
        path,
        lat,
        lon,
        radius_studs,
        limit,
        require_buildings,
        require_terrain,
        max_streaming_cost,
        max_estimated_memory_cost,
    )?
    .map(|window| delivery_plan_from_window("geo-point", &window)))
}

pub fn build_delivery_window_around_point(
    path: &Path,
    scene_id: &str,
    focus_x: f64,
    focus_z: f64,
    radius_studs: f64,
    limit: Option<usize>,
    require_buildings: bool,
    require_terrain: bool,
    max_streaming_cost: Option<f64>,
    max_estimated_memory_cost: Option<f64>,
) -> PlanetaryStoreResult<Option<PlanetaryDeliveryWindow>> {
    let Some(scene) = read_scene_catalog_entry(path, scene_id)? else {
        return Ok(None);
    };
    let chunks = read_scene_chunk_summary_around_point(
        path,
        scene_id,
        focus_x,
        focus_z,
        radius_studs,
        limit,
        require_buildings,
        require_terrain,
        max_streaming_cost,
        max_estimated_memory_cost,
    )?;
    let (chunk_count, total_feature_count, total_streaming_cost, total_estimated_memory_cost) =
        summarize_delivery_chunks(&chunks);
    Ok(Some(PlanetaryDeliveryWindow {
        scene,
        focus_lat: 0.0,
        focus_lon: 0.0,
        focus_x,
        focus_z,
        radius_studs,
        chunk_count,
        total_feature_count,
        total_streaming_cost,
        total_estimated_memory_cost,
        chunks,
    }))
}

pub fn build_delivery_plan_around_point(
    path: &Path,
    scene_id: &str,
    focus_x: f64,
    focus_z: f64,
    radius_studs: f64,
    limit: Option<usize>,
    require_buildings: bool,
    require_terrain: bool,
    max_streaming_cost: Option<f64>,
    max_estimated_memory_cost: Option<f64>,
) -> PlanetaryStoreResult<Option<PlanetaryDeliveryPlan>> {
    Ok(build_delivery_window_around_point(
        path,
        scene_id,
        focus_x,
        focus_z,
        radius_studs,
        limit,
        require_buildings,
        require_terrain,
        max_streaming_cost,
        max_estimated_memory_cost,
    )?
    .map(|window| delivery_plan_from_window("local-point", &window)))
}

pub fn build_delivery_window_for_scene_bbox(
    path: &Path,
    scene_id: &str,
    min_x: f64,
    min_z: f64,
    max_x: f64,
    max_z: f64,
    limit: Option<usize>,
    require_buildings: bool,
    require_terrain: bool,
    max_streaming_cost: Option<f64>,
    max_estimated_memory_cost: Option<f64>,
) -> PlanetaryStoreResult<Option<PlanetaryDeliveryWindow>> {
    let Some(scene) = read_scene_catalog_entry(path, scene_id)? else {
        return Ok(None);
    };
    let focus_x = (min_x + max_x) * 0.5;
    let focus_z = (min_z + max_z) * 0.5;
    let radius_studs = ((max_x - min_x) * 0.5).max((max_z - min_z) * 0.5);
    let chunks = read_scene_chunk_summary_subset(
        path,
        scene_id,
        min_x,
        min_z,
        max_x,
        max_z,
        limit,
        require_buildings,
        require_terrain,
        max_streaming_cost,
        max_estimated_memory_cost,
    )?;
    let (chunk_count, total_feature_count, total_streaming_cost, total_estimated_memory_cost) =
        summarize_delivery_chunks(&chunks);
    Ok(Some(PlanetaryDeliveryWindow {
        scene,
        focus_lat: 0.0,
        focus_lon: 0.0,
        focus_x,
        focus_z,
        radius_studs,
        chunk_count,
        total_feature_count,
        total_streaming_cost,
        total_estimated_memory_cost,
        chunks,
    }))
}

pub fn build_delivery_plan_for_scene_bbox(
    path: &Path,
    scene_id: &str,
    min_x: f64,
    min_z: f64,
    max_x: f64,
    max_z: f64,
    limit: Option<usize>,
    require_buildings: bool,
    require_terrain: bool,
    max_streaming_cost: Option<f64>,
    max_estimated_memory_cost: Option<f64>,
) -> PlanetaryStoreResult<Option<PlanetaryDeliveryPlan>> {
    Ok(build_delivery_window_for_scene_bbox(
        path,
        scene_id,
        min_x,
        min_z,
        max_x,
        max_z,
        limit,
        require_buildings,
        require_terrain,
        max_streaming_cost,
        max_estimated_memory_cost,
    )?
    .map(|window| delivery_plan_from_window("scene-bbox", &window)))
}

pub fn build_delivery_window_for_tile(
    path: &Path,
    zoom: u8,
    x: u32,
    y: u32,
    limit: Option<usize>,
    require_buildings: bool,
    require_terrain: bool,
    max_streaming_cost: Option<f64>,
    max_estimated_memory_cost: Option<f64>,
) -> PlanetaryStoreResult<Option<PlanetaryDeliveryWindow>> {
    let Some(scene) = find_best_scene_covering_tile(path, zoom, x, y)? else {
        return Ok(None);
    };
    let connection = open_store(path)?;
    let meta = read_scene_meta(&connection, &scene.scene_id)?;
    drop(connection);

    let tile_bbox = slippy_tile_bbox(zoom, x, y);
    let tile_center = tile_bbox.center();
    let focus = Mercator::project(tile_center, meta.bbox.center(), meta.meters_per_stud);
    let (min_x, min_z, max_x, max_z) =
        project_geo_bbox_to_local_bounds(tile_bbox, meta.bbox.center(), meta.meters_per_stud);
    let radius_studs = ((max_x - min_x) * 0.5).max((max_z - min_z) * 0.5);
    let chunks = read_scene_chunk_summary_for_tile(
        path,
        &scene.scene_id,
        zoom,
        x,
        y,
        limit,
        require_buildings,
        require_terrain,
        max_streaming_cost,
        max_estimated_memory_cost,
    )?;
    let (chunk_count, total_feature_count, total_streaming_cost, total_estimated_memory_cost) =
        summarize_delivery_chunks(&chunks);
    Ok(Some(PlanetaryDeliveryWindow {
        scene,
        focus_lat: tile_center.lat,
        focus_lon: tile_center.lon,
        focus_x: focus.x,
        focus_z: focus.z,
        radius_studs,
        chunk_count,
        total_feature_count,
        total_streaming_cost,
        total_estimated_memory_cost,
        chunks,
    }))
}

pub fn build_delivery_plan_for_tile(
    path: &Path,
    zoom: u8,
    x: u32,
    y: u32,
    limit: Option<usize>,
    require_buildings: bool,
    require_terrain: bool,
    max_streaming_cost: Option<f64>,
    max_estimated_memory_cost: Option<f64>,
) -> PlanetaryStoreResult<Option<PlanetaryDeliveryPlan>> {
    Ok(build_delivery_window_for_tile(
        path,
        zoom,
        x,
        y,
        limit,
        require_buildings,
        require_terrain,
        max_streaming_cost,
        max_estimated_memory_cost,
    )?
    .map(|window| delivery_plan_from_window("tile", &window)))
}

pub fn read_scene_chunk_summary_around_geo_point(
    path: &Path,
    scene_id: &str,
    lat: f64,
    lon: f64,
    radius_studs: f64,
    limit: Option<usize>,
    require_buildings: bool,
    require_terrain: bool,
    max_streaming_cost: Option<f64>,
    max_estimated_memory_cost: Option<f64>,
) -> PlanetaryStoreResult<Vec<PlanetaryChunkSummary>> {
    let connection = open_store(path)?;
    let meta = read_scene_meta(&connection, scene_id)?;
    let focus = Mercator::project(
        LatLon::new(lat, lon),
        meta.bbox.center(),
        meta.meters_per_stud,
    );
    drop(connection);
    read_scene_chunk_summary_around_point(
        path,
        scene_id,
        focus.x,
        focus.z,
        radius_studs,
        limit,
        require_buildings,
        require_terrain,
        max_streaming_cost,
        max_estimated_memory_cost,
    )
}

pub fn read_scene_manifest_subset_around_geo_point(
    path: &Path,
    scene_id: &str,
    lat: f64,
    lon: f64,
    radius_studs: f64,
    limit: Option<usize>,
    require_buildings: bool,
    require_terrain: bool,
    max_streaming_cost: Option<f64>,
    max_estimated_memory_cost: Option<f64>,
) -> PlanetaryStoreResult<arbx_roblox_export::StoredManifestSubset> {
    let chunk_ids = read_scene_chunk_summary_around_geo_point(
        path,
        scene_id,
        lat,
        lon,
        radius_studs,
        limit,
        require_buildings,
        require_terrain,
        max_streaming_cost,
        max_estimated_memory_cost,
    )?
    .into_iter()
    .map(|chunk| chunk.chunk_id)
    .collect::<Vec<_>>();
    read_scene_manifest_subset_by_chunk_ids(path, scene_id, &chunk_ids)
}

pub fn read_scene_chunk_summary_for_tile(
    path: &Path,
    scene_id: &str,
    zoom: u8,
    x: u32,
    y: u32,
    limit: Option<usize>,
    require_buildings: bool,
    require_terrain: bool,
    max_streaming_cost: Option<f64>,
    max_estimated_memory_cost: Option<f64>,
) -> PlanetaryStoreResult<Vec<PlanetaryChunkSummary>> {
    let connection = open_store(path)?;
    let meta = read_scene_meta(&connection, scene_id)?;
    let tile_bbox = slippy_tile_bbox(zoom, x, y);
    let (min_x, min_z, max_x, max_z) =
        project_geo_bbox_to_local_bounds(tile_bbox, meta.bbox.center(), meta.meters_per_stud);
    drop(connection);
    read_scene_chunk_summary_subset(
        path,
        scene_id,
        min_x,
        min_z,
        max_x,
        max_z,
        limit,
        require_buildings,
        require_terrain,
        max_streaming_cost,
        max_estimated_memory_cost,
    )
}

pub fn read_scene_manifest_subset_for_tile(
    path: &Path,
    scene_id: &str,
    zoom: u8,
    x: u32,
    y: u32,
) -> PlanetaryStoreResult<arbx_roblox_export::StoredManifestSubset> {
    let connection = open_store(path)?;
    let meta = read_scene_meta(&connection, scene_id)?;
    let tile_bbox = slippy_tile_bbox(zoom, x, y);
    let (min_x, min_z, max_x, max_z) =
        project_geo_bbox_to_local_bounds(tile_bbox, meta.bbox.center(), meta.meters_per_stud);
    drop(connection);
    read_scene_manifest_subset(path, scene_id, min_x, min_z, max_x, max_z)
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
    require_buildings: bool,
    require_terrain: bool,
    max_streaming_cost: Option<f64>,
    max_estimated_memory_cost: Option<f64>,
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

    let sql = String::from(
        "
        SELECT
            chunk_id,
            origin_x,
            origin_z,
            feature_count,
            streaming_cost,
            estimated_memory_cost,
            partition_version,
            has_terrain,
            road_count,
            rail_count,
            building_count,
            water_count,
            prop_count,
            landuse_count,
            barrier_count
        FROM chunks
        WHERE scene_id = ?1
          AND origin_x <= ?2
          AND origin_x + ?3 >= ?4
          AND origin_z <= ?5
          AND origin_z + ?3 >= ?6
          AND (?7 = 0 OR building_count > 0)
          AND (?8 = 0 OR has_terrain = 1)
        ORDER BY origin_z ASC, origin_x ASC
        ",
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map(
        params![
            scene_id,
            max_x,
            chunk_size_studs as f64,
            min_x,
            max_z,
            min_z,
            if require_buildings { 1 } else { 0 },
            if require_terrain { 1 } else { 0 },
        ],
        |row| {
            Ok(PlanetaryChunkSummary {
                chunk_id: row.get(0)?,
                origin_x: row.get(1)?,
                origin_z: row.get(2)?,
                feature_count: row.get::<_, i64>(3)? as usize,
                streaming_cost: row.get(4)?,
                estimated_memory_cost: row.get(5)?,
                partition_version: row.get(6)?,
                has_terrain: row.get::<_, i64>(7)? != 0,
                road_count: row.get::<_, i64>(8)? as usize,
                rail_count: row.get::<_, i64>(9)? as usize,
                building_count: row.get::<_, i64>(10)? as usize,
                water_count: row.get::<_, i64>(11)? as usize,
                prop_count: row.get::<_, i64>(12)? as usize,
                landuse_count: row.get::<_, i64>(13)? as usize,
                barrier_count: row.get::<_, i64>(14)? as usize,
                center_distance_sq: None,
            })
        },
    )?;

    let mut chunks = Vec::new();
    for row in rows {
        chunks.push(row?);
    }
    Ok(apply_chunk_summary_constraints(
        chunks,
        limit,
        max_streaming_cost,
        max_estimated_memory_cost,
    ))
}

pub fn read_scene_chunk_summary_around_point(
    path: &Path,
    scene_id: &str,
    focus_x: f64,
    focus_z: f64,
    radius_studs: f64,
    limit: Option<usize>,
    require_buildings: bool,
    require_terrain: bool,
    max_streaming_cost: Option<f64>,
    max_estimated_memory_cost: Option<f64>,
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

    let half = chunk_size_studs as f64 * 0.5;
    let radius_sq = radius_studs * radius_studs;
    let sql = String::from(
        "
        SELECT
            chunk_id,
            origin_x,
            origin_z,
            feature_count,
            streaming_cost,
            estimated_memory_cost,
            partition_version,
            has_terrain,
            road_count,
            rail_count,
            building_count,
            water_count,
            prop_count,
            landuse_count,
            barrier_count,
            ((origin_x + ?2) - ?3) * ((origin_x + ?2) - ?3)
              + ((origin_z + ?2) - ?4) * ((origin_z + ?2) - ?4) AS center_distance_sq
        FROM chunks
        WHERE scene_id = ?1
          AND ((origin_x + ?2) - ?3) * ((origin_x + ?2) - ?3)
            + ((origin_z + ?2) - ?4) * ((origin_z + ?2) - ?4) <= ?5
          AND (?6 = 0 OR building_count > 0)
          AND (?7 = 0 OR has_terrain = 1)
        ORDER BY center_distance_sq ASC, chunk_id ASC
        ",
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map(
        params![
            scene_id,
            half,
            focus_x,
            focus_z,
            radius_sq,
            if require_buildings { 1 } else { 0 },
            if require_terrain { 1 } else { 0 },
        ],
        |row| {
            Ok(PlanetaryChunkSummary {
                chunk_id: row.get(0)?,
                origin_x: row.get(1)?,
                origin_z: row.get(2)?,
                feature_count: row.get::<_, i64>(3)? as usize,
                streaming_cost: row.get(4)?,
                estimated_memory_cost: row.get(5)?,
                partition_version: row.get(6)?,
                has_terrain: row.get::<_, i64>(7)? != 0,
                road_count: row.get::<_, i64>(8)? as usize,
                rail_count: row.get::<_, i64>(9)? as usize,
                building_count: row.get::<_, i64>(10)? as usize,
                water_count: row.get::<_, i64>(11)? as usize,
                prop_count: row.get::<_, i64>(12)? as usize,
                landuse_count: row.get::<_, i64>(13)? as usize,
                barrier_count: row.get::<_, i64>(14)? as usize,
                center_distance_sq: Some(row.get(15)?),
            })
        },
    )?;

    let mut chunks = Vec::new();
    for row in rows {
        chunks.push(row?);
    }
    Ok(apply_chunk_summary_constraints(
        chunks,
        limit,
        max_streaming_cost,
        max_estimated_memory_cost,
    ))
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
            false,
            false,
            None,
            None,
        )
        .unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_id, "0_0");
    }

    #[test]
    fn planetary_store_filters_chunk_summary_subset_by_payload_classes() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let chunks = read_scene_chunk_summary_subset(
            &store_path,
            "sample_austin",
            -10.0,
            -10.0,
            800.0,
            300.0,
            None,
            true,
            true,
            None,
            None,
        )
        .unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_id, "0_0");
        assert!(chunks[0].has_terrain);
        assert!(chunks[0].building_count > 0);
    }

    #[test]
    fn planetary_store_reads_chunk_summary_around_point() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let chunks = read_scene_chunk_summary_around_point(
            &store_path,
            "sample_austin",
            300.0,
            128.0,
            300.0,
            Some(2),
            false,
            false,
            None,
            None,
        )
        .unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].chunk_id, "1_0");
        assert_eq!(chunks[1].chunk_id, "0_0");
        assert!(chunks[0].center_distance_sq.unwrap() <= chunks[1].center_distance_sq.unwrap());
    }

    #[test]
    fn planetary_store_applies_streaming_budget_to_point_selection() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let chunks = read_scene_chunk_summary_around_point(
            &store_path,
            "sample_austin",
            300.0,
            128.0,
            300.0,
            None,
            false,
            false,
            Some(8.0),
            None,
        )
        .unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_id, "1_0");
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

        let scenes =
            find_scenes_intersecting_geo_bbox(&store_path, 30.1, -97.9, 30.2, -97.8).unwrap();
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

        let summary =
            ingest_manifest_json(&store_path, &manifest_path, Some("json_scene")).unwrap();
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
        let ids: Vec<&str> = subset
            .chunks
            .iter()
            .map(|chunk| chunk.chunk_id.as_str())
            .collect();
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
            read_scene_manifest_subset(&store_path, "sample_austin", 200.0, 0.0, 500.0, 200.0)
                .unwrap();
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

    #[test]
    fn planetary_store_reads_chunk_summary_around_geo_point() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let chunks = read_scene_chunk_summary_around_geo_point(
            &store_path,
            "sample_austin",
            center.lat,
            center.lon,
            300.0,
            Some(2),
            false,
            false,
            None,
            None,
        )
        .unwrap();
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn planetary_store_reads_manifest_subset_around_geo_point() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let subset = read_scene_manifest_subset_around_geo_point(
            &store_path,
            "sample_austin",
            center.lat,
            center.lon,
            300.0,
            Some(2),
            false,
            false,
            None,
            None,
        )
        .unwrap();
        assert_eq!(subset.chunks.len(), 1);
    }

    #[test]
    fn planetary_store_prefers_smallest_scene_covering_geo_point() {
        let dir = tempdir().unwrap();
        let manifest_a_path = dir.path().join("sample_a.sqlite");
        let manifest_b_path = dir.path().join("sample_b.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let mut manifest_a = build_sample_multi_chunk(1, 1);
        manifest_a.meta.bbox = arbx_geo::BoundingBox::new(30.0, -98.0, 31.0, -97.0);
        let mut manifest_b = build_sample_multi_chunk(1, 1);
        manifest_b.meta.bbox = arbx_geo::BoundingBox::new(30.2, -97.9, 30.4, -97.7);
        write_manifest_sqlite(&manifest_a, &manifest_a_path).unwrap();
        write_manifest_sqlite(&manifest_b, &manifest_b_path).unwrap();

        ingest_manifest_sqlite(&store_path, &manifest_a_path, Some("large_scene")).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_b_path, Some("small_scene")).unwrap();

        let mut source_counts = BTreeMap::new();
        source_counts.insert("overpass".to_string(), 20);
        let summary = SourceTruthPackSummary {
            scene: "large_scene_truth".to_string(),
            feature_count: 40,
            retained_semantic_count: 10,
            semantic_lineage_count: 10,
            dropped_semantic_count: 0,
            collapse_count: 0,
            source_counts,
        };
        attach_truth_pack_summary(&store_path, "large_scene", &summary).unwrap();

        let window = build_delivery_window_around_geo_point(
            &store_path,
            30.3,
            -97.8,
            300.0,
            Some(1),
            false,
            false,
            None,
            None,
        )
        .unwrap()
        .unwrap();
        assert_eq!(window.scene.scene_id, "large_scene");
    }

    #[test]
    fn planetary_store_builds_delivery_window_around_geo_point() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let window = build_delivery_window_around_geo_point(
            &store_path,
            center.lat,
            center.lon,
            300.0,
            Some(2),
            false,
            false,
            None,
            None,
        )
        .unwrap()
        .unwrap();
        assert_eq!(window.scene.scene_id, "sample_austin");
        assert_eq!(window.chunks.len(), 1);
        assert_eq!(window.chunk_count, 1);
        assert_eq!(window.total_feature_count, window.chunks[0].feature_count);
        assert_eq!(window.total_streaming_cost, window.chunks[0].streaming_cost);
    }

    #[test]
    fn planetary_store_builds_delivery_window_around_local_point() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let window = build_delivery_window_around_point(
            &store_path,
            "sample_austin",
            300.0,
            128.0,
            300.0,
            Some(2),
            false,
            false,
            None,
            None,
        )
        .unwrap()
        .unwrap();
        assert_eq!(window.scene.scene_id, "sample_austin");
        assert_eq!(window.focus_x, 300.0);
        assert_eq!(window.focus_z, 128.0);
        assert_eq!(window.chunk_count, 2);
    }

    #[test]
    fn planetary_store_builds_delivery_window_for_scene_bbox() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let window = build_delivery_window_for_scene_bbox(
            &store_path,
            "sample_austin",
            200.0,
            0.0,
            500.0,
            200.0,
            Some(2),
            false,
            false,
            None,
            None,
        )
        .unwrap()
        .unwrap();
        assert_eq!(window.scene.scene_id, "sample_austin");
        assert_eq!(window.focus_x, 350.0);
        assert_eq!(window.focus_z, 100.0);
        assert_eq!(window.chunk_count, 2);
    }

    #[test]
    fn planetary_store_builds_delivery_window_for_tile() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let zoom = 10u8;
        let x = (((center.lon + 180.0) / 360.0) * 2f64.powi(zoom as i32)).floor() as u32;
        let lat_rad = center.lat.to_radians();
        let y = ((1.0 - ((lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI)) / 2.0
            * 2f64.powi(zoom as i32))
        .floor() as u32;

        let window = build_delivery_window_for_tile(
            &store_path,
            zoom,
            x,
            y,
            Some(2),
            false,
            false,
            None,
            None,
        )
        .unwrap()
        .unwrap();
        assert_eq!(window.scene.scene_id, "sample_austin");
        assert!(!window.chunks.is_empty());
        assert_eq!(window.chunk_count, window.chunks.len());
        assert!(window.total_feature_count >= window.chunks[0].feature_count);
        assert!(window.total_streaming_cost >= window.chunks[0].streaming_cost);
    }

    #[test]
    fn planetary_store_builds_delivery_plan_for_tile() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let zoom = 10u8;
        let x = (((center.lon + 180.0) / 360.0) * 2f64.powi(zoom as i32)).floor() as u32;
        let lat_rad = center.lat.to_radians();
        let y = ((1.0 - ((lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI)) / 2.0
            * 2f64.powi(zoom as i32))
        .floor() as u32;

        let plan = build_delivery_plan_for_tile(
            &store_path,
            zoom,
            x,
            y,
            Some(2),
            false,
            false,
            None,
            None,
        )
        .unwrap()
        .unwrap();
        assert_eq!(plan.scene_id, "sample_austin");
        assert_eq!(plan.selection_mode, "tile");
        assert_eq!(plan.chunk_count, plan.chunk_ids.len());
    }

    #[test]
    fn planetary_store_finds_scenes_covering_tile() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let mut manifest = build_sample_multi_chunk(1, 1);
        manifest.meta.bbox = arbx_geo::BoundingBox::new(30.0, -98.0, 30.5, -97.5);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("austin")).unwrap();

        let zoom = 10u8;
        let x = (((center.lon + 180.0) / 360.0) * 2f64.powi(zoom as i32)).floor() as u32;
        let lat_rad = center.lat.to_radians();
        let y = ((1.0 - ((lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI)) / 2.0
            * 2f64.powi(zoom as i32))
        .floor() as u32;

        let scenes = find_scenes_covering_tile(&store_path, zoom, x, y).unwrap();
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].scene_id, "austin");
    }

    #[test]
    fn planetary_store_reads_chunk_summary_for_tile() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let zoom = 10u8;
        let x = (((center.lon + 180.0) / 360.0) * 2f64.powi(zoom as i32)).floor() as u32;
        let lat_rad = center.lat.to_radians();
        let y = ((1.0 - ((lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI)) / 2.0
            * 2f64.powi(zoom as i32))
        .floor() as u32;

        let chunks = read_scene_chunk_summary_for_tile(
            &store_path,
            "sample_austin",
            zoom,
            x,
            y,
            Some(2),
            false,
            false,
            None,
            None,
        )
        .unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn planetary_store_reads_manifest_subset_for_tile() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let zoom = 10u8;
        let x = (((center.lon + 180.0) / 360.0) * 2f64.powi(zoom as i32)).floor() as u32;
        let lat_rad = center.lat.to_radians();
        let y = ((1.0 - ((lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI)) / 2.0
            * 2f64.powi(zoom as i32))
        .floor() as u32;

        let subset =
            read_scene_manifest_subset_for_tile(&store_path, "sample_austin", zoom, x, y).unwrap();
        assert!(!subset.chunks.is_empty());
    }

    #[test]
    fn planetary_store_attaches_truth_pack_summary() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("sample.sqlite");
        let store_path = dir.path().join("planetary.sqlite");

        let manifest = build_sample_multi_chunk(1, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();
        ingest_manifest_sqlite(&store_path, &manifest_path, Some("sample_austin")).unwrap();

        let mut source_counts = BTreeMap::new();
        source_counts.insert("overpass".to_string(), 5);
        let summary = SourceTruthPackSummary {
            scene: "austin".to_string(),
            feature_count: 12,
            retained_semantic_count: 8,
            semantic_lineage_count: 6,
            dropped_semantic_count: 2,
            collapse_count: 1,
            source_counts,
        };

        attach_truth_pack_summary(&store_path, "sample_austin", &summary).unwrap();
        let scene = read_scene_catalog_entry(&store_path, "sample_austin")
            .unwrap()
            .unwrap();
        assert_eq!(scene.truth_pack_scene.as_deref(), Some("austin"));
        assert_eq!(scene.truth_pack_feature_count, Some(12));
        assert_eq!(scene.truth_pack_collapse_count, Some(1));
        assert_eq!(
            scene
                .truth_pack_source_counts
                .as_ref()
                .and_then(|value| value.get("overpass"))
                .copied(),
            Some(5)
        );
    }
}
