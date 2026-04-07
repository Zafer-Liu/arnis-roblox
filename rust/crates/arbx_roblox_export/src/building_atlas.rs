//! Building facade texture atlas generation.
//!
//! Pre-computes a single PNG atlas per chunk containing facade tiles for the
//! near-streaming-ring buildings. Each entry in the atlas carries a UV rect so
//! the Roblox runtime can index the atlas directly without re-running the
//! per-building EditableImage path described in
//! `BuildingBuilder.lua::applyHeroPbrToMeshPart`.
//!
//! This is a Tranche-3 optimization for the planetary streaming plan: a single
//! 512×512 PNG replaces up to 16 individual EditableImage allocations per
//! chunk, and the procedural patterns mirror the Lua runtime ones in
//! `generateOfficePbrTextures`/`generateStonePbrTextures`/`generateMetalPbrTextures`.

use crate::manifest::BuildingShell;

/// Hard caps that keep atlases bounded — 16 buildings × 128² each fits in
/// 512² with room to spare and ensures the encoded PNG stays under a few KB.
pub const MAX_BUILDINGS_PER_ATLAS: usize = 16;
pub const ATLAS_RESOLUTION: u32 = 512;
pub const TILE_SMALL: u32 = 64;
pub const TILE_LARGE: u32 = 128;

/// Minimum building height (in studs) to be included in the atlas. Mirrors
/// `HERO_PBR_MIN_HEIGHT` in BuildingBuilder.lua so the Lua importer and the
/// Rust pre-pack agree on which buildings are "hero" candidates.
pub const ATLAS_MIN_HEIGHT_STUDS: f64 = 20.0;

/// UV rect referencing a single building tile inside a chunk atlas.
#[derive(Debug, Clone, PartialEq)]
pub struct AtlasUv {
    pub uv_x: f32,
    pub uv_y: f32,
    pub uv_width: f32,
    pub uv_height: f32,
}

/// Atlas entry: which building, where it lives in the atlas, and the facade
/// type that drove the procedural pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct AtlasEntry {
    pub building_id: String,
    pub uv_x: f32,
    pub uv_y: f32,
    pub uv_width: f32,
    pub uv_height: f32,
    pub facade_type: String,
}

impl AtlasEntry {
    pub fn uv(&self) -> AtlasUv {
        AtlasUv {
            uv_x: self.uv_x,
            uv_y: self.uv_y,
            uv_width: self.uv_width,
            uv_height: self.uv_height,
        }
    }
}

/// A chunk-scoped facade atlas with raw RGBA pixel data plus per-building
/// UV rects. `rgba_data` is raw 4-bytes-per-pixel; the JSON writer
/// base64-encodes it so the Lua consumer can decode directly into
/// `EditableImage:WritePixelsBuffer` without a PNG decoder.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildingAtlas {
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub rgba_data: Vec<u8>,
    pub entries: Vec<AtlasEntry>,
}

/// Classification: lower-cased usage/kind text → atlas facade type.
/// Mirrors `classifyPbrFacade` in BuildingBuilder.lua.
pub fn classify_facade(building: &BuildingShell) -> &'static str {
    let usage = building
        .usage
        .as_deref()
        .unwrap_or("default")
        .to_ascii_lowercase();
    match usage.as_str() {
        "commercial" | "office" | "retail" => "office",
        "industrial" | "warehouse" => "metal",
        _ => "stone",
    }
}

fn building_is_hero_candidate(b: &BuildingShell) -> bool {
    let has_name = b.name.as_deref().map(|n| !n.is_empty()).unwrap_or(false);
    has_name && b.height >= ATLAS_MIN_HEIGHT_STUDS
}

fn building_importance(b: &BuildingShell) -> f64 {
    // Match the runtime intuition: taller, named buildings deserve large tiles.
    b.height
}

