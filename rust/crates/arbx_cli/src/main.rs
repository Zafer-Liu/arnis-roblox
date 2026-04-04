use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use arbx_geo::{
    BoundingBox, ElevationProvider, FlatElevationProvider, HgtElevationProvider,
    OffsetElevationProvider, TerrariumElevationProvider,
};
use arbx_pipeline::{
    write_source_truth_pack_sqlite, write_source_truth_pack_summary, ElevationEnrichmentStage,
    NormalizeStage, PipelineContext, SourceTruthPack, SourceTruthPackSummary, TriangulateStage,
    ValidateStage,
};
use arbx_planetary_store::{
    attach_truth_pack_summary, build_delivery_plan_around_geo_point,
    build_delivery_plan_around_point, build_delivery_plan_for_scene_bbox,
    build_delivery_plan_for_tile, build_delivery_window_around_geo_point,
    build_delivery_window_around_point, build_delivery_window_for_scene_bbox,
    build_delivery_window_for_tile, build_delivery_window_from_plan, build_hydrated_route_session,
    build_route_delivery_schedule, build_route_delivery_session_for_geo_points,
    find_best_scene_covering_geo_point, find_best_scene_covering_tile,
    find_scenes_covering_geo_point, find_scenes_covering_tile, find_scenes_intersecting_geo_bbox,
    ingest_manifest_json, ingest_manifest_sqlite, init_planetary_store, list_scenes,
    merge_delivery_plans, read_chunk_summaries_by_refs, read_chunks_by_ids,
    read_scene_catalog_entry, read_scene_chunk_subset, read_scene_chunk_summary_around_geo_point,
    read_scene_chunk_summary_around_point, read_scene_chunk_summary_for_tile,
    read_scene_chunk_summary_subset, read_scene_manifest_subset_around_geo_point,
    read_scene_manifest_subset_by_chunk_ids, summarize_planetary_store,
};
use arbx_roblox_export::{
    build_sample_multi_chunk, export_to_chunks, read_manifest_sqlite_all, write_manifest_sqlite,
    write_runtime_lua_shards_from_sqlite, write_runtime_lua_shards_from_stored_subset,
    ChunkManifest, ExportConfig, RuntimeLuaShardsOptions, SatelliteTileProvider,
    StoredManifestSubset,
};
use rayon::prelude::*;
use serde::Serialize;
use serde_json::{json, Value};

const SCENE_INDEX_VERSION: u64 = 2;

#[derive(Debug, Serialize)]
struct DeliveryBundleRuntimeResult {
    chunk_count: usize,
    fragment_count: usize,
    shard_count: usize,
    index_path: String,
    shard_dir: String,
}

#[derive(Debug, Serialize)]
struct DeliveryBundleResult {
    plan: arbx_planetary_store::PlanetaryDeliveryPlan,
    out: Option<String>,
    plan_out: Option<String>,
    route_session_out: Option<String>,
    hydrated_route_out: Option<String>,
    schedule_out: Option<String>,
    step_lane_dir: Option<String>,
    step_summary_dir: Option<String>,
    step_window_dir: Option<String>,
    step_transition_dir: Option<String>,
    summary_out: Option<String>,
    window_out: Option<String>,
    manifest_out: Option<String>,
    runtime: Option<DeliveryBundleRuntimeResult>,
}

fn srtm_tile_name(lat: f64, lon: f64) -> String {
    let lat_i = lat.floor() as i32;
    let lon_i = lon.floor() as i32;
    let ns = if lat_i >= 0 { "N" } else { "S" };
    let ew = if lon_i >= 0 { "E" } else { "W" };
    format!("{}{:02}{}{:03}", ns, lat_i.abs(), ew, lon_i.abs())
}

fn help_text() -> String {
    r#"arbx_cli — Arnis HD Pipeline

Generates high-fidelity Roblox world manifests from real-world geodata.
Works for any location on Earth. Outputs Schema 0.4.0 JSON manifests.

USAGE:
  arbx_cli <COMMAND> [OPTIONS]

COMMANDS:
  compile    Build a chunk manifest from geodata sources
  sample     Emit a synthetic sample manifest for testing
  stats      Print statistics for a manifest file
  validate   Validate a manifest against the schema
  diff       Compare two manifest files
  scene-index Build a compact per-chunk scene-audit summary
  scene-audit Compare Studio scene markers against manifest truth
  emit-runtime-lua Emit bounded runtime Lua shards directly from a manifest SQLite store
  planetary-store Manage the canonical planetary SQLite store
  config     Emit a default world configuration JSON
  explain    Print the full pipeline architecture for agents

COMPILE OPTIONS:
  --source PATH          Input Overpass JSON file (omit for synthetic data)
  --live                 Fetch live from Overpass API (auto-cached to --cache-dir)
  --bbox S,W,N,E         Bounding box: min_lat,min_lon,max_lat,max_lon
                         Example: --bbox 30.26,-97.75,30.27,-97.74 (Austin TX)
  --out PATH             Output manifest file (default: stdout)
  --sqlite-out PATH      Also write a SQLite manifest store
  --truth-pack-out PATH  Write bounded source truth-pack SQLite (Overpass/live only)
  --truth-pack-summary-out PATH
                         Write compact source truth-pack summary JSON
  --world-name NAME      World name in manifest metadata (default: ExportedWorld)
  --meters-per-stud N    Scale factor (default: 0.3 = Roblox humanoid proportional)
  --terrain-cell-size N  Terrain grid precision in studs (default: 2, range: 1-32)
                         Lower = more detailed terrain, more memory
  --satellite [DIR]      Enable satellite material classification
                         Fetches ESRI z19 imagery, caches to DIR (default: out/tiles/satellite)
  --cache-dir PATH       Overpass API response cache (default: out/overpass)

QUALITY PROFILES:
  --profile insane       cell=1 sat=on  (256x256 grid, ~2GB RAM, M5 Max / workstation)
  --profile high         cell=2 sat=off (128x128 grid, ~512MB RAM) [default bounded detail]
  --profile balanced     cell=4 sat=off (64x64 grid, ~128MB RAM, 8GB machines)
  --profile fast         cell=8 sat=off (32x32 grid, ~32MB RAM, CI/testing)
  --yolo                 Alias for --profile insane

SAMPLE OPTIONS:
  --out PATH             Output file (default: stdout)
  --sqlite-out PATH      Also write a SQLite manifest store
  --grid X,Z             Multi-chunk grid dimensions (default: 1,1)

OTHER:
  --help, -h             Show this help
  --version, -V          Show version

EXAMPLES:
  # Austin downtown, maximum fidelity
  arbx_cli compile --source data/austin_overpass.json --yolo --out out/austin.json

  # Live fetch any city, high quality
  arbx_cli compile --live --bbox 35.68,139.75,35.69,139.76 --world-name Tokyo --out out/tokyo.json

  # CI/testing: fast synthetic export
  arbx_cli compile --profile fast --out out/test.json

  # Validate an existing manifest
  arbx_cli validate out/austin.json

  # Compare two exports
  arbx_cli diff out/v1.json out/v2.json

  # Build a compact chunk summary for fast scene audits
  arbx_cli scene-index --manifest out/austin.json --json-out out/austin.scene-index.json
  arbx_cli scene-index --manifest-sqlite out/austin.sqlite --json-out out/austin.scene-index.json

  # Compare Studio scene markers against manifest truth
  arbx_cli scene-audit --manifest-summary out/austin.scene-index.json --log studio.log --marker ARNIS_SCENE_EDIT
  arbx_cli scene-audit --manifest-sqlite out/austin.sqlite --log studio.log --marker ARNIS_SCENE_EDIT

  # Get pipeline info (for AI agents)
  arbx_cli explain

  # Initialize and ingest into the canonical planetary store
  arbx_cli planetary-store init --store out/planetary.sqlite
  arbx_cli planetary-store ingest-manifest --store out/planetary.sqlite --manifest-sqlite out/austin.sqlite --scene austin
  arbx_cli planetary-store subset --store out/planetary.sqlite --scene austin --bbox-studs 0,0,512,512

OUTPUT FORMAT:
  Schema 0.4.0 JSON manifest with:
  - metersPerStud: 0.3 (configurable)
  - Chunks with terrain grids, roads, buildings, water, props, landuse, barriers
  - DEM-derived elevation for all features
  - Satellite-classified roof/ground materials (when --satellite is used)
  - All coordinates in stud-space relative to chunk origins

EXIT CODES:
  0  Success
  1  Error (message on stderr)
"#
    .to_string()
}

fn print_help() {
    print!("{}", help_text());
}

fn write_manifest_outputs(
    manifest: &ChunkManifest,
    json_out: Option<&PathBuf>,
    sqlite_out: Option<&PathBuf>,
    verb: &str,
    duration: std::time::Duration,
) -> Result<(), String> {
    if let Some(path) = json_out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| format!("create dir failed: {err}"))?;
        }
        fs::write(path, manifest.to_json_pretty()).map_err(|err| format!("write failed: {err}"))?;
        println!("{verb} and wrote {} in {:?}", path.display(), duration);
    } else if sqlite_out.is_none() {
        print!("{}", manifest.to_json_pretty());
        eprintln!("{verb} in {:?}", duration);
    }

    if let Some(path) = sqlite_out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| format!("create dir failed: {err}"))?;
        }
        write_manifest_sqlite(manifest, path)
            .map_err(|err| format!("sqlite write failed: {err}"))?;
        println!(
            "Wrote SQLite manifest store {} in {:?}",
            path.display(),
            duration
        );
    }

    Ok(())
}

fn stem_without_suffixes(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    for suffix in [
        ".truth-pack.summary.json",
        ".truth-pack.sqlite",
        "-manifest.sqlite",
        "-manifest.json",
        ".sqlite",
        ".json",
    ] {
        if let Some(stem) = name.strip_suffix(suffix) {
            return Some(stem.to_string());
        }
    }
    path.file_stem()
        .and_then(|value| value.to_str())
        .map(|value| value.to_string())
}

fn infer_truth_pack_scene_name(
    truth_pack_out: Option<&PathBuf>,
    truth_pack_summary_out: Option<&PathBuf>,
    manifest_out: Option<&PathBuf>,
    manifest_sqlite_out: Option<&PathBuf>,
    world_name: &str,
) -> String {
    for path in [
        truth_pack_out,
        truth_pack_summary_out,
        manifest_out,
        manifest_sqlite_out,
    ]
    .into_iter()
    .flatten()
    {
        if let Some(stem) = stem_without_suffixes(path) {
            return stem;
        }
    }
    world_name.to_lowercase()
}

fn write_truth_pack_outputs(
    truth_pack: &SourceTruthPack,
    truth_pack_out: Option<&PathBuf>,
    truth_pack_summary_out: Option<&PathBuf>,
    manifest_out: Option<&PathBuf>,
    manifest_sqlite_out: Option<&PathBuf>,
    world_name: &str,
) -> Result<(), String> {
    let scene = infer_truth_pack_scene_name(
        truth_pack_out,
        truth_pack_summary_out,
        manifest_out,
        manifest_sqlite_out,
        world_name,
    );
    let summary = truth_pack.summary(scene);

    if let Some(path) = truth_pack_out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| format!("create dir failed: {err}"))?;
        }
        write_source_truth_pack_sqlite(truth_pack, path)
            .map_err(|err| format!("truth-pack sqlite write failed: {err:?}"))?;
        println!("Wrote source truth-pack {}", path.display());
    }

    if let Some(path) = truth_pack_summary_out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| format!("create dir failed: {err}"))?;
        }
        write_source_truth_pack_summary(&summary, path)
            .map_err(|err| format!("truth-pack summary write failed: {err:?}"))?;
        println!("Wrote source truth-pack summary {}", path.display());
    }

    Ok(())
}

fn cmd_sample(args: &[String]) -> Result<(), String> {
    let mut out_path: Option<PathBuf> = None;
    let mut sqlite_out_path: Option<PathBuf> = None;
    let mut grid = (1, 1);

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                let value = args.get(i + 1).ok_or("--out requires a path")?;
                out_path = Some(PathBuf::from(value));
                i += 2;
            }
            "--sqlite-out" => {
                let value = args.get(i + 1).ok_or("--sqlite-out requires a path")?;
                sqlite_out_path = Some(PathBuf::from(value));
                i += 2;
            }
            "--grid" => {
                let value = args.get(i + 1).ok_or("--grid requires X,Z")?;
                let parts: Vec<&str> = value.split(',').collect();
                if parts.len() != 2 {
                    return Err("--grid requires X,Z format".to_string());
                }
                let x = parts[0].parse::<i32>().map_err(|_| "invalid X in grid")?;
                let z = parts[1].parse::<i32>().map_err(|_| "invalid Z in grid")?;
                grid = (x, z);
                i += 2;
            }
            other => {
                return Err(format!("unknown argument to sample: {other}"));
            }
        }
    }

    let start = Instant::now();
    let manifest = build_sample_multi_chunk(grid.0, grid.1);
    let duration = start.elapsed();

    write_manifest_outputs(
        &manifest,
        out_path.as_ref(),
        sqlite_out_path.as_ref(),
        "Generated",
        duration,
    )
}

fn cmd_compile(args: &[String]) -> Result<(), String> {
    let mut out_path: Option<PathBuf> = None;
    let mut sqlite_out_path: Option<PathBuf> = None;
    let mut truth_pack_out_path: Option<PathBuf> = None;
    let mut truth_pack_summary_out_path: Option<PathBuf> = None;
    let mut source_path: Option<PathBuf> = None;
    // Default bbox covers downtown Austin. Overridden by --bbox to match the OSM fetch area.
    let mut bbox = BoundingBox::new(30.26, -97.75, 30.27, -97.74);
    // 0.3 = Roblox humanoid-scale. Use --meters-per-stud to override.
    let mut meters_per_stud: f64 = 0.3;
    // --live: fetch from the Overpass API instead of a local file.
    let mut live = false;
    // --cache-dir: where to store cached Overpass responses (default: out/overpass).
    let mut cache_dir = "out/overpass".to_string();
    // --satellite: optional satellite tile directory for material enrichment.
    let mut satellite_dir: Option<String> = None;
    // --terrain-cell-size: terrain grid cell size in studs (2 = high, 4 = balanced, 8 = fast).
    let mut terrain_cell_size: i32 = 2;
    // --world-name: name written into the manifest meta.
    let mut world_name = "ExportedWorld".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                let value = args.get(i + 1).ok_or("--out requires a path")?;
                out_path = Some(PathBuf::from(value));
                i += 2;
            }
            "--sqlite-out" => {
                let value = args.get(i + 1).ok_or("--sqlite-out requires a path")?;
                sqlite_out_path = Some(PathBuf::from(value));
                i += 2;
            }
            "--truth-pack-out" => {
                let value = args.get(i + 1).ok_or("--truth-pack-out requires a path")?;
                truth_pack_out_path = Some(PathBuf::from(value));
                i += 2;
            }
            "--truth-pack-summary-out" => {
                let value = args
                    .get(i + 1)
                    .ok_or("--truth-pack-summary-out requires a path")?;
                truth_pack_summary_out_path = Some(PathBuf::from(value));
                i += 2;
            }
            "--source" => {
                let value = args.get(i + 1).ok_or("--source requires a path")?;
                source_path = Some(PathBuf::from(value));
                i += 2;
            }
            "--bbox" => {
                let value = args
                    .get(i + 1)
                    .ok_or("--bbox requires MIN_LAT,MIN_LON,MAX_LAT,MAX_LON")?;
                let p: Vec<f64> = value
                    .split(',')
                    .map(|s| {
                        s.trim()
                            .parse::<f64>()
                            .map_err(|_| format!("invalid number in bbox: {}", s))
                    })
                    .collect::<Result<Vec<f64>, String>>()?;
                if p.len() != 4 {
                    return Err("--bbox requires 4 values".to_string());
                }
                bbox = BoundingBox::new(p[0], p[1], p[2], p[3]);
                i += 2;
            }
            "--world-name" => {
                world_name = args
                    .get(i + 1)
                    .ok_or("--world-name requires a name")?
                    .clone();
                i += 2;
            }
            "--meters-per-stud" => {
                let value = args
                    .get(i + 1)
                    .ok_or("--meters-per-stud requires a number")?;
                meters_per_stud = value
                    .parse::<f64>()
                    .map_err(|_| format!("invalid --meters-per-stud value: {value}"))?;
                if meters_per_stud <= 0.0 {
                    return Err("--meters-per-stud must be positive".to_string());
                }
                i += 2;
            }
            "--live" => {
                live = true;
                i += 1;
            }
            "--cache-dir" => {
                let value = args.get(i + 1).ok_or("--cache-dir requires a path")?;
                cache_dir = value.clone();
                i += 2;
            }
            "--terrain-cell-size" => {
                let value = args
                    .get(i + 1)
                    .ok_or("--terrain-cell-size requires a number")?;
                terrain_cell_size = value
                    .parse::<i32>()
                    .map_err(|_| format!("invalid --terrain-cell-size: {value}"))?;
                if !(1..=32).contains(&terrain_cell_size) {
                    return Err("--terrain-cell-size must be 1-32".to_string());
                }
                i += 2;
            }
            "--profile" => {
                let profile = args.get(i + 1).ok_or("--profile requires a preset name")?;
                apply_compile_profile(profile, &mut terrain_cell_size, &mut satellite_dir)?;
                i += 2;
            }
            "--yolo" => {
                terrain_cell_size = 1; // 256×256 grid = 65,536 cells per chunk
                                       // satellite enabled automatically
                if satellite_dir.is_none() {
                    satellite_dir = Some("out/tiles/satellite".to_string());
                }
                eprintln!(
                    "YOLO MODE (--profile insane): terrain cell=1, satellite=on, maximum fidelity"
                );
                i += 1;
            }
            "--satellite" => {
                // Optional tile directory argument: use it if the next token doesn't start with '-'
                if let Some(next) = args.get(i + 1) {
                    if !next.starts_with('-') {
                        satellite_dir = Some(next.clone());
                        i += 2;
                        continue;
                    }
                }
                satellite_dir = Some("out/tiles/satellite".to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown argument to compile: {other}"));
            }
        }
    }

    let start = Instant::now();
    let wants_truth_pack = truth_pack_out_path.is_some() || truth_pack_summary_out_path.is_some();

    let adapter: Box<dyn arbx_pipeline::SourceAdapter> = if let Some(path) = &source_path {
        // --source always uses the file-based adapter regardless of --live
        if path.to_string_lossy().ends_with(".json") {
            let content =
                fs::read_to_string(path).map_err(|e| format!("failed to read source: {}", e))?;
            if content.contains("\"elements\"") {
                Box::new(arbx_pipeline::OverpassAdapter {
                    path: path.clone(),
                    meters_per_stud,
                })
            } else {
                Box::new(arbx_pipeline::FileSourceAdapter { path: path.clone() })
            }
        } else {
            Box::new(arbx_pipeline::FileSourceAdapter { path: path.clone() })
        }
    } else if live {
        // --live with no --source: fetch from the Overpass API
        Box::new(arbx_pipeline::LiveOverpassAdapter {
            bbox,
            meters_per_stud,
            cache_dir,
        })
    } else {
        Box::new(arbx_pipeline::SyntheticAustinAdapter { meters_per_stud })
    };

    println!(
        "Compiling from {}... (meters_per_stud={meters_per_stud})",
        adapter.name()
    );

    let config = ExportConfig {
        meters_per_stud,
        terrain_cell_size,
        world_name: world_name.clone(),
        ..ExportConfig::default()
    };

    // ── Create elevation provider BEFORE the pipeline so the enrichment
    //    stage can inject DEM-derived Y values into every feature. ──────────

    // Compute the SRTM tile name from the bbox center (supports any worldwide location).
    let tile_name = srtm_tile_name(bbox.center().lat, bbox.center().lon);
    let hgt_path = PathBuf::from(format!("data/{}.hgt", tile_name));
    if !hgt_path.exists() {
        eprintln!("Attempting to download SRTM elevation tile {tile_name}.hgt...");
        let gz_path = PathBuf::from(format!("data/{}.hgt.gz", tile_name));
        let url = format!(
            "https://s3.amazonaws.com/elevation-tiles-prod/skadi/{}/{}.hgt.gz",
            &tile_name[..3],
            tile_name
        );
        let status = std::process::Command::new("curl")
            .args([
                "-L",
                "-o",
                gz_path.to_str().unwrap(),
                url.as_str(),
                "--silent",
                "--fail",
                "--user-agent",
                "arnis-roblox/1.0 (open-source educational project)",
                "--retry",
                "3",
                "--retry-delay",
                "5",
            ])
            .status();
        if status.map(|s| s.success()).unwrap_or(false) {
            let _ = std::process::Command::new("gunzip")
                .arg(gz_path.to_str().unwrap())
                .status();
            if hgt_path.exists() {
                eprintln!("Downloaded {tile_name}.hgt successfully.");
            }
        } else {
            eprintln!("Could not download SRTM tile, using flat elevation.");
        }
    }

    let elevation: Box<dyn arbx_geo::ElevationProvider> = {
        // Try Terrarium tiles first (no API key required, auto-cached).
        match TerrariumElevationProvider::new(&bbox, TerrariumElevationProvider::DEFAULT_ZOOM) {
            Ok(terrarium) => {
                let center = bbox.center();
                let base = terrarium.sample_height_at(center);
                eprintln!(
                    "Using Terrarium elevation, base offset = {:.1}m at bbox center",
                    base
                );
                Box::new(OffsetElevationProvider::new(Box::new(terrarium), base))
            }
            Err(e) => {
                eprintln!("Terrarium tiles unavailable ({e}), falling back to SRTM/flat.");
                if hgt_path.exists() {
                    let hgt = HgtElevationProvider::new(PathBuf::from("data"));
                    let base = hgt.sample_height_at(bbox.center());
                    eprintln!(
                        "Using SRTM elevation, base offset = {:.1}m at bbox center",
                        base
                    );
                    Box::new(OffsetElevationProvider::new(Box::new(hgt), base))
                } else {
                    eprintln!("No SRTM tile found, using flat elevation.");
                    Box::new(FlatElevationProvider { height: 0.0 })
                }
            }
        }
    };

    // ── Run pipeline with elevation enrichment as the final stage ──────────
    let enrichment = ElevationEnrichmentStage {
        elevation: elevation.as_ref(),
        meters_per_stud,
        bbox_center: bbox.center(),
    };

    let stages: [&dyn arbx_pipeline::PipelineStage; 4] = [
        &ValidateStage,
        &NormalizeStage,
        &TriangulateStage,
        &enrichment,
    ];

    let (source_features, truth_pack_opt) = adapter
        .load_features_and_truth_pack(bbox)
        .map_err(|e| format!("source load failed: {:?}", e))?;
    let truth_pack = if wants_truth_pack {
        Some(truth_pack_opt.ok_or_else(|| {
            "truth-pack output is only supported for OverpassAdapter / LiveOverpassAdapter compile sources".to_string()
        })?)
    } else {
        None
    };

    let mut ctx = PipelineContext::new(bbox, source_features);
    for stage in &stages {
        stage
            .run(&mut ctx)
            .map_err(|e| format!("pipeline failed: {:?}", e))?;
    }

    let mut sat_provider = satellite_dir.as_deref().map(SatelliteTileProvider::new);
    let manifest = export_to_chunks(
        ctx.features,
        ctx.bbox,
        &config,
        elevation.as_ref(),
        sat_provider.as_mut(),
    );
    let duration = start.elapsed();
    // Rust export remains the single authoritative partition function for additive chunkRefs metadata.
    write_manifest_outputs(
        &manifest,
        out_path.as_ref(),
        sqlite_out_path.as_ref(),
        "Compiled",
        duration,
    )?;

    if let Some(truth_pack) = truth_pack.as_ref() {
        write_truth_pack_outputs(
            truth_pack,
            truth_pack_out_path.as_ref(),
            truth_pack_summary_out_path.as_ref(),
            out_path.as_ref(),
            sqlite_out_path.as_ref(),
            &world_name,
        )?;
    }

    Ok(())
}

fn apply_compile_profile(
    profile: &str,
    terrain_cell_size: &mut i32,
    satellite_dir: &mut Option<String>,
) -> Result<(), String> {
    match profile {
        "insane" => {
            *terrain_cell_size = 1;
            if satellite_dir.is_none() {
                *satellite_dir = Some("out/tiles/satellite".to_string());
            }
            eprintln!("Profile: insane — cell=1, satellite=on (36GB+ RAM)");
        }
        "high" => {
            *terrain_cell_size = 2;
            eprintln!("Profile: high — cell=2, satellite=off (16GB+ RAM)");
        }
        "balanced" => {
            *terrain_cell_size = 4;
            eprintln!("Profile: balanced — cell=4 (8GB+ RAM)");
        }
        "fast" => {
            *terrain_cell_size = 8;
            eprintln!("Profile: fast — cell=8 (4GB+ RAM)");
        }
        other => {
            return Err(format!(
                "unknown profile: {other} (valid: insane, high, balanced, fast)"
            ));
        }
    }

    Ok(())
}

fn cmd_config(args: &[String]) -> Result<(), String> {
    let mut out_path: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                let value = args.get(i + 1).ok_or("--out requires a path")?;
                out_path = Some(PathBuf::from(value));
                i += 2;
            }
            other => {
                return Err(format!("unknown argument to config: {other}"));
            }
        }
    }

    let config_json = r#"{
  "metersPerStud": 0.3,
  "chunkSizeStuds": 256,
  "terrainMode": "voxel",
  "roadMode": "mesh",
  "buildingMode": "shellMesh",
  "streamingEnabled": true,
  "streamingTargetRadius": 1024,
  "instanceBudget": {
    "maxPerChunk": 1500,
    "maxPropsPerChunk": 250
  }
}"#;

    if let Some(path) = out_path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| format!("create dir failed: {err}"))?;
        }
        fs::write(&path, config_json).map_err(|err| format!("write failed: {err}"))?;
        println!("Wrote configuration to {}", path.display());
    } else {
        println!("{config_json}");
    }

    Ok(())
}

fn cmd_stats(args: &[String]) -> Result<(), String> {
    let path = args
        .first()
        .ok_or("stats requires a path to a manifest file")?;
    let content =
        fs::read_to_string(path).map_err(|e| format!("failed to read manifest: {}", e))?;

    let v: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("invalid JSON: {}", e))?;

    println!("Manifest: {}", path);
    println!("Version:  {}", v["schemaVersion"]);
    if let Some(meta) = v.get("meta") {
        println!("World:    {}", meta["worldName"]);
        println!("Features: {}", meta["totalFeatures"]);
        println!("Source:   {}", meta["source"]);
    }

    if let Some(chunks) = v["chunks"].as_array() {
        println!("Chunks:   {}", chunks.len());
        let mut total_roads = 0;
        let mut total_rails = 0;
        let mut total_bldgs = 0;
        let mut total_props = 0;
        for c in chunks {
            total_roads += c["roads"].as_array().map(|a| a.len() as u64).unwrap_or(0);
            total_rails += c["rails"].as_array().map(|a| a.len() as u64).unwrap_or(0);
            total_bldgs += c["buildings"]
                .as_array()
                .map(|a| a.len() as u64)
                .unwrap_or(0);
            total_props += c["props"].as_array().map(|a| a.len() as u64).unwrap_or(0);
        }
        println!("  - Roads:     {}", total_roads);
        println!("  - Rails:     {}", total_rails);
        println!("  - Buildings: {}", total_bldgs);
        println!("  - Props:     {}", total_props);
    }

    Ok(())
}

