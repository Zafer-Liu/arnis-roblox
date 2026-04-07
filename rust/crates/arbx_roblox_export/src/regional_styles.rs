//! Regional style packs for city-specific architectural palettes.
//!
//! Different regions of the world have characteristic building materials,
//! roof materials, and wall color palettes. This module lets the exporter
//! pick a region-aware override layer that sits on top of the default
//! [`crate::materials::StyleMapper`] behavior without replacing it.
//!
//! The design is intentionally non-invasive: when a regional style has no
//! entry for a particular `(usage, kind)` combination, the default
//! `StyleMapper` logic is used unchanged.

use crate::manifest::Color;
use arbx_geo::BoundingBox;
use std::collections::HashMap;

/// A regional architectural style pack.
///
/// `building_materials` maps a building usage tag (e.g. `"default"`,
/// `"commercial"`, `"residential"`, `"civic"`) to an ordered list of
/// preferred Roblox material names. The first entry is the dominant
/// material for that usage; downstream code is free to consult later
/// entries as fallbacks or variants.
///
/// `roof_materials` is an ordered list of preferred roof material names
/// for the region (dominant first).
///
/// `wall_colors` is a palette of characteristic wall colors for the
/// region; it is used as a source of warm/cool tint when a feature does
/// not carry an explicit OSM color tag.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalStyle {
    pub region_name: String,
    pub building_materials: HashMap<String, Vec<String>>,
    pub roof_materials: Vec<String>,
    pub wall_colors: Vec<Color>,
}

impl RegionalStyle {
    /// Returns the dominant building material for the given usage tag,
    /// if the regional style has an entry for it.
    pub fn building_material(&self, usage: &str) -> Option<&str> {
        self.building_materials
            .get(usage)
            .or_else(|| self.building_materials.get("default"))
            .and_then(|list| list.first().map(String::as_str))
    }

    /// Returns the dominant roof material for the region, if any.
    pub fn roof_material(&self) -> Option<&str> {
        self.roof_materials.first().map(String::as_str)
    }

    /// Returns a characteristic wall color for the given usage tag,
    /// selected deterministically from the palette. Returns `None`
    /// when the palette is empty.
    pub fn wall_color(&self, usage: &str) -> Option<Color> {
        if self.wall_colors.is_empty() {
            return None;
        }
        // Deterministic pick so the same usage always yields the same tint
        // within a region.
        let mut hash: u32 = 0x811c9dc5;
        for b in usage.as_bytes() {
            hash ^= *b as u32;
            hash = hash.wrapping_mul(0x01000193);
        }
        let idx = (hash as usize) % self.wall_colors.len();
        Some(self.wall_colors[idx])
    }
}

/// Resolve the regional style for a given world bounding box.
///
/// Detection uses the bbox centroid lat/lon and matches against a fixed
/// set of coarse geographic envelopes. Anything outside the known
/// envelopes falls back to [`default_style`].
pub fn resolve_regional_style(bbox: &BoundingBox) -> RegionalStyle {
    let center = bbox.center();
    resolve_regional_style_latlon(center.lat, center.lon)
}

/// Internal helper used by `resolve_regional_style` and by tests that
/// want to check a specific lat/lon without building a full `BoundingBox`.
pub fn resolve_regional_style_latlon(lat: f64, lon: f64) -> RegionalStyle {
    // Texas / Austin
    if (29.0..=33.0).contains(&lat) && (-100.0..=-96.0).contains(&lon) {
        return austin_style();
    }
    // Netherlands / Amsterdam
    if (51.0..=53.0).contains(&lat) && (3.0..=7.0).contains(&lon) {
        return amsterdam_style();
    }
    // Japan / Tokyo
    if (35.0..=36.0).contains(&lat) && (139.0..=140.0).contains(&lon) {
        return tokyo_style();
    }
    // California / San Francisco
    if (37.0..=38.0).contains(&lat) && (-123.0..=-121.0).contains(&lon) {
        return san_francisco_style();
    }
    default_style()
}

