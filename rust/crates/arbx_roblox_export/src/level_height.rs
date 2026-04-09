//! Level and height resolution — ported from osm2world's `LevelAndHeightData`.
//!
//! Resolves the complex interplay of OSM building tags (`height`,
//! `building:levels`, `roof:height`, `roof:angle`, `min_height`, etc.) into
//! concrete vertical intervals for wall and roof generation.

/// Fully resolved vertical intervals for one building or building:part.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedHeights {
    /// Ground level (from `min_height` or 0).
    pub base_y: f64,
    /// Wall top = base_y + wall_height.
    pub wall_height: f64,
    /// Roof peak height above wall top.
    pub roof_height: f64,
    /// Per-floor height (wall_height / level_count).
    pub floor_height: f64,
    /// Number of floors.
    pub level_count: u32,
}

/// Default per-floor height in metres (osm2world convention).
const DEFAULT_FLOOR_HEIGHT: f64 = 3.5;
/// Foundation/parapet allowance added when computing total height from levels.
const FOUNDATION_PARAPET: f64 = 2.0;

/// Resolve OSM building tags into concrete vertical intervals.
///
/// Mirrors osm2world's `LevelAndHeightData` resolution logic:
///
/// - If `height` is given, use it as the total height (base_y to top of roof).
/// - If only `levels` is given, compute `levels * 3.5 + 2.0`.
/// - `wall_height` = total_height - roof_height.
/// - `roof_height` from tag, or from `roof:angle` via `tan(angle) * max_dist_to_ridge`,
///   or default 0 for flat, 1/3 wall_height for gabled/hipped.
/// - `level_count` = levels tag, or `wall_height / 3.5` rounded (min 1).
/// - `floor_height` = `wall_height / level_count`.
pub fn resolve_heights(
    base_y: f64,
    height: Option<f64>,
    levels: Option<u32>,
    roof_height: Option<f64>,
    roof_angle: Option<f64>,
    roof_levels: Option<u32>,
    min_height: Option<f64>,
    max_dist_to_ridge: f64,
) -> ResolvedHeights {
    let effective_base = min_height.unwrap_or(base_y);

    // --- Total height ---
    let total_height = if let Some(h) = height {
        h
    } else if let Some(l) = levels {
        let roof_lvls = roof_levels.unwrap_or(0) as f64;
        (l as f64 + roof_lvls) * DEFAULT_FLOOR_HEIGHT + FOUNDATION_PARAPET
    } else {
        // No explicit height or levels — default 1-storey building.
        DEFAULT_FLOOR_HEIGHT + FOUNDATION_PARAPET
    };

    // --- Roof height ---
    let computed_roof_height = if let Some(rh) = roof_height {
        rh
    } else if let Some(angle_deg) = roof_angle {
        let angle_rad = angle_deg.to_radians();
        (angle_rad.tan() * max_dist_to_ridge).max(0.0)
    } else if let Some(rl) = roof_levels {
        rl as f64 * DEFAULT_FLOOR_HEIGHT
    } else {
        // Default: 0 (flat). Caller can override for gabled/hipped by passing
        // roof_height = Some(wall_height / 3.0) before calling.
        0.0
    };

    // Clamp roof height so wall_height stays positive.
    let clamped_roof = computed_roof_height.min(total_height * 0.9);

    let wall_h = (total_height - clamped_roof).max(DEFAULT_FLOOR_HEIGHT);

    // --- Level count ---
    let lvl_count = if let Some(l) = levels {
        l.max(1)
    } else {
        (wall_h / DEFAULT_FLOOR_HEIGHT).round().max(1.0) as u32
    };

    let floor_h = wall_h / lvl_count as f64;

    ResolvedHeights {
        base_y: effective_base,
        wall_height: wall_h,
        roof_height: clamped_roof,
        floor_height: floor_h,
        level_count: lvl_count,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    #[test]
    fn test_explicit_height_no_roof() {
        let r = resolve_heights(0.0, Some(12.0), None, None, None, None, None, 5.0);
        assert!(approx(r.base_y, 0.0));
        assert!(approx(r.wall_height, 12.0));
        assert!(approx(r.roof_height, 0.0));
        assert_eq!(r.level_count, 3); // 12 / 3.5 ~ 3.4 -> round to 3
        assert!(approx(r.floor_height, 4.0));
    }

    #[test]
    fn test_levels_only() {
        let r = resolve_heights(0.0, None, Some(4), None, None, None, None, 5.0);
        // total = 4 * 3.5 + 2.0 = 16.0
        assert!(approx(r.wall_height, 16.0));
        assert_eq!(r.level_count, 4);
        assert!(approx(r.floor_height, 4.0));
    }

    #[test]
    fn test_height_with_roof_height() {
        let r = resolve_heights(0.0, Some(15.0), Some(3), Some(3.0), None, None, None, 5.0);
        assert!(approx(r.wall_height, 12.0));
        assert!(approx(r.roof_height, 3.0));
        assert_eq!(r.level_count, 3);
        assert!(approx(r.floor_height, 4.0));
    }

    #[test]
    fn test_roof_angle() {
        // tan(45 deg) * 5.0 = 5.0
        let r = resolve_heights(0.0, Some(15.0), None, None, Some(45.0), None, None, 5.0);
        assert!(approx(r.roof_height, 5.0));
        assert!(approx(r.wall_height, 10.0));
    }

    #[test]
    fn test_min_height() {
        let r = resolve_heights(0.0, Some(20.0), None, None, None, None, Some(5.0), 5.0);
        assert!(approx(r.base_y, 5.0));
    }

    #[test]
    fn test_roof_levels_no_explicit_height() {
        let r = resolve_heights(0.0, None, Some(3), None, None, Some(1), None, 5.0);
        // total = (3 + 1) * 3.5 + 2.0 = 16.0, roof_height = 1 * 3.5 = 3.5
        assert!(approx(r.roof_height, 3.5));
        assert!(approx(r.wall_height, 12.5));
        assert_eq!(r.level_count, 3);
    }

    #[test]
    fn test_default_single_storey() {
        let r = resolve_heights(0.0, None, None, None, None, None, None, 5.0);
        assert!(approx(r.wall_height, DEFAULT_FLOOR_HEIGHT + FOUNDATION_PARAPET));
        assert_eq!(r.level_count, 2); // 5.5 / 3.5 ~ 1.57 -> round to 2
    }

    #[test]
    fn test_roof_height_clamped() {
        // Roof height larger than total -> clamped to 90% of total
        let r = resolve_heights(0.0, Some(10.0), None, Some(20.0), None, None, None, 5.0);
        assert!(approx(r.roof_height, 9.0)); // 90% of 10
        assert!(r.wall_height >= DEFAULT_FLOOR_HEIGHT);
    }
}
