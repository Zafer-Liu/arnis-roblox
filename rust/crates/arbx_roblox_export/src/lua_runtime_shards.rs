use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::manifest_store::{
    stream_manifest_sqlite_all, ManifestStoreResult, StoredChunkRecord, StoredManifestMeta,
    StoredManifestSubset,
};

const CHUNK_LIST_FIELDS: [&str; 8] = [
    "roads",
    "rails",
    "buildings",
    "water",
    "props",
    "landuse",
    "barriers",
    "rooms",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLuaShardsOptions {
    pub output_dir: PathBuf,
    pub index_name: String,
    pub shard_folder: String,
    pub chunks_per_shard: usize,
    pub max_bytes: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLuaShardsStats {
    pub chunk_count: usize,
    pub fragment_count: usize,
    pub shard_count: usize,
    pub index_path: PathBuf,
    pub shard_dir: PathBuf,
}

pub fn write_runtime_lua_shards_from_sqlite(
    manifest_sqlite: &Path,
    options: &RuntimeLuaShardsOptions,
) -> ManifestStoreResult<RuntimeLuaShardsStats> {
    let mut records: Vec<StoredChunkRecord> = Vec::new();
    let meta = stream_manifest_sqlite_all(manifest_sqlite, |record| {
        records.push(record);
        Ok(())
    })?;
    write_runtime_lua_shards_from_records(meta, records, options)
}

pub fn write_runtime_lua_shards_from_stored_subset(
    subset: &StoredManifestSubset,
    options: &RuntimeLuaShardsOptions,
) -> ManifestStoreResult<RuntimeLuaShardsStats> {
    write_runtime_lua_shards_from_records(subset.meta.clone(), subset.chunks.clone(), options)
}

fn write_runtime_lua_shards_from_records(
    meta: StoredManifestMeta,
    records: Vec<StoredChunkRecord>,
    options: &RuntimeLuaShardsOptions,
) -> ManifestStoreResult<RuntimeLuaShardsStats> {
    let output_dir = &options.output_dir;
    let shard_dir = output_dir.join(&options.shard_folder);
    fs::create_dir_all(output_dir)?;
    clear_existing_shards(&shard_dir, &options.index_name)?;

    let mut chunk_refs: Vec<Value> = Vec::new();
    let mut chunk_ref_index_by_id: HashMap<String, usize> = HashMap::new();
    let mut shard_names: Vec<String> = Vec::new();
    let mut shard_buffer: Vec<Value> = Vec::new();
    let mut shard_chunk_ids: HashSet<String> = HashSet::new();
    let mut shard_bytes = lua_empty_shard_len();
    let mut next_shard_index: usize = 1;
    let mut fragment_count: usize = 0;

    for record in records {
        let chunk = parse_chunk_object(&record)?;
        let chunk_id = record.chunk_id.clone();
        let ref_index = chunk_refs.len();
        chunk_ref_index_by_id.insert(chunk_id.clone(), ref_index);
        chunk_refs.push(build_chunk_ref_value(&record)?);

        let fragments = fragment_chunk_for_lua_shards(&chunk, options.max_bytes)?;
        fragment_count += fragments.len();
        for fragment in fragments {
            let fragment_chunk_id = fragment
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| "runtime fragment is missing string id".to_string())?
                .to_string();
            let fragment_bytes = lua_value_len(&Value::Object(fragment.clone()));
            let next_shard_bytes =
                shard_bytes + fragment_bytes + usize::from(!shard_buffer.is_empty());
            let would_add_new_chunk = !shard_chunk_ids.contains(&fragment_chunk_id);
            let would_exceed_chunk_limit =
                would_add_new_chunk && shard_chunk_ids.len() == options.chunks_per_shard;
            let would_exceed_byte_limit = options.max_bytes.is_some()
                && !shard_buffer.is_empty()
                && next_shard_bytes > options.max_bytes.unwrap();

            if would_exceed_chunk_limit || would_exceed_byte_limit {
                flush_shard_buffer(
                    &shard_dir,
                    &options.index_name,
                    &mut shard_buffer,
                    &mut chunk_refs,
                    &chunk_ref_index_by_id,
                    &mut shard_names,
                    &mut next_shard_index,
                )?;
                shard_chunk_ids.clear();
                shard_bytes = lua_empty_shard_len();
            }

            let next_shard_bytes =
                shard_bytes + fragment_bytes + usize::from(!shard_buffer.is_empty());
            shard_buffer.push(Value::Object(fragment));
            shard_chunk_ids.insert(fragment_chunk_id);
            shard_bytes = next_shard_bytes;
        }
    }

    if !shard_buffer.is_empty() {
        flush_shard_buffer(
            &shard_dir,
            &options.index_name,
            &mut shard_buffer,
            &mut chunk_refs,
            &chunk_ref_index_by_id,
            &mut shard_names,
            &mut next_shard_index,
        )?;
    }

    let index_path = output_dir.join(format!("{}.lua", options.index_name));
    write_lua_module(
        &index_path,
        &Value::Object(Map::from_iter([
            (
                "schemaVersion".to_string(),
                Value::String(meta.schema_version.clone()),
            ),
            ("meta".to_string(), build_meta_value(&meta)),
            (
                "shardFolder".to_string(),
                Value::String(options.shard_folder.clone()),
            ),
            (
                "shards".to_string(),
                Value::Array(
                    shard_names
                        .iter()
                        .cloned()
                        .map(Value::String)
                        .collect::<Vec<_>>(),
                ),
            ),
            (
                "chunkCount".to_string(),
                Value::from(chunk_refs.len() as u64),
            ),
            (
                "fragmentCount".to_string(),
                Value::from(fragment_count as u64),
            ),
            (
                "chunksPerShard".to_string(),
                Value::from(options.chunks_per_shard as u64),
            ),
            ("chunkRefs".to_string(), Value::Array(chunk_refs)),
        ])),
    )?;

    Ok(RuntimeLuaShardsStats {
        chunk_count: chunk_ref_index_by_id.len(),
        fragment_count,
        shard_count: shard_names.len(),
        index_path,
        shard_dir,
    })
}

fn parse_chunk_object(record: &StoredChunkRecord) -> ManifestStoreResult<Map<String, Value>> {
    let value: Value = serde_json::from_str(&record.chunk_json)?;
    match value {
        Value::Object(object) => Ok(object),
        _ => Err(format!(
            "manifest store chunk {} does not contain a JSON object",
            record.chunk_id
        )
        .into()),
    }
}

fn build_chunk_ref_value(record: &StoredChunkRecord) -> ManifestStoreResult<Value> {
    let mut object = Map::new();
    object.insert("id".to_string(), Value::String(record.chunk_id.clone()));
    object.insert(
        "originStuds".to_string(),
        Value::Object(Map::from_iter([
            ("x".to_string(), Value::from(record.origin_studs.x)),
            ("y".to_string(), Value::from(record.origin_studs.y)),
            ("z".to_string(), Value::from(record.origin_studs.z)),
        ])),
    );
    object.insert("shards".to_string(), Value::Array(Vec::new()));
    object.insert(
        "featureCount".to_string(),
        Value::from(record.feature_count as u64),
    );
    object.insert(
        "streamingCost".to_string(),
        Value::from(record.streaming_cost),
    );
    if let Some(estimated_memory_cost) = record.estimated_memory_cost {
        object.insert(
            "estimatedMemoryCost".to_string(),
            Value::from(estimated_memory_cost),
        );
    }
    object.insert(
        "partitionVersion".to_string(),
        Value::String(record.partition_version.clone()),
    );
    object.insert(
        "subplans".to_string(),
        serde_json::from_str::<Value>(&record.subplans_json)?,
    );
    Ok(Value::Object(object))
}

fn build_meta_value(meta: &crate::manifest_store::StoredManifestMeta) -> Value {
    Value::Object(Map::from_iter([
        (
            "worldName".to_string(),
            Value::String(meta.world_name.clone()),
        ),
        (
            "generator".to_string(),
            Value::String(meta.generator.clone()),
        ),
        ("source".to_string(), Value::String(meta.source.clone())),
        (
            "metersPerStud".to_string(),
            Value::from(meta.meters_per_stud),
        ),
        (
            "chunkSizeStuds".to_string(),
            Value::from(meta.chunk_size_studs as i64),
        ),
        (
            "bbox".to_string(),
            Value::Object(Map::from_iter([
                ("minLat".to_string(), Value::from(meta.bbox.min.lat)),
                ("minLon".to_string(), Value::from(meta.bbox.min.lon)),
                ("maxLat".to_string(), Value::from(meta.bbox.max.lat)),
                ("maxLon".to_string(), Value::from(meta.bbox.max.lon)),
            ])),
        ),
        (
            "totalFeatures".to_string(),
            Value::from(meta.total_features as u64),
        ),
        (
            "notes".to_string(),
            Value::Array(
                meta.notes
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect::<Vec<_>>(),
            ),
        ),
    ]))
}

fn flush_shard_buffer(
    shard_dir: &Path,
    index_name: &str,
    shard_buffer: &mut Vec<Value>,
    chunk_refs: &mut [Value],
    chunk_ref_index_by_id: &HashMap<String, usize>,
    shard_names: &mut Vec<String>,
    next_shard_index: &mut usize,
) -> ManifestStoreResult<()> {
    let shard_name = format!("{index_name}_{:03}", *next_shard_index);
    *next_shard_index += 1;
    let shard_path = shard_dir.join(format!("{shard_name}.lua"));

    for fragment in shard_buffer.iter() {
        let chunk_id = fragment
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "runtime fragment is missing string id".to_string())?;
        let ref_index = *chunk_ref_index_by_id
            .get(chunk_id)
            .ok_or_else(|| format!("missing chunkRef metadata for fragment chunk {chunk_id}"))?;
        let chunk_ref = chunk_refs
            .get_mut(ref_index)
            .and_then(Value::as_object_mut)
            .ok_or_else(|| format!("malformed chunkRef metadata for chunk {chunk_id}"))?;
        let shards = chunk_ref
            .get_mut("shards")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| format!("malformed shard list for chunk {chunk_id}"))?;
        shards.push(Value::String(shard_name.clone()));
    }

    write_lua_module(
        &shard_path,
        &Value::Object(Map::from_iter([(
            "chunks".to_string(),
            Value::Array(std::mem::take(shard_buffer)),
        )])),
    )?;
    shard_names.push(shard_name);
    Ok(())
}