fn cmd_validate(args: &[String]) -> Result<(), String> {
    let path = args
        .first()
        .ok_or("validate requires a path to a manifest file")?;
    let start = Instant::now();

    let content =
        fs::read_to_string(path).map_err(|e| format!("failed to read manifest: {}", e))?;
    let v: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("invalid JSON: {}", e))?;

    // Validate top-level structure
    let schema_version = v
        .get("schemaVersion")
        .and_then(|v| v.as_str())
        .ok_or("missing or invalid schemaVersion")?;

    if schema_version != "0.4.0" {
        return Err(format!(
            "unsupported schemaVersion: {}; only 0.4.0 manifests are supported",
            schema_version
        ));
    }

    // Validate meta section
    let meta = v.get("meta").ok_or("missing meta section")?;
    let required_meta = [
        "worldName",
        "generator",
        "source",
        "metersPerStud",
        "chunkSizeStuds",
        "bbox",
        "totalFeatures",
    ];
    for field in &required_meta {
        if meta.get(field).is_none() {
            return Err(format!("meta missing required field: {}", field));
        }
    }

    // Validate bbox
    let bbox = meta.get("bbox").ok_or("missing bbox")?;
    let bbox_fields = ["minLat", "minLon", "maxLat", "maxLon"];
    for field in &bbox_fields {
        if bbox.get(field).and_then(|v| v.as_f64()).is_none() {
            return Err(format!("bbox missing required field: {}", field));
        }
    }

    // Validate chunks
    let chunks = v
        .get("chunks")
        .and_then(|v| v.as_array())
        .ok_or("missing or invalid chunks array")?;

    if chunks.is_empty() {
        return Err("chunks array is empty".to_string());
    }

    for (i, chunk) in chunks.iter().enumerate() {
        let prefix = format!("chunks[{}]", i);

        // Validate chunk id
        if chunk.get("id").and_then(|v| v.as_str()).is_none() {
            return Err(format!("{} missing id", prefix));
        }

        // Validate originStuds
        let origin = chunk
            .get("originStuds")
            .ok_or(format!("{} missing originStuds", prefix))?;
        let origin_fields = ["x", "y", "z"];
        for field in &origin_fields {
            if origin.get(field).and_then(|v| v.as_f64()).is_none() {
                return Err(format!("{}.originStuds missing field: {}", prefix, field));
            }
        }

        // Validate terrain if present
        if let Some(terrain) = chunk.get("terrain") {
            let terrain_fields = ["cellSizeStuds", "width", "depth", "heights", "material"];
            for field in &terrain_fields {
                if terrain.get(field).is_none() {
                    return Err(format!("{}.terrain missing field: {}", prefix, field));
                }
            }
            let heights = terrain
                .get("heights")
                .and_then(|v| v.as_array())
                .ok_or(format!("{}.terrain.heights must be an array", prefix))?;
            let width = terrain.get("width").and_then(|v| v.as_u64()).unwrap_or(0);
            let depth = terrain.get("depth").and_then(|v| v.as_u64()).unwrap_or(0);
            let expected = width * depth;
            if heights.len() as u64 != expected {
                return Err(format!(
                    "{}.terrain.heights length mismatch: expected {}, got {}",
                    prefix,
                    expected,
                    heights.len()
                ));
            }
        }

        // Validate roads
        if let Some(roads) = chunk.get("roads").and_then(|v| v.as_array()) {
            for (j, road) in roads.iter().enumerate() {
                let road_prefix = format!("{}.roads[{}]", prefix, j);
                if road.get("id").and_then(|v| v.as_str()).is_none() {
                    return Err(format!("{} missing id", road_prefix));
                }
                if road.get("widthStuds").and_then(|v| v.as_f64()).is_none() {
                    return Err(format!("{} missing widthStuds", road_prefix));
                }
                let points = road
                    .get("points")
                    .and_then(|v| v.as_array())
                    .ok_or(format!("{} missing points", road_prefix))?;
                if points.len() < 2 {
                    return Err(format!(
                        "{}.points must have at least 2 points",
                        road_prefix
                    ));
                }
            }
        }

        // Validate buildings
        if let Some(buildings) = chunk.get("buildings").and_then(|v| v.as_array()) {
            for (j, bldg) in buildings.iter().enumerate() {
                let bldg_prefix = format!("{}.buildings[{}]", prefix, j);
                if bldg.get("id").and_then(|v| v.as_str()).is_none() {
                    return Err(format!("{} missing id", bldg_prefix));
                }
                if bldg.get("footprint").and_then(|v| v.as_array()).is_none() {
                    return Err(format!("{} missing footprint", bldg_prefix));
                }
                if bldg.get("baseY").and_then(|v| v.as_f64()).is_none() {
                    return Err(format!("{} missing baseY", bldg_prefix));
                }
                if bldg.get("height").and_then(|v| v.as_f64()).is_none() {
                    return Err(format!("{} missing height", bldg_prefix));
                }
            }
        }
    }

    let duration = start.elapsed();
    println!("✓ Manifest validated successfully: {}", path);
    println!("  Version: {}", schema_version);
    println!("  Chunks: {}", chunks.len());
    println!("  Validated in {:?}", duration);

    Ok(())
}

fn schema_version_difference_message(v1: &Value, v2: &Value) -> Option<String> {
    if v1["schemaVersion"] != v2["schemaVersion"] {
        Some(format!(
            "Schema versions differ: {} vs {}",
            v1["schemaVersion"], v2["schemaVersion"]
        ))
    } else {
        None
    }
}

fn cmd_diff(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("diff requires two manifest paths".to_string());
    }
    let m1_path = &args[0];
    let m2_path = &args[1];

    let m1_content =
        fs::read_to_string(m1_path).map_err(|e| format!("failed to read {}: {}", m1_path, e))?;
    let m2_content =
        fs::read_to_string(m2_path).map_err(|e| format!("failed to read {}: {}", m2_path, e))?;

    let v1: serde_json::Value = serde_json::from_str(&m1_content)
        .map_err(|e| format!("invalid JSON in {}: {}", m1_path, e))?;
    let v2: serde_json::Value = serde_json::from_str(&m2_content)
        .map_err(|e| format!("invalid JSON in {}: {}", m2_path, e))?;

    if let Some(message) = schema_version_difference_message(&v1, &v2) {
        println!("{message}");
    }

    let c1 = v1["chunks"].as_array().map(|a| a.len()).unwrap_or(0);
    let c2 = v2["chunks"].as_array().map(|a| a.len()).unwrap_or(0);

    if c1 != c2 {
        println!("Chunk count differs: {} vs {}", c1, c2);
    } else {
        println!("Both manifests have {} chunks", c1);
    }

    let f1 = v1["meta"]["totalFeatures"].as_u64().unwrap_or(0);
    let f2 = v2["meta"]["totalFeatures"].as_u64().unwrap_or(0);

    if f1 != f2 {
        println!("Total features differ: {} vs {}", f1, f2);
    }

    Ok(())
}

fn safe_f64(value: Option<&Value>) -> f64 {
    value.and_then(Value::as_f64).unwrap_or(0.0)
}

fn parse_latest_marker_from_str(log_content: &str, marker: &str) -> Result<Value, String> {
    let prefix = format!("{marker} ");
    let mut latest_payload: Option<Value> = None;
    let mut latest_error: Option<String> = None;
    for line in log_content.lines() {
        let Some(marker_index) = line.find(&prefix) else {
            continue;
        };
        let payload = &line[marker_index + prefix.len()..];
        match serde_json::from_str::<Value>(payload.trim()) {
            Ok(value) if value.is_object() => {
                latest_payload = Some(value);
            }
            Ok(_) => {}
            Err(err) => {
                latest_error = Some(format!("invalid {marker} payload: {err}"));
            }
        }
    }
    if let Some(payload) = latest_payload {
        Ok(payload)
    } else if let Some(error) = latest_error {
        Err(error)
    } else {
        Err(format!("no {marker} marker found in log"))
    }
}

fn chunk_center(chunk: &Value, chunk_size: f64) -> (f64, f64) {
    let origin = chunk.get("originStuds").and_then(Value::as_object);
    let origin_x = origin
        .and_then(|value| value.get("x"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let origin_z = origin
        .and_then(|value| value.get("z"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    (origin_x + chunk_size * 0.5, origin_z + chunk_size * 0.5)
}

fn ratio(actual: f64, expected: f64) -> f64 {
    if expected <= 0.0 {
        if actual <= 0.0 {
            1.0
        } else {
            0.0
        }
    } else {
        actual / expected
    }
}

fn build_manifest_zone_summary(manifest: &Value, payload: &Value) -> Result<Value, String> {
    let meta = manifest
        .get("meta")
        .and_then(Value::as_object)
        .ok_or("manifest missing meta")?;
    let chunk_size = safe_f64(meta.get("chunkSizeStuds"));
    let focus = payload.get("focus").and_then(Value::as_object);
    let scene = payload.get("scene").and_then(Value::as_object);
    let focus_x = focus
        .and_then(|value| value.get("x"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let focus_z = focus
        .and_then(|value| value.get("z"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let radius = payload.get("radius").and_then(Value::as_f64).unwrap_or(0.0);
    let radius_sq = radius * radius;
    let requested_chunk_ids: HashSet<String> = scene
        .and_then(|value| value.get("chunkIds"))
        .and_then(Value::as_array)
        .map(|chunk_ids| {
            chunk_ids
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default();

    let chunks = manifest
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or("manifest missing chunks")?;

    #[derive(Default, Clone)]
    struct ChunkCounts {
        chunk_count: usize,
        road_count: usize,
        roads_with_sidewalks: usize,
        roads_with_crossings: usize,
        building_count: usize,
        chunks_with_roads: usize,
        chunks_with_sidewalk_roads: usize,
        chunks_with_crossing_roads: usize,
        chunks_with_buildings: usize,
    }

    let candidate_chunks: Vec<&Value> = if requested_chunk_ids.is_empty() {
        chunks
            .iter()
            .filter(|chunk| {
                let (center_x, center_z) = chunk_center(chunk, chunk_size.max(256.0));
                let dx = center_x - focus_x;
                let dz = center_z - focus_z;
                radius <= 0.0 || dx * dx + dz * dz <= radius_sq
            })
            .collect()
    } else {
        chunks
            .iter()
            .filter(|chunk| {
                chunk
                    .get("id")
                    .and_then(Value::as_str)
                    .map(|id| requested_chunk_ids.contains(id))
                    .unwrap_or(false)
            })
            .collect()
    };

    let chunk_ids: Vec<String> = candidate_chunks
        .iter()
        .filter_map(|chunk| {
            chunk
                .get("id")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
        .collect();

    let counts = candidate_chunks
        .par_iter()
        .map(|chunk| {
            let roads = chunk.get("roads").and_then(Value::as_array);
            let buildings = chunk.get("buildings").and_then(Value::as_array);
            let road_count = chunk
                .get("roadCount")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or_else(|| roads.map_or(0, Vec::len));
            let roads_with_sidewalks = chunk
                .get("roadsWithSidewalks")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or_else(|| {
                    roads.map_or(0, |values| {
                        values
                            .iter()
                            .filter(|road| {
                                road.get("hasSidewalk")
                                    .and_then(Value::as_bool)
                                    .unwrap_or(false)
                                    || road
                                        .get("subkind")
                                        .and_then(Value::as_str)
                                        .is_some_and(|value| value == "sidewalk")
                                    || road
                                        .get("sidewalk")
                                        .and_then(Value::as_str)
                                        .is_some_and(|value| value != "no")
                            })
                            .count()
                    })
                });
            let roads_with_crossings = chunk
                .get("roadsWithCrossings")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or_else(|| {
                    roads.map_or(0, |values| {
                        values
                            .iter()
                            .filter(|road| {
                                road.get("subkind")
                                    .and_then(Value::as_str)
                                    .is_some_and(|value| value == "crossing")
                            })
                            .count()
                    })
                });
            let building_count = chunk
                .get("buildingCount")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or_else(|| buildings.map_or(0, Vec::len));
            let chunks_with_roads = chunk
                .get("chunksWithRoads")
                .and_then(Value::as_bool)
                .unwrap_or_else(|| roads.is_some_and(|value| !value.is_empty()));
            let chunks_with_sidewalk_roads = chunk
                .get("chunksWithSidewalkRoads")
                .and_then(Value::as_bool)
                .unwrap_or(roads_with_sidewalks > 0);
            let chunks_with_crossing_roads = chunk
                .get("chunksWithCrossingRoads")
                .and_then(Value::as_bool)
                .unwrap_or(roads_with_crossings > 0);
            let chunks_with_buildings = chunk
                .get("chunksWithBuildings")
                .and_then(Value::as_bool)
                .unwrap_or_else(|| buildings.is_some_and(|value| !value.is_empty()));
            ChunkCounts {
                chunk_count: 1,
                road_count,
                roads_with_sidewalks,
                roads_with_crossings,
                building_count,
                chunks_with_roads: usize::from(chunks_with_roads),
                chunks_with_sidewalk_roads: usize::from(chunks_with_sidewalk_roads),
                chunks_with_crossing_roads: usize::from(chunks_with_crossing_roads),
                chunks_with_buildings: usize::from(chunks_with_buildings),
            }
        })
        .reduce(ChunkCounts::default, |mut left, right| {
            left.chunk_count += right.chunk_count;
            left.road_count += right.road_count;
            left.roads_with_sidewalks += right.roads_with_sidewalks;
            left.roads_with_crossings += right.roads_with_crossings;
            left.building_count += right.building_count;
            left.chunks_with_roads += right.chunks_with_roads;
            left.chunks_with_sidewalk_roads += right.chunks_with_sidewalk_roads;
            left.chunks_with_crossing_roads += right.chunks_with_crossing_roads;
            left.chunks_with_buildings += right.chunks_with_buildings;
            left
        });

    Ok(json!({
        "chunkCount": counts.chunk_count,
        "chunkIds": chunk_ids,
        "roadCount": counts.road_count,
        "roadsWithSidewalks": counts.roads_with_sidewalks,
        "roadsWithCrossings": counts.roads_with_crossings,
        "buildingCount": counts.building_count,
        "chunksWithRoads": counts.chunks_with_roads,
        "chunksWithSidewalkRoads": counts.chunks_with_sidewalk_roads,
        "chunksWithCrossingRoads": counts.chunks_with_crossing_roads,
        "chunksWithBuildings": counts.chunks_with_buildings,
    }))
}

fn build_manifest_value_from_stored_subset(subset: StoredManifestSubset) -> Result<Value, String> {
    let chunks = subset
        .chunks
        .into_iter()
        .map(|record| {
            serde_json::from_str::<Value>(&record.chunk_json)
                .map_err(|err| format!("invalid chunk JSON in manifest store: {err}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json!({
        "schemaVersion": subset.meta.schema_version,
        "meta": {
            "worldName": subset.meta.world_name,
            "generator": subset.meta.generator,
            "source": subset.meta.source,
            "metersPerStud": subset.meta.meters_per_stud,
            "chunkSizeStuds": subset.meta.chunk_size_studs,
            "bbox": {
                "min": {
                    "lat": subset.meta.bbox.min.lat,
                    "lon": subset.meta.bbox.min.lon,
                },
                "max": {
                    "lat": subset.meta.bbox.max.lat,
                    "lon": subset.meta.bbox.max.lon,
                },
            },
            "totalFeatures": subset.meta.total_features,
            "notes": subset.meta.notes,
        },
        "chunks": chunks,
    }))
}

fn build_scene_manifest_index_from_sqlite(path: &Path) -> Result<Value, String> {
    let subset = read_manifest_sqlite_all(path)
        .map_err(|err| format!("failed to read manifest store {}: {err}", path.display()))?;
    let manifest = build_manifest_value_from_stored_subset(subset)?;
    build_scene_manifest_index_from_value(&manifest)
}

fn build_scene_audit_report_from_sqlite(
    path: &Path,
    log_content: &str,
    marker: &str,
) -> Result<Value, String> {
    let subset = read_manifest_sqlite_all(path)
        .map_err(|err| format!("failed to read manifest store {}: {err}", path.display()))?;
    let manifest = build_manifest_value_from_stored_subset(subset)?;
    build_scene_audit_report_from_value(&manifest, log_content, marker)
}

fn build_scene_manifest_index_from_str(manifest_content: &str) -> Result<Value, String> {
    let manifest: Value = serde_json::from_str(manifest_content)
        .map_err(|err| format!("invalid manifest JSON: {err}"))?;
    build_scene_manifest_index_from_value(&manifest)
}

fn build_scene_manifest_index_from_value(manifest: &Value) -> Result<Value, String> {
    let meta = manifest
        .get("meta")
        .and_then(Value::as_object)
        .ok_or("manifest missing meta")?;
    let chunks = manifest
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or("manifest missing chunks")?;

    let summarized_chunks: Vec<Value> = chunks
        .par_iter()
        .map(|chunk| {
            let roads = chunk.get("roads").and_then(Value::as_array);
            let buildings = chunk.get("buildings").and_then(Value::as_array);
            let roads_with_sidewalks = roads.map_or(0, |values| {
                values
                    .iter()
                    .filter(|road| {
                        road.get("hasSidewalk").and_then(Value::as_bool).unwrap_or(false)
                            || road
                                .get("subkind")
                                .and_then(Value::as_str)
                                .is_some_and(|value| value == "sidewalk")
                            || road
                                .get("sidewalk")
                                .and_then(Value::as_str)
                                .is_some_and(|value| value != "no")
                    })
                    .count()
            });
            let roads_with_crossings = roads.map_or(0, |values| {
                values
                    .iter()
                    .filter(|road| {
                        road
                            .get("subkind")
                            .and_then(Value::as_str)
                            .is_some_and(|value| value == "crossing")
                    })
                    .count()
            });
            json!({
                "id": chunk.get("id").cloned().unwrap_or(Value::Null),
                "originStuds": chunk.get("originStuds").cloned().unwrap_or_else(|| json!({"x": 0.0, "y": 0.0, "z": 0.0})),
                "roadCount": roads.map_or(0, Vec::len),
                "roadsWithSidewalks": roads_with_sidewalks,
                "roadsWithCrossings": roads_with_crossings,
                "buildingCount": buildings.map_or(0, Vec::len),
                "chunksWithRoads": roads.is_some_and(|value| !value.is_empty()),
                "chunksWithSidewalkRoads": roads_with_sidewalks > 0,
                "chunksWithCrossingRoads": roads_with_crossings > 0,
                "chunksWithBuildings": buildings.is_some_and(|value| !value.is_empty()),
            })
        })
        .collect();

    Ok(json!({
        "meta": {
            "worldName": meta.get("worldName").cloned().unwrap_or(Value::Null),
            "chunkSizeStuds": meta.get("chunkSizeStuds").cloned().unwrap_or(json!(256)),
            "schemaVersion": manifest.get("schemaVersion").cloned().unwrap_or(Value::Null),
            "sceneIndexVersion": SCENE_INDEX_VERSION,
        },
        "chunks": summarized_chunks,
    }))
}

fn build_scene_audit_report_from_str(
    manifest_content: &str,
    log_content: &str,
    marker: &str,
) -> Result<Value, String> {
    let manifest: Value = serde_json::from_str(manifest_content)
        .map_err(|err| format!("invalid manifest JSON: {err}"))?;
    build_scene_audit_report_from_value(&manifest, log_content, marker)
}

fn build_scene_audit_report_from_value(
    manifest: &Value,
    log_content: &str,
    marker: &str,
) -> Result<Value, String> {
    let mut payload = parse_latest_marker_from_str(log_content, marker)?;
    let mut scene = payload
        .get("scene")
        .cloned()
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({}));
    let chunk_marker = format!("{marker}_CHUNKS");
    if let Ok(chunk_payload) = parse_latest_marker_from_str(log_content, &chunk_marker) {
        if let Some(chunk_ids) = chunk_payload.get("chunkIds").cloned() {
            if let Some(scene_object) = scene.as_object_mut() {
                scene_object.insert("chunkIds".to_string(), chunk_ids);
            }
        }
    }
    if let Some(payload_object) = payload.as_object_mut() {
        payload_object.insert("scene".to_string(), scene.clone());
    }
    let manifest_summary = build_manifest_zone_summary(&manifest, &payload)?;

    let scene_chunk_count = scene.get("chunkCount").and_then(Value::as_u64).unwrap_or(0);
    let scene_building_models = scene
        .get("buildingModelCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let scene_buildings_with_direct_shell = scene
        .get("buildingModelsWithDirectShell")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let scene_buildings_missing_direct_shell = scene
        .get("buildingModelsMissingDirectShell")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let scene_merged_building_mesh_parts = scene
        .get("mergedBuildingMeshPartCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let scene_building_shell_parts = scene
        .get("buildingShellPartCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let scene_road_chunks = scene
        .get("chunksWithRoadGeometry")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let scene_road_mesh_parts = scene
        .get("roadMeshPartCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let scene_sidewalk_surface_parts = scene
        .get("sidewalkSurfacePartCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let scene_crossing_surface_parts = scene
        .get("crossingSurfacePartCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let scene_curb_surface_parts = scene
        .get("curbSurfacePartCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let scene_chunks_with_sidewalk_surfaces = scene
        .get("chunksWithSidewalkSurfaces")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let scene_chunks_with_crossing_surfaces = scene
        .get("chunksWithCrossingSurfaces")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let scene_chunks_with_curb_surfaces = scene
        .get("chunksWithCurbSurfaces")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let scene_road_detail_parts = scene
        .get("roadDetailPartCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let manifest_chunk_count = manifest_summary
        .get("chunkCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let manifest_building_count = manifest_summary
        .get("buildingCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let manifest_road_chunk_count = manifest_summary
        .get("chunksWithRoads")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let manifest_sidewalk_road_count = manifest_summary
        .get("roadsWithSidewalks")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let manifest_crossing_road_count = manifest_summary
        .get("roadsWithCrossings")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let manifest_sidewalk_road_chunks = manifest_summary
        .get("chunksWithSidewalkRoads")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let manifest_crossing_road_chunks = manifest_summary
        .get("chunksWithCrossingRoads")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    let mut findings = Vec::new();
    if scene_chunk_count < manifest_chunk_count {
        findings.push(json!({
            "severity": "high",
            "code": "scene_chunk_gap",
            "message": format!(
                "scene built {} chunks but manifest expected {}",
                scene_chunk_count, manifest_chunk_count
            )
        }));
    }
    if scene_building_models < manifest_building_count {
        findings.push(json!({
            "severity": "high",
            "code": "missing_building_models",
            "message": format!(
                "scene built {} building models but manifest expected {}",
                scene_building_models, manifest_building_count
            )
        }));
    }
    if manifest_road_chunk_count > 0 && scene_road_chunks < manifest_road_chunk_count {
        findings.push(json!({
            "severity": "high",
            "code": "missing_road_geometry",
            "message": format!(
                "scene has road geometry in {} chunks but manifest expected {}",
                scene_road_chunks, manifest_road_chunk_count
            )
        }));
    }
    if manifest_sidewalk_road_chunks > 0
        && scene_chunks_with_sidewalk_surfaces < manifest_sidewalk_road_chunks
    {
        findings.push(json!({
            "severity": "high",
            "code": "missing_sidewalk_surfaces",
            "message": format!(
                "scene has sidewalk surfaces in {} chunks but manifest expected {} chunks with sidewalk roads",
                scene_chunks_with_sidewalk_surfaces, manifest_sidewalk_road_chunks
            )
        }));
    }
    if manifest_sidewalk_road_chunks > 0
        && scene_chunks_with_curb_surfaces < manifest_sidewalk_road_chunks
    {
        findings.push(json!({
            "severity": "medium",
            "code": "missing_curb_surfaces",
            "message": format!(
                "scene has curb surfaces in {} chunks but manifest expected {} chunks with sidewalk roads",
                scene_chunks_with_curb_surfaces, manifest_sidewalk_road_chunks
            )
        }));
    }
    if manifest_crossing_road_chunks > 0
        && scene_chunks_with_crossing_surfaces < manifest_crossing_road_chunks
    {
        findings.push(json!({
            "severity": if scene_chunks_with_crossing_surfaces == 0 { "high" } else { "medium" },
            "code": "missing_crossing_surfaces",
            "message": format!(
                "scene has crossing surfaces in {} chunks but manifest expected {} chunks with crossing ways",
                scene_chunks_with_crossing_surfaces, manifest_crossing_road_chunks
            )
        }));
    }

    let missing_direct_shell_ratio = ratio(
        scene_buildings_missing_direct_shell as f64,
        scene_building_models as f64,
    );
    let merged_building_mesh_ratio = ratio(
        scene_merged_building_mesh_parts as f64,
        scene_building_models as f64,
    );
    let road_detail_to_mesh_ratio = if scene_road_mesh_parts == 0 {
        if scene_road_detail_parts == 0 {
            1.0
        } else {
            f64::INFINITY
        }
    } else {
        scene_road_detail_parts as f64 / scene_road_mesh_parts as f64
    };

    if scene_buildings_missing_direct_shell > 0 {
        findings.push(json!({
            "severity": if missing_direct_shell_ratio >= 0.2 { "high" } else { "medium" },
            "code": "missing_direct_building_shells",
            "message": format!(
                "{} of {} building models are missing direct shell geometry",
                scene_buildings_missing_direct_shell, scene_building_models
            )
        }));
    }

    if scene_merged_building_mesh_parts > scene_building_shell_parts && scene_building_models > 0 {
        findings.push(json!({
            "severity": "medium",
            "code": "merged_building_geometry_dominates",
            "message": format!(
                "merged building mesh parts ({}) exceed direct shell parts ({}) in the audited scene",
                scene_merged_building_mesh_parts, scene_building_shell_parts
            )
        }));
    }

    if scene_road_detail_parts > 0 && road_detail_to_mesh_ratio > 8.0 {
        findings.push(json!({
            "severity": "medium",
            "code": "road_detail_overwhelms_mesh",
            "message": format!(
                "road detail parts ({}) outweigh road mesh parts ({}) by {:.2}x",
                scene_road_detail_parts, scene_road_mesh_parts, road_detail_to_mesh_ratio
            )
        }));
    }

    Ok(json!({
        "generatedAt": format!("{:?}", std::time::SystemTime::now()),
        "phase": payload.get("phase").cloned().unwrap_or(Value::Null),
        "rootName": payload.get("rootName").cloned().unwrap_or(Value::Null),
        "focus": payload.get("focus").cloned().unwrap_or(Value::Null),
        "radius": payload.get("radius").cloned().unwrap_or(Value::Null),
        "scene": scene,
        "manifest": manifest_summary,
        "summary": {
            "marker": marker,
            "chunk_ratio": ratio(scene_chunk_count as f64, manifest_chunk_count as f64),
            "building_model_ratio": ratio(scene_building_models as f64, manifest_building_count as f64),
            "road_geometry_ratio": ratio(scene_road_chunks as f64, manifest_road_chunk_count as f64),
            "sidewalk_chunk_presence_ratio": ratio(scene_chunks_with_sidewalk_surfaces as f64, manifest_sidewalk_road_chunks as f64),
            "crossing_chunk_presence_ratio": ratio(scene_chunks_with_crossing_surfaces as f64, manifest_crossing_road_chunks as f64),
            "curb_chunk_presence_ratio": ratio(scene_chunks_with_curb_surfaces as f64, manifest_sidewalk_road_chunks as f64),
            "sidewalk_surface_part_count": scene_sidewalk_surface_parts,
            "crossing_surface_part_count": scene_crossing_surface_parts,
            "curb_surface_part_count": scene_curb_surface_parts,
            "manifest_roads_with_sidewalks": manifest_sidewalk_road_count,
            "manifest_roads_with_crossings": manifest_crossing_road_count,
            "missing_direct_shell_ratio": missing_direct_shell_ratio,
            "direct_shell_coverage_ratio": ratio(scene_buildings_with_direct_shell as f64, scene_building_models as f64),
            "merged_building_mesh_ratio": merged_building_mesh_ratio,
            "road_detail_to_mesh_ratio": road_detail_to_mesh_ratio,
        },
        "findings": findings,
    }))
}

fn cmd_scene_index(args: &[String]) -> Result<(), String> {
    let mut manifest_path: Option<PathBuf> = None;
    let mut manifest_sqlite_path: Option<PathBuf> = None;
    let mut json_out: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--manifest" => {
                manifest_path = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--manifest requires a path")?,
                ));
                i += 2;
            }
            "--manifest-sqlite" => {
                manifest_sqlite_path = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--manifest-sqlite requires a path")?,
                ));
                i += 2;
            }
            "--json-out" => {
                json_out = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--json-out requires a path")?,
                ));
                i += 2;
            }
            other => return Err(format!("unknown argument to scene-index: {other}")),
        }
    }

    let index = match (manifest_path.as_ref(), manifest_sqlite_path.as_ref()) {
        (Some(_), Some(_)) => {
            return Err("provide only one of --manifest or --manifest-sqlite".to_string())
        }
        (Some(path), None) => {
            let manifest_content = fs::read_to_string(path)
                .map_err(|err| format!("failed to read manifest {}: {err}", path.display()))?;
            build_scene_manifest_index_from_str(&manifest_content)?
        }
        (None, Some(path)) => build_scene_manifest_index_from_sqlite(path)?,
        (None, None) => return Err("--manifest or --manifest-sqlite is required".to_string()),
    };
    let output = serde_json::to_string_pretty(&index)
        .map_err(|err| format!("failed to encode scene index: {err}"))?;

    if let Some(path) = json_out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| format!("create dir failed: {err}"))?;
        }
        fs::write(&path, output).map_err(|err| format!("write failed: {err}"))?;
    } else {
        println!("{output}");
    }
    Ok(())
}

fn cmd_scene_audit(args: &[String]) -> Result<(), String> {
    let mut manifest_path: Option<PathBuf> = None;
    let mut manifest_sqlite_path: Option<PathBuf> = None;
    let mut manifest_summary_path: Option<PathBuf> = None;
    let mut log_path: Option<PathBuf> = None;
    let mut marker = "ARNIS_SCENE_PLAY".to_string();
    let mut json_out: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--manifest" => {
                manifest_path = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--manifest requires a path")?,
                ));
                i += 2;
            }
            "--manifest-sqlite" => {
                manifest_sqlite_path = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--manifest-sqlite requires a path")?,
                ));
                i += 2;
            }
            "--manifest-summary" => {
                manifest_summary_path = Some(PathBuf::from(
                    args.get(i + 1)
                        .ok_or("--manifest-summary requires a path")?,
                ));
                i += 2;
            }
            "--log" => {
                log_path = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--log requires a path")?,
                ));
                i += 2;
            }
            "--marker" => {
                marker = args.get(i + 1).ok_or("--marker requires a value")?.clone();
                i += 2;
            }
            "--json-out" => {
                json_out = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--json-out requires a path")?,
                ));
                i += 2;
            }
            other => return Err(format!("unknown argument to scene-audit: {other}")),
        }
    }

    let log_path = log_path.ok_or("--log is required")?;
    let log_content = fs::read_to_string(&log_path)
        .map_err(|err| format!("failed to read log {}: {err}", log_path.display()))?;
    let (mut report, manifest_source_label) = if let Some(path) = manifest_summary_path.as_ref() {
        if manifest_path.is_some() || manifest_sqlite_path.is_some() {
            return Err(
                "provide either --manifest-summary or a raw manifest source, not both".to_string(),
            );
        }
        let manifest_content = fs::read_to_string(path)
            .map_err(|err| format!("failed to read manifest source {}: {err}", path.display()))?;
        (
            build_scene_audit_report_from_str(&manifest_content, &log_content, &marker)?,
            path.display().to_string(),
        )
    } else if let Some(path) = manifest_path.as_ref() {
        if manifest_sqlite_path.is_some() {
            return Err("provide only one of --manifest or --manifest-sqlite".to_string());
        }
        let manifest_content = fs::read_to_string(path)
            .map_err(|err| format!("failed to read manifest source {}: {err}", path.display()))?;
        (
            build_scene_audit_report_from_str(&manifest_content, &log_content, &marker)?,
            path.display().to_string(),
        )
    } else if let Some(path) = manifest_sqlite_path.as_ref() {
        (
            build_scene_audit_report_from_sqlite(path, &log_content, &marker)?,
            path.display().to_string(),
        )
    } else {
        return Err("--manifest, --manifest-sqlite, or --manifest-summary is required".to_string());
    };
    if let Some(object) = report.as_object_mut() {
        object.insert(
            "manifestPath".to_string(),
            Value::String(manifest_source_label),
        );
        object.insert(
            "logPath".to_string(),
            Value::String(log_path.display().to_string()),
        );
    }
    let output = serde_json::to_string_pretty(&report)
        .map_err(|err| format!("failed to encode report: {err}"))?;

    if let Some(path) = json_out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| format!("create dir failed: {err}"))?;
        }
        fs::write(&path, output).map_err(|err| format!("write failed: {err}"))?;
    } else {
        println!("{output}");
    }

    Ok(())
}