/// Austin, TX: warm limestone, brick, concrete, terra cotta roofs.
pub fn austin_style() -> RegionalStyle {
    let mut building_materials: HashMap<String, Vec<String>> = HashMap::new();
    building_materials.insert(
        "default".to_string(),
        vec![
            "Limestone".to_string(),
            "Brick".to_string(),
            "Concrete".to_string(),
        ],
    );
    building_materials.insert(
        "civic".to_string(),
        vec!["Limestone".to_string(), "Marble".to_string()],
    );
    building_materials.insert(
        "government".to_string(),
        vec!["Limestone".to_string(), "Marble".to_string()],
    );
    building_materials.insert(
        "university".to_string(),
        vec!["Limestone".to_string(), "Brick".to_string()],
    );
    building_materials.insert(
        "residential".to_string(),
        vec!["Brick".to_string(), "Concrete".to_string()],
    );
    building_materials.insert(
        "commercial".to_string(),
        vec!["Concrete".to_string(), "Limestone".to_string()],
    );

    RegionalStyle {
        region_name: "austin".to_string(),
        building_materials,
        roof_materials: vec![
            "Slate".to_string(),
            "Concrete".to_string(),
            "Metal".to_string(),
        ],
        wall_colors: vec![
            Color::new(0xdd, 0xd3, 0xc0), // warm limestone
            Color::new(0xc8, 0xa1, 0x88), // warm brick
            Color::new(0xb0, 0x67, 0x4f), // terracotta brick
            Color::new(0xd7, 0xd0, 0xc4), // pale limestone
        ],
    }
}

/// Amsterdam: brick dominant, clay tile roofs, gabled townhouses.
pub fn amsterdam_style() -> RegionalStyle {
    let mut building_materials: HashMap<String, Vec<String>> = HashMap::new();
    building_materials.insert(
        "default".to_string(),
        vec![
            "Brick".to_string(),
            "Sandstone".to_string(),
            "Concrete".to_string(),
        ],
    );
    building_materials.insert(
        "residential".to_string(),
        vec!["Brick".to_string(), "Sandstone".to_string()],
    );
    building_materials.insert(
        "apartments".to_string(),
        vec!["Brick".to_string(), "Sandstone".to_string()],
    );
    building_materials.insert(
        "terrace".to_string(),
        vec!["Brick".to_string()],
    );
    building_materials.insert(
        "house".to_string(),
        vec!["Brick".to_string()],
    );
    building_materials.insert(
        "commercial".to_string(),
        vec!["Brick".to_string(), "Concrete".to_string()],
    );
    building_materials.insert(
        "civic".to_string(),
        vec!["Sandstone".to_string(), "Brick".to_string()],
    );

    RegionalStyle {
        region_name: "amsterdam".to_string(),
        building_materials,
        roof_materials: vec![
            "Slate".to_string(),
            "Brick".to_string(),
            "Metal".to_string(),
        ],
        wall_colors: vec![
            Color::new(0x8b, 0x3a, 0x2f), // deep red brick
            Color::new(0xa8, 0x4e, 0x3c), // classic brick
            Color::new(0xc4, 0x63, 0x3c), // red sandstone
            Color::new(0x6e, 0x2b, 0x20), // dark clay
        ],
    }
}

/// Tokyo: concrete, metal, glass. Cool modern palette.
pub fn tokyo_style() -> RegionalStyle {
    let mut building_materials: HashMap<String, Vec<String>> = HashMap::new();
    building_materials.insert(
        "default".to_string(),
        vec![
            "Concrete".to_string(),
            "Glass".to_string(),
            "Metal".to_string(),
        ],
    );
    building_materials.insert(
        "commercial".to_string(),
        vec!["Glass".to_string(), "Concrete".to_string()],
    );
    building_materials.insert(
        "office".to_string(),
        vec!["Glass".to_string(), "Concrete".to_string(), "Metal".to_string()],
    );
    building_materials.insert(
        "residential".to_string(),
        vec!["Concrete".to_string(), "Metal".to_string()],
    );
    building_materials.insert(
        "apartments".to_string(),
        vec!["Concrete".to_string()],
    );
    building_materials.insert(
        "industrial".to_string(),
        vec!["Metal".to_string(), "Concrete".to_string()],
    );
    building_materials.insert(
        "temple".to_string(),
        vec!["WoodPlanks".to_string(), "Slate".to_string()],
    );

    RegionalStyle {
        region_name: "tokyo".to_string(),
        building_materials,
        roof_materials: vec![
            "Metal".to_string(),
            "Concrete".to_string(),
            "Slate".to_string(),
        ],
        wall_colors: vec![
            Color::new(0xbf, 0xc5, 0xcb), // cool concrete
            Color::new(0x9a, 0xa3, 0xad), // steel grey
            Color::new(0xd6, 0xdc, 0xe2), // pale glass
            Color::new(0x72, 0x7a, 0x84), // dark steel
        ],
    }
}