fn clear_existing_shards(shard_dir: &Path, index_name: &str) -> ManifestStoreResult<()> {
    if shard_dir.exists() {
        for entry in fs::read_dir(shard_dir)? {
            let path = entry?.path();
            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if file_name.starts_with(&format!("{index_name}_")) && file_name.ends_with(".lua") {
                fs::remove_file(path)?;
            }
        }
    } else {
        fs::create_dir_all(shard_dir)?;
    }
    Ok(())
}

fn fragment_chunk_for_lua_shards(
    chunk: &Map<String, Value>,
    max_bytes: Option<usize>,
) -> ManifestStoreResult<Vec<Map<String, Value>>> {
    if max_bytes.is_none() {
        return Ok(vec![chunk.clone()]);
    }
    let max_bytes = max_bytes.unwrap();
    let mut fragments = Vec::new();

    let base_fragment = base_chunk_fragment(chunk);
    if lua_payload_len(&Value::Object(Map::from_iter([(
        "chunks".to_string(),
        Value::Array(vec![Value::Object(base_fragment.clone())]),
    )])))
        > max_bytes
    {
        return Err(format!(
            "runtime chunk {} base metadata exceeds max bytes {max_bytes}",
            chunk_id(chunk)?
        )
        .into());
    }
    fragments.push(base_fragment);

    if let Some(Value::Object(terrain)) = chunk.get("terrain") {
        for terrain_key in ["heights", "materials"] {
            let Some(terrain_value) = terrain.get(terrain_key) else {
                continue;
            };
            match terrain_value {
                Value::Array(items) => {
                    let terrain_fragments = fragment_list_payloads(
                        chunk_id(chunk)?,
                        items,
                        max_bytes,
                        &format!("terrain field {terrain_key}"),
                        |fragment_items| {
                            Map::from_iter([
                                (
                                    "id".to_string(),
                                    Value::String(chunk_id(chunk).unwrap().to_string()),
                                ),
                                (
                                    "terrain".to_string(),
                                    Value::Object(Map::from_iter([(
                                        terrain_key.to_string(),
                                        Value::Array(fragment_items.to_vec()),
                                    )])),
                                ),
                            ])
                        },
                    )?;
                    fragments.extend(terrain_fragments);
                }
                other => {
                    let fragment = Map::from_iter([
                        (
                            "id".to_string(),
                            Value::String(chunk_id(chunk)?.to_string()),
                        ),
                        (
                            "terrain".to_string(),
                            Value::Object(Map::from_iter([(
                                terrain_key.to_string(),
                                other.clone(),
                            )])),
                        ),
                    ]);
                    if lua_payload_len(&Value::Object(Map::from_iter([(
                        "chunks".to_string(),
                        Value::Array(vec![Value::Object(fragment.clone())]),
                    )])))
                        > max_bytes
                    {
                        return Err(format!(
                            "runtime chunk {} terrain field {terrain_key} exceeds max bytes {max_bytes}",
                            chunk_id(chunk)?
                        )
                        .into());
                    }
                    fragments.push(fragment);
                }
            }
        }
    }

    for field in CHUNK_LIST_FIELDS {
        let Some(Value::Array(items)) = chunk.get(field) else {
            continue;
        };
        if items.is_empty() {
            continue;
        }

        let list_fragments = fragment_list_payloads(
            chunk_id(chunk)?,
            items,
            max_bytes,
            &format!("field {field}"),
            |fragment_items| {
                Map::from_iter([
                    (
                        "id".to_string(),
                        Value::String(chunk_id(chunk).unwrap().to_string()),
                    ),
                    (field.to_string(), Value::Array(fragment_items.to_vec())),
                ])
            },
        )?;
        fragments.extend(list_fragments);
    }

    Ok(fragments)
}