fn cmd_emit_runtime_lua(args: &[String]) -> Result<(), String> {
    let mut manifest_sqlite: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut index_name = "AustinManifestIndex".to_string();
    let mut shard_folder = "AustinManifestChunks".to_string();
    let mut chunks_per_shard: usize = 32;
    let mut max_bytes: Option<usize> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--manifest-sqlite" => {
                manifest_sqlite = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--manifest-sqlite requires a path")?,
                ));
                i += 2;
            }
            "--output-dir" => {
                output_dir = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--output-dir requires a path")?,
                ));
                i += 2;
            }
            "--index-name" => {
                index_name = args
                    .get(i + 1)
                    .ok_or("--index-name requires a value")?
                    .to_string();
                i += 2;
            }
            "--shard-folder" => {
                shard_folder = args
                    .get(i + 1)
                    .ok_or("--shard-folder requires a value")?
                    .to_string();
                i += 2;
            }
            "--chunks-per-shard" => {
                chunks_per_shard = args
                    .get(i + 1)
                    .ok_or("--chunks-per-shard requires a value")?
                    .parse::<usize>()
                    .map_err(|err| format!("invalid --chunks-per-shard: {err}"))?;
                i += 2;
            }
            "--max-bytes" => {
                max_bytes = Some(
                    args.get(i + 1)
                        .ok_or("--max-bytes requires a value")?
                        .parse::<usize>()
                        .map_err(|err| format!("invalid --max-bytes: {err}"))?,
                );
                i += 2;
            }
            other => return Err(format!("unknown argument to emit-runtime-lua: {other}")),
        }
    }

    let manifest_sqlite = manifest_sqlite.ok_or("--manifest-sqlite is required".to_string())?;
    let output_dir = output_dir.ok_or("--output-dir is required".to_string())?;
    if chunks_per_shard == 0 {
        return Err("--chunks-per-shard must be at least 1".to_string());
    }

    let stats = write_runtime_lua_shards_from_sqlite(
        &manifest_sqlite,
        &RuntimeLuaShardsOptions {
            output_dir,
            index_name,
            shard_folder,
            chunks_per_shard,
            max_bytes,
        },
    )
    .map_err(|err| format!("emit-runtime-lua failed: {err}"))?;

    println!("Wrote index module to {}", stats.index_path.display());
    println!(
        "Wrote {} shard modules to {}",
        stats.shard_count,
        stats.shard_dir.display()
    );
    Ok(())
}

fn resolve_planetary_scene_for_selection(
    store_path: &Path,
    explicit_scene_id: Option<String>,
    point: Option<(f64, f64)>,
    tile: Option<(u8, u32, u32)>,
    command_name: &str,
) -> Result<String, String> {
    if let Some(scene_id) = explicit_scene_id {
        return Ok(scene_id);
    }
    if let Some((lat, lon)) = point {
        let scene = find_best_scene_covering_geo_point(store_path, lat, lon)
            .map_err(|err| {
                format!("planetary-store {command_name} point scene resolution failed: {err}")
            })?
            .ok_or_else(|| {
                format!(
                    "planetary-store {command_name} found no scene covering geo point {lat},{lon}"
                )
            })?;
        return Ok(scene.scene_id);
    }
    if let Some((zoom, x, y)) = tile {
        let scene = find_best_scene_covering_tile(store_path, zoom, x, y)
            .map_err(|err| {
                format!("planetary-store {command_name} tile scene resolution failed: {err}")
            })?
            .ok_or_else(|| {
                format!(
                    "planetary-store {command_name} found no scene covering tile {zoom},{x},{y}"
                )
            })?;
        return Ok(scene.scene_id);
    }
    Err(format!(
        "planetary-store {command_name} requires --scene ID unless using --point or --tile"
    ))
}

fn read_delivery_plan(
    plan_path: &Path,
) -> Result<arbx_planetary_store::PlanetaryDeliveryPlan, String> {
    let text = fs::read_to_string(plan_path).map_err(|err| {
        format!(
            "failed reading delivery plan {}: {err}",
            plan_path.display()
        )
    })?;
    serde_json::from_str(&text).map_err(|err| {
        format!(
            "failed parsing delivery plan {}: {err}",
            plan_path.display()
        )
    })
}

fn read_route_session(
    session_path: &Path,
) -> Result<arbx_planetary_store::PlanetaryRouteSession, String> {
    let text = fs::read_to_string(session_path).map_err(|err| {
        format!(
            "failed reading route session {}: {err}",
            session_path.display()
        )
    })?;
    serde_json::from_str(&text).map_err(|err| {
        format!(
            "failed parsing route session {}: {err}",
            session_path.display()
        )
    })
}

fn plan_chunk_refs(
    plan: &arbx_planetary_store::PlanetaryDeliveryPlan,
) -> Vec<arbx_planetary_store::PlanetaryChunkRef> {
    if !plan.chunk_refs.is_empty() {
        return plan.chunk_refs.clone();
    }
    plan.chunk_ids
        .iter()
        .map(|chunk_id| arbx_planetary_store::PlanetaryChunkRef {
            scene_id: plan.scene_id.clone(),
            chunk_id: chunk_id.clone(),
        })
        .collect()
}

fn plan_scene_ids(plan: &arbx_planetary_store::PlanetaryDeliveryPlan) -> Vec<String> {
    let mut scene_ids = plan_chunk_refs(plan)
        .into_iter()
        .map(|chunk_ref| chunk_ref.scene_id)
        .collect::<Vec<_>>();
    scene_ids.sort();
    scene_ids.dedup();
    scene_ids
}

fn single_scene_plan_scene_id(
    plan: &arbx_planetary_store::PlanetaryDeliveryPlan,
    command_name: &str,
) -> Result<String, String> {
    let scene_ids = plan_scene_ids(plan);
    if scene_ids.len() != 1 {
        return Err(format!(
            "planetary-store {command_name} requires a single-scene delivery artifact; use --session for cross-scene route truth"
        ));
    }
    Ok(scene_ids[0].clone())
}

fn single_scene_plan_chunk_ids(
    plan: &arbx_planetary_store::PlanetaryDeliveryPlan,
    scene_id: &str,
) -> Vec<String> {
    plan_chunk_refs(plan)
        .into_iter()
        .filter(|chunk_ref| chunk_ref.scene_id == scene_id)
        .map(|chunk_ref| chunk_ref.chunk_id)
        .collect()
}

fn read_chunk_summary_for_plan(
    store_path: &Path,
    plan: &arbx_planetary_store::PlanetaryDeliveryPlan,
) -> Result<Vec<arbx_planetary_store::PlanetaryChunkSummary>, String> {
    read_chunk_summaries_by_refs(store_path, &plan_chunk_refs(plan))
        .map_err(|err| format!("failed reading delivery plan chunk summaries: {err}"))
}

fn read_manifest_subset_for_plan_single_scene(
    store_path: &Path,
    plan: &arbx_planetary_store::PlanetaryDeliveryPlan,
    command_name: &str,
) -> Result<StoredManifestSubset, String> {
    let scene_id = single_scene_plan_scene_id(plan, command_name)?;
    let chunk_ids = single_scene_plan_chunk_ids(plan, &scene_id);
    read_scene_manifest_subset_by_chunk_ids(store_path, &scene_id, &chunk_ids)
        .map_err(|err| format!("planetary-store {command_name} failed: {err}"))
}

fn parse_lat_lon_pair(value: &str, flag_name: &str) -> Result<(f64, f64), String> {
    let parts: Vec<f64> = value
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<f64>()
                .map_err(|_| format!("invalid number in {flag_name}: {}", part))
        })
        .collect::<Result<Vec<f64>, String>>()?;
    if parts.len() != 2 {
        return Err(format!("{flag_name} requires two comma-separated numbers"));
    }
    Ok((parts[0], parts[1]))
}

fn resolve_planetary_store_path(
    explicit_store_path: Option<PathBuf>,
    plan: Option<&arbx_planetary_store::PlanetaryDeliveryPlan>,
    session: Option<&arbx_planetary_store::PlanetaryRouteSession>,
    command_name: &str,
) -> Result<PathBuf, String> {
    if let Some(path) = explicit_store_path {
        return Ok(path);
    }
    if let Some(plan) = plan {
        return Ok(PathBuf::from(&plan.planetary_store_path));
    }
    if let Some(session) = session {
        return Ok(PathBuf::from(&session.planetary_store_path));
    }
    Err(format!(
        "planetary-store {command_name} requires --store PATH unless using --plan or --session"
    ))
}

fn write_delivery_plan_json(
    plan: &arbx_planetary_store::PlanetaryDeliveryPlan,
    out_path: &Path,
) -> Result<(), String> {
    let plan_json = serde_json::to_string_pretty(plan)
        .map_err(|err| format!("delivery-plan json failed: {err}"))?;
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("delivery-plan create dir failed: {err}"))?;
    }
    fs::write(out_path, format!("{plan_json}\n"))
        .map_err(|err| format!("delivery-plan write failed: {err}"))?;
    Ok(())
}

fn write_json_file<T: serde::Serialize>(
    value: &T,
    out_path: &Path,
    label: &str,
) -> Result<(), String> {
    let payload =
        serde_json::to_string_pretty(value).map_err(|err| format!("{label} json failed: {err}"))?;
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("{label} create dir failed: {err}"))?;
    }
    fs::write(out_path, format!("{payload}\n"))
        .map_err(|err| format!("{label} write failed: {err}"))?;
    Ok(())
}

fn write_route_schedule_lane_files(
    schedule: &arbx_planetary_store::PlanetaryRouteDeliverySchedule,
    out_dir: &Path,
    label: &str,
) -> Result<(), String> {
    fs::create_dir_all(out_dir).map_err(|err| format!("{label} create dir failed: {err}"))?;
    for step in &schedule.steps {
        let active_path = out_dir.join(format!("step-{:03}-active.json", step.step_index));
        let prefetch_path = out_dir.join(format!("step-{:03}-prefetch.json", step.step_index));
        let retain_path = out_dir.join(format!("step-{:03}-retain.json", step.step_index));
        write_json_file(&step.active, &active_path, label)?;
        write_json_file(&step.prefetch, &prefetch_path, label)?;
        write_json_file(&step.retain, &retain_path, label)?;
    }
    Ok(())
}

