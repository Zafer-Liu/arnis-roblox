//! Compile-time road mesh strip generation.
//!
//! Mirrors the ribbon-along-polyline algorithm from `RoadBuilder.lua`'s
//! `RoadMeshAccumulator:addRoadStrip` so that geometry can be baked into the
//! manifest at export time instead of computed at Roblox runtime.

use std::fmt::Write as _;

/// Pre-computed road mesh strip geometry (flat vertex/index buffers).
#[derive(Debug, Clone, PartialEq)]
pub struct RoadMeshStrip {
    /// Interleaved [x, y, z, x, y, z, ...] vertex positions.
    pub vertices: Vec<f32>,
    /// Triangle indices (3 per triangle) referencing vertex positions.
    pub triangles: Vec<u32>,
    /// Interleaved [nx, ny, nz, ...] per-vertex normals.
    pub normals: Vec<f32>,
}

impl RoadMeshStrip {
    /// Number of vertices in this strip.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / 3
    }

    /// Number of triangles in this strip.
    pub fn triangle_count(&self) -> usize {
        self.triangles.len() / 3
    }
}

/// Build a road mesh strip from a 3-D polyline.
///
/// For each consecutive pair of points in `points`, this creates a box-shaped
/// ribbon segment with 8 vertices and 12 triangles (6 quad faces).  The
/// geometry exactly matches the Lua `addRoadStrip` logic so the Roblox client
/// can skip runtime mesh generation when a pre-baked strip is present.
///
/// # Parameters
/// - `points` — ordered 3-D polyline of the road centreline (world-space studs).
/// - `width` — full road width in studs.
/// - `surface_lift` — vertical offset above each polyline point for the road surface.
/// - `thickness` — vertical thickness of the road slab.
/// - `lateral_offset` — perpendicular offset from the centreline in studs
///   (positive = right of travel direction, negative = left). Mirrors the Lua
///   `addRoadStrip` `sideOffset` parameter.
pub fn build_road_strip(
    points: &[(f64, f64, f64)],
    width: f64,
    surface_lift: f64,
    thickness: f64,
    lateral_offset: f64,
) -> RoadMeshStrip {
    let mut vertices: Vec<f32> = Vec::new();
    let mut triangles: Vec<u32> = Vec::new();
    let mut normals: Vec<f32> = Vec::new();

    let thickness = thickness.max(0.05);

    // Per-face vertex approach: 4 verts per face * 6 faces = 24 vertices per
    // segment, with flat per-face normals matching the Lua addQuad calls.
    for window in points.windows(2) {
        let (p1, p2) = (window[0], window[1]);

        let dx = p2.0 - p1.0;
        let dz = p2.2 - p1.2;
        let horiz_len = (dx * dx + dz * dz).sqrt();
        if horiz_len < 0.01 {
            continue;
        }

        let hx = dx / horiz_len;
        let hz = dz / horiz_len;
        let px = -hz;
        let pz = hx;
        let half_w = width * 0.5;

        let y1 = p1.1 + surface_lift;
        let y2 = p2.1 + surface_lift;

        // Lateral offset shifts the entire strip perpendicular to travel direction.
        let lx = px * lateral_offset;
        let lz = pz * lateral_offset;

        let v1 = (p1.0 - px * half_w + lx, y1, p1.2 - pz * half_w + lz);
        let v2 = (p1.0 + px * half_w + lx, y1, p1.2 + pz * half_w + lz);
        let v3 = (p2.0 + px * half_w + lx, y2, p2.2 + pz * half_w + lz);
        let v4 = (p2.0 - px * half_w + lx, y2, p2.2 - pz * half_w + lz);

        let b1 = (v1.0, v1.1 - thickness, v1.2);
        let b2 = (v2.0, v2.1 - thickness, v2.2);
        let b3 = (v3.0, v3.1 - thickness, v3.2);
        let b4 = (v4.0, v4.1 - thickness, v4.2);

        // 6 face quads — same winding as Lua addRoadStrip:
        //   top:    v1, v2, v3, v4   normal (0,  1, 0)
        //   bottom: b4, b3, b2, b1   normal (0, -1, 0)
        //   front:  v4, v3, b3, b4   normal (+hx, 0, +hz)  (horizDir)
        //   back:   v2, v1, b1, b2   normal (-hx, 0, -hz)
        //   left:   v1, v4, b4, b1   normal (-px, 0, -pz)  (-unitPerp)
        //   right:  v3, v2, b2, b3   normal (+px, 0, +pz)  (+unitPerp)
        let faces: [([_; 4], [f32; 3]); 6] = [
            ([v1, v2, v3, v4], [0.0, 1.0, 0.0]),
            ([b4, b3, b2, b1], [0.0, -1.0, 0.0]),
            ([v4, v3, b3, b4], [hx as f32, 0.0, hz as f32]),
            ([v2, v1, b1, b2], [-hx as f32, 0.0, -hz as f32]),
            ([v1, v4, b4, b1], [-px as f32, 0.0, -pz as f32]),
            ([v3, v2, b2, b3], [px as f32, 0.0, pz as f32]),
        ];

        for (quad, normal) in &faces {
            let base = (vertices.len() / 3) as u32;
            for &(qx, qy, qz) in quad {
                vertices.push(qx as f32);
                vertices.push(qy as f32);
                vertices.push(qz as f32);
                normals.push(normal[0]);
                normals.push(normal[1]);
                normals.push(normal[2]);
            }
            // Two triangles per quad: (0,1,2) and (0,2,3)
            triangles.push(base);
            triangles.push(base + 1);
            triangles.push(base + 2);
            triangles.push(base);
            triangles.push(base + 2);
            triangles.push(base + 3);
        }
    }

    RoadMeshStrip {
        vertices,
        triangles,
        normals,
    }
}