/// Pack facade tiles for the heroic buildings of a chunk into a single PNG
/// atlas. Returns `None` when there are zero hero candidates.
pub fn build_chunk_atlas(buildings: &[BuildingShell]) -> Option<BuildingAtlas> {
    // Stable selection: filter, then sort by (importance desc, id asc) so the
    // packing layout — and therefore the resulting PNG bytes — is deterministic
    // for a given set of buildings regardless of input ordering.
    let mut candidates: Vec<&BuildingShell> =
        buildings.iter().filter(|b| building_is_hero_candidate(b)).collect();
    if candidates.is_empty() {
        return None;
    }
    candidates.sort_by(|a, b| {
        building_importance(b)
            .partial_cmp(&building_importance(a))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });
    candidates.truncate(MAX_BUILDINGS_PER_ATLAS);

    let atlas_w = ATLAS_RESOLUTION;
    let atlas_h = ATLAS_RESOLUTION;
    let mut pixels = vec![0u8; (atlas_w * atlas_h * 4) as usize];

    let mut entries: Vec<AtlasEntry> = Vec::with_capacity(candidates.len());

    // Simple shelf packer: iterate row-major across an 8×8 grid of 64-stud
    // cells. A "large" tile (128²) consumes a 2×2 block. We try to place each
    // candidate at the first cell where it fits.
    let cell = TILE_SMALL; // 64
    let cells_per_side = atlas_w / cell; // 8
    let total_cells = (cells_per_side * cells_per_side) as usize;
    let mut occupied = vec![false; total_cells];
    let cells_per_side_us = cells_per_side as usize;

    let cell_index = |cx: usize, cy: usize| cy * cells_per_side_us + cx;

    let try_place_large = |occupied: &mut [bool]| -> Option<(u32, u32)> {
        for cy in 0..(cells_per_side_us - 1) {
            for cx in 0..(cells_per_side_us - 1) {
                let i00 = cell_index(cx, cy);
                let i10 = cell_index(cx + 1, cy);
                let i01 = cell_index(cx, cy + 1);
                let i11 = cell_index(cx + 1, cy + 1);
                if !occupied[i00] && !occupied[i10] && !occupied[i01] && !occupied[i11] {
                    occupied[i00] = true;
                    occupied[i10] = true;
                    occupied[i01] = true;
                    occupied[i11] = true;
                    return Some((cx as u32 * cell, cy as u32 * cell));
                }
            }
        }
        None
    };

    let try_place_small = |occupied: &mut [bool]| -> Option<(u32, u32)> {
        for cy in 0..cells_per_side_us {
            for cx in 0..cells_per_side_us {
                let i = cell_index(cx, cy);
                if !occupied[i] {
                    occupied[i] = true;
                    return Some((cx as u32 * cell, cy as u32 * cell));
                }
            }
        }
        None
    };

    for b in &candidates {
        let facade_type = classify_facade(b);
        // Larger buildings (height >= 60 studs ≈ ~18m at 0.3 m/stud) get the
        // big tile. Everything else gets the small tile.
        let want_large = b.height >= 60.0;
        let placement = if want_large {
            try_place_large(&mut occupied).or_else(|| try_place_small(&mut occupied))
        } else {
            try_place_small(&mut occupied)
        };
        let (px, py) = match placement {
            Some(p) => p,
            None => break, // atlas is full
        };
        let tile_size = if want_large
            && px + TILE_LARGE <= atlas_w
            && py + TILE_LARGE <= atlas_h
            // The large packer above guarantees the 2×2 block exists; small
            // fallback yields a 64-tile.
            && occupied_block_is_large(&occupied, px, py, cell, cells_per_side_us)
        {
            TILE_LARGE
        } else {
            TILE_SMALL
        };

        write_facade_tile(&mut pixels, atlas_w, px, py, tile_size, facade_type);

        let uv_x = px as f32 / atlas_w as f32;
        let uv_y = py as f32 / atlas_h as f32;
        let uv_w = tile_size as f32 / atlas_w as f32;
        let uv_h = tile_size as f32 / atlas_h as f32;

        entries.push(AtlasEntry {
            building_id: b.id.clone(),
            uv_x,
            uv_y,
            uv_width: uv_w,
            uv_height: uv_h,
            facade_type: facade_type.to_string(),
        });
    }

    Some(BuildingAtlas {
        atlas_width: atlas_w,
        atlas_height: atlas_h,
        rgba_data: pixels,
        entries,
    })
}

/// Verify the 2×2 block at (px,py) is owned by the same allocation, i.e. all
/// four cells are marked occupied. Used to confirm we actually got a large
/// placement before writing 128 pixels of facade content.
fn occupied_block_is_large(
    occupied: &[bool],
    px: u32,
    py: u32,
    cell: u32,
    cells_per_side: usize,
) -> bool {
    let cx = (px / cell) as usize;
    let cy = (py / cell) as usize;
    if cx + 1 >= cells_per_side || cy + 1 >= cells_per_side {
        return false;
    }
    let idx = |cx: usize, cy: usize| cy * cells_per_side + cx;
    occupied[idx(cx, cy)]
        && occupied[idx(cx + 1, cy)]
        && occupied[idx(cx, cy + 1)]
        && occupied[idx(cx + 1, cy + 1)]
}

/// Stamp a procedural facade tile into the atlas at (origin_x, origin_y).
/// Patterns mirror the Lua hero-PBR generators so runtime visuals stay
/// consistent whether or not the atlas optimization is enabled.
fn write_facade_tile(
    pixels: &mut [u8],
    atlas_w: u32,
    origin_x: u32,
    origin_y: u32,
    tile_size: u32,
    facade_type: &str,
) {
    match facade_type {
        "office" => write_office_tile(pixels, atlas_w, origin_x, origin_y, tile_size),
        "metal" => write_metal_tile(pixels, atlas_w, origin_x, origin_y, tile_size),
        _ => write_stone_tile(pixels, atlas_w, origin_x, origin_y, tile_size),
    }
}