fn cmd_planetary_store(args: &[String]) -> Result<(), String> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        return Err(
            "planetary-store requires a subcommand: init | ingest-manifest | ingest-json | attach-truth-pack-summary | summary | list-scenes | scene | subset | subset-summary | fetch-chunks | emit-manifest-subset | emit-runtime-lua | find-scenes | delivery-window | delivery-plan | route-plan | route-session | hydrate-route-session | schedule-route-session | materialize-route-schedule | merge-delivery-plans | delivery-bundle | tile-scenes".to_string(),
        );
    };

    match subcommand {
        "init" => {
            let mut store_path: Option<PathBuf> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    other => {
                        return Err(format!("unknown argument to planetary-store init: {other}"))
                    }
                }
            }
            let store_path = store_path.ok_or("planetary-store init requires --store PATH")?;
            init_planetary_store(&store_path)
                .map_err(|err| format!("planetary-store init failed: {err}"))?;
            println!("Initialized planetary store {}", store_path.display());
            Ok(())
        }
        "ingest-manifest" => {
            let mut store_path: Option<PathBuf> = None;
            let mut manifest_sqlite_path: Option<PathBuf> = None;
            let mut scene_id: Option<String> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--manifest-sqlite" => {
                        let value = args.get(i + 1).ok_or("--manifest-sqlite requires a path")?;
                        manifest_sqlite_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--scene" => {
                        let value = args.get(i + 1).ok_or("--scene requires a scene id")?;
                        scene_id = Some(value.clone());
                        i += 2;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store ingest-manifest: {other}"
                        ))
                    }
                }
            }
            let store_path =
                store_path.ok_or("planetary-store ingest-manifest requires --store PATH")?;
            let manifest_sqlite_path = manifest_sqlite_path
                .ok_or("planetary-store ingest-manifest requires --manifest-sqlite PATH")?;
            let summary =
                ingest_manifest_sqlite(&store_path, &manifest_sqlite_path, scene_id.as_deref())
                    .map_err(|err| format!("planetary-store ingest-manifest failed: {err}"))?;
            println!(
                "Ingested scene {} ({}) with {} chunks and {} features into {}",
                summary.scene_id,
                summary.world_name,
                summary.chunk_count,
                summary.total_features,
                store_path.display()
            );
            Ok(())
        }
        "ingest-json" => {
            let mut store_path: Option<PathBuf> = None;
            let mut manifest_json_path: Option<PathBuf> = None;
            let mut scene_id: Option<String> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--manifest-json" => {
                        let value = args.get(i + 1).ok_or("--manifest-json requires a path")?;
                        manifest_json_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--scene" => {
                        let value = args.get(i + 1).ok_or("--scene requires a scene id")?;
                        scene_id = Some(value.clone());
                        i += 2;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store ingest-json: {other}"
                        ))
                    }
                }
            }
            let store_path =
                store_path.ok_or("planetary-store ingest-json requires --store PATH")?;
            let manifest_json_path = manifest_json_path
                .ok_or("planetary-store ingest-json requires --manifest-json PATH")?;
            let summary =
                ingest_manifest_json(&store_path, &manifest_json_path, scene_id.as_deref())
                    .map_err(|err| format!("planetary-store ingest-json failed: {err}"))?;
            println!(
                "Ingested JSON scene {} ({}) with {} chunks and {} features into {}",
                summary.scene_id,
                summary.world_name,
                summary.chunk_count,
                summary.total_features,
                store_path.display()
            );
            Ok(())
        }
        "attach-truth-pack-summary" => {
            let mut store_path: Option<PathBuf> = None;
            let mut scene_id: Option<String> = None;
            let mut truth_pack_summary_path: Option<PathBuf> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--scene" => {
                        let value = args.get(i + 1).ok_or("--scene requires a scene id")?;
                        scene_id = Some(value.clone());
                        i += 2;
                    }
                    "--truth-pack-summary" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--truth-pack-summary requires a path")?;
                        truth_pack_summary_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store attach-truth-pack-summary: {other}"
                        ))
                    }
                }
            }
            let store_path = store_path
                .ok_or("planetary-store attach-truth-pack-summary requires --store PATH")?;
            let scene_id =
                scene_id.ok_or("planetary-store attach-truth-pack-summary requires --scene ID")?;
            let truth_pack_summary_path = truth_pack_summary_path.ok_or(
                "planetary-store attach-truth-pack-summary requires --truth-pack-summary PATH",
            )?;
            let summary_text = fs::read_to_string(&truth_pack_summary_path)
                .map_err(|err| format!("failed to read truth-pack summary: {err}"))?;
            let summary: SourceTruthPackSummary = serde_json::from_str(&summary_text)
                .map_err(|err| format!("invalid truth-pack summary JSON: {err}"))?;
            attach_truth_pack_summary(&store_path, &scene_id, &summary).map_err(|err| {
                format!("planetary-store attach-truth-pack-summary failed: {err}")
            })?;
            println!(
                "Attached truth-pack summary {} to scene {} in {}",
                truth_pack_summary_path.display(),
                scene_id,
                store_path.display()
            );
            Ok(())
        }
        "summary" => {
            let mut store_path: Option<PathBuf> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store summary: {other}"
                        ))
                    }
                }
            }
            let store_path = store_path.ok_or("planetary-store summary requires --store PATH")?;
            let summary = summarize_planetary_store(&store_path)
                .map_err(|err| format!("planetary-store summary failed: {err}"))?;
            println!(
                "Planetary store {}: {} scenes, {} chunks, {} features",
                store_path.display(),
                summary.scene_count,
                summary.chunk_count,
                summary.total_features
            );
            Ok(())
        }
        "list-scenes" => {
            let mut store_path: Option<PathBuf> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store list-scenes: {other}"
                        ))
                    }
                }
            }
            let store_path =
                store_path.ok_or("planetary-store list-scenes requires --store PATH")?;
            let scenes = list_scenes(&store_path)
                .map_err(|err| format!("planetary-store list-scenes failed: {err}"))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&scenes)
                    .map_err(|err| format!("list-scenes json failed: {err}"))?
            );
            Ok(())
        }
        "scene" => {
            let mut store_path: Option<PathBuf> = None;
            let mut scene_id: Option<String> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--scene" => {
                        let value = args.get(i + 1).ok_or("--scene requires a scene id")?;
                        scene_id = Some(value.clone());
                        i += 2;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store scene: {other}"
                        ))
                    }
                }
            }
            let store_path = store_path.ok_or("planetary-store scene requires --store PATH")?;
            let scene_id = scene_id.ok_or("planetary-store scene requires --scene ID")?;
            let scene = read_scene_catalog_entry(&store_path, &scene_id)
                .map_err(|err| format!("planetary-store scene failed: {err}"))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&scene)
                    .map_err(|err| format!("scene json failed: {err}"))?
            );
            Ok(())
        }
        "subset" => {
            let mut store_path: Option<PathBuf> = None;
            let mut scene_id: Option<String> = None;
            let mut bbox_studs: Option<(f64, f64, f64, f64)> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--scene" => {
                        let value = args.get(i + 1).ok_or("--scene requires a scene id")?;
                        scene_id = Some(value.clone());
                        i += 2;
                    }
                    "--bbox-studs" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--bbox-studs requires MIN_X,MIN_Z,MAX_X,MAX_Z")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim().parse::<f64>().map_err(|_| {
                                    format!("invalid number in --bbox-studs: {}", part)
                                })
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 4 {
                            return Err(
                                "--bbox-studs requires four comma-separated numbers".to_string()
                            );
                        }
                        bbox_studs = Some((parts[0], parts[1], parts[2], parts[3]));
                        i += 2;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store subset: {other}"
                        ))
                    }
                }
            }
            let store_path = store_path.ok_or("planetary-store subset requires --store PATH")?;
            let scene_id = scene_id.ok_or("planetary-store subset requires --scene ID")?;
            let (min_x, min_z, max_x, max_z) = bbox_studs
                .ok_or("planetary-store subset requires --bbox-studs MIN_X,MIN_Z,MAX_X,MAX_Z")?;
            let subset =
                read_scene_chunk_subset(&store_path, &scene_id, min_x, min_z, max_x, max_z)
                    .map_err(|err| format!("planetary-store subset failed: {err}"))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&subset)
                    .map_err(|err| format!("subset json failed: {err}"))?
            );
            Ok(())
        }
        "subset-summary" => {
            let mut store_path: Option<PathBuf> = None;
            let mut scene_id: Option<String> = None;
            let mut plan_path: Option<PathBuf> = None;
            let mut session_path: Option<PathBuf> = None;
            let mut bbox_studs: Option<(f64, f64, f64, f64)> = None;
            let mut around_studs: Option<(f64, f64)> = None;
            let mut point: Option<(f64, f64)> = None;
            let mut tile: Option<(u8, u32, u32)> = None;
            let mut radius_studs: Option<f64> = None;
            let mut limit: Option<usize> = None;
            let mut max_streaming_cost: Option<f64> = None;
            let mut max_estimated_memory_cost: Option<f64> = None;
            let mut require_buildings = false;
            let mut require_terrain = false;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--scene" => {
                        let value = args.get(i + 1).ok_or("--scene requires a scene id")?;
                        scene_id = Some(value.clone());
                        i += 2;
                    }
                    "--plan" => {
                        let value = args.get(i + 1).ok_or("--plan requires a path")?;
                        plan_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--session" => {
                        let value = args.get(i + 1).ok_or("--session requires a path")?;
                        session_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--bbox-studs" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--bbox-studs requires MIN_X,MIN_Z,MAX_X,MAX_Z")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim().parse::<f64>().map_err(|_| {
                                    format!("invalid number in --bbox-studs: {}", part)
                                })
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 4 {
                            return Err(
                                "--bbox-studs requires four comma-separated numbers".to_string()
                            );
                        }
                        bbox_studs = Some((parts[0], parts[1], parts[2], parts[3]));
                        i += 2;
                    }
                    "--around-studs" => {
                        let value = args.get(i + 1).ok_or("--around-studs requires X,Z")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim().parse::<f64>().map_err(|_| {
                                    format!("invalid number in --around-studs: {}", part)
                                })
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 2 {
                            return Err(
                                "--around-studs requires two comma-separated numbers".to_string()
                            );
                        }
                        around_studs = Some((parts[0], parts[1]));
                        i += 2;
                    }
                    "--point" => {
                        let value = args.get(i + 1).ok_or("--point requires LAT,LON")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim()
                                    .parse::<f64>()
                                    .map_err(|_| format!("invalid number in --point: {}", part))
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 2 {
                            return Err("--point requires two comma-separated numbers".to_string());
                        }
                        point = Some((parts[0], parts[1]));
                        i += 2;
                    }
                    "--tile" => {
                        let value = args.get(i + 1).ok_or("--tile requires Z,X,Y")?;
                        let parts: Vec<u64> = value
                            .split(',')
                            .map(|part| {
                                part.trim()
                                    .parse::<u64>()
                                    .map_err(|_| format!("invalid number in --tile: {}", part))
                            })
                            .collect::<Result<Vec<u64>, String>>()?;
                        if parts.len() != 3 {
                            return Err(
                                "--tile requires three comma-separated integers".to_string()
                            );
                        }
                        tile = Some((parts[0] as u8, parts[1] as u32, parts[2] as u32));
                        i += 2;
                    }
                    "--radius-studs" => {
                        let value = args.get(i + 1).ok_or("--radius-studs requires a number")?;
                        radius_studs = Some(
                            value
                                .parse::<f64>()
                                .map_err(|_| format!("invalid --radius-studs value: {value}"))?,
                        );
                        i += 2;
                    }
                    "--limit" => {
                        let value = args.get(i + 1).ok_or("--limit requires a number")?;
                        limit = Some(
                            value
                                .parse::<usize>()
                                .map_err(|_| format!("invalid --limit value: {value}"))?,
                        );
                        i += 2;
                    }
                    "--max-streaming-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-streaming-cost requires a number")?;
                        max_streaming_cost =
                            Some(value.parse::<f64>().map_err(|_| {
                                format!("invalid --max-streaming-cost value: {value}")
                            })?);
                        i += 2;
                    }
                    "--max-estimated-memory-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-estimated-memory-cost requires a number")?;
                        max_estimated_memory_cost = Some(value.parse::<f64>().map_err(|_| {
                            format!("invalid --max-estimated-memory-cost value: {value}")
                        })?);
                        i += 2;
                    }
                    "--require-buildings" => {
                        require_buildings = true;
                        i += 1;
                    }
                    "--require-terrain" => {
                        require_terrain = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store subset-summary: {other}"
                        ))
                    }
                }
            }
            if let Some(plan_path) = plan_path {
                let plan = read_delivery_plan(&plan_path)?;
                let store_path =
                    resolve_planetary_store_path(store_path, Some(&plan), None, "subset-summary")?;
                let subset = read_chunk_summary_for_plan(&store_path, &plan)
                    .map_err(|err| format!("planetary-store subset-summary plan failed: {err}"))?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&subset)
                        .map_err(|err| format!("subset-summary json failed: {err}"))?
                );
                return Ok(());
            }
            if let Some(session_path) = session_path {
                let session = read_route_session(&session_path)?;
                let store_path = resolve_planetary_store_path(
                    store_path,
                    Some(&session.merged_plan),
                    Some(&session),
                    "subset-summary",
                )?;
                let subset = read_chunk_summary_for_plan(&store_path, &session.merged_plan)
                    .map_err(|err| {
                        format!("planetary-store subset-summary session failed: {err}")
                    })?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&subset)
                        .map_err(|err| format!("subset-summary json failed: {err}"))?
                );
                return Ok(());
            }
            let store_path =
                resolve_planetary_store_path(store_path, None, None, "subset-summary")?;
            let scene_id = if bbox_studs.is_some() || around_studs.is_some() {
                scene_id.ok_or(
                    "planetary-store subset-summary requires --scene ID when using --bbox-studs or --around-studs",
                )?
            } else {
                resolve_planetary_scene_for_selection(
                    &store_path,
                    scene_id,
                    point,
                    tile,
                    "subset-summary",
                )?
            };
            let subset = if let Some((min_x, min_z, max_x, max_z)) = bbox_studs {
                read_scene_chunk_summary_subset(
                    &store_path,
                    &scene_id,
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
                .map_err(|err| format!("planetary-store subset-summary bbox failed: {err}"))?
            } else if let Some((focus_x, focus_z)) = around_studs {
                let radius_studs = radius_studs.ok_or(
                    "planetary-store subset-summary requires --radius-studs when using --around-studs",
                )?;
                read_scene_chunk_summary_around_point(
                    &store_path,
                    &scene_id,
                    focus_x,
                    focus_z,
                    radius_studs,
                    limit,
                    require_buildings,
                    require_terrain,
                    max_streaming_cost,
                    max_estimated_memory_cost,
                )
                .map_err(|err| {
                    format!("planetary-store subset-summary around-studs failed: {err}")
                })?
            } else if let Some((lat, lon)) = point {
                let radius_studs = radius_studs.ok_or(
                    "planetary-store subset-summary requires --radius-studs when using --point",
                )?;
                read_scene_chunk_summary_around_geo_point(
                    &store_path,
                    &scene_id,
                    lat,
                    lon,
                    radius_studs,
                    limit,
                    require_buildings,
                    require_terrain,
                    max_streaming_cost,
                    max_estimated_memory_cost,
                )
                .map_err(|err| format!("planetary-store subset-summary point failed: {err}"))?
            } else if let Some((zoom, x, y)) = tile {
                read_scene_chunk_summary_for_tile(
                    &store_path,
                    &scene_id,
                    zoom,
                    x,
                    y,
                    limit,
                    require_buildings,
                    require_terrain,
                    max_streaming_cost,
                    max_estimated_memory_cost,
                )
                .map_err(|err| format!("planetary-store subset-summary tile failed: {err}"))?
            } else {
                return Err(
                    "planetary-store subset-summary requires --bbox-studs, --around-studs, --point, or --tile"
                        .to_string(),
                );
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&subset)
                    .map_err(|err| format!("subset-summary json failed: {err}"))?
            );
            Ok(())
        }
        "fetch-chunks" => {
            let mut store_path: Option<PathBuf> = None;
            let mut scene_id: Option<String> = None;
            let mut chunk_ids: Option<Vec<String>> = None;
            let mut plan_path: Option<PathBuf> = None;
            let mut session_path: Option<PathBuf> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--scene" => {
                        let value = args.get(i + 1).ok_or("--scene requires a scene id")?;
                        scene_id = Some(value.clone());
                        i += 2;
                    }
                    "--chunk-ids" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--chunk-ids requires a comma-separated list")?;
                        chunk_ids = Some(
                            value
                                .split(',')
                                .map(str::trim)
                                .filter(|value| !value.is_empty())
                                .map(ToString::to_string)
                                .collect(),
                        );
                        i += 2;
                    }
                    "--plan" => {
                        let value = args.get(i + 1).ok_or("--plan requires a path")?;
                        plan_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--session" => {
                        let value = args.get(i + 1).ok_or("--session requires a path")?;
                        session_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store fetch-chunks: {other}"
                        ))
                    }
                }
            }
            if let Some(plan_path) = plan_path {
                let plan = read_delivery_plan(&plan_path)?;
                let store_path =
                    resolve_planetary_store_path(store_path, Some(&plan), None, "fetch-chunks")?;
                let refs = plan_chunk_refs(&plan);
                let mut by_scene = std::collections::BTreeMap::<String, Vec<String>>::new();
                for chunk_ref in refs {
                    by_scene
                        .entry(chunk_ref.scene_id)
                        .or_default()
                        .push(chunk_ref.chunk_id);
                }
                let subsets = by_scene
                    .into_iter()
                    .map(|(scene_id, chunk_ids)| {
                        read_chunks_by_ids(&store_path, &scene_id, &chunk_ids)
                            .map_err(|err| format!("planetary-store fetch-chunks failed: {err}"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&subsets)
                        .map_err(|err| format!("fetch-chunks json failed: {err}"))?
                );
                return Ok(());
            }
            if let Some(session_path) = session_path {
                let session = read_route_session(&session_path)?;
                let store_path = resolve_planetary_store_path(
                    store_path,
                    Some(&session.merged_plan),
                    Some(&session),
                    "fetch-chunks",
                )?;
                let refs = plan_chunk_refs(&session.merged_plan);
                let mut by_scene = std::collections::BTreeMap::<String, Vec<String>>::new();
                for chunk_ref in refs {
                    by_scene
                        .entry(chunk_ref.scene_id)
                        .or_default()
                        .push(chunk_ref.chunk_id);
                }
                let subsets = by_scene
                    .into_iter()
                    .map(|(scene_id, chunk_ids)| {
                        read_chunks_by_ids(&store_path, &scene_id, &chunk_ids)
                            .map_err(|err| format!("planetary-store fetch-chunks failed: {err}"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&subsets)
                        .map_err(|err| format!("fetch-chunks json failed: {err}"))?
                );
                return Ok(());
            }
            let store_path = resolve_planetary_store_path(store_path, None, None, "fetch-chunks")?;
            let scene_id = scene_id.ok_or("planetary-store fetch-chunks requires --scene ID")?;
            let chunk_ids =
                chunk_ids.ok_or("planetary-store fetch-chunks requires --chunk-ids ID1,ID2,...")?;
            let subset = read_chunks_by_ids(&store_path, &scene_id, &chunk_ids)
                .map_err(|err| format!("planetary-store fetch-chunks failed: {err}"))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&subset)
                    .map_err(|err| format!("fetch-chunks json failed: {err}"))?
            );
            Ok(())
        }
        "emit-manifest-subset" => {
            let mut store_path: Option<PathBuf> = None;
            let mut scene_id: Option<String> = None;
            let mut bbox_studs: Option<(f64, f64, f64, f64)> = None;
            let mut around_studs: Option<(f64, f64)> = None;
            let mut point: Option<(f64, f64)> = None;
            let mut tile: Option<(u8, u32, u32)> = None;
            let mut radius_studs: Option<f64> = None;
            let mut chunk_ids: Option<Vec<String>> = None;
            let mut plan_path: Option<PathBuf> = None;
            let mut session_path: Option<PathBuf> = None;
            let mut out_path: Option<PathBuf> = None;
            let mut limit: Option<usize> = None;
            let mut max_streaming_cost: Option<f64> = None;
            let mut max_estimated_memory_cost: Option<f64> = None;
            let mut require_buildings = false;
            let mut require_terrain = false;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--scene" => {
                        let value = args.get(i + 1).ok_or("--scene requires a scene id")?;
                        scene_id = Some(value.clone());
                        i += 2;
                    }
                    "--bbox-studs" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--bbox-studs requires MIN_X,MIN_Z,MAX_X,MAX_Z")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim().parse::<f64>().map_err(|_| {
                                    format!("invalid number in --bbox-studs: {}", part)
                                })
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 4 {
                            return Err(
                                "--bbox-studs requires four comma-separated numbers".to_string()
                            );
                        }
                        bbox_studs = Some((parts[0], parts[1], parts[2], parts[3]));
                        i += 2;
                    }
                    "--around-studs" => {
                        let value = args.get(i + 1).ok_or("--around-studs requires X,Z")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim().parse::<f64>().map_err(|_| {
                                    format!("invalid number in --around-studs: {}", part)
                                })
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 2 {
                            return Err(
                                "--around-studs requires two comma-separated numbers".to_string()
                            );
                        }
                        around_studs = Some((parts[0], parts[1]));
                        i += 2;
                    }
                    "--point" => {
                        let value = args.get(i + 1).ok_or("--point requires LAT,LON")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim()
                                    .parse::<f64>()
                                    .map_err(|_| format!("invalid number in --point: {}", part))
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 2 {
                            return Err("--point requires two comma-separated numbers".to_string());
                        }
                        point = Some((parts[0], parts[1]));
                        i += 2;
                    }
                    "--tile" => {
                        let value = args.get(i + 1).ok_or("--tile requires Z,X,Y")?;
                        let parts: Vec<u64> = value
                            .split(',')
                            .map(|part| {
                                part.trim()
                                    .parse::<u64>()
                                    .map_err(|_| format!("invalid number in --tile: {}", part))
                            })
                            .collect::<Result<Vec<u64>, String>>()?;
                        if parts.len() != 3 {
                            return Err(
                                "--tile requires three comma-separated integers".to_string()
                            );
                        }
                        tile = Some((parts[0] as u8, parts[1] as u32, parts[2] as u32));
                        i += 2;
                    }
                    "--radius-studs" => {
                        let value = args.get(i + 1).ok_or("--radius-studs requires a number")?;
                        radius_studs = Some(
                            value
                                .parse::<f64>()
                                .map_err(|_| format!("invalid --radius-studs value: {value}"))?,
                        );
                        i += 2;
                    }
                    "--chunk-ids" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--chunk-ids requires a comma-separated list")?;
                        chunk_ids = Some(
                            value
                                .split(',')
                                .map(str::trim)
                                .filter(|value| !value.is_empty())
                                .map(ToString::to_string)
                                .collect(),
                        );
                        i += 2;
                    }
                    "--plan" => {
                        let value = args.get(i + 1).ok_or("--plan requires a path")?;
                        plan_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--session" => {
                        let value = args.get(i + 1).ok_or("--session requires a path")?;
                        session_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--out" => {
                        let value = args.get(i + 1).ok_or("--out requires a path")?;
                        out_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--limit" => {
                        let value = args.get(i + 1).ok_or("--limit requires a number")?;
                        limit = Some(
                            value
                                .parse::<usize>()
                                .map_err(|_| format!("invalid --limit value: {value}"))?,
                        );
                        i += 2;
                    }
                    "--max-streaming-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-streaming-cost requires a number")?;
                        max_streaming_cost =
                            Some(value.parse::<f64>().map_err(|_| {
                                format!("invalid --max-streaming-cost value: {value}")
                            })?);
                        i += 2;
                    }
                    "--max-estimated-memory-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-estimated-memory-cost requires a number")?;
                        max_estimated_memory_cost = Some(value.parse::<f64>().map_err(|_| {
                            format!("invalid --max-estimated-memory-cost value: {value}")
                        })?);
                        i += 2;
                    }
                    "--require-buildings" => {
                        require_buildings = true;
                        i += 1;
                    }
                    "--require-terrain" => {
                        require_terrain = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store emit-manifest-subset: {other}"
                        ))
                    }
                }
            }
            if let Some(plan_path) = plan_path {
                let plan = read_delivery_plan(&plan_path)?;
                let store_path = resolve_planetary_store_path(
                    store_path,
                    Some(&plan),
                    None,
                    "emit-manifest-subset",
                )?;
                let subset = read_manifest_subset_for_plan_single_scene(
                    &store_path,
                    &plan,
                    "emit-manifest-subset",
                )
                .map_err(|err| {
                    format!("planetary-store emit-manifest-subset plan failed: {err}")
                })?;
                let manifest = build_manifest_value_from_stored_subset(subset)?;
                let manifest_json = serde_json::to_string_pretty(&manifest)
                    .map_err(|err| format!("emit-manifest-subset json failed: {err}"))?;
                if let Some(out_path) = out_path {
                    if let Some(parent) = out_path.parent() {
                        fs::create_dir_all(parent).map_err(|err| {
                            format!("emit-manifest-subset create dir failed: {err}")
                        })?;
                    }
                    fs::write(&out_path, format!("{manifest_json}\n"))
                        .map_err(|err| format!("emit-manifest-subset write failed: {err}"))?;
                    println!("Wrote manifest subset {}", out_path.display());
                } else {
                    println!("{manifest_json}");
                }
                return Ok(());
            }
            if let Some(session_path) = session_path {
                let session = read_route_session(&session_path)?;
                let store_path = resolve_planetary_store_path(
                    store_path,
                    Some(&session.merged_plan),
                    Some(&session),
                    "emit-manifest-subset",
                )?;
                let subset = read_manifest_subset_for_plan_single_scene(
                    &store_path,
                    &session.merged_plan,
                    "emit-manifest-subset",
                )
                .map_err(|err| {
                    format!("planetary-store emit-manifest-subset session failed: {err}")
                })?;
                let manifest = build_manifest_value_from_stored_subset(subset)?;
                let manifest_json = serde_json::to_string_pretty(&manifest)
                    .map_err(|err| format!("emit-manifest-subset json failed: {err}"))?;
                if let Some(out_path) = out_path {
                    if let Some(parent) = out_path.parent() {
                        fs::create_dir_all(parent).map_err(|err| {
                            format!("emit-manifest-subset create dir failed: {err}")
                        })?;
                    }
                    fs::write(&out_path, format!("{manifest_json}\n"))
                        .map_err(|err| format!("emit-manifest-subset write failed: {err}"))?;
                    println!("Wrote manifest subset {}", out_path.display());
                } else {
                    println!("{manifest_json}");
                }
                return Ok(());
            }
            let store_path =
                resolve_planetary_store_path(store_path, None, None, "emit-manifest-subset")?;
            let scene_id = resolve_planetary_scene_for_selection(
                &store_path,
                scene_id,
                point,
                tile,
                "emit-manifest-subset",
            )?;
            let subset = if let Some((min_x, min_z, max_x, max_z)) = bbox_studs {
                let chunk_ids = read_scene_chunk_summary_subset(
                    &store_path,
                    &scene_id,
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
                .map_err(|err| format!("planetary-store emit-manifest-subset bbox failed: {err}"))?
                .into_iter()
                .map(|chunk| chunk.chunk_id)
                .collect::<Vec<_>>();
                read_scene_manifest_subset_by_chunk_ids(&store_path, &scene_id, &chunk_ids)
                    .map_err(|err| {
                        format!("planetary-store emit-manifest-subset bbox chunk selection failed: {err}")
                    })?
            } else if let Some((focus_x, focus_z)) = around_studs {
                let radius_studs = radius_studs.ok_or(
                    "planetary-store emit-manifest-subset requires --radius-studs when using --around-studs",
                )?;
                let chunk_ids = read_scene_chunk_summary_around_point(
                    &store_path,
                    &scene_id,
                    focus_x,
                    focus_z,
                    radius_studs,
                    limit,
                    require_buildings,
                    require_terrain,
                    max_streaming_cost,
                    max_estimated_memory_cost,
                )
                .map_err(|err| {
                    format!("planetary-store emit-manifest-subset around-studs failed: {err}")
                })?
                .into_iter()
                .map(|chunk| chunk.chunk_id)
                .collect::<Vec<_>>();
                read_scene_manifest_subset_by_chunk_ids(&store_path, &scene_id, &chunk_ids)
                    .map_err(|err| format!("planetary-store emit-manifest-subset around chunk selection failed: {err}"))?
            } else if let Some((lat, lon)) = point {
                let radius_studs = radius_studs.ok_or(
                    "planetary-store emit-manifest-subset requires --radius-studs when using --point",
                )?;
                read_scene_manifest_subset_around_geo_point(
                    &store_path,
                    &scene_id,
                    lat,
                    lon,
                    radius_studs,
                    limit,
                    require_buildings,
                    require_terrain,
                    max_streaming_cost,
                    max_estimated_memory_cost,
                )
                .map_err(|err| {
                    format!("planetary-store emit-manifest-subset point failed: {err}")
                })?
            } else if let Some((zoom, x, y)) = tile {
                let chunk_ids = read_scene_chunk_summary_for_tile(
                    &store_path,
                    &scene_id,
                    zoom,
                    x,
                    y,
                    limit,
                    require_buildings,
                    require_terrain,
                    max_streaming_cost,
                    max_estimated_memory_cost,
                )
                .map_err(|err| format!("planetary-store emit-manifest-subset tile failed: {err}"))?
                .into_iter()
                .map(|chunk| chunk.chunk_id)
                .collect::<Vec<_>>();
                read_scene_manifest_subset_by_chunk_ids(&store_path, &scene_id, &chunk_ids)
                    .map_err(|err| {
                        format!("planetary-store emit-manifest-subset tile chunk selection failed: {err}")
                    })?
            } else if let Some(chunk_ids) = chunk_ids {
                read_scene_manifest_subset_by_chunk_ids(&store_path, &scene_id, &chunk_ids)
                    .map_err(|err| {
                        format!("planetary-store emit-manifest-subset chunk-ids failed: {err}")
                    })?
            } else {
                return Err(
                    "planetary-store emit-manifest-subset requires --bbox-studs, --around-studs, --point, --tile, or --chunk-ids"
                        .to_string(),
                );
            };
            let manifest = build_manifest_value_from_stored_subset(subset)?;
            let manifest_json = serde_json::to_string_pretty(&manifest)
                .map_err(|err| format!("emit-manifest-subset json failed: {err}"))?;
            if let Some(out_path) = out_path {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|err| format!("emit-manifest-subset create dir failed: {err}"))?;
                }
                fs::write(&out_path, format!("{manifest_json}\n"))
                    .map_err(|err| format!("emit-manifest-subset write failed: {err}"))?;
                println!("Wrote manifest subset {}", out_path.display());
            } else {
                println!("{manifest_json}");
            }
            Ok(())
        }
        "emit-runtime-lua" => {
            let mut store_path: Option<PathBuf> = None;
            let mut scene_id: Option<String> = None;
            let mut bbox_studs: Option<(f64, f64, f64, f64)> = None;
            let mut around_studs: Option<(f64, f64)> = None;
            let mut point: Option<(f64, f64)> = None;
            let mut tile: Option<(u8, u32, u32)> = None;
            let mut radius_studs: Option<f64> = None;
            let mut chunk_ids: Option<Vec<String>> = None;
            let mut plan_path: Option<PathBuf> = None;
            let mut session_path: Option<PathBuf> = None;
            let mut output_dir: Option<PathBuf> = None;
            let mut index_name = "PlanetaryManifestIndex".to_string();
            let mut shard_folder = "PlanetaryManifestChunks".to_string();
            let mut chunks_per_shard: usize = 32;
            let mut max_bytes: Option<usize> = None;
            let mut limit: Option<usize> = None;
            let mut max_streaming_cost: Option<f64> = None;
            let mut max_estimated_memory_cost: Option<f64> = None;
            let mut require_buildings = false;
            let mut require_terrain = false;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--scene" => {
                        let value = args.get(i + 1).ok_or("--scene requires a scene id")?;
                        scene_id = Some(value.clone());
                        i += 2;
                    }
                    "--bbox-studs" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--bbox-studs requires MIN_X,MIN_Z,MAX_X,MAX_Z")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim().parse::<f64>().map_err(|_| {
                                    format!("invalid number in --bbox-studs: {}", part)
                                })
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 4 {
                            return Err(
                                "--bbox-studs requires four comma-separated numbers".to_string()
                            );
                        }
                        bbox_studs = Some((parts[0], parts[1], parts[2], parts[3]));
                        i += 2;
                    }
                    "--around-studs" => {
                        let value = args.get(i + 1).ok_or("--around-studs requires X,Z")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim().parse::<f64>().map_err(|_| {
                                    format!("invalid number in --around-studs: {}", part)
                                })
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 2 {
                            return Err(
                                "--around-studs requires two comma-separated numbers".to_string()
                            );
                        }
                        around_studs = Some((parts[0], parts[1]));
                        i += 2;
                    }
                    "--point" => {
                        let value = args.get(i + 1).ok_or("--point requires LAT,LON")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim()
                                    .parse::<f64>()
                                    .map_err(|_| format!("invalid number in --point: {}", part))
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 2 {
                            return Err("--point requires two comma-separated numbers".to_string());
                        }
                        point = Some((parts[0], parts[1]));
                        i += 2;
                    }
                    "--tile" => {
                        let value = args.get(i + 1).ok_or("--tile requires Z,X,Y")?;
                        let parts: Vec<u64> = value
                            .split(',')
                            .map(|part| {
                                part.trim()
                                    .parse::<u64>()
                                    .map_err(|_| format!("invalid number in --tile: {}", part))
                            })
                            .collect::<Result<Vec<u64>, String>>()?;
                        if parts.len() != 3 {
                            return Err(
                                "--tile requires three comma-separated integers".to_string()
                            );
                        }
                        tile = Some((parts[0] as u8, parts[1] as u32, parts[2] as u32));
                        i += 2;
                    }
                    "--radius-studs" => {
                        let value = args.get(i + 1).ok_or("--radius-studs requires a number")?;
                        radius_studs = Some(
                            value
                                .parse::<f64>()
                                .map_err(|_| format!("invalid --radius-studs value: {value}"))?,
                        );
                        i += 2;
                    }
                    "--chunk-ids" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--chunk-ids requires a comma-separated list")?;
                        chunk_ids = Some(
                            value
                                .split(',')
                                .map(str::trim)
                                .filter(|value| !value.is_empty())
                                .map(ToString::to_string)
                                .collect(),
                        );
                        i += 2;
                    }
                    "--plan" => {
                        let value = args.get(i + 1).ok_or("--plan requires a path")?;
                        plan_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--session" => {
                        let value = args.get(i + 1).ok_or("--session requires a path")?;
                        session_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--output-dir" => {
                        let value = args.get(i + 1).ok_or("--output-dir requires a path")?;
                        output_dir = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--index-name" => {
                        index_name = args
                            .get(i + 1)
                            .ok_or("--index-name requires a value")?
                            .to_string();
                        i += 2;
                    }
                    "--shard-folder" => {
                        shard_folder = args
                            .get(i + 1)
                            .ok_or("--shard-folder requires a value")?
                            .to_string();
                        i += 2;
                    }
                    "--chunks-per-shard" => {
                        chunks_per_shard = args
                            .get(i + 1)
                            .ok_or("--chunks-per-shard requires a value")?
                            .parse::<usize>()
                            .map_err(|err| format!("invalid --chunks-per-shard: {err}"))?;
                        i += 2;
                    }
                    "--max-bytes" => {
                        max_bytes = Some(
                            args.get(i + 1)
                                .ok_or("--max-bytes requires a value")?
                                .parse::<usize>()
                                .map_err(|err| format!("invalid --max-bytes: {err}"))?,
                        );
                        i += 2;
                    }
                    "--limit" => {
                        let value = args.get(i + 1).ok_or("--limit requires a number")?;
                        limit = Some(
                            value
                                .parse::<usize>()
                                .map_err(|_| format!("invalid --limit value: {value}"))?,
                        );
                        i += 2;
                    }
                    "--max-streaming-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-streaming-cost requires a number")?;
                        max_streaming_cost =
                            Some(value.parse::<f64>().map_err(|_| {
                                format!("invalid --max-streaming-cost value: {value}")
                            })?);
                        i += 2;
                    }
                    "--max-estimated-memory-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-estimated-memory-cost requires a number")?;
                        max_estimated_memory_cost = Some(value.parse::<f64>().map_err(|_| {
                            format!("invalid --max-estimated-memory-cost value: {value}")
                        })?);
                        i += 2;
                    }
                    "--require-buildings" => {
                        require_buildings = true;
                        i += 1;
                    }
                    "--require-terrain" => {
                        require_terrain = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store emit-runtime-lua: {other}"
                        ))
                    }
                }
            }
            let output_dir =
                output_dir.ok_or("planetary-store emit-runtime-lua requires --output-dir PATH")?;
            if let Some(plan_path) = plan_path {
                let plan = read_delivery_plan(&plan_path)?;
                let store_path = resolve_planetary_store_path(
                    store_path,
                    Some(&plan),
                    None,
                    "emit-runtime-lua",
                )?;
                let subset = read_manifest_subset_for_plan_single_scene(
                    &store_path,
                    &plan,
                    "emit-runtime-lua",
                )
                .map_err(|err| format!("planetary-store emit-runtime-lua plan failed: {err}"))?;
                let stats = write_runtime_lua_shards_from_stored_subset(
                    &subset,
                    &RuntimeLuaShardsOptions {
                        output_dir,
                        index_name,
                        shard_folder,
                        chunks_per_shard,
                        max_bytes,
                    },
                )
                .map_err(|err| format!("planetary-store emit-runtime-lua failed: {err}"))?;
                println!("Wrote index module to {}", stats.index_path.display());
                println!(
                    "Wrote {} shard modules to {}",
                    stats.shard_count,
                    stats.shard_dir.display()
                );
                return Ok(());
            }
            if let Some(session_path) = session_path {
                let session = read_route_session(&session_path)?;
                let store_path = resolve_planetary_store_path(
                    store_path,
                    Some(&session.merged_plan),
                    Some(&session),
                    "emit-runtime-lua",
                )?;
                let subset = read_manifest_subset_for_plan_single_scene(
                    &store_path,
                    &session.merged_plan,
                    "emit-runtime-lua",
                )
                .map_err(|err| format!("planetary-store emit-runtime-lua session failed: {err}"))?;
                let stats = write_runtime_lua_shards_from_stored_subset(
                    &subset,
                    &RuntimeLuaShardsOptions {
                        output_dir,
                        index_name,
                        shard_folder,
                        chunks_per_shard,
                        max_bytes,
                    },
                )
                .map_err(|err| format!("planetary-store emit-runtime-lua failed: {err}"))?;
                println!("Wrote index module to {}", stats.index_path.display());
                println!(
                    "Wrote {} shard modules to {}",
                    stats.shard_count,
                    stats.shard_dir.display()
                );
                return Ok(());
            }
            let store_path =
                resolve_planetary_store_path(store_path, None, None, "emit-runtime-lua")?;
            let scene_id = resolve_planetary_scene_for_selection(
                &store_path,
                scene_id,
                point,
                tile,
                "emit-runtime-lua",
            )?;
            let subset = if let Some((min_x, min_z, max_x, max_z)) = bbox_studs {
                let chunk_ids = read_scene_chunk_summary_subset(
                    &store_path,
                    &scene_id,
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
                .map_err(|err| format!("planetary-store emit-runtime-lua bbox failed: {err}"))?
                .into_iter()
                .map(|chunk| chunk.chunk_id)
                .collect::<Vec<_>>();
                read_scene_manifest_subset_by_chunk_ids(&store_path, &scene_id, &chunk_ids)
                    .map_err(|err| {
                        format!(
                            "planetary-store emit-runtime-lua bbox chunk selection failed: {err}"
                        )
                    })?
            } else if let Some((focus_x, focus_z)) = around_studs {
                let radius_studs = radius_studs.ok_or(
                    "planetary-store emit-runtime-lua requires --radius-studs when using --around-studs",
                )?;
                let chunk_ids = read_scene_chunk_summary_around_point(
                    &store_path,
                    &scene_id,
                    focus_x,
                    focus_z,
                    radius_studs,
                    limit,
                    require_buildings,
                    require_terrain,
                    max_streaming_cost,
                    max_estimated_memory_cost,
                )
                .map_err(|err| {
                    format!("planetary-store emit-runtime-lua around-studs failed: {err}")
                })?
                .into_iter()
                .map(|chunk| chunk.chunk_id)
                .collect::<Vec<_>>();
                read_scene_manifest_subset_by_chunk_ids(&store_path, &scene_id, &chunk_ids)
                    .map_err(|err| {
                        format!(
                            "planetary-store emit-runtime-lua around chunk selection failed: {err}"
                        )
                    })?
            } else if let Some((lat, lon)) = point {
                let radius_studs = radius_studs.ok_or(
                    "planetary-store emit-runtime-lua requires --radius-studs when using --point",
                )?;
                read_scene_manifest_subset_around_geo_point(
                    &store_path,
                    &scene_id,
                    lat,
                    lon,
                    radius_studs,
                    limit,
                    require_buildings,
                    require_terrain,
                    max_streaming_cost,
                    max_estimated_memory_cost,
                )
                .map_err(|err| format!("planetary-store emit-runtime-lua point failed: {err}"))?
            } else if let Some((zoom, x, y)) = tile {
                let chunk_ids = read_scene_chunk_summary_for_tile(
                    &store_path,
                    &scene_id,
                    zoom,
                    x,
                    y,
                    limit,
                    require_buildings,
                    require_terrain,
                    max_streaming_cost,
                    max_estimated_memory_cost,
                )
                .map_err(|err| format!("planetary-store emit-runtime-lua tile failed: {err}"))?
                .into_iter()
                .map(|chunk| chunk.chunk_id)
                .collect::<Vec<_>>();
                read_scene_manifest_subset_by_chunk_ids(&store_path, &scene_id, &chunk_ids)
                    .map_err(|err| {
                        format!(
                            "planetary-store emit-runtime-lua tile chunk selection failed: {err}"
                        )
                    })?
            } else if let Some(chunk_ids) = chunk_ids {
                read_scene_manifest_subset_by_chunk_ids(&store_path, &scene_id, &chunk_ids)
                    .map_err(|err| {
                        format!("planetary-store emit-runtime-lua chunk-ids failed: {err}")
                    })?
            } else {
                return Err(
                    "planetary-store emit-runtime-lua requires --bbox-studs, --around-studs, --point, --tile, or --chunk-ids"
                        .to_string(),
                );
            };
            let stats = write_runtime_lua_shards_from_stored_subset(
                &subset,
                &RuntimeLuaShardsOptions {
                    output_dir,
                    index_name,
                    shard_folder,
                    chunks_per_shard,
                    max_bytes,
                },
            )
            .map_err(|err| format!("planetary-store emit-runtime-lua failed: {err}"))?;
            println!("Wrote index module to {}", stats.index_path.display());
            println!(
                "Wrote {} shard modules to {}",
                stats.shard_count,
                stats.shard_dir.display()
            );
            Ok(())
        }
        "find-scenes" => {
            let mut store_path: Option<PathBuf> = None;
            let mut bbox: Option<(f64, f64, f64, f64)> = None;
            let mut point: Option<(f64, f64)> = None;
            let mut tile: Option<(u8, u32, u32)> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--bbox" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--bbox requires MIN_LAT,MIN_LON,MAX_LAT,MAX_LON")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim()
                                    .parse::<f64>()
                                    .map_err(|_| format!("invalid number in --bbox: {}", part))
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 4 {
                            return Err("--bbox requires four comma-separated numbers".to_string());
                        }
                        bbox = Some((parts[0], parts[1], parts[2], parts[3]));
                        i += 2;
                    }
                    "--point" => {
                        let value = args.get(i + 1).ok_or("--point requires LAT,LON")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim()
                                    .parse::<f64>()
                                    .map_err(|_| format!("invalid number in --point: {}", part))
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 2 {
                            return Err("--point requires two comma-separated numbers".to_string());
                        }
                        point = Some((parts[0], parts[1]));
                        i += 2;
                    }
                    "--tile" => {
                        let value = args.get(i + 1).ok_or("--tile requires Z,X,Y")?;
                        let parts: Vec<u64> = value
                            .split(',')
                            .map(|part| {
                                part.trim()
                                    .parse::<u64>()
                                    .map_err(|_| format!("invalid number in --tile: {}", part))
                            })
                            .collect::<Result<Vec<u64>, String>>()?;
                        if parts.len() != 3 {
                            return Err(
                                "--tile requires three comma-separated integers".to_string()
                            );
                        }
                        tile = Some((parts[0] as u8, parts[1] as u32, parts[2] as u32));
                        i += 2;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store find-scenes: {other}"
                        ))
                    }
                }
            }
            let store_path =
                store_path.ok_or("planetary-store find-scenes requires --store PATH")?;
            let scenes = if let Some((min_lat, min_lon, max_lat, max_lon)) = bbox {
                find_scenes_intersecting_geo_bbox(&store_path, min_lat, min_lon, max_lat, max_lon)
                    .map_err(|err| format!("planetary-store find-scenes bbox failed: {err}"))?
            } else if let Some((lat, lon)) = point {
                find_scenes_covering_geo_point(&store_path, lat, lon)
                    .map_err(|err| format!("planetary-store find-scenes point failed: {err}"))?
            } else if let Some((zoom, x, y)) = tile {
                find_scenes_covering_tile(&store_path, zoom, x, y)
                    .map_err(|err| format!("planetary-store find-scenes tile failed: {err}"))?
            } else {
                return Err(
                    "planetary-store find-scenes requires --bbox, --point, or --tile".to_string(),
                );
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&scenes)
                    .map_err(|err| format!("find-scenes json failed: {err}"))?
            );
            Ok(())
        }
        "tile-scenes" => {
            let mut store_path: Option<PathBuf> = None;
            let mut tile: Option<(u8, u32, u32)> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--tile" => {
                        let value = args.get(i + 1).ok_or("--tile requires Z,X,Y")?;
                        let parts: Vec<u64> = value
                            .split(',')
                            .map(|part| {
                                part.trim()
                                    .parse::<u64>()
                                    .map_err(|_| format!("invalid number in --tile: {}", part))
                            })
                            .collect::<Result<Vec<u64>, String>>()?;
                        if parts.len() != 3 {
                            return Err(
                                "--tile requires three comma-separated integers".to_string()
                            );
                        }
                        tile = Some((parts[0] as u8, parts[1] as u32, parts[2] as u32));
                        i += 2;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store tile-scenes: {other}"
                        ))
                    }
                }
            }
            let store_path =
                store_path.ok_or("planetary-store tile-scenes requires --store PATH")?;
            let (zoom, x, y) = tile.ok_or("planetary-store tile-scenes requires --tile Z,X,Y")?;
            let scenes = find_scenes_covering_tile(&store_path, zoom, x, y)
                .map_err(|err| format!("planetary-store tile-scenes failed: {err}"))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&scenes)
                    .map_err(|err| format!("tile-scenes json failed: {err}"))?
            );
            Ok(())
        }
        "delivery-window" => {
            let mut store_path: Option<PathBuf> = None;
            let mut plan_path: Option<PathBuf> = None;
            let mut session_path: Option<PathBuf> = None;
            let mut scene_id: Option<String> = None;
            let mut bbox_studs: Option<(f64, f64, f64, f64)> = None;
            let mut around_studs: Option<(f64, f64)> = None;
            let mut point: Option<(f64, f64)> = None;
            let mut tile: Option<(u8, u32, u32)> = None;
            let mut radius_studs: Option<f64> = None;
            let mut limit: Option<usize> = None;
            let mut max_streaming_cost: Option<f64> = None;
            let mut max_estimated_memory_cost: Option<f64> = None;
            let mut require_buildings = false;
            let mut require_terrain = false;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--scene" => {
                        let value = args.get(i + 1).ok_or("--scene requires a scene id")?;
                        scene_id = Some(value.clone());
                        i += 2;
                    }
                    "--plan" => {
                        let value = args.get(i + 1).ok_or("--plan requires a path")?;
                        plan_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--session" => {
                        let value = args.get(i + 1).ok_or("--session requires a path")?;
                        session_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--bbox-studs" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--bbox-studs requires MIN_X,MIN_Z,MAX_X,MAX_Z")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim().parse::<f64>().map_err(|_| {
                                    format!("invalid number in --bbox-studs: {}", part)
                                })
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 4 {
                            return Err(
                                "--bbox-studs requires four comma-separated numbers".to_string()
                            );
                        }
                        bbox_studs = Some((parts[0], parts[1], parts[2], parts[3]));
                        i += 2;
                    }
                    "--around-studs" => {
                        let value = args.get(i + 1).ok_or("--around-studs requires X,Z")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim().parse::<f64>().map_err(|_| {
                                    format!("invalid number in --around-studs: {}", part)
                                })
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 2 {
                            return Err(
                                "--around-studs requires two comma-separated numbers".to_string()
                            );
                        }
                        around_studs = Some((parts[0], parts[1]));
                        i += 2;
                    }
                    "--point" => {
                        let value = args.get(i + 1).ok_or("--point requires LAT,LON")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim()
                                    .parse::<f64>()
                                    .map_err(|_| format!("invalid number in --point: {}", part))
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 2 {
                            return Err("--point requires two comma-separated numbers".to_string());
                        }
                        point = Some((parts[0], parts[1]));
                        i += 2;
                    }
                    "--tile" => {
                        let value = args.get(i + 1).ok_or("--tile requires Z,X,Y")?;
                        let parts: Vec<u64> = value
                            .split(',')
                            .map(|part| {
                                part.trim()
                                    .parse::<u64>()
                                    .map_err(|_| format!("invalid number in --tile: {}", part))
                            })
                            .collect::<Result<Vec<u64>, String>>()?;
                        if parts.len() != 3 {
                            return Err(
                                "--tile requires three comma-separated integers".to_string()
                            );
                        }
                        tile = Some((parts[0] as u8, parts[1] as u32, parts[2] as u32));
                        i += 2;
                    }
                    "--radius-studs" => {
                        let value = args.get(i + 1).ok_or("--radius-studs requires a number")?;
                        radius_studs = Some(
                            value
                                .parse::<f64>()
                                .map_err(|_| format!("invalid --radius-studs value: {value}"))?,
                        );
                        i += 2;
                    }
                    "--limit" => {
                        let value = args.get(i + 1).ok_or("--limit requires a number")?;
                        limit = Some(
                            value
                                .parse::<usize>()
                                .map_err(|_| format!("invalid --limit value: {value}"))?,
                        );
                        i += 2;
                    }
                    "--max-streaming-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-streaming-cost requires a number")?;
                        max_streaming_cost =
                            Some(value.parse::<f64>().map_err(|_| {
                                format!("invalid --max-streaming-cost value: {value}")
                            })?);
                        i += 2;
                    }
                    "--max-estimated-memory-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-estimated-memory-cost requires a number")?;
                        max_estimated_memory_cost = Some(value.parse::<f64>().map_err(|_| {
                            format!("invalid --max-estimated-memory-cost value: {value}")
                        })?);
                        i += 2;
                    }
                    "--require-buildings" => {
                        require_buildings = true;
                        i += 1;
                    }
                    "--require-terrain" => {
                        require_terrain = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store delivery-window: {other}"
                        ))
                    }
                }
            }
            let window = if let Some(plan_path) = plan_path {
                let plan = read_delivery_plan(&plan_path)?;
                let store_path =
                    resolve_planetary_store_path(store_path, Some(&plan), None, "delivery-window")?;
                build_delivery_window_from_plan(&store_path, &plan)
            } else if let Some(session_path) = session_path {
                let session = read_route_session(&session_path)?;
                let store_path = resolve_planetary_store_path(
                    store_path,
                    Some(&session.merged_plan),
                    Some(&session),
                    "delivery-window",
                )?;
                build_delivery_window_from_plan(&store_path, &session.merged_plan)
            } else if let Some((min_x, min_z, max_x, max_z)) = bbox_studs {
                let store_path =
                    resolve_planetary_store_path(store_path, None, None, "delivery-window")?;
                let scene_id = scene_id.ok_or(
                    "planetary-store delivery-window requires --scene ID when using --bbox-studs",
                )?;
                build_delivery_window_for_scene_bbox(
                    &store_path,
                    &scene_id,
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
            } else if let Some((focus_x, focus_z)) = around_studs {
                let store_path =
                    resolve_planetary_store_path(store_path, None, None, "delivery-window")?;
                let scene_id = scene_id.ok_or(
                    "planetary-store delivery-window requires --scene ID when using --around-studs",
                )?;
                let radius_studs = radius_studs.ok_or(
                    "planetary-store delivery-window requires --radius-studs R when using --around-studs",
                )?;
                build_delivery_window_around_point(
                    &store_path,
                    &scene_id,
                    focus_x,
                    focus_z,
                    radius_studs,
                    limit,
                    require_buildings,
                    require_terrain,
                    max_streaming_cost,
                    max_estimated_memory_cost,
                )
            } else if let Some((lat, lon)) = point {
                let store_path =
                    resolve_planetary_store_path(store_path, None, None, "delivery-window")?;
                let radius_studs = radius_studs.ok_or(
                    "planetary-store delivery-window requires --radius-studs R when using --point",
                )?;
                build_delivery_window_around_geo_point(
                    &store_path,
                    lat,
                    lon,
                    radius_studs,
                    limit,
                    require_buildings,
                    require_terrain,
                    max_streaming_cost,
                    max_estimated_memory_cost,
                )
            } else if let Some((zoom, x, y)) = tile {
                let store_path =
                    resolve_planetary_store_path(store_path, None, None, "delivery-window")?;
                build_delivery_window_for_tile(
                    &store_path,
                    zoom,
                    x,
                    y,
                    limit,
                    require_buildings,
                    require_terrain,
                    max_streaming_cost,
                    max_estimated_memory_cost,
                )
            } else {
                return Err(
                    "planetary-store delivery-window requires --bbox-studs, --around-studs, --point LAT,LON, or --tile Z,X,Y"
                        .to_string()
                );
            }
            .map_err(|err| format!("planetary-store delivery-window failed: {err}"))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&window)
                    .map_err(|err| format!("delivery-window json failed: {err}"))?
            );
            Ok(())
        }
        "delivery-plan" => {
            let mut store_path: Option<PathBuf> = None;
            let mut scene_id: Option<String> = None;
            let mut bbox_studs: Option<(f64, f64, f64, f64)> = None;
            let mut around_studs: Option<(f64, f64)> = None;
            let mut point: Option<(f64, f64)> = None;
            let mut tile: Option<(u8, u32, u32)> = None;
            let mut radius_studs: Option<f64> = None;
            let mut out_path: Option<PathBuf> = None;
            let mut limit: Option<usize> = None;
            let mut max_streaming_cost: Option<f64> = None;
            let mut max_estimated_memory_cost: Option<f64> = None;
            let mut require_buildings = false;
            let mut require_terrain = false;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--scene" => {
                        let value = args.get(i + 1).ok_or("--scene requires a scene id")?;
                        scene_id = Some(value.clone());
                        i += 2;
                    }
                    "--bbox-studs" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--bbox-studs requires MIN_X,MIN_Z,MAX_X,MAX_Z")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim().parse::<f64>().map_err(|_| {
                                    format!("invalid number in --bbox-studs: {}", part)
                                })
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 4 {
                            return Err(
                                "--bbox-studs requires four comma-separated numbers".to_string()
                            );
                        }
                        bbox_studs = Some((parts[0], parts[1], parts[2], parts[3]));
                        i += 2;
                    }
                    "--around-studs" => {
                        let value = args.get(i + 1).ok_or("--around-studs requires X,Z")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim().parse::<f64>().map_err(|_| {
                                    format!("invalid number in --around-studs: {}", part)
                                })
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 2 {
                            return Err(
                                "--around-studs requires two comma-separated numbers".to_string()
                            );
                        }
                        around_studs = Some((parts[0], parts[1]));
                        i += 2;
                    }
                    "--point" => {
                        let value = args.get(i + 1).ok_or("--point requires LAT,LON")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim()
                                    .parse::<f64>()
                                    .map_err(|_| format!("invalid number in --point: {}", part))
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 2 {
                            return Err("--point requires two comma-separated numbers".to_string());
                        }
                        point = Some((parts[0], parts[1]));
                        i += 2;
                    }
                    "--tile" => {
                        let value = args.get(i + 1).ok_or("--tile requires Z,X,Y")?;
                        let parts: Vec<u64> = value
                            .split(',')
                            .map(|part| {
                                part.trim()
                                    .parse::<u64>()
                                    .map_err(|_| format!("invalid number in --tile: {}", part))
                            })
                            .collect::<Result<Vec<u64>, String>>()?;
                        if parts.len() != 3 {
                            return Err(
                                "--tile requires three comma-separated integers".to_string()
                            );
                        }
                        tile = Some((parts[0] as u8, parts[1] as u32, parts[2] as u32));
                        i += 2;
                    }
                    "--radius-studs" => {
                        let value = args.get(i + 1).ok_or("--radius-studs requires a number")?;
                        radius_studs = Some(
                            value
                                .parse::<f64>()
                                .map_err(|_| format!("invalid --radius-studs value: {value}"))?,
                        );
                        i += 2;
                    }
                    "--out" => {
                        let value = args.get(i + 1).ok_or("--out requires a path")?;
                        out_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--limit" => {
                        let value = args.get(i + 1).ok_or("--limit requires a number")?;
                        limit = Some(
                            value
                                .parse::<usize>()
                                .map_err(|_| format!("invalid --limit value: {value}"))?,
                        );
                        i += 2;
                    }
                    "--max-streaming-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-streaming-cost requires a number")?;
                        max_streaming_cost =
                            Some(value.parse::<f64>().map_err(|_| {
                                format!("invalid --max-streaming-cost value: {value}")
                            })?);
                        i += 2;
                    }
                    "--max-estimated-memory-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-estimated-memory-cost requires a number")?;
                        max_estimated_memory_cost = Some(value.parse::<f64>().map_err(|_| {
                            format!("invalid --max-estimated-memory-cost value: {value}")
                        })?);
                        i += 2;
                    }
                    "--require-buildings" => {
                        require_buildings = true;
                        i += 1;
                    }
                    "--require-terrain" => {
                        require_terrain = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store delivery-plan: {other}"
                        ))
                    }
                }
            }
            let store_path = resolve_planetary_store_path(store_path, None, None, "delivery-plan")?;
            let plan = if let Some((min_x, min_z, max_x, max_z)) = bbox_studs {
                let scene_id = scene_id.ok_or(
                    "planetary-store delivery-plan requires --scene ID when using --bbox-studs",
                )?;
                build_delivery_plan_for_scene_bbox(
                    &store_path,
                    &scene_id,
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
            } else if let Some((focus_x, focus_z)) = around_studs {
                let scene_id = scene_id.ok_or(
                    "planetary-store delivery-plan requires --scene ID when using --around-studs",
                )?;
                let radius_studs = radius_studs.ok_or(
                    "planetary-store delivery-plan requires --radius-studs R when using --around-studs",
                )?;
                build_delivery_plan_around_point(
                    &store_path,
                    &scene_id,
                    focus_x,
                    focus_z,
                    radius_studs,
                    limit,
                    require_buildings,
                    require_terrain,
                    max_streaming_cost,
                    max_estimated_memory_cost,
                )
            } else if let Some((lat, lon)) = point {
                let radius_studs = radius_studs.ok_or(
                    "planetary-store delivery-plan requires --radius-studs R when using --point",
                )?;
                build_delivery_plan_around_geo_point(
                    &store_path,
                    lat,
                    lon,
                    radius_studs,
                    limit,
                    require_buildings,
                    require_terrain,
                    max_streaming_cost,
                    max_estimated_memory_cost,
                )
            } else if let Some((zoom, x, y)) = tile {
                build_delivery_plan_for_tile(
                    &store_path,
                    zoom,
                    x,
                    y,
                    limit,
                    require_buildings,
                    require_terrain,
                    max_streaming_cost,
                    max_estimated_memory_cost,
                )
            } else {
                return Err(
                    "planetary-store delivery-plan requires --bbox-studs, --around-studs, --point LAT,LON, or --tile Z,X,Y"
                        .to_string(),
                );
            }
            .map_err(|err| format!("planetary-store delivery-plan failed: {err}"))?;
            let plan_json = serde_json::to_string_pretty(&plan)
                .map_err(|err| format!("delivery-plan json failed: {err}"))?;
            if let Some(out_path) = out_path {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|err| format!("delivery-plan create dir failed: {err}"))?;
                }
                fs::write(&out_path, format!("{plan_json}\n"))
                    .map_err(|err| format!("delivery-plan write failed: {err}"))?;
                println!("Wrote delivery plan {}", out_path.display());
            } else {
                println!("{plan_json}");
            }
            Ok(())
        }
        "route-plan" => {
            let mut store_path: Option<PathBuf> = None;
            let mut points: Vec<(f64, f64)> = Vec::new();
            let mut radius_studs: Option<f64> = None;
            let mut out_path: Option<PathBuf> = None;
            let mut limit: Option<usize> = None;
            let mut max_streaming_cost: Option<f64> = None;
            let mut max_estimated_memory_cost: Option<f64> = None;
            let mut require_buildings = false;
            let mut require_terrain = false;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--point" => {
                        let value = args.get(i + 1).ok_or("--point requires LAT,LON")?;
                        points.push(parse_lat_lon_pair(value, "--point")?);
                        i += 2;
                    }
                    "--radius-studs" => {
                        let value = args.get(i + 1).ok_or("--radius-studs requires a number")?;
                        radius_studs = Some(
                            value
                                .parse::<f64>()
                                .map_err(|_| format!("invalid --radius-studs value: {value}"))?,
                        );
                        i += 2;
                    }
                    "--out" => {
                        let value = args.get(i + 1).ok_or("--out requires a path")?;
                        out_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--limit" => {
                        let value = args.get(i + 1).ok_or("--limit requires a number")?;
                        limit = Some(
                            value
                                .parse::<usize>()
                                .map_err(|_| format!("invalid --limit value: {value}"))?,
                        );
                        i += 2;
                    }
                    "--max-streaming-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-streaming-cost requires a number")?;
                        max_streaming_cost =
                            Some(value.parse::<f64>().map_err(|_| {
                                format!("invalid --max-streaming-cost value: {value}")
                            })?);
                        i += 2;
                    }
                    "--max-estimated-memory-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-estimated-memory-cost requires a number")?;
                        max_estimated_memory_cost = Some(value.parse::<f64>().map_err(|_| {
                            format!("invalid --max-estimated-memory-cost value: {value}")
                        })?);
                        i += 2;
                    }
                    "--require-buildings" => {
                        require_buildings = true;
                        i += 1;
                    }
                    "--require-terrain" => {
                        require_terrain = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store route-plan: {other}"
                        ))
                    }
                }
            }
            if points.is_empty() {
                return Err(
                    "planetary-store route-plan requires at least one --point LAT,LON".to_string(),
                );
            }
            let store_path = resolve_planetary_store_path(store_path, None, None, "route-plan")?;
            let radius_studs =
                radius_studs.ok_or("planetary-store route-plan requires --radius-studs R")?;
            let session = build_route_delivery_session_for_geo_points(
                &store_path,
                &points,
                radius_studs,
                limit,
                require_buildings,
                require_terrain,
                max_streaming_cost,
                max_estimated_memory_cost,
            )
            .map_err(|err| format!("planetary-store route-plan failed: {err}"))?;
            let merged = session
                .ok_or("planetary-store route-plan found no plans to merge")?
                .merged_plan;
            if let Some(out_path) = out_path.as_ref() {
                write_delivery_plan_json(&merged, out_path)?;
                println!("Wrote route plan {}", out_path.display());
            } else {
                let merged_json = serde_json::to_string_pretty(&merged)
                    .map_err(|err| format!("route-plan json failed: {err}"))?;
                println!("{merged_json}");
            }
            Ok(())
        }
        "route-session" => {
            let mut store_path: Option<PathBuf> = None;
            let mut points: Vec<(f64, f64)> = Vec::new();
            let mut radius_studs: Option<f64> = None;
            let mut out_path: Option<PathBuf> = None;
            let mut limit: Option<usize> = None;
            let mut max_streaming_cost: Option<f64> = None;
            let mut max_estimated_memory_cost: Option<f64> = None;
            let mut require_buildings = false;
            let mut require_terrain = false;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--point" => {
                        let value = args.get(i + 1).ok_or("--point requires LAT,LON")?;
                        points.push(parse_lat_lon_pair(value, "--point")?);
                        i += 2;
                    }
                    "--radius-studs" => {
                        let value = args.get(i + 1).ok_or("--radius-studs requires a number")?;
                        radius_studs = Some(
                            value
                                .parse::<f64>()
                                .map_err(|_| format!("invalid --radius-studs value: {value}"))?,
                        );
                        i += 2;
                    }
                    "--out" => {
                        let value = args.get(i + 1).ok_or("--out requires a path")?;
                        out_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--limit" => {
                        let value = args.get(i + 1).ok_or("--limit requires a number")?;
                        limit = Some(
                            value
                                .parse::<usize>()
                                .map_err(|_| format!("invalid --limit value: {value}"))?,
                        );
                        i += 2;
                    }
                    "--max-streaming-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-streaming-cost requires a number")?;
                        max_streaming_cost =
                            Some(value.parse::<f64>().map_err(|_| {
                                format!("invalid --max-streaming-cost value: {value}")
                            })?);
                        i += 2;
                    }
                    "--max-estimated-memory-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-estimated-memory-cost requires a number")?;
                        max_estimated_memory_cost = Some(value.parse::<f64>().map_err(|_| {
                            format!("invalid --max-estimated-memory-cost value: {value}")
                        })?);
                        i += 2;
                    }
                    "--require-buildings" => {
                        require_buildings = true;
                        i += 1;
                    }
                    "--require-terrain" => {
                        require_terrain = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store route-session: {other}"
                        ))
                    }
                }
            }
            if points.is_empty() {
                return Err(
                    "planetary-store route-session requires at least one --point LAT,LON"
                        .to_string(),
                );
            }
            let store_path = resolve_planetary_store_path(store_path, None, None, "route-session")?;
            let radius_studs =
                radius_studs.ok_or("planetary-store route-session requires --radius-studs R")?;
            let session = build_route_delivery_session_for_geo_points(
                &store_path,
                &points,
                radius_studs,
                limit,
                require_buildings,
                require_terrain,
                max_streaming_cost,
                max_estimated_memory_cost,
            )
            .map_err(|err| format!("planetary-store route-session failed: {err}"))?
            .ok_or("planetary-store route-session found no plans to merge")?;
            if let Some(out_path) = out_path.as_ref() {
                write_json_file(&session, out_path, "route-session")?;
                println!("Wrote route session {}", out_path.display());
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&session)
                        .map_err(|err| format!("route-session json failed: {err}"))?
                );
            }
            Ok(())
        }
        "hydrate-route-session" => {
            let mut store_path: Option<PathBuf> = None;
            let mut session_path: Option<PathBuf> = None;
            let mut out_path: Option<PathBuf> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--session" => {
                        let value = args.get(i + 1).ok_or("--session requires a path")?;
                        session_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--out" => {
                        let value = args.get(i + 1).ok_or("--out requires a path")?;
                        out_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store hydrate-route-session: {other}"
                        ))
                    }
                }
            }
            let session_path = session_path
                .ok_or("planetary-store hydrate-route-session requires --session PATH")?;
            let session = read_route_session(&session_path)?;
            let store_path = resolve_planetary_store_path(
                store_path,
                Some(&session.merged_plan),
                Some(&session),
                "hydrate-route-session",
            )?;
            let hydrated = build_hydrated_route_session(&store_path, &session)
                .map_err(|err| format!("planetary-store hydrate-route-session failed: {err}"))?
                .ok_or("planetary-store hydrate-route-session found no matching scene/chunks")?;
            if let Some(out_path) = out_path.as_ref() {
                write_json_file(&hydrated, out_path, "hydrate-route-session")?;
                println!("Wrote hydrated route session {}", out_path.display());
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&hydrated)
                        .map_err(|err| format!("hydrate-route-session json failed: {err}"))?
                );
            }
            Ok(())
        }
        "schedule-route-session" => {
            let mut store_path: Option<PathBuf> = None;
            let mut session_path: Option<PathBuf> = None;
            let mut out_path: Option<PathBuf> = None;
            let mut ahead_steps: usize = 1;
            let mut behind_steps: usize = 1;
            let mut max_prefetch_streaming_cost: Option<f64> = None;
            let mut max_prefetch_estimated_memory_cost: Option<f64> = None;
            let mut max_retain_streaming_cost: Option<f64> = None;
            let mut max_retain_estimated_memory_cost: Option<f64> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--session" => {
                        let value = args.get(i + 1).ok_or("--session requires a path")?;
                        session_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--ahead-steps" => {
                        let value = args.get(i + 1).ok_or("--ahead-steps requires a number")?;
                        ahead_steps = value
                            .parse::<usize>()
                            .map_err(|_| format!("invalid --ahead-steps value: {value}"))?;
                        i += 2;
                    }
                    "--behind-steps" => {
                        let value = args.get(i + 1).ok_or("--behind-steps requires a number")?;
                        behind_steps = value
                            .parse::<usize>()
                            .map_err(|_| format!("invalid --behind-steps value: {value}"))?;
                        i += 2;
                    }
                    "--max-prefetch-streaming-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-prefetch-streaming-cost requires a number")?;
                        max_prefetch_streaming_cost = Some(value.parse::<f64>().map_err(|_| {
                            format!("invalid --max-prefetch-streaming-cost value: {value}")
                        })?);
                        i += 2;
                    }
                    "--max-prefetch-estimated-memory-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-prefetch-estimated-memory-cost requires a number")?;
                        max_prefetch_estimated_memory_cost =
                            Some(value.parse::<f64>().map_err(|_| {
                                format!(
                                    "invalid --max-prefetch-estimated-memory-cost value: {value}"
                                )
                            })?);
                        i += 2;
                    }
                    "--max-retain-streaming-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-retain-streaming-cost requires a number")?;
                        max_retain_streaming_cost = Some(value.parse::<f64>().map_err(|_| {
                            format!("invalid --max-retain-streaming-cost value: {value}")
                        })?);
                        i += 2;
                    }
                    "--max-retain-estimated-memory-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-retain-estimated-memory-cost requires a number")?;
                        max_retain_estimated_memory_cost =
                            Some(value.parse::<f64>().map_err(|_| {
                                format!("invalid --max-retain-estimated-memory-cost value: {value}")
                            })?);
                        i += 2;
                    }
                    "--out" => {
                        let value = args.get(i + 1).ok_or("--out requires a path")?;
                        out_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store schedule-route-session: {other}"
                        ))
                    }
                }
            }
            let session_path = session_path
                .ok_or("planetary-store schedule-route-session requires --session PATH")?;
            let session = read_route_session(&session_path)?;
            let store_path = resolve_planetary_store_path(
                store_path,
                Some(&session.merged_plan),
                Some(&session),
                "schedule-route-session",
            )?;
            let hydrated = build_hydrated_route_session(&store_path, &session)
                .map_err(|err| format!("planetary-store schedule-route-session failed: {err}"))?
                .ok_or("planetary-store schedule-route-session found no matching scene/chunks")?;
            let schedule = build_route_delivery_schedule(
                &hydrated,
                ahead_steps,
                behind_steps,
                max_prefetch_streaming_cost,
                max_prefetch_estimated_memory_cost,
                max_retain_streaming_cost,
                max_retain_estimated_memory_cost,
            );
            if let Some(out_path) = out_path.as_ref() {
                write_json_file(&schedule, out_path, "schedule-route-session")?;
                println!("Wrote route delivery schedule {}", out_path.display());
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&schedule)
                        .map_err(|err| format!("schedule-route-session json failed: {err}"))?
                );
            }
            Ok(())
        }
        "materialize-route-schedule" => {
            let mut store_path: Option<PathBuf> = None;
            let mut session_path: Option<PathBuf> = None;
            let mut schedule_path: Option<PathBuf> = None;
            let mut out_dir: Option<PathBuf> = None;
            let mut ahead_steps: usize = 1;
            let mut behind_steps: usize = 1;
            let mut max_prefetch_streaming_cost: Option<f64> = None;
            let mut max_prefetch_estimated_memory_cost: Option<f64> = None;
            let mut max_retain_streaming_cost: Option<f64> = None;
            let mut max_retain_estimated_memory_cost: Option<f64> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--session" => {
                        let value = args.get(i + 1).ok_or("--session requires a path")?;
                        session_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--schedule" => {
                        let value = args.get(i + 1).ok_or("--schedule requires a path")?;
                        schedule_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--out-dir" => {
                        let value = args.get(i + 1).ok_or("--out-dir requires a path")?;
                        out_dir = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--ahead-steps" => {
                        let value = args.get(i + 1).ok_or("--ahead-steps requires a number")?;
                        ahead_steps = value
                            .parse::<usize>()
                            .map_err(|_| format!("invalid --ahead-steps value: {value}"))?;
                        i += 2;
                    }
                    "--behind-steps" => {
                        let value = args.get(i + 1).ok_or("--behind-steps requires a number")?;
                        behind_steps = value
                            .parse::<usize>()
                            .map_err(|_| format!("invalid --behind-steps value: {value}"))?;
                        i += 2;
                    }
                    "--max-prefetch-streaming-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-prefetch-streaming-cost requires a number")?;
                        max_prefetch_streaming_cost = Some(value.parse::<f64>().map_err(|_| {
                            format!("invalid --max-prefetch-streaming-cost value: {value}")
                        })?);
                        i += 2;
                    }
                    "--max-prefetch-estimated-memory-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-prefetch-estimated-memory-cost requires a number")?;
                        max_prefetch_estimated_memory_cost =
                            Some(value.parse::<f64>().map_err(|_| {
                                format!(
                                    "invalid --max-prefetch-estimated-memory-cost value: {value}"
                                )
                            })?);
                        i += 2;
                    }
                    "--max-retain-streaming-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-retain-streaming-cost requires a number")?;
                        max_retain_streaming_cost = Some(value.parse::<f64>().map_err(|_| {
                            format!("invalid --max-retain-streaming-cost value: {value}")
                        })?);
                        i += 2;
                    }
                    "--max-retain-estimated-memory-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-retain-estimated-memory-cost requires a number")?;
                        max_retain_estimated_memory_cost =
                            Some(value.parse::<f64>().map_err(|_| {
                                format!("invalid --max-retain-estimated-memory-cost value: {value}")
                            })?);
                        i += 2;
                    }
                    other => {
                        return Err(format!(
                        "unknown argument to planetary-store materialize-route-schedule: {other}"
                    ))
                    }
                }
            }
            let out_dir = out_dir
                .ok_or("planetary-store materialize-route-schedule requires --out-dir PATH")?;
            if let Some(schedule_path) = schedule_path {
                let text = fs::read_to_string(&schedule_path).map_err(|err| {
                    format!(
                        "failed reading route schedule {}: {err}",
                        schedule_path.display()
                    )
                })?;
                let schedule = serde_json::from_str::<
                    arbx_planetary_store::PlanetaryRouteDeliverySchedule,
                >(&text)
                .map_err(|err| {
                    format!(
                        "failed parsing route schedule {}: {err}",
                        schedule_path.display()
                    )
                })?;
                write_route_schedule_lane_files(&schedule, &out_dir, "materialize-route-schedule")?;
                println!("Wrote route schedule lane files {}", out_dir.display());
                return Ok(());
            }
            let session_path = session_path
                .ok_or("planetary-store materialize-route-schedule requires --session PATH or --schedule PATH")?;
            let session = read_route_session(&session_path)?;
            let store_path = resolve_planetary_store_path(
                store_path,
                Some(&session.merged_plan),
                Some(&session),
                "materialize-route-schedule",
            )?;
            let hydrated = build_hydrated_route_session(&store_path, &session)
                .map_err(|err| format!("planetary-store materialize-route-schedule failed: {err}"))?
                .ok_or(
                    "planetary-store materialize-route-schedule found no matching scene/chunks",
                )?;
            let schedule = build_route_delivery_schedule(
                &hydrated,
                ahead_steps,
                behind_steps,
                max_prefetch_streaming_cost,
                max_prefetch_estimated_memory_cost,
                max_retain_streaming_cost,
                max_retain_estimated_memory_cost,
            );
            write_route_schedule_lane_files(&schedule, &out_dir, "materialize-route-schedule")?;
            println!("Wrote route schedule lane files {}", out_dir.display());
            Ok(())
        }
        "merge-delivery-plans" => {
            let mut plan_paths: Vec<PathBuf> = Vec::new();
            let mut out_path: Option<PathBuf> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--plan" => {
                        let value = args.get(i + 1).ok_or("--plan requires a path")?;
                        plan_paths.push(PathBuf::from(value));
                        i += 2;
                    }
                    "--out" => {
                        let value = args.get(i + 1).ok_or("--out requires a path")?;
                        out_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store merge-delivery-plans: {other}"
                        ))
                    }
                }
            }
            if plan_paths.is_empty() {
                return Err(
                    "planetary-store merge-delivery-plans requires at least one --plan PATH"
                        .to_string(),
                );
            }
            let plans = plan_paths
                .iter()
                .map(|path| read_delivery_plan(path))
                .collect::<Result<Vec<_>, _>>()?;
            let merged = merge_delivery_plans(&plans)
                .map_err(|err| format!("planetary-store merge-delivery-plans failed: {err}"))?
                .ok_or("planetary-store merge-delivery-plans found no plans to merge")?;
            if let Some(out_path) = out_path.as_ref() {
                write_delivery_plan_json(&merged, out_path)?;
                println!("Wrote merged delivery plan {}", out_path.display());
            } else {
                let merged_json = serde_json::to_string_pretty(&merged)
                    .map_err(|err| format!("merge-delivery-plans json failed: {err}"))?;
                println!("{merged_json}");
            }
            Ok(())
        }
        "delivery-bundle" => {
            let mut store_path: Option<PathBuf> = None;
            let mut scene_id: Option<String> = None;
            let mut bbox_studs: Option<(f64, f64, f64, f64)> = None;
            let mut around_studs: Option<(f64, f64)> = None;
            let mut points: Vec<(f64, f64)> = Vec::new();
            let mut tile: Option<(u8, u32, u32)> = None;
            let mut radius_studs: Option<f64> = None;
            let mut plan_paths: Vec<PathBuf> = Vec::new();
            let mut session_path: Option<PathBuf> = None;
            let mut out_path: Option<PathBuf> = None;
            let mut plan_out: Option<PathBuf> = None;
            let mut route_session_out: Option<PathBuf> = None;
            let mut hydrated_route_out: Option<PathBuf> = None;
            let mut schedule_out: Option<PathBuf> = None;
            let mut step_lane_dir: Option<PathBuf> = None;
            let mut step_summary_dir: Option<PathBuf> = None;
            let mut step_window_dir: Option<PathBuf> = None;
            let mut step_transition_dir: Option<PathBuf> = None;
            let mut ahead_steps: usize = 1;
            let mut behind_steps: usize = 1;
            let mut max_prefetch_streaming_cost: Option<f64> = None;
            let mut max_prefetch_estimated_memory_cost: Option<f64> = None;
            let mut max_retain_streaming_cost: Option<f64> = None;
            let mut max_retain_estimated_memory_cost: Option<f64> = None;
            let mut summary_out: Option<PathBuf> = None;
            let mut window_out: Option<PathBuf> = None;
            let mut manifest_out: Option<PathBuf> = None;
            let mut output_dir: Option<PathBuf> = None;
            let mut index_name = "PlanetaryManifestIndex".to_string();
            let mut shard_folder = "PlanetaryManifestChunks".to_string();
            let mut chunks_per_shard: usize = 32;
            let mut max_bytes: Option<usize> = None;
            let mut limit: Option<usize> = None;
            let mut max_streaming_cost: Option<f64> = None;
            let mut max_estimated_memory_cost: Option<f64> = None;
            let mut require_buildings = false;
            let mut require_terrain = false;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--store" => {
                        let value = args.get(i + 1).ok_or("--store requires a path")?;
                        store_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--scene" => {
                        let value = args.get(i + 1).ok_or("--scene requires a scene id")?;
                        scene_id = Some(value.clone());
                        i += 2;
                    }
                    "--bbox-studs" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--bbox-studs requires MIN_X,MIN_Z,MAX_X,MAX_Z")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim().parse::<f64>().map_err(|_| {
                                    format!("invalid number in --bbox-studs: {}", part)
                                })
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 4 {
                            return Err(
                                "--bbox-studs requires four comma-separated numbers".to_string()
                            );
                        }
                        bbox_studs = Some((parts[0], parts[1], parts[2], parts[3]));
                        i += 2;
                    }
                    "--around-studs" => {
                        let value = args.get(i + 1).ok_or("--around-studs requires X,Z")?;
                        let parts: Vec<f64> = value
                            .split(',')
                            .map(|part| {
                                part.trim().parse::<f64>().map_err(|_| {
                                    format!("invalid number in --around-studs: {}", part)
                                })
                            })
                            .collect::<Result<Vec<f64>, String>>()?;
                        if parts.len() != 2 {
                            return Err(
                                "--around-studs requires two comma-separated numbers".to_string()
                            );
                        }
                        around_studs = Some((parts[0], parts[1]));
                        i += 2;
                    }
                    "--point" => {
                        let value = args.get(i + 1).ok_or("--point requires LAT,LON")?;
                        points.push(parse_lat_lon_pair(value, "--point")?);
                        i += 2;
                    }
                    "--tile" => {
                        let value = args.get(i + 1).ok_or("--tile requires Z,X,Y")?;
                        let parts: Vec<u64> = value
                            .split(',')
                            .map(|part| {
                                part.trim()
                                    .parse::<u64>()
                                    .map_err(|_| format!("invalid number in --tile: {}", part))
                            })
                            .collect::<Result<Vec<u64>, String>>()?;
                        if parts.len() != 3 {
                            return Err(
                                "--tile requires three comma-separated integers".to_string()
                            );
                        }
                        tile = Some((parts[0] as u8, parts[1] as u32, parts[2] as u32));
                        i += 2;
                    }
                    "--radius-studs" => {
                        let value = args.get(i + 1).ok_or("--radius-studs requires a number")?;
                        radius_studs = Some(
                            value
                                .parse::<f64>()
                                .map_err(|_| format!("invalid --radius-studs value: {value}"))?,
                        );
                        i += 2;
                    }
                    "--plan" => {
                        let value = args.get(i + 1).ok_or("--plan requires a path")?;
                        plan_paths.push(PathBuf::from(value));
                        i += 2;
                    }
                    "--session" => {
                        let value = args.get(i + 1).ok_or("--session requires a path")?;
                        session_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--out" => {
                        let value = args.get(i + 1).ok_or("--out requires a path")?;
                        out_path = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--plan-out" => {
                        let value = args.get(i + 1).ok_or("--plan-out requires a path")?;
                        plan_out = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--route-session-out" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--route-session-out requires a path")?;
                        route_session_out = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--hydrated-route-out" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--hydrated-route-out requires a path")?;
                        hydrated_route_out = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--schedule-out" => {
                        let value = args.get(i + 1).ok_or("--schedule-out requires a path")?;
                        schedule_out = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--step-lane-dir" => {
                        let value = args.get(i + 1).ok_or("--step-lane-dir requires a path")?;
                        step_lane_dir = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--step-summary-dir" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--step-summary-dir requires a path")?;
                        step_summary_dir = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--step-window-dir" => {
                        let value = args.get(i + 1).ok_or("--step-window-dir requires a path")?;
                        step_window_dir = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--step-transition-dir" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--step-transition-dir requires a path")?;
                        step_transition_dir = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--summary-out" => {
                        let value = args.get(i + 1).ok_or("--summary-out requires a path")?;
                        summary_out = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--window-out" => {
                        let value = args.get(i + 1).ok_or("--window-out requires a path")?;
                        window_out = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--manifest-out" => {
                        let value = args.get(i + 1).ok_or("--manifest-out requires a path")?;
                        manifest_out = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--output-dir" => {
                        let value = args.get(i + 1).ok_or("--output-dir requires a path")?;
                        output_dir = Some(PathBuf::from(value));
                        i += 2;
                    }
                    "--index-name" => {
                        index_name = args
                            .get(i + 1)
                            .ok_or("--index-name requires a value")?
                            .to_string();
                        i += 2;
                    }
                    "--shard-folder" => {
                        shard_folder = args
                            .get(i + 1)
                            .ok_or("--shard-folder requires a value")?
                            .to_string();
                        i += 2;
                    }
                    "--chunks-per-shard" => {
                        chunks_per_shard = args
                            .get(i + 1)
                            .ok_or("--chunks-per-shard requires a value")?
                            .parse::<usize>()
                            .map_err(|err| format!("invalid --chunks-per-shard: {err}"))?;
                        i += 2;
                    }
                    "--max-bytes" => {
                        max_bytes = Some(
                            args.get(i + 1)
                                .ok_or("--max-bytes requires a value")?
                                .parse::<usize>()
                                .map_err(|err| format!("invalid --max-bytes: {err}"))?,
                        );
                        i += 2;
                    }
                    "--ahead-steps" => {
                        let value = args.get(i + 1).ok_or("--ahead-steps requires a number")?;
                        ahead_steps = value
                            .parse::<usize>()
                            .map_err(|_| format!("invalid --ahead-steps value: {value}"))?;
                        i += 2;
                    }
                    "--behind-steps" => {
                        let value = args.get(i + 1).ok_or("--behind-steps requires a number")?;
                        behind_steps = value
                            .parse::<usize>()
                            .map_err(|_| format!("invalid --behind-steps value: {value}"))?;
                        i += 2;
                    }
                    "--max-prefetch-streaming-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-prefetch-streaming-cost requires a number")?;
                        max_prefetch_streaming_cost = Some(value.parse::<f64>().map_err(|_| {
                            format!("invalid --max-prefetch-streaming-cost value: {value}")
                        })?);
                        i += 2;
                    }
                    "--max-prefetch-estimated-memory-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-prefetch-estimated-memory-cost requires a number")?;
                        max_prefetch_estimated_memory_cost =
                            Some(value.parse::<f64>().map_err(|_| {
                                format!(
                                    "invalid --max-prefetch-estimated-memory-cost value: {value}"
                                )
                            })?);
                        i += 2;
                    }
                    "--max-retain-streaming-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-retain-streaming-cost requires a number")?;
                        max_retain_streaming_cost = Some(value.parse::<f64>().map_err(|_| {
                            format!("invalid --max-retain-streaming-cost value: {value}")
                        })?);
                        i += 2;
                    }
                    "--max-retain-estimated-memory-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-retain-estimated-memory-cost requires a number")?;
                        max_retain_estimated_memory_cost =
                            Some(value.parse::<f64>().map_err(|_| {
                                format!("invalid --max-retain-estimated-memory-cost value: {value}")
                            })?);
                        i += 2;
                    }
                    "--limit" => {
                        let value = args.get(i + 1).ok_or("--limit requires a number")?;
                        limit = Some(
                            value
                                .parse::<usize>()
                                .map_err(|_| format!("invalid --limit value: {value}"))?,
                        );
                        i += 2;
                    }
                    "--max-streaming-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-streaming-cost requires a number")?;
                        max_streaming_cost =
                            Some(value.parse::<f64>().map_err(|_| {
                                format!("invalid --max-streaming-cost value: {value}")
                            })?);
                        i += 2;
                    }
                    "--max-estimated-memory-cost" => {
                        let value = args
                            .get(i + 1)
                            .ok_or("--max-estimated-memory-cost requires a number")?;
                        max_estimated_memory_cost = Some(value.parse::<f64>().map_err(|_| {
                            format!("invalid --max-estimated-memory-cost value: {value}")
                        })?);
                        i += 2;
                    }
                    "--require-buildings" => {
                        require_buildings = true;
                        i += 1;
                    }
                    "--require-terrain" => {
                        require_terrain = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown argument to planetary-store delivery-bundle: {other}"
                        ))
                    }
                }
            }

            let route_session = if let Some(session_path) = session_path.as_ref() {
                Some(read_route_session(session_path)?)
            } else if points.len() > 1 {
                let store_path = resolve_planetary_store_path(
                    store_path.clone(),
                    None,
                    None,
                    "delivery-bundle",
                )?;
                let radius_studs = radius_studs.ok_or(
                    "planetary-store delivery-bundle requires --radius-studs R when using repeated --point",
                )?;
                build_route_delivery_session_for_geo_points(
                    &store_path,
                    &points,
                    radius_studs,
                    limit,
                    require_buildings,
                    require_terrain,
                    max_streaming_cost,
                    max_estimated_memory_cost,
                )
                .map_err(|err| format!("planetary-store delivery-bundle failed: {err}"))?
            } else {
                None
            };

            let plan = if let Some(session) = route_session.as_ref() {
                session.merged_plan.clone()
            } else if !plan_paths.is_empty() {
                let plans = plan_paths
                    .iter()
                    .map(|path| read_delivery_plan(path))
                    .collect::<Result<Vec<_>, _>>()?;
                merge_delivery_plans(&plans)
                    .map_err(|err| format!("planetary-store delivery-bundle failed: {err}"))?
                    .ok_or("planetary-store delivery-bundle found no matching scene/chunks")?
            } else {
                let store_path = resolve_planetary_store_path(
                    store_path.clone(),
                    None,
                    route_session.as_ref(),
                    "delivery-bundle",
                )?;
                let plan = if let Some((min_x, min_z, max_x, max_z)) = bbox_studs {
                    let scene_id = scene_id.ok_or(
                        "planetary-store delivery-bundle requires --scene ID when using --bbox-studs",
                    )?;
                    build_delivery_plan_for_scene_bbox(
                        &store_path,
                        &scene_id,
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
                } else if let Some((focus_x, focus_z)) = around_studs {
                    let scene_id = scene_id.ok_or(
                        "planetary-store delivery-bundle requires --scene ID when using --around-studs",
                    )?;
                    let radius_studs = radius_studs.ok_or(
                        "planetary-store delivery-bundle requires --radius-studs R when using --around-studs",
                    )?;
                    build_delivery_plan_around_point(
                        &store_path,
                        &scene_id,
                        focus_x,
                        focus_z,
                        radius_studs,
                        limit,
                        require_buildings,
                        require_terrain,
                        max_streaming_cost,
                        max_estimated_memory_cost,
                    )
                } else if points.len() == 1 {
                    let (lat, lon) = points[0];
                    let radius_studs = radius_studs.ok_or(
                        "planetary-store delivery-bundle requires --radius-studs R when using --point",
                    )?;
                    build_delivery_plan_around_geo_point(
                        &store_path,
                        lat,
                        lon,
                        radius_studs,
                        limit,
                        require_buildings,
                        require_terrain,
                        max_streaming_cost,
                        max_estimated_memory_cost,
                    )
                } else if let Some((zoom, x, y)) = tile {
                    build_delivery_plan_for_tile(
                        &store_path,
                        zoom,
                        x,
                        y,
                        limit,
                        require_buildings,
                        require_terrain,
                        max_streaming_cost,
                        max_estimated_memory_cost,
                    )
                } else {
                    return Err(
                        "planetary-store delivery-bundle requires --plan, --session, or a selector (--bbox-studs, --around-studs, repeated --point, or --tile)"
                            .to_string(),
                    );
                }
                .map_err(|err| format!("planetary-store delivery-bundle failed: {err}"))?
                .ok_or("planetary-store delivery-bundle found no matching scene/chunks")?;
                plan
            };

            let store_path = resolve_planetary_store_path(
                store_path,
                Some(&plan),
                route_session.as_ref(),
                "delivery-bundle",
            )?;

            if let Some(plan_out) = plan_out.as_ref() {
                write_delivery_plan_json(&plan, plan_out)?;
            }
            if let Some(route_session_out) = route_session_out.as_ref() {
                if let Some(route_session) = route_session.as_ref() {
                    write_json_file(
                        route_session,
                        route_session_out,
                        "delivery-bundle route-session",
                    )?;
                }
            }

            let hydrated_route = if let Some(route_session) = route_session.as_ref() {
                build_hydrated_route_session(&store_path, route_session).map_err(|err| {
                    format!("planetary-store delivery-bundle route hydration failed: {err}")
                })?
            } else {
                None
            };
            let route_schedule = hydrated_route.as_ref().map(|hydrated| {
                build_route_delivery_schedule(
                    hydrated,
                    ahead_steps,
                    behind_steps,
                    max_prefetch_streaming_cost,
                    max_prefetch_estimated_memory_cost,
                    max_retain_streaming_cost,
                    max_retain_estimated_memory_cost,
                )
            });
            if let Some(hydrated_route_out) = hydrated_route_out.as_ref() {
                if let Some(hydrated_route) = hydrated_route.as_ref() {
                    write_json_file(
                        hydrated_route,
                        hydrated_route_out,
                        "delivery-bundle hydrated-route",
                    )?;
                }
            }
            if let Some(schedule_out) = schedule_out.as_ref() {
                if let Some(route_schedule) = route_schedule.as_ref() {
                    write_json_file(
                        route_schedule,
                        schedule_out,
                        "delivery-bundle route schedule",
                    )?;
                }
            }
            if let Some(route_schedule) = route_schedule.as_ref() {
                if let Some(step_lane_dir) = step_lane_dir.as_ref() {
                    write_route_schedule_lane_files(
                        route_schedule,
                        step_lane_dir,
                        "delivery-bundle step lanes",
                    )?;
                }
            }
            if let Some(hydrated_route) = hydrated_route.as_ref() {
                if let Some(step_summary_dir) = step_summary_dir.as_ref() {
                    fs::create_dir_all(step_summary_dir).map_err(|err| {
                        format!("delivery-bundle step-summary-dir create failed: {err}")
                    })?;
                    for step in &hydrated_route.steps {
                        let out_path = step_summary_dir
                            .join(format!("step-{:03}-summary.json", step.step_index));
                        write_json_file(
                            &step.window.chunks,
                            &out_path,
                            "delivery-bundle step summary",
                        )?;
                    }
                }
                if let Some(step_window_dir) = step_window_dir.as_ref() {
                    fs::create_dir_all(step_window_dir).map_err(|err| {
                        format!("delivery-bundle step-window-dir create failed: {err}")
                    })?;
                    for step in &hydrated_route.steps {
                        let out_path = step_window_dir
                            .join(format!("step-{:03}-window.json", step.step_index));
                        write_json_file(&step.window, &out_path, "delivery-bundle step window")?;
                    }
                }
                if let Some(step_transition_dir) = step_transition_dir.as_ref() {
                    fs::create_dir_all(step_transition_dir).map_err(|err| {
                        format!("delivery-bundle step-transition-dir create failed: {err}")
                    })?;
                    for step in &hydrated_route.steps {
                        let out_path = step_transition_dir
                            .join(format!("step-{:03}-transition.json", step.step_index));
                        let transition = json!({
                            "stepIndex": step.step_index,
                            "entering": step.entering,
                            "retained": step.retained,
                            "leaving": step.leaving,
                        });
                        write_json_file(&transition, &out_path, "delivery-bundle step transition")?;
                    }
                }
            }

            let summary = read_chunk_summary_for_plan(&store_path, &plan)
                .map_err(|err| format!("planetary-store delivery-bundle summary failed: {err}"))?;
            if let Some(summary_out) = summary_out.as_ref() {
                write_json_file(&summary, summary_out, "delivery-bundle summary")?;
            }

            if let Some(window_out) = window_out.as_ref() {
                let window =
                    build_delivery_window_from_plan(&store_path, &plan).map_err(|err| {
                        format!("planetary-store delivery-bundle window failed: {err}")
                    })?;
                write_json_file(&window, window_out, "delivery-bundle window")?;
            }

            let mut result = DeliveryBundleResult {
                plan: plan.clone(),
                out: out_path.as_ref().map(|path| path.display().to_string()),
                plan_out: plan_out.as_ref().map(|path| path.display().to_string()),
                route_session_out: route_session_out
                    .as_ref()
                    .map(|path| path.display().to_string()),
                hydrated_route_out: hydrated_route_out
                    .as_ref()
                    .map(|path| path.display().to_string()),
                schedule_out: schedule_out.as_ref().map(|path| path.display().to_string()),
                step_lane_dir: step_lane_dir
                    .as_ref()
                    .map(|path| path.display().to_string()),
                step_summary_dir: step_summary_dir
                    .as_ref()
                    .map(|path| path.display().to_string()),
                step_window_dir: step_window_dir
                    .as_ref()
                    .map(|path| path.display().to_string()),
                step_transition_dir: step_transition_dir
                    .as_ref()
                    .map(|path| path.display().to_string()),
                summary_out: summary_out.as_ref().map(|path| path.display().to_string()),
                window_out: window_out.as_ref().map(|path| path.display().to_string()),
                manifest_out: manifest_out.as_ref().map(|path| path.display().to_string()),
                runtime: None,
            };

            if let Some(manifest_out) = manifest_out.as_ref() {
                let subset = read_manifest_subset_for_plan_single_scene(
                    &store_path,
                    &plan,
                    "delivery-bundle subset",
                )
                .map_err(|err| format!("planetary-store delivery-bundle subset failed: {err}"))?;
                let manifest = build_manifest_value_from_stored_subset(subset.clone())?;
                write_json_file(&manifest, manifest_out, "delivery-bundle manifest")?;
            }

            if let Some(output_dir) = output_dir {
                let subset = read_manifest_subset_for_plan_single_scene(
                    &store_path,
                    &plan,
                    "delivery-bundle subset",
                )
                .map_err(|err| format!("planetary-store delivery-bundle subset failed: {err}"))?;
                let stats = write_runtime_lua_shards_from_stored_subset(
                    &subset,
                    &RuntimeLuaShardsOptions {
                        output_dir,
                        index_name,
                        shard_folder,
                        chunks_per_shard,
                        max_bytes,
                    },
                )
                .map_err(|err| format!("planetary-store delivery-bundle runtime failed: {err}"))?;
                result.runtime = Some(DeliveryBundleRuntimeResult {
                    chunk_count: stats.chunk_count,
                    fragment_count: stats.fragment_count,
                    shard_count: stats.shard_count,
                    index_path: stats.index_path.display().to_string(),
                    shard_dir: stats.shard_dir.display().to_string(),
                });
            }

            if let Some(out_path) = out_path.as_ref() {
                write_json_file(&result, out_path, "delivery-bundle")?;
                println!("Wrote delivery bundle {}", out_path.display());
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result)
                        .map_err(|err| format!("delivery-bundle json failed: {err}"))?
                );
            }
            Ok(())
        }
        other => Err(format!("unknown planetary-store subcommand: {other}")),
    }
}

