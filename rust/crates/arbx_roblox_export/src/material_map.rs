//! OSM tag -> Roblox material/color mapping with deterministic diversity seeding.
//!
//! Designed for Roblox 2026's MaterialVariant system where each material string
//! maps to a Roblox Enum.Material and an optional MaterialVariant name.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A resolved Roblox material with optional color override and variant name.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedMaterial {
    /// Roblox material name (e.g., "Brick", "Concrete", "Glass").
    pub material: String,
    /// RGB color override, if any.
    pub color: Option<(u8, u8, u8)>,
    /// MaterialVariant name for diversity.
    pub variant: Option<String>,
}

// ---------------------------------------------------------------------------
// Deterministic hashing
// ---------------------------------------------------------------------------

fn hash_building_id(building_id: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    building_id.hash(&mut hasher);
    hasher.finish()
}

fn pick_index(len: usize, seed: u64) -> usize {
    (seed as usize) % len
}

// ---------------------------------------------------------------------------
// CSS color parsing
// ---------------------------------------------------------------------------

/// Parse a CSS color name or `#RRGGBB` hex string into an RGB tuple.
pub fn parse_css_color(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.trim();
    // Try hex first
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some((r, g, b));
        }
        if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            return Some((r, g, b));
        }
        return None;
    }
    // Named CSS colors (common subset)
    match s.to_ascii_lowercase().as_str() {
        "white" => Some((255, 255, 255)),
        "black" => Some((0, 0, 0)),
        "red" => Some((255, 0, 0)),
        "green" => Some((0, 128, 0)),
        "blue" => Some((0, 0, 255)),
        "yellow" => Some((255, 255, 0)),
        "orange" => Some((255, 165, 0)),
        "brown" => Some((139, 69, 19)),
        "grey" | "gray" => Some((128, 128, 128)),
        "beige" => Some((245, 245, 220)),
        "cream" => Some((255, 253, 208)),
        "tan" => Some((210, 180, 140)),
        "pink" => Some((255, 192, 203)),
        "purple" => Some((128, 0, 128)),
        "ivory" => Some((255, 255, 240)),
        "silver" => Some((192, 192, 192)),
        "maroon" => Some((128, 0, 0)),
        "navy" => Some((0, 0, 128)),
        "teal" => Some((0, 128, 128)),
        "olive" => Some((128, 128, 0)),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Wall material resolution
// ---------------------------------------------------------------------------

/// Palette entry for deterministic diversity when no OSM tag is present.
struct WallPaletteEntry {
    material: &'static str,
    color: (u8, u8, u8),
    variant: &'static str,
}

const RESIDENTIAL_WALL_PALETTE: &[WallPaletteEntry] = &[
    WallPaletteEntry { material: "Brick",      color: (178, 102, 75),  variant: "brick_red" },
    WallPaletteEntry { material: "Brick",      color: (150, 110, 80),  variant: "brick_brown" },
    WallPaletteEntry { material: "Brick",      color: (200, 170, 130), variant: "brick_tan" },
    WallPaletteEntry { material: "WoodPlanks", color: (194, 154, 115), variant: "wood_natural" },
    WallPaletteEntry { material: "Concrete",   color: (230, 220, 210), variant: "stucco_white" },
];

const COMMERCIAL_WALL_PALETTE: &[WallPaletteEntry] = &[
    WallPaletteEntry { material: "Concrete",     color: (200, 196, 188), variant: "concrete_light" },
    WallPaletteEntry { material: "Glass",        color: (180, 210, 230), variant: "glass_blue" },
    WallPaletteEntry { material: "Glass",        color: (200, 220, 210), variant: "glass_green" },
    WallPaletteEntry { material: "SmoothPlastic", color: (220, 215, 205), variant: "panel_cream" },
];

const INDUSTRIAL_WALL_PALETTE: &[WallPaletteEntry] = &[
    WallPaletteEntry { material: "Metal", color: (144, 150, 154), variant: "metal_grey" },
    WallPaletteEntry { material: "Metal", color: (120, 125, 130), variant: "metal_dark" },
    WallPaletteEntry { material: "Metal", color: (160, 155, 145), variant: "metal_tan" },
];

const DEFAULT_WALL_PALETTE: &[WallPaletteEntry] = &[
    WallPaletteEntry { material: "Concrete", color: (189, 184, 176), variant: "concrete_default" },
    WallPaletteEntry { material: "Concrete", color: (200, 195, 185), variant: "concrete_warm" },
    WallPaletteEntry { material: "Brick",    color: (175, 130, 100), variant: "brick_warm" },
    WallPaletteEntry { material: "Concrete", color: (180, 180, 185), variant: "concrete_cool" },
];

/// Map an OSM wall material tag to a Roblox material name.
fn osm_wall_material_to_roblox(tag: &str) -> (&'static str, Option<(u8, u8, u8)>) {
    match tag.to_ascii_lowercase().as_str() {
        "brick" | "masonry" => ("Brick", None),
        "concrete" | "cement" => ("Concrete", None),
        "glass" => ("Glass", None),
        "metal" | "steel" | "aluminium" | "aluminum" => ("Metal", None),
        "wood" | "timber" => ("WoodPlanks", None),
        "stone" | "granite" | "limestone" => ("Limestone", None),
        "sandstone" => ("Sandstone", None),
        "marble" => ("Marble", None),
        "plaster" | "stucco" | "render" => ("Concrete", Some((230, 220, 210))),
        "clapboard" | "siding" => ("WoodPlanks", None),
        _ => ("Concrete", None),
    }
}

/// Resolve wall material from OSM tags.
///
/// Priority: explicit material/facade/cladding tag > colour tag > usage-based
/// palette with deterministic diversity seeding.
pub fn resolve_wall_material(
    material_tag: Option<&str>,
    facade_tag: Option<&str>,
    cladding_tag: Option<&str>,
    colour_tag: Option<&str>,
    usage: Option<&str>,
    building_id: &str,
) -> ResolvedMaterial {
    // 1. Explicit material tag (building:material, building:facade:material, building:cladding)
    let explicit_tag = material_tag.or(facade_tag).or(cladding_tag);
    if let Some(tag) = explicit_tag {
        let (mat, default_color) = osm_wall_material_to_roblox(tag);
        let color = colour_tag.and_then(parse_css_color).or(default_color);
        return ResolvedMaterial {
            material: mat.to_string(),
            color,
            variant: None,
        };
    }

    // 2. Only colour tag, no material — use concrete with the parsed colour
    if let Some(colour) = colour_tag {
        if let Some(rgb) = parse_css_color(colour) {
            return ResolvedMaterial {
                material: "Concrete".to_string(),
                color: Some(rgb),
                variant: None,
            };
        }
    }

    // 3. Usage-based palette with diversity seeding
    let seed = hash_building_id(building_id);
    let palette = match usage.unwrap_or("") {
        "residential" | "house" | "detached" | "apartments" | "terrace" | "dormitory" => {
            RESIDENTIAL_WALL_PALETTE
        }
        "commercial" | "retail" | "office" | "hotel" | "supermarket" | "restaurant" | "bank" => {
            COMMERCIAL_WALL_PALETTE
        }
        "industrial" | "warehouse" | "factory" | "garage" => INDUSTRIAL_WALL_PALETTE,
        _ => DEFAULT_WALL_PALETTE,
    };
    let entry = &palette[pick_index(palette.len(), seed)];
    ResolvedMaterial {
        material: entry.material.to_string(),
        color: Some(entry.color),
        variant: Some(entry.variant.to_string()),
    }
}

// ---------------------------------------------------------------------------
// Roof material resolution
// ---------------------------------------------------------------------------

struct RoofPaletteEntry {
    material: &'static str,
    color: (u8, u8, u8),
    variant: &'static str,
}

const RESIDENTIAL_ROOF_PALETTE: &[RoofPaletteEntry] = &[
    RoofPaletteEntry { material: "Brick",      color: (180, 80, 60),   variant: "tile_terracotta" },
    RoofPaletteEntry { material: "Slate",      color: (90, 90, 100),   variant: "slate_dark" },
    RoofPaletteEntry { material: "Asphalt",    color: (70, 70, 75),    variant: "shingle_dark" },
    RoofPaletteEntry { material: "Asphalt",    color: (100, 95, 85),   variant: "shingle_brown" },
    RoofPaletteEntry { material: "WoodPlanks", color: (140, 110, 75),  variant: "wood_shake" },
];

const COMMERCIAL_ROOF_PALETTE: &[RoofPaletteEntry] = &[
    RoofPaletteEntry { material: "Concrete", color: (170, 165, 160), variant: "concrete_flat" },
    RoofPaletteEntry { material: "Metal",    color: (160, 165, 170), variant: "metal_standing" },
    RoofPaletteEntry { material: "Concrete", color: (185, 180, 175), variant: "concrete_light" },
];

const DEFAULT_ROOF_PALETTE: &[RoofPaletteEntry] = &[
    RoofPaletteEntry { material: "Concrete", color: (160, 155, 150), variant: "roof_default" },
    RoofPaletteEntry { material: "Asphalt",  color: (80, 80, 85),    variant: "asphalt_dark" },
    RoofPaletteEntry { material: "Metal",    color: (150, 155, 160), variant: "metal_grey" },
];

fn osm_roof_material_to_roblox(tag: &str) -> (&'static str, Option<(u8, u8, u8)>) {
    match tag.to_ascii_lowercase().as_str() {
        "slate" => ("Slate", None),
        "tile" | "tiles" | "clay" => ("Brick", Some((180, 80, 60))),
        "metal" | "tin" | "copper" => ("Metal", None),
        "concrete" | "tar" => ("Concrete", None),
        "thatch" | "grass" => ("Grass", None),
        "asphalt" | "asphalt_shingle" => ("Asphalt", None),
        "wood" | "shingle" => ("WoodPlanks", None),
        _ => ("Concrete", None),
    }
}

/// Resolve roof material from OSM tags.
///
/// Priority: explicit material tag > colour tag > shape-based/usage-based
/// palette with deterministic diversity seeding.
pub fn resolve_roof_material(
    material_tag: Option<&str>,
    colour_tag: Option<&str>,
    roof_shape: &str,
    usage: Option<&str>,
    building_id: &str,
) -> ResolvedMaterial {
    // 1. Explicit roof:material tag
    if let Some(tag) = material_tag {
        let (mat, default_color) = osm_roof_material_to_roblox(tag);
        let color = colour_tag.and_then(parse_css_color).or(default_color);
        return ResolvedMaterial {
            material: mat.to_string(),
            color,
            variant: None,
        };
    }

    // 2. Only colour tag
    if let Some(colour) = colour_tag {
        if let Some(rgb) = parse_css_color(colour) {
            return ResolvedMaterial {
                material: "Concrete".to_string(),
                color: Some(rgb),
                variant: None,
            };
        }
    }

    // 3. Flat roofs get concrete/metal regardless of usage
    if roof_shape == "flat" {
        let seed = hash_building_id(building_id);
        let flat_palette: &[RoofPaletteEntry] = &[
            RoofPaletteEntry { material: "Concrete", color: (170, 165, 160), variant: "flat_concrete" },
            RoofPaletteEntry { material: "Metal",    color: (160, 165, 170), variant: "flat_metal" },
            RoofPaletteEntry { material: "Concrete", color: (185, 180, 175), variant: "flat_light" },
        ];
        let entry = &flat_palette[pick_index(flat_palette.len(), seed)];
        return ResolvedMaterial {
            material: entry.material.to_string(),
            color: Some(entry.color),
            variant: Some(entry.variant.to_string()),
        };
    }

    // 4. Usage-based palette with diversity seeding
    let seed = hash_building_id(building_id);
    let palette = match usage.unwrap_or("") {
        "residential" | "house" | "detached" | "apartments" | "terrace" | "dormitory" => {
            RESIDENTIAL_ROOF_PALETTE
        }
        "commercial" | "retail" | "office" | "hotel" | "supermarket" | "restaurant" | "bank" => {
            COMMERCIAL_ROOF_PALETTE
        }
        _ => DEFAULT_ROOF_PALETTE,
    };
    let entry = &palette[pick_index(palette.len(), seed)];
    ResolvedMaterial {
        material: entry.material.to_string(),
        color: Some(entry.color),
        variant: Some(entry.variant.to_string()),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_brick_tag() {
        let r = resolve_wall_material(Some("brick"), None, None, None, None, "bldg_1");
        assert_eq!(r.material, "Brick");
    }

    #[test]
    fn explicit_glass_tag() {
        let r = resolve_wall_material(Some("glass"), None, None, None, None, "bldg_2");
        assert_eq!(r.material, "Glass");
    }

    #[test]
    fn explicit_metal_tag() {
        let r = resolve_wall_material(Some("steel"), None, None, None, None, "bldg_3");
        assert_eq!(r.material, "Metal");
    }

    #[test]
    fn facade_tag_overrides_material() {
        let r = resolve_wall_material(None, Some("glass"), None, None, None, "bldg_4");
        assert_eq!(r.material, "Glass");
    }

    #[test]
    fn cladding_tag_fallback() {
        let r = resolve_wall_material(None, None, Some("wood"), None, None, "bldg_5");
        assert_eq!(r.material, "WoodPlanks");
    }

    #[test]
    fn plaster_gets_lighter_color() {
        let r = resolve_wall_material(Some("plaster"), None, None, None, None, "bldg_6");
        assert_eq!(r.material, "Concrete");
        assert_eq!(r.color, Some((230, 220, 210)));
    }

    #[test]
    fn no_tag_residential_gets_palette_variant() {
        let r = resolve_wall_material(None, None, None, None, Some("residential"), "bldg_7");
        // Must pick from the residential palette
        let valid_materials = ["Brick", "WoodPlanks", "Concrete"];
        assert!(
            valid_materials.contains(&r.material.as_str()),
            "unexpected material: {}",
            r.material
        );
        assert!(r.variant.is_some(), "residential fallback should have variant");
    }

    #[test]
    fn deterministic_same_id() {
        let r1 = resolve_wall_material(None, None, None, None, Some("residential"), "stable_id");
        let r2 = resolve_wall_material(None, None, None, None, Some("residential"), "stable_id");
        assert_eq!(r1, r2, "same building_id must produce identical results");
    }

    #[test]
    fn diversity_different_ids() {
        // With enough different IDs, we should see at least 2 distinct variants
        let variants: std::collections::HashSet<_> = (0..20)
            .map(|i| {
                let r = resolve_wall_material(
                    None, None, None, None,
                    Some("residential"),
                    &format!("bldg_{}", i),
                );
                r.variant.unwrap()
            })
            .collect();
        assert!(
            variants.len() >= 2,
            "expected diversity across building IDs, got {:?}",
            variants
        );
    }

    #[test]
    fn css_color_red() {
        assert_eq!(parse_css_color("red"), Some((255, 0, 0)));
    }

    #[test]
    fn css_color_hex() {
        assert_eq!(parse_css_color("#FF8800"), Some((255, 136, 0)));
    }

    #[test]
    fn css_color_hex_lowercase() {
        assert_eq!(parse_css_color("#ff8800"), Some((255, 136, 0)));
    }

    #[test]
    fn css_color_hex_short() {
        assert_eq!(parse_css_color("#f80"), Some((255, 136, 0)));
    }

    #[test]
    fn css_color_grey_both_spellings() {
        assert_eq!(parse_css_color("grey"), parse_css_color("gray"));
    }

    #[test]
    fn colour_tag_applied_to_explicit_material() {
        let r = resolve_wall_material(Some("brick"), None, None, Some("red"), None, "bldg_c");
        assert_eq!(r.material, "Brick");
        assert_eq!(r.color, Some((255, 0, 0)));
    }

    #[test]
    fn colour_tag_only_no_material() {
        let r = resolve_wall_material(None, None, None, Some("#AABB00"), None, "bldg_co");
        assert_eq!(r.material, "Concrete");
        assert_eq!(r.color, Some((170, 187, 0)));
    }

    // --- Roof tests ---

    #[test]
    fn roof_explicit_slate() {
        let r = resolve_roof_material(Some("slate"), None, "gabled", None, "r1");
        assert_eq!(r.material, "Slate");
    }

    #[test]
    fn roof_explicit_tile_gets_terracotta() {
        let r = resolve_roof_material(Some("tile"), None, "hipped", None, "r2");
        assert_eq!(r.material, "Brick");
        assert_eq!(r.color, Some((180, 80, 60)));
    }

    #[test]
    fn roof_explicit_thatch() {
        let r = resolve_roof_material(Some("thatch"), None, "gabled", None, "r3");
        assert_eq!(r.material, "Grass");
    }

    #[test]
    fn roof_flat_shape_defaults() {
        let r = resolve_roof_material(None, None, "flat", Some("commercial"), "r4");
        let valid = ["Concrete", "Metal"];
        assert!(valid.contains(&r.material.as_str()), "flat roof got: {}", r.material);
    }

    #[test]
    fn roof_deterministic() {
        let r1 = resolve_roof_material(None, None, "gabled", Some("residential"), "roof_stable");
        let r2 = resolve_roof_material(None, None, "gabled", Some("residential"), "roof_stable");
        assert_eq!(r1, r2);
    }

    #[test]
    fn roof_colour_tag() {
        let r = resolve_roof_material(Some("metal"), Some("#112233"), "gabled", None, "rc");
        assert_eq!(r.material, "Metal");
        assert_eq!(r.color, Some((17, 34, 51)));
    }
}