// ── JSON serialisation helpers (used by manifest.rs) ────────────────────────

impl RoadMeshStrip {
    /// Write this mesh strip as a JSON object into the manifest serialiser.
    pub fn write_json(&self, out: &mut String, indent: usize) {
        write_indent(out, indent);
        out.push_str("{\n");

        write_indent(out, indent + 2);
        out.push_str("\"vertices\": [");
        for (i, v) in self.vertices.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            write_f32(out, *v);
        }
        out.push_str("],\n");

        write_indent(out, indent + 2);
        out.push_str("\"triangles\": [");
        for (i, t) in self.triangles.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            write!(out, "{}", t).unwrap();
        }
        out.push_str("],\n");

        write_indent(out, indent + 2);
        out.push_str("\"normals\": [");
        for (i, n) in self.normals.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            write_f32(out, *n);
        }
        out.push_str("]\n");

        write_indent(out, indent);
        out.push('}');
    }
}

fn write_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

fn write_f32(out: &mut String, value: f32) {
    if value.fract() == 0.0 {
        write!(out, "{:.0}", value).unwrap();
    } else {
        let formatted = format!("{:.4}", value);
        let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
        out.push_str(trimmed);
    }
}

// ── Road mesh bundle (surface + sidewalks + curbs) ─────────────────────────

/// Constants matching Lua `RoadBuilder.lua`.
pub const CURB_THICKNESS: f64 = 0.35;
pub const CURB_SURFACE_LIFT: f64 = 0.45;
pub const PAVEMENT_SURFACE_LIFT: f64 = 0.25;
/// Default sidewalk strip thickness (same as main road slab).
const SIDEWALK_SLAB_THICKNESS: f64 = 0.2;
/// Default curb slab thickness (same as curb visual thickness).
const CURB_SLAB_THICKNESS: f64 = CURB_THICKNESS;

/// Pre-computed bundle of road mesh strips: main surface plus optional
/// sidewalk and curb strips on each side.
#[derive(Debug, Clone, PartialEq)]
pub struct RoadMeshBundle {
    /// Main road surface strip.
    pub surface: RoadMeshStrip,
    /// Left sidewalk strip (negative lateral offset from centreline).
    pub sidewalk_left: Option<RoadMeshStrip>,
    /// Right sidewalk strip (positive lateral offset from centreline).
    pub sidewalk_right: Option<RoadMeshStrip>,
    /// Left curb strip (negative lateral offset, between road edge and sidewalk).
    pub curb_left: Option<RoadMeshStrip>,
    /// Right curb strip (positive lateral offset, between road edge and sidewalk).
    pub curb_right: Option<RoadMeshStrip>,
}

/// Sidewalk mode parsed from the manifest `sidewalk` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidewalkMode {
    Both,
    Left,
    Right,
    No,
}

impl SidewalkMode {
    /// Parse from the canonical manifest string value.
    pub fn from_str_opt(s: Option<&str>) -> Self {
        match s {
            Some("both") => Self::Both,
            Some("left") => Self::Left,
            Some("right") => Self::Right,
            _ => Self::No,
        }
    }