fn fragment_list_payloads<F>(
    chunk_id: &str,
    values: &[Value],
    max_bytes: usize,
    field_label: &str,
    fragment_builder: F,
) -> ManifestStoreResult<Vec<Map<String, Value>>>
where
    F: Fn(&[Value]) -> Map<String, Value>,
{
    let empty_fragment = fragment_builder(&[]);
    let empty_len = lua_payload_len(&Value::Object(Map::from_iter([(
        "chunks".to_string(),
        Value::Array(vec![Value::Object(empty_fragment)]),
    )])));
    let item_lengths = values.iter().map(lua_value_len).collect::<Vec<_>>();

    let mut fragments = Vec::new();
    let mut start = 0usize;
    let mut current_len = empty_len;
    let mut current_count = 0usize;

    for (index, item_len) in item_lengths.iter().enumerate() {
        let next_len = current_len + item_len + usize::from(current_count > 0);
        if current_count == 0 {
            if next_len > max_bytes {
                return Err(format!(
                    "runtime chunk {chunk_id} {field_label} contains an entry larger than max bytes {max_bytes}"
                )
                .into());
            }
            current_len = next_len;
            current_count = 1;
            continue;
        }

        if next_len > max_bytes {
            fragments.push(fragment_builder(&values[start..index]));
            start = index;
            current_len = empty_len + item_len;
            current_count = 1;
            continue;
        }

        current_len = next_len;
        current_count += 1;
    }

    if current_count > 0 {
        fragments.push(fragment_builder(&values[start..]));
    }

    Ok(fragments)
}