/// San Francisco: painted wooden row houses, stucco, concrete.
pub fn san_francisco_style() -> RegionalStyle {
    let mut building_materials: HashMap<String, Vec<String>> = HashMap::new();
    building_materials.insert(
        "default".to_string(),
        vec![
            "WoodPlanks".to_string(),
            "SmoothPlastic".to_string(),
            "Concrete".to_string(),
        ],
    );
    building_materials.insert(
        "residential".to_string(),
        vec!["WoodPlanks".to_string(), "SmoothPlastic".to_string()],
    );
    building_materials.insert(
        "house".to_string(),
        vec!["WoodPlanks".to_string()],
    );
    building_materials.insert(
        "detached".to_string(),
        vec!["WoodPlanks".to_string()],
    );
    building_materials.insert(
        "apartments".to_string(),
        vec!["SmoothPlastic".to_string(), "WoodPlanks".to_string()],
    );
    building_materials.insert(
        "commercial".to_string(),
        vec!["Concrete".to_string(), "SmoothPlastic".to_string()],
    );

    RegionalStyle {
        region_name: "san_francisco".to_string(),
        building_materials,
        roof_materials: vec![
            "Slate".to_string(),
            "Metal".to_string(),
            "Concrete".to_string(),
        ],
        wall_colors: vec![
            Color::new(0xe8, 0xc8, 0x8c), // painted yellow
            Color::new(0xd4, 0x93, 0x7e), // terracotta painted
            Color::new(0xbf, 0xd6, 0xd2), // pale teal painted
            Color::new(0xe6, 0xde, 0xcf), // cream stucco
        ],
    }
}

/// Default / unknown region: preserves the existing `StyleMapper` behavior
/// by having no overrides. An empty map means every lookup falls through.
pub fn default_style() -> RegionalStyle {
    RegionalStyle {
        region_name: "default".to_string(),
        building_materials: HashMap::new(),
        roof_materials: Vec::new(),
        wall_colors: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn austin_detected_from_capitol_bbox() {
        let bbox = BoundingBox::new(30.264, -97.750, 30.266, -97.748);
        let style = resolve_regional_style(&bbox);
        assert_eq!(style.region_name, "austin");
    }

    #[test]
    fn amsterdam_detected_from_dam_square() {
        let bbox = BoundingBox::new(52.371, 4.892, 52.373, 4.894);
        let style = resolve_regional_style(&bbox);
        assert_eq!(style.region_name, "amsterdam");
    }

    #[test]
    fn tokyo_detected_from_shibuya() {
        let bbox = BoundingBox::new(35.658, 139.700, 35.660, 139.702);
        let style = resolve_regional_style(&bbox);
        assert_eq!(style.region_name, "tokyo");
    }

    #[test]
    fn san_francisco_detected_from_civic_center() {
        let bbox = BoundingBox::new(37.778, -122.419, 37.780, -122.417);
        let style = resolve_regional_style(&bbox);
        assert_eq!(style.region_name, "san_francisco");
    }

    #[test]
    fn unknown_region_falls_back_to_default() {
        // Middle of the Atlantic
        let bbox = BoundingBox::new(0.0, -30.0, 0.1, -29.9);
        let style = resolve_regional_style(&bbox);
        assert_eq!(style.region_name, "default");
        assert!(style.building_materials.is_empty());
        assert!(style.roof_materials.is_empty());
        assert!(style.wall_colors.is_empty());
    }

    #[test]
    fn austin_prefers_limestone_for_civic() {
        let style = austin_style();
        assert_eq!(style.building_material("civic"), Some("Limestone"));
        assert_eq!(style.building_material("government"), Some("Limestone"));
    }

    #[test]
    fn amsterdam_prefers_brick_for_residential() {
        let style = amsterdam_style();
        assert_eq!(style.building_material("residential"), Some("Brick"));
        assert_eq!(style.building_material("apartments"), Some("Brick"));
    }

    #[test]
    fn tokyo_prefers_glass_for_offices() {
        let style = tokyo_style();
        assert_eq!(style.building_material("office"), Some("Glass"));
        assert_eq!(style.building_material("commercial"), Some("Glass"));
    }

    #[test]
    fn san_francisco_prefers_wood_for_houses() {
        let style = san_francisco_style();
        assert_eq!(style.building_material("house"), Some("WoodPlanks"));
        assert_eq!(style.building_material("residential"), Some("WoodPlanks"));
    }

    #[test]
    fn default_style_has_no_overrides() {
        let style = default_style();
        assert_eq!(style.building_material("house"), None);
        assert_eq!(style.roof_material(), None);
        assert_eq!(style.wall_color("house"), None);
    }

    #[test]
    fn unknown_usage_falls_back_to_default_entry_in_region() {
        let style = austin_style();
        // "warehouse" is not a key in the Austin pack; it should fall
        // back to the pack's own "default" entry, which is limestone.
        assert_eq!(style.building_material("warehouse"), Some("Limestone"));
    }

    #[test]
    fn wall_color_is_deterministic_per_usage() {
        let style = austin_style();
        let a = style.wall_color("civic");
        let b = style.wall_color("civic");
        assert_eq!(a, b);
        assert!(a.is_some());
    }
}