fn explain_text() -> String {
    r#"ARNIS HD PIPELINE — Architecture Overview

DATA FLOW:
  Input → Overpass JSON / Live API → Feature Extraction → Elevation Enrichment
  → Chunking → Satellite Classification → Schema 0.4.0 Manifest

SCHEMA VERSION: 0.4.0
SCALE: 1 stud = 0.3 meters (configurable)

FEATURE TYPES IN MANIFEST:
  terrain    Height grid with per-cell materials (satellite-classified)
  roads      Polylines with lanes, surface, elevated/tunnel flags, sidewalk mode
  rails      Polylines with track count
  buildings  Polygon shells with height, roof shape/color/material, usage, rooms
  water      Ribbons (rivers) or polygons (lakes) with surfaceY, holes for islands
  props      Point instances: 25+ types (trees, lamps, fountains, bollards, etc.)
  landuse    Ground polygons (parks, parking, forest, etc.)
  barriers   Linear features (walls, fences, hedges, guard rails)

ELEVATION:
  All feature Y positions are DEM-derived (Terrarium/SRTM).
  Roblox builders read manifest values directly — no runtime re-sampling.

SATELLITE CLASSIFICATION:
  When --satellite is enabled, the pipeline:
  1. Fetches ESRI World Imagery tiles at z19 (~0.3m/pixel)
  2. Classifies building roofs: Asphalt/Metal/Brick/WoodPlanks/Slate/Concrete
  3. Classifies terrain ground cover: Grass/LeafyGrass/Concrete/Asphalt/Rock/Ground
  4. Sets roof colors from satellite pixel values

ROBLOX IMPORT:
  The manifest is consumed by ImportService in Roblox Studio.
  Builders create Parts, EditableMesh, and Terrain voxels.
  WorldConfig.lua controls all rendering parameters.
  LOD system uses CollectionService tagging for distance culling.
  Day/night cycle toggles street lights and window glow.

RUST CRATES:
  arbx_geo             BoundingBox, elevation providers (Terrarium, SRTM, Flat)
  arbx_pipeline        Feature extraction, pipeline stages (validate/normalize/triangulate/enrich)
  arbx_roblox_export   Chunker, builders, satellite tile provider, manifest serialisation
  arbx_cli             CLI entry point (this binary)

ROBLOX MODULES:
  ImportService        Orchestrates chunk loading and builder dispatch
  StreamingService     Loads/unloads chunks based on player proximity
  ChunkSchema          Lua-side schema definition matching the JSON manifest
  WorldConfig          Rendering knobs: scale, LOD, instance budgets
  Profiler             Timing and instance-count telemetry

PIPELINE STAGES (in order):
  1. ValidateStage       Reject malformed or unsupported input features
  2. NormalizeStage      Canonicalise tags, units, and coordinate winding
  3. TriangulateStage    Decompose polygons for mesh builders
  4. ElevationEnrichment Inject DEM-derived Y offsets into every feature

MANIFEST TOP-LEVEL STRUCTURE:
  { "schemaVersion": "0.4.0",
    "meta": { worldName, generator, source, metersPerStud, chunkSizeStuds, bbox, totalFeatures },
    "chunks": [ { id, originStuds, terrain, roads, rails, buildings, water, props, landuse, barriers } ]
  }
"#
    .to_string()
}