fn base_chunk_fragment(chunk: &Map<String, Value>) -> Map<String, Value> {
    let mut fragment = Map::new();
    fragment.insert(
        "id".to_string(),
        chunk.get("id").cloned().unwrap_or(Value::Null),
    );
    for (key, value) in chunk {
        if key == "id" || key == "partitionVersion" || key == "subplans" {
            continue;
        }
        if CHUNK_LIST_FIELDS.contains(&key.as_str()) && value.is_array() {
            continue;
        }
        if key == "terrain" {
            if let Value::Object(terrain) = value {
                let terrain_fragment = terrain
                    .iter()
                    .filter(|(nested_key, _)| {
                        *nested_key != "heights" && *nested_key != "materials"
                    })
                    .map(|(nested_key, nested_value)| (nested_key.clone(), nested_value.clone()))
                    .collect::<Map<String, Value>>();
                if !terrain_fragment.is_empty() {
                    fragment.insert("terrain".to_string(), Value::Object(terrain_fragment));
                }
                continue;
            }
        }
        fragment.insert(key.clone(), value.clone());
    }
    fragment
}

fn chunk_id(chunk: &Map<String, Value>) -> ManifestStoreResult<&str> {
    chunk
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "runtime chunk is missing string id".to_string().into())
}

fn write_lua_module(path: &Path, data: &Value) -> ManifestStoreResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut output = String::from("return ");
    write_lua_value(data, &mut output);
    output.push('\n');
    fs::write(path, output)?;
    Ok(())
}

fn lua_payload_len(value: &Value) -> usize {
    let mut output = String::from("return ");
    write_lua_value(value, &mut output);
    output.push('\n');
    output.len()
}

fn lua_empty_shard_len() -> usize {
    lua_payload_len(&Value::Object(Map::from_iter([(
        "chunks".to_string(),
        Value::Array(Vec::new()),
    )])))
}

fn lua_value_len(value: &Value) -> usize {
    let mut output = String::new();
    write_lua_value(value, &mut output);
    output.len()
}

fn write_lua_value(value: &Value, output: &mut String) {
    match value {
        Value::Object(object) => {
            output.push('{');
            for (index, (key, nested)) in object.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_lua_key(key, output);
                output.push('=');
                write_lua_value(nested, output);
            }
            output.push('}');
        }
        Value::Array(items) => {
            output.push('{');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_lua_value(item, output);
            }
            output.push('}');
        }
        Value::String(text) => write_lua_string(text, output),
        Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        Value::Null => output.push_str("nil"),
        Value::Number(number) => output.push_str(&number.to_string()),
    }
}

fn write_lua_key(key: &str, output: &mut String) {
    if is_lua_identifier(key) {
        output.push_str(key);
        return;
    }
    output.push('[');
    write_lua_string(key, output);
    output.push(']');
}

fn is_lua_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn write_lua_string(value: &str, output: &mut String) {
    output.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            _ => output.push(ch),
        }
    }
    output.push('"');
}