fn pixel_offset(atlas_w: u32, x: u32, y: u32) -> usize {
    ((y * atlas_w + x) * 4) as usize
}

fn write_office_tile(pixels: &mut [u8], atlas_w: u32, ox: u32, oy: u32, size: u32) {
    // Office: cool concrete/glass with a regular grid of mullion seams.
    let panel_spacing = (size / 8).max(1);
    for y in 0..size {
        for x in 0..size {
            let off = pixel_offset(atlas_w, ox + x, oy + y);
            let is_joint = (x % panel_spacing < 1) || (y % panel_spacing < 1);
            // Albedo: light blue glass on panels, darker mullions on joints.
            let (r, g, b) = if is_joint { (90, 100, 120) } else { (170, 195, 220) };
            pixels[off] = r;
            pixels[off + 1] = g;
            pixels[off + 2] = b;
            pixels[off + 3] = 255;
        }
    }
}

fn write_stone_tile(pixels: &mut [u8], atlas_w: u32, ox: u32, oy: u32, size: u32) {
    // Stone: warm noise-driven masonry with vertical weathering.
    for y in 0..size {
        for x in 0..size {
            let off = pixel_offset(atlas_w, ox + x, oy + y);
            let n = pseudo_noise(x as i32, y as i32);
            let weather = (y as f32 / size as f32) * 30.0;
            let base = 165.0 - weather + (n - 0.5) * 40.0;
            let v = base.clamp(60.0, 220.0) as u8;
            pixels[off] = v;
            pixels[off + 1] = (v as f32 * 0.92) as u8;
            pixels[off + 2] = (v as f32 * 0.82) as u8;
            pixels[off + 3] = 255;
        }
    }
}

fn write_metal_tile(pixels: &mut [u8], atlas_w: u32, ox: u32, oy: u32, size: u32) {
    // Metal: cool steel ribs at fixed spacing, corrosion specks driven by noise.
    let rib_spacing = (size / 6).max(1);
    for y in 0..size {
        for x in 0..size {
            let off = pixel_offset(atlas_w, ox + x, oy + y);
            let is_rib = x % rib_spacing < 2;
            let n = pseudo_noise(x as i32, y as i32);
            let corrosion = if n > 0.7 { 30.0 } else { 0.0 };
            let base = if is_rib { 140.0 } else { 180.0 };
            let v = (base - corrosion + (n - 0.5) * 20.0).clamp(60.0, 230.0) as u8;
            pixels[off] = v;
            pixels[off + 1] = v;
            pixels[off + 2] = (v as f32 * 1.05).min(255.0) as u8;
            pixels[off + 3] = 255;
        }
    }
}