fn cmd_explain() {
    print!("{}", explain_text());
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let Some(command) = args.first().map(String::as_str) else {
        print_help();
        return;
    };

    let result = match command {
        "sample" => cmd_sample(&args[1..]),
        "compile" => cmd_compile(&args[1..]),
        "config" => cmd_config(&args[1..]),
        "stats" => cmd_stats(&args[1..]),
        "validate" => cmd_validate(&args[1..]),
        "diff" => cmd_diff(&args[1..]),
        "scene-index" => cmd_scene_index(&args[1..]),
        "scene-audit" => cmd_scene_audit(&args[1..]),
        "emit-runtime-lua" => cmd_emit_runtime_lua(&args[1..]),
        "planetary-store" => cmd_planetary_store(&args[1..]),
        "explain" => {
            cmd_explain();
            Ok(())
        }
        "--help" | "-h" | "help" => {
            print_help();
            Ok(())
        }
        "--version" | "-V" | "version" => {
            println!("arbx_cli 0.4.0 (arnis-roblox HD pipeline)");
            Ok(())
        }
        other => Err(format!("unknown command: {other}")),
    };

    if let Err(message) = result {
        eprintln!("error: {message}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn srtm_tile_name_austin() {
        // Austin, TX: center ~30.265, -97.745 → SW corner N30W098
        assert_eq!(srtm_tile_name(30.265, -97.745), "N30W098");
    }

    #[test]
    fn srtm_tile_name_tokyo() {
        // Tokyo: center ~35.685, 139.695 → SW corner N35E139
        assert_eq!(srtm_tile_name(35.685, 139.695), "N35E139");
    }

    #[test]
    fn srtm_tile_name_london() {
        // London: center ~51.505, -0.125 → SW corner N51W001
        assert_eq!(srtm_tile_name(51.505, -0.125), "N51W001");
    }

    #[test]
    fn srtm_tile_name_southern_hemisphere() {
        // Santiago, Chile: center ~-33.455, -70.645 → SW corner S34W071
        assert_eq!(srtm_tile_name(-33.455, -70.645), "S34W071");
    }

    #[test]
    fn srtm_tile_name_eastern_zero() {
        // Exactly on the prime meridian, northern hemisphere → N51E000
        assert_eq!(srtm_tile_name(51.5, 0.0), "N51E000");
    }

    fn write_temp_manifest(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn diff_identical_manifests() {
        let content = r#"{
            "schemaVersion": "0.4.0",
            "meta": { "totalFeatures": 10 },
            "chunks": [{}, {}]
        }"#;
        let f1 = write_temp_manifest(content);
        let f2 = write_temp_manifest(content);
        assert!(cmd_diff(&[
            f1.path().to_str().unwrap().to_string(),
            f2.path().to_str().unwrap().to_string()
        ])
        .is_ok());
    }

    #[test]
    fn diff_same_version_manifests() {
        let c1 = r#"{ "schemaVersion": "0.4.0", "meta": { "totalFeatures": 10 }, "chunks": [] }"#;
        let c2 = r#"{ "schemaVersion": "0.4.0", "meta": { "totalFeatures": 11 }, "chunks": [] }"#;
        let f1 = write_temp_manifest(c1);
        let f2 = write_temp_manifest(c2);
        assert!(cmd_diff(&[
            f1.path().to_str().unwrap().to_string(),
            f2.path().to_str().unwrap().to_string()
        ])
        .is_ok());
    }

    #[test]
    fn diff_reports_schema_version_difference() {
        let v1: Value =
            serde_json::from_str(r#"{ "schemaVersion": "0.4.0", "meta": {}, "chunks": [] }"#)
                .unwrap();
        let v2: Value =
            serde_json::from_str(r#"{ "schemaVersion": "0.2.0", "meta": {}, "chunks": [] }"#)
                .unwrap();

        let message = schema_version_difference_message(&v1, &v2)
            .expect("expected a schema-version difference message");
        assert_eq!(message, "Schema versions differ: \"0.4.0\" vs \"0.2.0\"");
    }

    #[test]
    fn help_text_is_0_4_0_only() {
        let help = help_text();
        assert!(help.contains("Outputs Schema 0.4.0 JSON manifests."));
        assert!(help.contains("--profile high         cell=2 sat=off"));
        assert!(!help.contains("Migrations"));
        assert!(!help.contains("0.1.0"));
        assert!(!help.contains("0.2.0"));
        assert!(!help.contains("0.3.0"));
    }

    #[test]
    fn high_profile_keeps_cell_two_without_enabling_satellite() {
        let mut terrain_cell_size = 8;
        let mut satellite_dir = None;

        apply_compile_profile("high", &mut terrain_cell_size, &mut satellite_dir).unwrap();

        assert_eq!(terrain_cell_size, 2);
        assert!(satellite_dir.is_none());
    }

    #[test]
    fn explain_text_is_0_4_0_only() {
        let explain = explain_text();
        assert!(explain.contains("SCHEMA VERSION: 0.4.0"));
        assert!(explain.contains(r#"  { "schemaVersion": "0.4.0","#));
        assert!(!explain.contains("Migrations"));
        assert!(!explain.contains("0.1.0"));
        assert!(!explain.contains("0.2.0"));
        assert!(!explain.contains("0.3.0"));
    }

    #[test]
    fn validate_rejects_legacy_schema_versions_with_0_4_0_only_language() {
        let content = r#"{
            "schemaVersion": "0.1.0",
            "meta": {
                "worldName": "LegacyWorld",
                "generator": "unit-test",
                "source": "synthetic",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256,
                "bbox": { "minLat": 0, "minLon": 0, "maxLat": 1, "maxLon": 1 },
                "totalFeatures": 0
            },
            "chunks": [{
                "id": "0_0",
                "originStuds": { "x": 0, "y": 0, "z": 0 }
            }]
        }"#;
        let manifest = write_temp_manifest(content);
        let err = cmd_validate(&[manifest.path().to_str().unwrap().to_string()])
            .expect_err("legacy schema should be rejected");
        assert!(err.contains("only 0.4.0 manifests are supported"));
    }

    #[test]
    fn sample_command_writes_sqlite_manifest_store() {
        let db = NamedTempFile::new().unwrap();
        let db_path = db.path().to_path_buf();
        drop(db);

        cmd_sample(&[
            "--grid".to_string(),
            "2,2".to_string(),
            "--sqlite-out".to_string(),
            db_path.to_string_lossy().into_owned(),
        ])
        .unwrap();

        let subset = arbx_roblox_export::manifest_store::read_manifest_sqlite_subset(
            &db_path,
            &["0_0".to_string(), "1_1".to_string()],
        )
        .unwrap();
        assert_eq!(subset.chunks.len(), 2);
        assert_eq!(subset.chunks[0].chunk_id, "0_0");
        assert_eq!(subset.chunks[1].chunk_id, "1_1");

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn scene_index_reads_sqlite_manifest_store() {
        let manifest = build_sample_multi_chunk(2, 1);
        let db = NamedTempFile::new().unwrap();
        let db_path = db.path().to_path_buf();
        drop(db);
        write_manifest_sqlite(&manifest, &db_path).unwrap();

        let index = build_scene_manifest_index_from_sqlite(&db_path).unwrap();

        assert_eq!(index["meta"]["worldName"], "SampleAustinLikeBlock");
        assert_eq!(index["meta"]["schemaVersion"], "0.4.0");
        assert_eq!(index["meta"]["sceneIndexVersion"], SCENE_INDEX_VERSION);
        assert_eq!(index["chunks"].as_array().unwrap().len(), 2);
        assert_eq!(index["chunks"][0]["id"], "0_0");
        assert_eq!(index["chunks"][0]["roadCount"], 1);
        assert_eq!(index["chunks"][0]["buildingCount"], 1);
        assert_eq!(index["chunks"][1]["id"], "1_0");
        assert_eq!(index["chunks"][1]["roadCount"], 0);
        assert_eq!(index["chunks"][1]["buildingCount"], 0);

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn scene_audit_reads_sqlite_manifest_store() {
        let manifest = build_sample_multi_chunk(2, 1);
        let db = NamedTempFile::new().unwrap();
        let db_path = db.path().to_path_buf();
        drop(db);
        write_manifest_sqlite(&manifest, &db_path).unwrap();
        let log = r#"2026-03-20 18:35:04.000  ARNIS_SCENE_EDIT {"phase":"edit","focus":{"x":64.0,"z":64.0},"radius":200.0,"rootName":"GeneratedWorld_AustinPreview","scene":{"chunkCount":1,"chunkIds":["0_0"],"buildingModelCount":1,"chunksWithBuildingModels":1,"roadTaggedPartCount":1,"chunksWithRoadGeometry":1,"roadMeshPartCount":1,"roadDetailPartCount":0}}"#;

        let report =
            build_scene_audit_report_from_sqlite(&db_path, log, "ARNIS_SCENE_EDIT").unwrap();

        assert_eq!(report["manifest"]["chunkCount"], 1);
        assert_eq!(report["manifest"]["roadCount"], 1);
        assert_eq!(report["manifest"]["buildingCount"], 1);
        assert_eq!(report["summary"]["chunk_ratio"], 1.0);
        assert_eq!(report["summary"]["building_model_ratio"], 1.0);
        assert_eq!(report["summary"]["road_geometry_ratio"], 1.0);
        assert!(report["findings"].as_array().unwrap().is_empty());

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn scene_index_command_accepts_manifest_sqlite() {
        let manifest = build_sample_multi_chunk(2, 1);
        let db = NamedTempFile::new().unwrap();
        let db_path = db.path().to_path_buf();
        drop(db);
        write_manifest_sqlite(&manifest, &db_path).unwrap();
        let out = NamedTempFile::new().unwrap();
        let out_path = out.path().to_path_buf();
        drop(out);

        cmd_scene_index(&[
            "--manifest-sqlite".to_string(),
            db_path.to_string_lossy().into_owned(),
            "--json-out".to_string(),
            out_path.to_string_lossy().into_owned(),
        ])
        .unwrap();

        let index: Value =
            serde_json::from_str(&std::fs::read_to_string(&out_path).unwrap()).unwrap();
        assert_eq!(index["meta"]["worldName"], "SampleAustinLikeBlock");
        assert_eq!(index["chunks"].as_array().unwrap().len(), 2);

        let _ = std::fs::remove_file(db_path);
        let _ = std::fs::remove_file(out_path);
    }

    #[test]
    fn scene_audit_command_accepts_manifest_sqlite() {
        let manifest = build_sample_multi_chunk(2, 1);
        let db = NamedTempFile::new().unwrap();
        let db_path = db.path().to_path_buf();
        drop(db);
        write_manifest_sqlite(&manifest, &db_path).unwrap();
        let mut log = NamedTempFile::new().unwrap();
        writeln!(
            log,
            "2026-03-20 18:35:04.000  ARNIS_SCENE_EDIT {{\"phase\":\"edit\",\"focus\":{{\"x\":64.0,\"z\":64.0}},\"radius\":200.0,\"rootName\":\"GeneratedWorld_AustinPreview\",\"scene\":{{\"chunkCount\":1,\"chunkIds\":[\"0_0\"],\"buildingModelCount\":1,\"chunksWithBuildingModels\":1,\"roadTaggedPartCount\":1,\"chunksWithRoadGeometry\":1,\"roadMeshPartCount\":1,\"roadDetailPartCount\":0}}}}"
        )
        .unwrap();
        let out = NamedTempFile::new().unwrap();
        let out_path = out.path().to_path_buf();
        drop(out);

        cmd_scene_audit(&[
            "--manifest-sqlite".to_string(),
            db_path.to_string_lossy().into_owned(),
            "--log".to_string(),
            log.path().to_string_lossy().into_owned(),
            "--marker".to_string(),
            "ARNIS_SCENE_EDIT".to_string(),
            "--json-out".to_string(),
            out_path.to_string_lossy().into_owned(),
        ])
        .unwrap();

        let report: Value =
            serde_json::from_str(&std::fs::read_to_string(&out_path).unwrap()).unwrap();
        assert_eq!(report["manifest"]["chunkCount"], 1);
        assert_eq!(report["manifest"]["roadCount"], 1);
        assert_eq!(report["summary"]["chunk_ratio"], 1.0);

        let _ = std::fs::remove_file(db_path);
        let _ = std::fs::remove_file(out_path);
    }

    #[test]
    fn scene_audit_uses_latest_marker_and_exact_chunk_ids() {
        let manifest = r#"{
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "SceneAuditTown",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256
            },
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": { "x": 0, "y": 0, "z": 0 },
                    "roads": [{ "id": "road_1" }, { "id": "road_2" }],
                    "buildings": [{ "id": "bldg_1" }, { "id": "bldg_2" }],
                    "water": [],
                    "props": [],
                    "landuse": [],
                    "barriers": [],
                    "rails": []
                },
                {
                    "id": "1_0",
                    "originStuds": { "x": 256, "y": 0, "z": 0 },
                    "roads": [{ "id": "road_3" }],
                    "buildings": [{ "id": "bldg_3" }],
                    "water": [],
                    "props": [],
                    "landuse": [],
                    "barriers": [],
                    "rails": []
                }
            ]
        }"#;
        let log = r#"2026-03-20 18:35:01.000  ARNIS_SCENE_PLAY {"phase":"play","focus":{"x":64.0,"z":64.0},"radius":350.0,"rootName":"GeneratedWorld_Austin","scene":{"chunkCount":1,"buildingModelCount":1,"chunksWithBuildingModels":1,"roadTaggedPartCount":1,"chunksWithRoadGeometry":1}}
2026-03-20 18:35:02.000  noise
2026-03-20 18:35:03.000  ARNIS_SCENE_PLAY {"phase":"play","focus":{"x":128.0,"z":128.0},"radius":400.0,"rootName":"GeneratedWorld_Austin","scene":{"chunkCount":2,"chunkIds":["0_0","1_0"],"buildingModelCount":1,"buildingDetailPartCount":0,"chunksWithBuildingModels":1,"roadTaggedPartCount":0,"chunksWithRoadGeometry":0,"meshPartCount":0,"basePartCount":0}}"#;

        let report = build_scene_audit_report_from_str(manifest, log, "ARNIS_SCENE_PLAY").unwrap();

        assert_eq!(report["scene"]["chunkCount"], 2);
        assert_eq!(report["manifest"]["chunkCount"], 2);
        assert_eq!(report["manifest"]["buildingCount"], 3);
        assert_eq!(report["manifest"]["roadCount"], 3);
        assert_eq!(
            report["summary"]["building_model_ratio"].as_f64().unwrap(),
            1.0 / 3.0
        );
        assert_eq!(
            report["summary"]["road_geometry_ratio"].as_f64().unwrap(),
            0.0
        );
        assert_eq!(
            report["findings"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|finding| finding.get("code").and_then(|code| code.as_str()))
                .collect::<Vec<_>>(),
            vec!["missing_building_models", "missing_road_geometry"]
        );
    }

    #[test]
    fn scene_audit_ignores_truncated_later_marker_if_last_valid_marker_exists() {
        let manifest = r#"{
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "SceneAuditTown",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256
            },
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": { "x": 0, "y": 0, "z": 0 },
                    "roads": [],
                    "buildings": [],
                    "water": [],
                    "props": [],
                    "landuse": [],
                    "barriers": [],
                    "rails": []
                }
            ]
        }"#;
        let log = r#"2026-03-20 18:35:03.000  ARNIS_SCENE_EDIT {"phase":"edit","focus":{"x":128.0,"z":128.0},"radius":400.0,"rootName":"GeneratedWorld_Austin","scene":{"chunkCount":1,"buildingModelCount":0,"chunksWithBuildingModels":0,"roadTaggedPartCount":0,"chunksWithRoadGeometry":0,"meshPartCount":0,"basePartCount":0}}
2026-03-20 18:35:04.000  ARNIS_SCENE_EDIT {"phase":"edit","focus":{"x":128.0,"z":128.0},"radius":400.0,"rootName":"GeneratedWorld_Austin","scene":{"chunkCount":1,"buildingModelCount":0,"chunksWithBuildingModels":0"#;

        let report = build_scene_audit_report_from_str(manifest, log, "ARNIS_SCENE_EDIT").unwrap();

        assert_eq!(report["phase"], "edit");
        assert_eq!(report["scene"]["chunkCount"], 1);
    }

    #[test]
    fn scene_audit_merges_chunk_marker_into_scene_payload() {
        let manifest = r#"{
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "SceneAuditTown",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256
            },
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": { "x": 0, "y": 0, "z": 0 },
                    "roads": [{ "id": "road_1" }, { "id": "road_2" }],
                    "buildings": [{ "id": "bldg_1" }, { "id": "bldg_2" }],
                    "water": [],
                    "props": [],
                    "landuse": [],
                    "barriers": [],
                    "rails": []
                },
                {
                    "id": "1_0",
                    "originStuds": { "x": 256, "y": 0, "z": 0 },
                    "roads": [{ "id": "road_3" }],
                    "buildings": [{ "id": "bldg_3" }],
                    "water": [],
                    "props": [],
                    "landuse": [],
                    "barriers": [],
                    "rails": []
                }
            ]
        }"#;
        let log = r#"2026-03-20 18:35:03.000  ARNIS_SCENE_PLAY_CHUNKS {"phase":"play","chunkIds":["0_0","1_0"]}