    pub fn has_left(self) -> bool {
        matches!(self, Self::Both | Self::Left)
    }

    pub fn has_right(self) -> bool {
        matches!(self, Self::Both | Self::Right)
    }
}

/// Compute the sidewalk width in studs, matching the Lua `RoadProfile.getSidewalkWidth`.
///
/// Formula: `clamp(road_width * 0.25, 2.5, 4.0)`, but returns 0 when
/// `has_sidewalk` is false.
pub fn sidewalk_width(road_width: f64, has_sidewalk: bool) -> f64 {
    if !has_sidewalk {
        return 0.0;
    }
    (road_width * 0.25).clamp(2.5, 4.0)
}

/// Compute the edge buffer width in studs, matching `RoadProfile.getEdgeBufferWidth`.
pub fn edge_buffer_width(road_width: f64, has_sidewalk: bool) -> f64 {
    if sidewalk_width(road_width, has_sidewalk) > 0.0 {
        return 0.75;
    }
    if road_width >= 12.0 {
        return 0.75;
    }
    if road_width >= 8.0 {
        return 0.5;
    }
    0.25
}

/// Build the complete road mesh bundle: main surface plus optional sidewalk
/// and curb strips based on the sidewalk mode.
///
/// # Parameters
/// - `points` — ordered 3-D polyline of the road centreline (world-space studs).
/// - `road_width` — full road carriageway width in studs.
/// - `surface_lift` — vertical offset for the main road surface.
/// - `thickness` — vertical thickness of the main road slab.
/// - `has_sidewalk` — whether the road feature declares attached sidewalks.
/// - `sidewalk_mode` — which sides get sidewalk geometry.
pub fn build_road_bundle(
    points: &[(f64, f64, f64)],
    road_width: f64,
    surface_lift: f64,
    thickness: f64,
    has_sidewalk: bool,
    sidewalk_mode: SidewalkMode,
) -> RoadMeshBundle {
    // Main road surface — no lateral offset.
    let surface = build_road_strip(points, road_width, surface_lift, thickness, 0.0);

    let sw_width = sidewalk_width(road_width, has_sidewalk);
    let edge_buf = edge_buffer_width(road_width, has_sidewalk);
    let sw_strip_width = sw_width + edge_buf;
    let has_geometry = sw_width > 0.0;

    let make_left = has_geometry && sidewalk_mode.has_left();
    let make_right = has_geometry && sidewalk_mode.has_right();

    let sidewalk_left = if make_left {
        let offset = -(road_width * 0.5 + sw_strip_width * 0.5);
        Some(build_road_strip(
            points,
            sw_strip_width,
            PAVEMENT_SURFACE_LIFT,
            SIDEWALK_SLAB_THICKNESS,
            offset,
        ))
    } else {
        None
    };

    let sidewalk_right = if make_right {
        let offset = road_width * 0.5 + sw_strip_width * 0.5;
        Some(build_road_strip(
            points,
            sw_strip_width,
            PAVEMENT_SURFACE_LIFT,
            SIDEWALK_SLAB_THICKNESS,
            offset,
        ))
    } else {
        None
    };

    let curb_left = if make_left {
        let offset = -(road_width * 0.5 + CURB_THICKNESS * 0.5);
        Some(build_road_strip(
            points,
            CURB_THICKNESS,
            CURB_SURFACE_LIFT,
            CURB_SLAB_THICKNESS,
            offset,
        ))
    } else {
        None
    };

    let curb_right = if make_right {
        let offset = road_width * 0.5 + CURB_THICKNESS * 0.5;
        Some(build_road_strip(
            points,
            CURB_THICKNESS,
            CURB_SURFACE_LIFT,
            CURB_SLAB_THICKNESS,
            offset,
        ))
    } else {
        None
    };

    RoadMeshBundle {
        surface,
        sidewalk_left,
        sidewalk_right,
        curb_left,
        curb_right,
    }
}