/// Tiny deterministic hash → [0,1). Mirrors the Lua noise helpers used in
/// generateStone/MetalPbrTextures.
fn pseudo_noise(x: i32, y: i32) -> f32 {
    let mut n = (x as i64).wrapping_mul(374_761_393).wrapping_add((y as i64).wrapping_mul(668_265_263));
    n ^= n >> 13;
    n = n.wrapping_mul(1_274_126_177);
    n ^= n >> 16;
    let m = (n.rem_euclid(256)) as f32;
    m / 255.0
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{BuildingShell, GroundPoint};

    fn make_building(id: &str, height: f64, usage: Option<&str>) -> BuildingShell {
        BuildingShell {
            id: id.to_string(),
            footprint: vec![
                GroundPoint::new(0.0, 0.0),
                GroundPoint::new(10.0, 0.0),
                GroundPoint::new(10.0, 10.0),
                GroundPoint::new(0.0, 10.0),
            ],
            holes: vec![],
            indices: None,
            material: "Concrete".to_string(),
            wall_color: None,
            roof_color: None,
            roof_shape: Some("flat".to_string()),
            roof_material: None,
            usage: usage.map(|s| s.to_string()),
            min_height: None,
            base_y: 0.0,
            height,
            height_m: None,
            levels: None,
            roof_levels: None,
            roof: "flat".to_string(),
            facade_style: None,
            structure_type: None,
            rooms: Vec::new(),
            roof_height: None,
            roof_direction: None,
            roof_angle: None,
            name: Some(format!("Hero {id}")),
            shell_mesh: None,
            atlas_uv: None,
        }
    }

    #[test]
    fn empty_building_list_yields_no_atlas() {
        assert!(build_chunk_atlas(&[]).is_none());
    }

    #[test]
    fn buildings_below_height_threshold_yield_no_atlas() {
        let buildings = vec![make_building("b1", 5.0, Some("office"))];
        assert!(build_chunk_atlas(&buildings).is_none());
    }

    #[test]
    fn unnamed_buildings_are_excluded() {
        let mut b = make_building("b1", 80.0, Some("office"));
        b.name = None;
        assert!(build_chunk_atlas(&[b]).is_none());
    }

    #[test]
    fn small_set_packs_each_building() {
        let buildings = vec![
            make_building("a", 25.0, Some("residential")),
            make_building("b", 30.0, Some("office")),
            make_building("c", 22.0, Some("warehouse")),
        ];
        let atlas = build_chunk_atlas(&buildings).expect("atlas should exist");
        assert_eq!(atlas.atlas_width, ATLAS_RESOLUTION);
        assert_eq!(atlas.atlas_height, ATLAS_RESOLUTION);
        assert_eq!(atlas.entries.len(), 3);
        // Raw RGBA: 512*512*4 = 1,048,576 bytes
        assert_eq!(atlas.rgba_data.len(), (ATLAS_RESOLUTION * ATLAS_RESOLUTION * 4) as usize);
    }

    #[test]
    fn entries_have_unique_uv_rects() {
        let buildings: Vec<BuildingShell> = (0..8)
            .map(|i| make_building(&format!("b{i}"), 25.0 + i as f64, Some("office")))
            .collect();
        let atlas = build_chunk_atlas(&buildings).expect("atlas should exist");
        for i in 0..atlas.entries.len() {
            for j in (i + 1)..atlas.entries.len() {
                let a = &atlas.entries[i];
                let b = &atlas.entries[j];
                let same = a.uv_x == b.uv_x && a.uv_y == b.uv_y;
                assert!(!same, "entries {i}/{j} share an origin in the atlas");
            }
        }
    }

    #[test]
    fn atlas_caps_at_max_buildings() {
        let buildings: Vec<BuildingShell> = (0..32)
            .map(|i| make_building(&format!("b{i:02}"), 25.0, Some("office")))
            .collect();
        let atlas = build_chunk_atlas(&buildings).expect("atlas should exist");
        assert!(atlas.entries.len() <= MAX_BUILDINGS_PER_ATLAS);
        assert_eq!(atlas.entries.len(), MAX_BUILDINGS_PER_ATLAS);
    }

    #[test]
    fn classify_facade_uses_office_metal_stone_buckets() {
        let office = make_building("o", 30.0, Some("office"));
        let retail = make_building("r", 30.0, Some("retail"));
        let warehouse = make_building("w", 30.0, Some("warehouse"));
        let residential = make_building("h", 30.0, Some("residential"));

        assert_eq!(classify_facade(&office), "office");
        assert_eq!(classify_facade(&retail), "office");
        assert_eq!(classify_facade(&warehouse), "metal");
        assert_eq!(classify_facade(&residential), "stone");
    }

    #[test]
    fn entries_carry_facade_type_palette_per_building() {
        let buildings = vec![
            make_building("a", 25.0, Some("office")),
            make_building("b", 25.0, Some("warehouse")),
            make_building("c", 25.0, Some("residential")),
        ];
        let atlas = build_chunk_atlas(&buildings).expect("atlas should exist");
        let by_id: std::collections::HashMap<&str, &str> = atlas
            .entries
            .iter()
            .map(|e| (e.building_id.as_str(), e.facade_type.as_str()))
            .collect();
        assert_eq!(by_id.get("a"), Some(&"office"));
        assert_eq!(by_id.get("b"), Some(&"metal"));
        assert_eq!(by_id.get("c"), Some(&"stone"));
    }

    #[test]
    fn tall_buildings_get_large_tiles_when_room_available() {
        let buildings = vec![make_building("tall", 120.0, Some("office"))];
        let atlas = build_chunk_atlas(&buildings).expect("atlas should exist");
        let entry = &atlas.entries[0];
        let tile_w = entry.uv_width * ATLAS_RESOLUTION as f32;
        assert!(
            (tile_w - TILE_LARGE as f32).abs() < 0.5,
            "expected large 128-stud tile for very tall building, got {tile_w}"
        );
    }

    #[test]
    fn deterministic_byte_output_for_fixed_inputs() {
        let buildings = vec![
            make_building("a", 25.0, Some("office")),
            make_building("b", 30.0, Some("warehouse")),
        ];
        let a1 = build_chunk_atlas(&buildings).unwrap();
        let a2 = build_chunk_atlas(&buildings).unwrap();
        assert_eq!(a1.rgba_data, a2.rgba_data);
        assert_eq!(a1.entries, a2.entries);
    }
}