2026-03-20 18:35:04.000  ARNIS_SCENE_PLAY {"phase":"play","focus":{"x":64.0,"z":64.0},"radius":50.0,"rootName":"GeneratedWorld_Austin","scene":{"chunkCount":2,"buildingModelCount":1,"chunksWithBuildingModels":1,"roadTaggedPartCount":0,"chunksWithRoadGeometry":0,"meshPartCount":0,"basePartCount":0}}"#;

        let report = build_scene_audit_report_from_str(manifest, log, "ARNIS_SCENE_PLAY").unwrap();

        assert_eq!(report["manifest"]["chunkCount"], 2);
        assert_eq!(report["manifest"]["buildingCount"], 3);
    }

    #[test]
    fn scene_audit_flags_chunk_gap_without_chunk_ids() {
        let manifest = r#"{
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "SceneAuditTown",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256
            },
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": { "x": 0, "y": 0, "z": 0 },
                    "roads": [],
                    "buildings": [],
                    "water": [],
                    "props": [],
                    "landuse": [],
                    "barriers": [],
                    "rails": []
                },
                {
                    "id": "1_0",
                    "originStuds": { "x": 256, "y": 0, "z": 0 },
                    "roads": [],
                    "buildings": [],
                    "water": [],
                    "props": [],
                    "landuse": [],
                    "barriers": [],
                    "rails": []
                }
            ]
        }"#;
        let log = r#"2026-03-20 18:35:03.000  ARNIS_SCENE_EDIT {"phase":"edit","focus":{"x":128.0,"z":128.0},"radius":400.0,"rootName":"GeneratedWorld_Austin","scene":{"chunkCount":1,"buildingModelCount":0,"chunksWithBuildingModels":0,"roadTaggedPartCount":0,"chunksWithRoadGeometry":0,"meshPartCount":0,"basePartCount":0}}"#;

        let report = build_scene_audit_report_from_str(manifest, log, "ARNIS_SCENE_EDIT").unwrap();
        let codes = report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|finding| finding.get("code").and_then(|code| code.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(report["manifest"]["chunkCount"], 2);
        assert!(codes.contains(&"scene_chunk_gap"));
    }

    #[test]
    fn scene_audit_flags_shell_loss_and_road_detail_skew() {
        let manifest = r#"{
            "meta": {
                "worldName": "SceneAuditTown",
                "chunkSizeStuds": 256
            },
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": { "x": 0, "y": 0, "z": 0 },
                    "roadCount": 1,
                    "buildingCount": 3,
                    "chunksWithRoads": true,
                    "chunksWithBuildings": true
                }
            ]
        }"#;
        let log = r#"2026-03-20 18:35:04.000  ARNIS_SCENE_EDIT {"phase":"edit","focus":{"x":128.0,"z":128.0},"radius":200.0,"rootName":"GeneratedWorld_AustinPreview","scene":{"chunkCount":1,"buildingModelCount":3,"chunksWithBuildingModels":1,"roadTaggedPartCount":4,"chunksWithRoadGeometry":1,"meshPartCount":1,"basePartCount":40,"buildingModelsWithDirectShell":1,"buildingModelsMissingDirectShell":2,"mergedBuildingMeshPartCount":7,"buildingShellPartCount":3,"roadMeshPartCount":1,"roadDetailPartCount":30}}"#;

        let report = build_scene_audit_report_from_str(manifest, log, "ARNIS_SCENE_EDIT").unwrap();
        let codes = report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|finding| finding.get("code").and_then(|code| code.as_str()))
            .collect::<Vec<_>>();

        assert!(codes.contains(&"missing_direct_building_shells"));
        assert!(codes.contains(&"merged_building_geometry_dominates"));
        assert!(codes.contains(&"road_detail_overwhelms_mesh"));
    }

    #[test]
    fn scene_audit_flags_missing_sidewalk_and_curb_surfaces() {
        let manifest = r#"{
            "meta": {
                "worldName": "SceneAuditTown",
                "chunkSizeStuds": 256
            },
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": { "x": 0, "y": 0, "z": 0 },
                    "roads": [
                        { "id": "road_1", "hasSidewalk": true }
                    ],
                    "buildings": [],
                    "water": [],
                    "props": [],
                    "landuse": [],
                    "barriers": [],
                    "rails": []
                }
            ]
        }"#;
        let log = r#"2026-03-20 18:35:04.000  ARNIS_SCENE_EDIT {"phase":"edit","focus":{"x":128.0,"z":128.0},"radius":200.0,"rootName":"GeneratedWorld_AustinPreview","scene":{"chunkCount":1,"buildingModelCount":0,"chunksWithBuildingModels":0,"roadTaggedPartCount":1,"chunksWithRoadGeometry":1,"roadMeshPartCount":1,"roadDetailPartCount":0,"sidewalkSurfacePartCount":0,"curbSurfacePartCount":0,"chunksWithSidewalkSurfaces":0,"chunksWithCurbSurfaces":0}}"#;

        let report = build_scene_audit_report_from_str(manifest, log, "ARNIS_SCENE_EDIT").unwrap();
        let codes = report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|finding| finding.get("code").and_then(|code| code.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(report["manifest"]["roadsWithSidewalks"], 1);
        assert_eq!(report["manifest"]["chunksWithSidewalkRoads"], 1);
        assert!(codes.contains(&"missing_sidewalk_surfaces"));
        assert!(codes.contains(&"missing_curb_surfaces"));
    }

    #[test]
    fn scene_audit_flags_missing_crossing_surfaces() {
        let manifest = r#"{
            "meta": {
                "worldName": "SceneAuditTown",
                "chunkSizeStuds": 256
            },
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": { "x": 0, "y": 0, "z": 0 },
                    "roads": [
                        { "id": "road_1", "kind": "footway", "subkind": "crossing" }
                    ],
                    "buildings": [],
                    "water": [],
                    "props": [],
                    "landuse": [],
                    "barriers": [],
                    "rails": []
                }
            ]
        }"#;
        let log = r#"2026-03-20 18:35:04.000  ARNIS_SCENE_EDIT {"phase":"edit","focus":{"x":128.0,"z":128.0},"radius":200.0,"rootName":"GeneratedWorld_AustinPreview","scene":{"chunkCount":1,"buildingModelCount":0,"chunksWithBuildingModels":0,"roadTaggedPartCount":0,"chunksWithRoadGeometry":0,"roadMeshPartCount":0,"roadDetailPartCount":0,"crossingSurfacePartCount":0,"chunksWithCrossingSurfaces":0}}"#;

        let report = build_scene_audit_report_from_str(manifest, log, "ARNIS_SCENE_EDIT").unwrap();
        let codes = report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|finding| finding.get("code").and_then(|code| code.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(report["manifest"]["roadsWithCrossings"], 1);
        assert_eq!(report["manifest"]["chunksWithCrossingRoads"], 1);
        assert!(codes.contains(&"missing_crossing_surfaces"));
    }

    #[test]
    fn scene_index_summarizes_chunk_counts_for_fast_audits() {
        let manifest = r#"{
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "SceneAuditTown",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256
            },
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": { "x": 0, "y": 0, "z": 0 },
                    "roads": [{ "id": "road_1" }],
                    "buildings": [{ "id": "bldg_1" }, { "id": "bldg_2" }],
                    "water": [],
                    "props": [],
                    "landuse": [],
                    "barriers": [],
                    "rails": []
                }
            ]
        }"#;

        let index = build_scene_manifest_index_from_str(manifest).unwrap();

        assert_eq!(index["meta"]["chunkSizeStuds"], 256);
        assert_eq!(index["meta"]["sceneIndexVersion"], SCENE_INDEX_VERSION);
        assert_eq!(index["chunks"][0]["id"], "0_0");
        assert_eq!(index["chunks"][0]["roadCount"], 1);
        assert_eq!(index["chunks"][0]["buildingCount"], 2);
        assert_eq!(index["chunks"][0]["chunksWithRoads"], true);
        assert_eq!(index["chunks"][0]["chunksWithBuildings"], true);
    }

    #[test]
    fn scene_index_counts_sidewalk_subkind_roads() {
        let manifest = r#"{
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "SceneAuditTown",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256
            },
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": { "x": 0, "y": 0, "z": 0 },
                    "roads": [
                        { "id": "road_1", "kind": "footway", "subkind": "sidewalk" },
                        { "id": "road_2", "kind": "residential", "hasSidewalk": true },
                        { "id": "road_3", "kind": "path" }
                    ],
                    "buildings": [],
                    "water": [],
                    "props": [],
                    "landuse": [],
                    "barriers": [],
                    "rails": []
                }
            ]
        }"#;

        let index = build_scene_manifest_index_from_str(manifest).unwrap();

        assert_eq!(index["chunks"][0]["roadsWithSidewalks"], 2);
        assert_eq!(index["chunks"][0]["chunksWithSidewalkRoads"], true);
        assert_eq!(index["chunks"][0]["roadsWithCrossings"], 0);
        assert_eq!(index["chunks"][0]["chunksWithCrossingRoads"], false);
    }

    #[test]
    fn scene_index_counts_crossing_subkind_roads() {
        let manifest = r#"{
            "schemaVersion": "0.4.0",
            "meta": {
                "worldName": "SceneAuditTown",
                "metersPerStud": 0.3,
                "chunkSizeStuds": 256
            },
            "chunks": [
                {
                    "id": "0_0",
                    "originStuds": { "x": 0, "y": 0, "z": 0 },
                    "roads": [
                        { "id": "road_1", "kind": "footway", "subkind": "crossing" },
                        { "id": "road_2", "kind": "pedestrian", "subkind": "crossing" },
                        { "id": "road_3", "kind": "path" }
                    ],
                    "buildings": [],
                    "water": [],
                    "props": [],
                    "landuse": [],
                    "barriers": [],
                    "rails": []
                }
            ]
        }"#;

        let index = build_scene_manifest_index_from_str(manifest).unwrap();

        assert_eq!(index["chunks"][0]["roadsWithCrossings"], 2);
        assert_eq!(index["chunks"][0]["chunksWithCrossingRoads"], true);
    }

    #[test]
    fn emit_runtime_lua_command_accepts_manifest_sqlite() {
        let manifest = build_sample_multi_chunk(2, 1);
        let db = NamedTempFile::new().unwrap();
        let db_path = db.path().to_path_buf();
        drop(db);
        write_manifest_sqlite(&manifest, &db_path).unwrap();

        let tempdir = tempfile::tempdir().unwrap();
        let output_dir = tempdir.path().join("SampleData");

        cmd_emit_runtime_lua(&[
            "--manifest-sqlite".to_string(),
            db_path.to_string_lossy().into_owned(),
            "--output-dir".to_string(),
            output_dir.to_string_lossy().into_owned(),
            "--index-name".to_string(),
            "TestManifestIndex".to_string(),
            "--shard-folder".to_string(),
            "TestManifestChunks".to_string(),
            "--chunks-per-shard".to_string(),
            "1".to_string(),
            "--max-bytes".to_string(),
            "1200".to_string(),
        ])
        .unwrap();

        let index_text = std::fs::read_to_string(output_dir.join("TestManifestIndex.lua")).unwrap();
        let shard_count = std::fs::read_dir(output_dir.join("TestManifestChunks"))
            .unwrap()
            .count();

        assert!(index_text.contains("chunkCount=2"));
        assert!(index_text.contains("fragmentCount="));
        assert!(index_text.contains("chunkRefs="));
        assert!(shard_count >= 2);
    }

    #[test]
    fn emit_runtime_lua_keeps_each_shard_within_max_bytes() {
        let manifest = build_sample_multi_chunk(2, 2);
        let db = NamedTempFile::new().unwrap();
        let db_path = db.path().to_path_buf();
        drop(db);
        write_manifest_sqlite(&manifest, &db_path).unwrap();

        let tempdir = tempfile::tempdir().unwrap();
        let output_dir = tempdir.path().join("SampleData");

        cmd_emit_runtime_lua(&[
            "--manifest-sqlite".to_string(),
            db_path.to_string_lossy().into_owned(),
            "--output-dir".to_string(),
            output_dir.to_string_lossy().into_owned(),
            "--index-name".to_string(),
            "TestManifestIndex".to_string(),
            "--shard-folder".to_string(),
            "TestManifestChunks".to_string(),
            "--chunks-per-shard".to_string(),
            "32".to_string(),
            "--max-bytes".to_string(),
            "1200".to_string(),
        ])
        .unwrap();

        let shard_dir = output_dir.join("TestManifestChunks");
        let shard_sizes = std::fs::read_dir(&shard_dir)
            .unwrap()
            .map(|entry| {
                let path = entry.unwrap().path();
                std::fs::metadata(path).unwrap().len() as usize
            })
            .collect::<Vec<_>>();

        assert!(
            !shard_sizes.is_empty(),
            "expected runtime emitter to write at least one shard"
        );
        assert!(
            shard_sizes.iter().all(|size| *size <= 1200),
            "expected every runtime shard module to respect --max-bytes; saw sizes {shard_sizes:?}"
        );
    }

    #[test]
    fn planetary_store_init_and_summary_work() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");

        cmd_planetary_store(&[
            "init".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "summary".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_ingests_manifest_sqlite() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let manifest = build_sample_multi_chunk(2, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_subset_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "subset".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--bbox-studs".to_string(),
            "200,0,500,200".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_list_scenes_and_scene_work() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let manifest = build_sample_multi_chunk(2, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "list-scenes".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "scene".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_subset_summary_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "subset-summary".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--bbox-studs".to_string(),
            "200,0,500,200".to_string(),
            "--limit".to_string(),
            "1".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_subset_summary_filters_payload_classes() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "subset-summary".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--bbox-studs".to_string(),
            "-10,-10,800,300".to_string(),
            "--require-buildings".to_string(),
            "--require-terrain".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_subset_summary_around_studs_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "subset-summary".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--around-studs".to_string(),
            "300,128".to_string(),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--limit".to_string(),
            "2".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_subset_summary_point_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "subset-summary".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--limit".to_string(),
            "2".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_subset_summary_point_budget_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "subset-summary".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--max-streaming-cost".to_string(),
            "24".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_subset_summary_point_without_scene_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "subset-summary".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--limit".to_string(),
            "1".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_subset_summary_plan_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let plan_path = tempdir.path().join("delivery-plan.json");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-plan".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            plan_path.display().to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "subset-summary".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--plan".to_string(),
            plan_path.display().to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_find_scenes_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let mut manifest = build_sample_multi_chunk(1, 1);
        manifest.meta.bbox = arbx_geo::BoundingBox::new(30.0, -98.0, 30.5, -97.5);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "find-scenes".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            "30.2,-97.8".to_string(),
        ])
        .unwrap();

        let center = manifest.meta.bbox.center();
        let zoom = 10u8;
        let x = (((center.lon + 180.0) / 360.0) * 2f64.powi(zoom as i32)).floor() as u32;
        let lat_rad = center.lat.to_radians();
        let y = ((1.0 - ((lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI)) / 2.0
            * 2f64.powi(zoom as i32))
        .floor() as u32;

        cmd_planetary_store(&[
            "tile-scenes".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--tile".to_string(),
            format!("{zoom},{x},{y}"),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_ingest_json_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.json");
        let manifest = build_sample_multi_chunk(2, 1);
        std::fs::write(&manifest_path, manifest.to_json_pretty()).unwrap();

        cmd_planetary_store(&[
            "ingest-json".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-json".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "json_scene".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_attach_truth_pack_summary_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let truth_pack_summary_path = tempdir.path().join("truth-pack-summary.json");
        let manifest = build_sample_multi_chunk(1, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        let summary = SourceTruthPackSummary {
            scene: "austin".to_string(),
            feature_count: 12,
            retained_semantic_count: 8,
            semantic_lineage_count: 6,
            dropped_semantic_count: 2,
            collapse_count: 1,
            source_counts: std::collections::BTreeMap::from([("overpass".to_string(), 5usize)]),
        };
        std::fs::write(
            &truth_pack_summary_path,
            serde_json::to_string_pretty(&summary).unwrap(),
        )
        .unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "attach-truth-pack-summary".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--truth-pack-summary".to_string(),
            truth_pack_summary_path.display().to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_fetch_chunks_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "fetch-chunks".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--chunk-ids".to_string(),
            "0_0,2_0".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_delivery_plan_and_fetch_chunks_plan_work() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let plan_path = tempdir.path().join("delivery-plan.json");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-plan".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            plan_path.display().to_string(),
        ])
        .unwrap();

        let plan: Value =
            serde_json::from_str(&std::fs::read_to_string(&plan_path).unwrap()).unwrap();
        assert_eq!(plan["scene_id"], "austin");
        assert!(!plan["chunk_ids"].as_array().unwrap().is_empty());

        cmd_planetary_store(&[
            "fetch-chunks".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--plan".to_string(),
            plan_path.display().to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_merge_delivery_plans_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let plan_a_path = tempdir.path().join("delivery-a.json");
        let plan_b_path = tempdir.path().join("delivery-b.json");
        let merged_path = tempdir.path().join("delivery-merged.json");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-plan".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            plan_a_path.display().to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-plan".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--around-studs".to_string(),
            "300,128".to_string(),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            plan_b_path.display().to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "merge-delivery-plans".to_string(),
            "--plan".to_string(),
            plan_a_path.display().to_string(),
            "--plan".to_string(),
            plan_b_path.display().to_string(),
            "--out".to_string(),
            merged_path.display().to_string(),
        ])
        .unwrap();

        let merged: Value =
            serde_json::from_str(&std::fs::read_to_string(&merged_path).unwrap()).unwrap();
        assert_eq!(merged["selection_mode"], "merged");
        assert_eq!(merged["source_plan_count"], 2);
        assert_eq!(merged["source_selector_count"], 2);
        assert_eq!(merged["chunk_count"], 2);
    }

    #[test]
    fn planetary_store_route_plan_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let route_path = tempdir.path().join("route-plan.json");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "route-plan".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon + 0.0002),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            route_path.display().to_string(),
        ])
        .unwrap();

        let route: Value =
            serde_json::from_str(&std::fs::read_to_string(&route_path).unwrap()).unwrap();
        assert_eq!(route["selection_mode"], "merged");
        assert_eq!(route["source_plan_count"], 2);
        assert_eq!(route["source_selector_count"], 2);
        assert!(route["chunk_count"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn planetary_store_route_session_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let session_path = tempdir.path().join("route-session.json");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "route-session".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon + 0.0002),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            session_path.display().to_string(),
        ])
        .unwrap();

        let session: Value =
            serde_json::from_str(&std::fs::read_to_string(&session_path).unwrap()).unwrap();
        assert_eq!(session["step_count"], 2);
        assert_eq!(session["steps"].as_array().unwrap().len(), 2);
        assert_eq!(session["merged_plan"]["selection_mode"], "merged");
    }

    #[test]
    fn planetary_store_hydrate_route_session_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let session_path = tempdir.path().join("route-session.json");
        let hydrated_path = tempdir.path().join("hydrated-route-session.json");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "route-session".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon + 0.0002),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            session_path.display().to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "hydrate-route-session".to_string(),
            "--session".to_string(),
            session_path.display().to_string(),
            "--out".to_string(),
            hydrated_path.display().to_string(),
        ])
        .unwrap();

        let hydrated: Value =
            serde_json::from_str(&std::fs::read_to_string(&hydrated_path).unwrap()).unwrap();
        assert_eq!(hydrated["steps"].as_array().unwrap().len(), 2);
        assert_eq!(hydrated["steps"][0]["entering"]["chunk_count"], 1);
        assert_eq!(hydrated["steps"][1]["retained"]["chunk_count"], 1);
    }

    #[test]
    fn planetary_store_schedule_route_session_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let session_path = tempdir.path().join("route-session.json");
        let schedule_path = tempdir.path().join("route-schedule.json");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "route-session".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon + 0.0002),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon + 0.0004),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            session_path.display().to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "schedule-route-session".to_string(),
            "--session".to_string(),
            session_path.display().to_string(),
            "--ahead-steps".to_string(),
            "1".to_string(),
            "--behind-steps".to_string(),
            "1".to_string(),
            "--max-prefetch-streaming-cost".to_string(),
            "8".to_string(),
            "--out".to_string(),
            schedule_path.display().to_string(),
        ])
        .unwrap();

        let schedule: Value =
            serde_json::from_str(&std::fs::read_to_string(&schedule_path).unwrap()).unwrap();
        assert_eq!(schedule["steps"].as_array().unwrap().len(), 3);
        assert_eq!(schedule["steps"][0]["active"]["chunk_count"], 1);
        assert_eq!(schedule["ahead_steps"], 1);
        assert_eq!(schedule["behind_steps"], 1);
        assert_eq!(schedule["max_prefetch_streaming_cost"], 8.0);
        assert!(
            schedule["steps"][0]["prefetch"]["total_streaming_cost"]
                .as_f64()
                .unwrap()
                <= 8.0
        );
    }

    #[test]
    fn planetary_store_materialize_route_schedule_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let session_path = tempdir.path().join("route-session.json");
        let lane_dir = tempdir.path().join("route-lanes");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "route-session".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon + 0.0002),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            session_path.display().to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "materialize-route-schedule".to_string(),
            "--session".to_string(),
            session_path.display().to_string(),
            "--out-dir".to_string(),
            lane_dir.display().to_string(),
        ])
        .unwrap();

        assert!(lane_dir.join("step-000-active.json").exists());
        assert!(lane_dir.join("step-000-prefetch.json").exists());
        assert!(lane_dir.join("step-000-retain.json").exists());
    }

    #[test]
    fn planetary_store_emit_manifest_subset_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let out_path = tempdir.path().join("subset.json");
        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "emit-manifest-subset".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--chunk-ids".to_string(),
            "0_0,2_0".to_string(),
            "--out".to_string(),
            out_path.display().to_string(),
        ])
        .unwrap();

        let output = std::fs::read_to_string(&out_path).unwrap();
        let manifest: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(manifest["chunks"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn planetary_store_emit_manifest_subset_around_studs_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let out_path = tempdir.path().join("subset.json");
        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "emit-manifest-subset".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--around-studs".to_string(),
            "300,128".to_string(),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--limit".to_string(),
            "2".to_string(),
            "--out".to_string(),
            out_path.display().to_string(),
        ])
        .unwrap();

        let output = std::fs::read_to_string(&out_path).unwrap();
        let manifest: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(manifest["chunks"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn planetary_store_emit_manifest_subset_point_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let out_path = tempdir.path().join("subset.json");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "emit-manifest-subset".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--limit".to_string(),
            "2".to_string(),
            "--out".to_string(),
            out_path.display().to_string(),
        ])
        .unwrap();

        let output = std::fs::read_to_string(&out_path).unwrap();
        let manifest: Value = serde_json::from_str(&output).unwrap();
        assert!(!manifest["chunks"].as_array().unwrap().is_empty());
    }

    #[test]
    fn planetary_store_emit_manifest_subset_point_without_scene_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let out_path = tempdir.path().join("subset.json");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "emit-manifest-subset".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--limit".to_string(),
            "2".to_string(),
            "--out".to_string(),
            out_path.display().to_string(),
        ])
        .unwrap();

        let output = std::fs::read_to_string(&out_path).unwrap();
        let manifest: Value = serde_json::from_str(&output).unwrap();
        assert!(!manifest["chunks"].as_array().unwrap().is_empty());
    }

    #[test]
    fn planetary_store_subset_summary_tile_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        let zoom = 10u8;
        let x = (((center.lon + 180.0) / 360.0) * 2f64.powi(zoom as i32)).floor() as u32;
        let lat_rad = center.lat.to_radians();
        let y = ((1.0 - ((lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI)) / 2.0
            * 2f64.powi(zoom as i32))
        .floor() as u32;

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "subset-summary".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--tile".to_string(),
            format!("{zoom},{x},{y}"),
            "--limit".to_string(),
            "2".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_emit_manifest_subset_tile_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let out_path = tempdir.path().join("subset.json");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        let zoom = 10u8;
        let x = (((center.lon + 180.0) / 360.0) * 2f64.powi(zoom as i32)).floor() as u32;
        let lat_rad = center.lat.to_radians();
        let y = ((1.0 - ((lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI)) / 2.0
            * 2f64.powi(zoom as i32))
        .floor() as u32;

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "emit-manifest-subset".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--tile".to_string(),
            format!("{zoom},{x},{y}"),
            "--out".to_string(),
            out_path.display().to_string(),
        ])
        .unwrap();

        let output = std::fs::read_to_string(&out_path).unwrap();
        let manifest: Value = serde_json::from_str(&output).unwrap();
        assert!(!manifest["chunks"].as_array().unwrap().is_empty());
    }

    #[test]
    fn planetary_store_emit_runtime_lua_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let output_dir = tempdir.path().join("runtime");
        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "emit-runtime-lua".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--chunk-ids".to_string(),
            "0_0,2_0".to_string(),
            "--output-dir".to_string(),
            output_dir.display().to_string(),
            "--index-name".to_string(),
            "PlanetaryIndex".to_string(),
            "--shard-folder".to_string(),
            "PlanetaryChunks".to_string(),
        ])
        .unwrap();

        let index_path = output_dir.join("PlanetaryIndex.lua");
        assert!(index_path.exists());
    }

    #[test]
    fn planetary_store_emit_runtime_lua_plan_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let plan_path = tempdir.path().join("delivery-plan.json");
        let output_dir = tempdir.path().join("runtime");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-plan".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            plan_path.display().to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "emit-runtime-lua".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--plan".to_string(),
            plan_path.display().to_string(),
            "--output-dir".to_string(),
            output_dir.display().to_string(),
            "--index-name".to_string(),
            "PlanetaryIndex".to_string(),
            "--shard-folder".to_string(),
            "PlanetaryChunks".to_string(),
        ])
        .unwrap();

        let index_path = output_dir.join("PlanetaryIndex.lua");
        assert!(index_path.exists());
    }

    #[test]
    fn planetary_store_emit_runtime_lua_plan_without_store_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let plan_path = tempdir.path().join("delivery-plan.json");
        let output_dir = tempdir.path().join("runtime");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-plan".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            plan_path.display().to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "emit-runtime-lua".to_string(),
            "--plan".to_string(),
            plan_path.display().to_string(),
            "--output-dir".to_string(),
            output_dir.display().to_string(),
            "--index-name".to_string(),
            "PlanetaryIndex".to_string(),
            "--shard-folder".to_string(),
            "PlanetaryChunks".to_string(),
        ])
        .unwrap();

        let index_path = output_dir.join("PlanetaryIndex.lua");
        assert!(index_path.exists());
    }

    #[test]
    fn planetary_store_delivery_bundle_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let bundle_path = tempdir.path().join("delivery-bundle.json");
        let plan_path = tempdir.path().join("delivery-plan.json");
        let summary_path = tempdir.path().join("delivery-summary.json");
        let window_path = tempdir.path().join("delivery-window.json");
        let subset_path = tempdir.path().join("subset.json");
        let output_dir = tempdir.path().join("runtime");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-bundle".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            bundle_path.display().to_string(),
            "--plan-out".to_string(),
            plan_path.display().to_string(),
            "--summary-out".to_string(),
            summary_path.display().to_string(),
            "--window-out".to_string(),
            window_path.display().to_string(),
            "--manifest-out".to_string(),
            subset_path.display().to_string(),
            "--output-dir".to_string(),
            output_dir.display().to_string(),
            "--index-name".to_string(),
            "PlanetaryIndex".to_string(),
            "--shard-folder".to_string(),
            "PlanetaryChunks".to_string(),
        ])
        .unwrap();

        assert!(bundle_path.exists());
        assert!(plan_path.exists());
        assert!(summary_path.exists());
        assert!(window_path.exists());
        assert!(subset_path.exists());
        assert!(output_dir.join("PlanetaryIndex.lua").exists());
    }

    #[test]
    fn planetary_store_delivery_bundle_accepts_multiple_plans() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let plan_a_path = tempdir.path().join("delivery-a.json");
        let plan_b_path = tempdir.path().join("delivery-b.json");
        let bundle_path = tempdir.path().join("delivery-bundle.json");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-plan".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            plan_a_path.display().to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-plan".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--around-studs".to_string(),
            "300,128".to_string(),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            plan_b_path.display().to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-bundle".to_string(),
            "--plan".to_string(),
            plan_a_path.display().to_string(),
            "--plan".to_string(),
            plan_b_path.display().to_string(),
            "--out".to_string(),
            bundle_path.display().to_string(),
        ])
        .unwrap();

        let bundle: Value =
            serde_json::from_str(&std::fs::read_to_string(&bundle_path).unwrap()).unwrap();
        assert_eq!(bundle["plan"]["selection_mode"], "merged");
        assert_eq!(bundle["plan"]["chunk_count"], 2);
    }

    #[test]
    fn planetary_store_delivery_bundle_accepts_route_plan() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let route_path = tempdir.path().join("route-plan.json");
        let bundle_path = tempdir.path().join("delivery-bundle.json");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "route-plan".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon + 0.0002),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            route_path.display().to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-bundle".to_string(),
            "--plan".to_string(),
            route_path.display().to_string(),
            "--out".to_string(),
            bundle_path.display().to_string(),
        ])
        .unwrap();

        let bundle: Value =
            serde_json::from_str(&std::fs::read_to_string(&bundle_path).unwrap()).unwrap();
        assert_eq!(bundle["plan"]["selection_mode"], "merged");
        assert_eq!(bundle["plan"]["source_plan_count"], 2);
        assert_eq!(bundle["plan"]["source_selector_count"], 2);
        assert!(bundle["plan"]["chunk_count"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn planetary_store_delivery_bundle_accepts_route_session() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let session_path = tempdir.path().join("route-session.json");
        let bundle_path = tempdir.path().join("delivery-bundle.json");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "route-session".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon + 0.0002),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            session_path.display().to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-bundle".to_string(),
            "--session".to_string(),
            session_path.display().to_string(),
            "--out".to_string(),
            bundle_path.display().to_string(),
        ])
        .unwrap();

        let bundle: Value =
            serde_json::from_str(&std::fs::read_to_string(&bundle_path).unwrap()).unwrap();
        assert_eq!(bundle["plan"]["selection_mode"], "merged");
        assert_eq!(bundle["plan"]["source_plan_count"], 2);
        assert_eq!(bundle["plan"]["source_selector_count"], 2);
        assert!(bundle["plan"]["chunk_count"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn planetary_store_delivery_bundle_writes_hydrated_route_outputs() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let route_session_path = tempdir.path().join("route-session.json");
        let hydrated_route_path = tempdir.path().join("hydrated-route.json");
        let schedule_path = tempdir.path().join("route-schedule.json");
        let step_summary_dir = tempdir.path().join("step-summaries");
        let step_window_dir = tempdir.path().join("step-windows");
        let step_transition_dir = tempdir.path().join("step-transitions");
        let bundle_path = tempdir.path().join("delivery-bundle.json");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-bundle".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon + 0.0002),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--route-session-out".to_string(),
            route_session_path.display().to_string(),
            "--hydrated-route-out".to_string(),
            hydrated_route_path.display().to_string(),
            "--schedule-out".to_string(),
            schedule_path.display().to_string(),
            "--step-summary-dir".to_string(),
            step_summary_dir.display().to_string(),
            "--step-window-dir".to_string(),
            step_window_dir.display().to_string(),
            "--step-transition-dir".to_string(),
            step_transition_dir.display().to_string(),
            "--out".to_string(),
            bundle_path.display().to_string(),
        ])
        .unwrap();

        let bundle: Value =
            serde_json::from_str(&std::fs::read_to_string(&bundle_path).unwrap()).unwrap();
        assert_eq!(
            bundle["route_session_out"],
            route_session_path.display().to_string()
        );
        assert_eq!(
            bundle["hydrated_route_out"],
            hydrated_route_path.display().to_string()
        );
        assert_eq!(bundle["schedule_out"], schedule_path.display().to_string());
        assert_eq!(
            bundle["step_summary_dir"],
            step_summary_dir.display().to_string()
        );
        assert_eq!(
            bundle["step_window_dir"],
            step_window_dir.display().to_string()
        );
        assert_eq!(
            bundle["step_transition_dir"],
            step_transition_dir.display().to_string()
        );
        let hydrated: Value =
            serde_json::from_str(&std::fs::read_to_string(&hydrated_route_path).unwrap()).unwrap();
        assert_eq!(hydrated["steps"].as_array().unwrap().len(), 2);
        let schedule: Value =
            serde_json::from_str(&std::fs::read_to_string(&schedule_path).unwrap()).unwrap();
        assert_eq!(schedule["steps"].as_array().unwrap().len(), 2);
        assert!(step_summary_dir.join("step-000-summary.json").exists());
        assert!(step_window_dir.join("step-000-window.json").exists());
        assert!(step_transition_dir
            .join("step-000-transition.json")
            .exists());
    }

    #[test]
    fn planetary_store_route_session_cross_scene_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_a_path = tempdir.path().join("sample-a.sqlite");
        let manifest_b_path = tempdir.path().join("sample-b.sqlite");
        let session_path = tempdir.path().join("route-session.json");

        let manifest_a = build_sample_multi_chunk(1, 1);
        let center_a = manifest_a.meta.bbox.center();
        let mut manifest_b = build_sample_multi_chunk(1, 1);
        manifest_b.meta.world_name = "SampleAustinLikeBlockB".to_string();
        manifest_b.meta.bbox = arbx_geo::BoundingBox::new(30.30, -97.70, 30.302, -97.698);
        let center_b = manifest_b.meta.bbox.center();
        write_manifest_sqlite(&manifest_a, &manifest_a_path).unwrap();
        write_manifest_sqlite(&manifest_b, &manifest_b_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_a_path.display().to_string(),
            "--scene".to_string(),
            "austin_a".to_string(),
        ])
        .unwrap();
        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_b_path.display().to_string(),
            "--scene".to_string(),
            "austin_b".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "route-session".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center_a.lat, center_a.lon),
            "--point".to_string(),
            format!("{},{}", center_b.lat, center_b.lon),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            session_path.display().to_string(),
        ])
        .unwrap();

        let session: Value =
            serde_json::from_str(&std::fs::read_to_string(&session_path).unwrap()).unwrap();
        assert_eq!(session["scene_ids"].as_array().unwrap().len(), 2);
        assert_eq!(session["merged_plan"]["scene_id"], "__multi_scene__");
    }

    #[test]
    fn planetary_store_delivery_bundle_accepts_multiple_points_directly() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let bundle_path = tempdir.path().join("delivery-bundle.json");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-bundle".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon + 0.0002),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            bundle_path.display().to_string(),
        ])
        .unwrap();

        let bundle: Value =
            serde_json::from_str(&std::fs::read_to_string(&bundle_path).unwrap()).unwrap();
        assert_eq!(bundle["plan"]["selection_mode"], "merged");
        assert_eq!(bundle["plan"]["source_plan_count"], 2);
        assert_eq!(bundle["plan"]["source_selector_count"], 2);
        assert!(bundle["plan"]["chunk_count"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn planetary_store_emit_runtime_lua_around_studs_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let output_dir = tempdir.path().join("runtime");
        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "emit-runtime-lua".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--around-studs".to_string(),
            "300,128".to_string(),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--limit".to_string(),
            "2".to_string(),
            "--output-dir".to_string(),
            output_dir.display().to_string(),
            "--index-name".to_string(),
            "PlanetaryIndex".to_string(),
            "--shard-folder".to_string(),
            "PlanetaryChunks".to_string(),
        ])
        .unwrap();

        let index_path = output_dir.join("PlanetaryIndex.lua");
        assert!(index_path.exists());
    }

    #[test]
    fn planetary_store_emit_runtime_lua_point_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let output_dir = tempdir.path().join("runtime");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "emit-runtime-lua".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--limit".to_string(),
            "2".to_string(),
            "--output-dir".to_string(),
            output_dir.display().to_string(),
            "--index-name".to_string(),
            "PlanetaryIndex".to_string(),
            "--shard-folder".to_string(),
            "PlanetaryChunks".to_string(),
        ])
        .unwrap();

        let index_path = output_dir.join("PlanetaryIndex.lua");
        assert!(index_path.exists());
    }

    #[test]
    fn planetary_store_emit_runtime_lua_tile_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let output_dir = tempdir.path().join("runtime");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        let zoom = 10u8;
        let x = (((center.lon + 180.0) / 360.0) * 2f64.powi(zoom as i32)).floor() as u32;
        let lat_rad = center.lat.to_radians();
        let y = ((1.0 - ((lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI)) / 2.0
            * 2f64.powi(zoom as i32))
        .floor() as u32;

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "emit-runtime-lua".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--tile".to_string(),
            format!("{zoom},{x},{y}"),
            "--output-dir".to_string(),
            output_dir.display().to_string(),
            "--index-name".to_string(),
            "PlanetaryIndex".to_string(),
            "--shard-folder".to_string(),
            "PlanetaryChunks".to_string(),
        ])
        .unwrap();

        let index_path = output_dir.join("PlanetaryIndex.lua");
        assert!(index_path.exists());
    }

    #[test]
    fn planetary_store_emit_runtime_lua_tile_without_scene_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let output_dir = tempdir.path().join("runtime");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        let zoom = 10u8;
        let x = (((center.lon + 180.0) / 360.0) * 2f64.powi(zoom as i32)).floor() as u32;
        let lat_rad = center.lat.to_radians();
        let y = ((1.0 - ((lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI)) / 2.0
            * 2f64.powi(zoom as i32))
        .floor() as u32;

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "emit-runtime-lua".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--tile".to_string(),
            format!("{zoom},{x},{y}"),
            "--output-dir".to_string(),
            output_dir.display().to_string(),
            "--index-name".to_string(),
            "PlanetaryIndex".to_string(),
            "--shard-folder".to_string(),
            "PlanetaryChunks".to_string(),
        ])
        .unwrap();

        let index_path = output_dir.join("PlanetaryIndex.lua");
        assert!(index_path.exists());
    }

    #[test]
    fn planetary_store_delivery_window_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-window".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--limit".to_string(),
            "2".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_delivery_window_around_studs_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-window".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--around-studs".to_string(),
            "300,128".to_string(),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--max-streaming-cost".to_string(),
            "24".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_delivery_window_bbox_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let manifest = build_sample_multi_chunk(3, 1);
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-window".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
            "--bbox-studs".to_string(),
            "200,0,500,200".to_string(),
            "--limit".to_string(),
            "2".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_delivery_window_plan_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let plan_path = tempdir.path().join("delivery-plan.json");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-plan".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--point".to_string(),
            format!("{},{}", center.lat, center.lon),
            "--radius-studs".to_string(),
            "300".to_string(),
            "--out".to_string(),
            plan_path.display().to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-window".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--plan".to_string(),
            plan_path.display().to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn planetary_store_delivery_window_tile_works() {
        let tempdir = tempfile::tempdir().unwrap();
        let store_path = tempdir.path().join("planetary.sqlite");
        let manifest_path = tempdir.path().join("sample.sqlite");
        let manifest = build_sample_multi_chunk(3, 1);
        let center = manifest.meta.bbox.center();
        write_manifest_sqlite(&manifest, &manifest_path).unwrap();

        let zoom = 10u8;
        let x = (((center.lon + 180.0) / 360.0) * 2f64.powi(zoom as i32)).floor() as u32;
        let lat_rad = center.lat.to_radians();
        let y = ((1.0 - ((lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI)) / 2.0
            * 2f64.powi(zoom as i32))
        .floor() as u32;

        cmd_planetary_store(&[
            "ingest-manifest".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--manifest-sqlite".to_string(),
            manifest_path.display().to_string(),
            "--scene".to_string(),
            "austin".to_string(),
        ])
        .unwrap();

        cmd_planetary_store(&[
            "delivery-window".to_string(),
            "--store".to_string(),
            store_path.display().to_string(),
            "--tile".to_string(),
            format!("{zoom},{x},{y}"),
            "--limit".to_string(),
            "2".to_string(),
        ])
        .unwrap();
    }
}