impl RoadMeshBundle {
    /// Write this bundle as a JSON object.
    pub fn write_json(&self, out: &mut String, indent: usize) {
        write_indent(out, indent);
        out.push_str("{\n");

        write_indent(out, indent + 2);
        out.push_str("\"surface\": ");
        self.surface.write_json(out, 0);

        if let Some(ref sw) = self.sidewalk_left {
            out.push_str(",\n");
            write_indent(out, indent + 2);
            out.push_str("\"sidewalkLeft\": ");
            sw.write_json(out, 0);
        }

        if let Some(ref sw) = self.sidewalk_right {
            out.push_str(",\n");
            write_indent(out, indent + 2);
            out.push_str("\"sidewalkRight\": ");
            sw.write_json(out, 0);
        }

        if let Some(ref c) = self.curb_left {
            out.push_str(",\n");
            write_indent(out, indent + 2);
            out.push_str("\"curbLeft\": ");
            c.write_json(out, 0);
        }

        if let Some(ref c) = self.curb_right {
            out.push_str(",\n");
            write_indent(out, indent + 2);
            out.push_str("\"curbRight\": ");
            c.write_json(out, 0);
        }

        out.push('\n');
        write_indent(out, indent);
        out.push('}');
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_two_point_segment() {
        let points = vec![(0.0, 5.0, 0.0), (10.0, 5.0, 0.0)];
        let strip = build_road_strip(&points, 6.0, 0.15, 0.2, 0.0);

        // 1 segment => 6 faces => 24 vertices, 12 triangles
        assert_eq!(strip.vertex_count(), 24);
        assert_eq!(strip.triangle_count(), 12);
        assert_eq!(strip.normals.len(), strip.vertices.len());

        // All triangle indices should be in-bounds.
        let vc = strip.vertex_count() as u32;
        for &idx in &strip.triangles {
            assert!(idx < vc, "triangle index {} out of bounds (vc={})", idx, vc);
        }
    }

    #[test]
    fn straight_segment_top_normals_point_up() {
        let points = vec![(0.0, 0.0, 0.0), (10.0, 0.0, 0.0)];
        let strip = build_road_strip(&points, 4.0, 0.1, 0.2, 0.0);

        // First face is the top quad; its 4 vertices should have normal (0, 1, 0).
        for i in 0..4 {
            let nx = strip.normals[i * 3];
            let ny = strip.normals[i * 3 + 1];
            let nz = strip.normals[i * 3 + 2];
            assert!(
                (nx.abs() < 1e-6) && ((ny - 1.0).abs() < 1e-6) && (nz.abs() < 1e-6),
                "top normal at vertex {} = ({}, {}, {})",
                i,
                nx,
                ny,
                nz
            );
        }
    }

    #[test]
    fn bottom_normals_point_down() {
        let points = vec![(0.0, 0.0, 0.0), (10.0, 0.0, 0.0)];
        let strip = build_road_strip(&points, 4.0, 0.1, 0.2, 0.0);

        // Second face (indices 4..8) is bottom; normals should be (0, -1, 0).
        for i in 4..8 {
            let ny = strip.normals[i * 3 + 1];
            assert!(
                (ny + 1.0).abs() < 1e-6,
                "bottom normal Y at vertex {} = {}",
                i,
                ny
            );
        }
    }

    #[test]
    fn side_normals_are_horizontal() {
        let points = vec![(0.0, 0.0, 0.0), (10.0, 0.0, 0.0)];
        let strip = build_road_strip(&points, 4.0, 0.1, 0.2, 0.0);

        // Faces 3-6 (vertices 8..24) are the four side faces — all normals Y=0.
        for i in 8..24 {
            let ny = strip.normals[i * 3 + 1];
            assert!(
                ny.abs() < 1e-6,
                "side normal Y at vertex {} = {} (expected 0)",
                i,
                ny
            );
        }
    }

    #[test]
    fn three_point_curved_road() {
        // L-shaped road: two segments.
        let points = vec![
            (0.0, 0.0, 0.0),
            (10.0, 0.0, 0.0),
            (10.0, 0.0, 10.0),
        ];
        let strip = build_road_strip(&points, 4.0, 0.15, 0.2, 0.0);

        // 2 segments => 48 vertices, 24 triangles
        assert_eq!(strip.vertex_count(), 48);
        assert_eq!(strip.triangle_count(), 24);

        // All indices in bounds.
        let vc = strip.vertex_count() as u32;
        for &idx in &strip.triangles {
            assert!(idx < vc);
        }
    }

    #[test]
    fn degenerate_segment_skipped() {
        // Two identical points — should produce empty mesh.
        let points = vec![(5.0, 0.0, 5.0), (5.0, 0.0, 5.0)];
        let strip = build_road_strip(&points, 4.0, 0.1, 0.2, 0.0);
        assert_eq!(strip.vertex_count(), 0);
        assert_eq!(strip.triangle_count(), 0);
    }

    #[test]
    fn thickness_clamped_to_minimum() {
        let points = vec![(0.0, 0.0, 0.0), (10.0, 0.0, 0.0)];
        let strip = build_road_strip(&points, 4.0, 0.1, 0.01, 0.0); // below 0.05 minimum

        // Verify top/bottom Y separation is at least 0.05.
        // Top face vertex 0 Y vs bottom face vertex 4 Y.
        let top_y = strip.vertices[1]; // vertex 0, Y component
        let bot_y = strip.vertices[4 * 3 + 1]; // vertex 4, Y component
        let actual_thickness = top_y - bot_y;
        assert!(
            (actual_thickness - 0.05).abs() < 1e-4,
            "expected thickness ~0.05, got {}",
            actual_thickness
        );
    }

    #[test]
    fn json_round_trip_structure() {
        let points = vec![(0.0, 0.0, 0.0), (5.0, 0.0, 0.0)];
        let strip = build_road_strip(&points, 2.0, 0.1, 0.2, 0.0);

        let mut json = String::new();
        strip.write_json(&mut json, 0);

        // Should be valid-ish JSON with the three arrays.
        assert!(json.contains("\"vertices\""));
        assert!(json.contains("\"triangles\""));
        assert!(json.contains("\"normals\""));
    }

    // ── Lateral offset tests ───────────────────────────────────────────

    #[test]
    fn lateral_offset_shifts_vertices() {
        let points = vec![(0.0, 0.0, 0.0), (10.0, 0.0, 0.0)];
        let strip_center = build_road_strip(&points, 4.0, 0.1, 0.2, 0.0);
        let strip_right = build_road_strip(&points, 4.0, 0.1, 0.2, 5.0);

        // Road goes along X, so perp is Z.  Offset +5 should shift Z by +5.
        // Compare first vertex Z: center strip vs offset strip.
        let z_center = strip_center.vertices[2]; // vertex 0, Z
        let z_offset = strip_right.vertices[2];
        assert!(
            (z_offset - z_center - 5.0).abs() < 1e-4,
            "expected Z shift of +5, got {} vs {}",
            z_offset,
            z_center
        );
    }

    #[test]
    fn negative_lateral_offset_shifts_opposite() {
        let points = vec![(0.0, 0.0, 0.0), (10.0, 0.0, 0.0)];
        let strip_left = build_road_strip(&points, 4.0, 0.1, 0.2, -3.0);
        let strip_center = build_road_strip(&points, 4.0, 0.1, 0.2, 0.0);

        let z_center = strip_center.vertices[2];
        let z_left = strip_left.vertices[2];
        assert!(
            (z_left - z_center + 3.0).abs() < 1e-4,
            "expected Z shift of -3, got {} vs {}",
            z_left,
            z_center
        );
    }

    // ── Sidewalk mode tests ────────────────────────────────────────────

    #[test]
    fn sidewalk_mode_parsing() {
        assert_eq!(SidewalkMode::from_str_opt(Some("both")), SidewalkMode::Both);
        assert_eq!(SidewalkMode::from_str_opt(Some("left")), SidewalkMode::Left);
        assert_eq!(
            SidewalkMode::from_str_opt(Some("right")),
            SidewalkMode::Right
        );
        assert_eq!(SidewalkMode::from_str_opt(Some("no")), SidewalkMode::No);
        assert_eq!(
            SidewalkMode::from_str_opt(Some("separate")),
            SidewalkMode::No
        );
        assert_eq!(SidewalkMode::from_str_opt(None), SidewalkMode::No);
    }

    #[test]
    fn sidewalk_width_formula() {
        // width=8 => 0.25*8=2.0 => clamped to 2.5
        assert!((sidewalk_width(8.0, true) - 2.5).abs() < 1e-6);
        // width=12 => 0.25*12=3.0 => in range
        assert!((sidewalk_width(12.0, true) - 3.0).abs() < 1e-6);
        // width=20 => 0.25*20=5.0 => clamped to 4.0
        assert!((sidewalk_width(20.0, true) - 4.0).abs() < 1e-6);
        // no sidewalk => 0
        assert!((sidewalk_width(12.0, false)).abs() < 1e-6);
    }

    #[test]
    fn edge_buffer_width_formula() {
        // has sidewalk => always 0.75
        assert!((edge_buffer_width(8.0, true) - 0.75).abs() < 1e-6);
        // no sidewalk, wide road => 0.75
        assert!((edge_buffer_width(14.0, false) - 0.75).abs() < 1e-6);
        // no sidewalk, medium road => 0.5
        assert!((edge_buffer_width(10.0, false) - 0.5).abs() < 1e-6);
        // no sidewalk, narrow road => 0.25
        assert!((edge_buffer_width(6.0, false) - 0.25).abs() < 1e-6);
    }

    // ── Bundle tests ───────────────────────────────────────────────────

    #[test]
    fn bundle_no_sidewalks() {
        let points = vec![(0.0, 0.0, 0.0), (10.0, 0.0, 0.0)];
        let bundle = build_road_bundle(&points, 8.0, 0.2, 0.2, false, SidewalkMode::No);

        assert!(bundle.surface.vertex_count() > 0);
        assert!(bundle.sidewalk_left.is_none());
        assert!(bundle.sidewalk_right.is_none());
        assert!(bundle.curb_left.is_none());
        assert!(bundle.curb_right.is_none());
    }

    #[test]
    fn bundle_both_sidewalks() {
        let points = vec![(0.0, 0.0, 0.0), (10.0, 0.0, 0.0)];
        let bundle = build_road_bundle(&points, 12.0, 0.2, 0.2, true, SidewalkMode::Both);

        assert!(bundle.surface.vertex_count() > 0);
        assert!(bundle.sidewalk_left.is_some());
        assert!(bundle.sidewalk_right.is_some());
        assert!(bundle.curb_left.is_some());
        assert!(bundle.curb_right.is_some());

        // Sidewalk and curb should each have proper geometry.
        let sw_left = bundle.sidewalk_left.unwrap();
        assert_eq!(sw_left.vertex_count(), 24); // 1 segment
        let curb_left = bundle.curb_left.unwrap();
        assert_eq!(curb_left.vertex_count(), 24);
    }

    #[test]
    fn bundle_left_only() {
        let points = vec![(0.0, 0.0, 0.0), (10.0, 0.0, 0.0)];
        let bundle = build_road_bundle(&points, 12.0, 0.2, 0.2, true, SidewalkMode::Left);

        assert!(bundle.sidewalk_left.is_some());
        assert!(bundle.sidewalk_right.is_none());
        assert!(bundle.curb_left.is_some());
        assert!(bundle.curb_right.is_none());
    }

    #[test]
    fn bundle_right_only() {
        let points = vec![(0.0, 0.0, 0.0), (10.0, 0.0, 0.0)];
        let bundle = build_road_bundle(&points, 12.0, 0.2, 0.2, true, SidewalkMode::Right);

        assert!(bundle.sidewalk_left.is_none());
        assert!(bundle.sidewalk_right.is_some());
        assert!(bundle.curb_left.is_none());
        assert!(bundle.curb_right.is_some());
    }

    #[test]
    fn bundle_json_surface_only() {
        let points = vec![(0.0, 0.0, 0.0), (5.0, 0.0, 0.0)];
        let bundle = build_road_bundle(&points, 4.0, 0.1, 0.2, false, SidewalkMode::No);

        let mut json = String::new();
        bundle.write_json(&mut json, 0);

        assert!(json.contains("\"surface\""));
        assert!(!json.contains("\"sidewalkLeft\""));
        assert!(!json.contains("\"curbLeft\""));
    }

    #[test]
    fn bundle_json_with_sidewalks() {
        let points = vec![(0.0, 0.0, 0.0), (5.0, 0.0, 0.0)];
        let bundle = build_road_bundle(&points, 12.0, 0.2, 0.2, true, SidewalkMode::Both);

        let mut json = String::new();
        bundle.write_json(&mut json, 0);

        assert!(json.contains("\"surface\""));
        assert!(json.contains("\"sidewalkLeft\""));
        assert!(json.contains("\"sidewalkRight\""));
        assert!(json.contains("\"curbLeft\""));
        assert!(json.contains("\"curbRight\""));
    }

    #[test]
    fn bundle_has_sidewalk_false_overrides_mode() {
        // Even with mode=Both, if has_sidewalk is false, no geometry generated.
        let points = vec![(0.0, 0.0, 0.0), (10.0, 0.0, 0.0)];
        let bundle = build_road_bundle(&points, 12.0, 0.2, 0.2, false, SidewalkMode::Both);

        assert!(bundle.sidewalk_left.is_none());
        assert!(bundle.sidewalk_right.is_none());
        assert!(bundle.curb_left.is_none());
        assert!(bundle.curb_right.is_none());
    }
}
