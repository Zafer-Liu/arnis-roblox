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
pub fn build_road_strip(
    points: &[(f64, f64, f64)],
    width: f64,
    surface_lift: f64,
    thickness: f64,
) -> RoadMeshStrip {
    let mut vertices: Vec<f32> = Vec::new();
    let mut triangles: Vec<u32> = Vec::new();
    let mut normals: Vec<f32> = Vec::new();

    let thickness = thickness.max(0.05);

    for window in points.windows(2) {
        let (p1, p2) = (window[0], window[1]);

        let dx = p2.0 - p1.0;
        let dz = p2.2 - p1.2;
        let horiz_len = (dx * dx + dz * dz).sqrt();
        if horiz_len < 0.01 {
            continue;
        }

        // Normalised horizontal direction and perpendicular (flat).
        let hx = dx / horiz_len;
        let hz = dz / horiz_len;
        let px = -hz; // perpendicular X
        let pz = hx; // perpendicular Z

        let half_w = width * 0.5;

        // Top surface vertices — follow per-endpoint Y.
        let y1 = p1.1 + surface_lift;
        let y2 = p2.1 + surface_lift;

        // v1..v4 = top quad (near-left, near-right, far-right, far-left)
        let v1 = (p1.0 - px * half_w, y1, p1.2 - pz * half_w);
        let v2 = (p1.0 + px * half_w, y1, p1.2 + pz * half_w);
        let v3 = (p2.0 + px * half_w, y2, p2.2 + pz * half_w);
        let v4 = (p2.0 - px * half_w, y2, p2.2 - pz * half_w);

        // b1..b4 = bottom quad
        let b1 = (v1.0, v1.1 - thickness, v1.2);
        let b2 = (v2.0, v2.1 - thickness, v2.2);
        let b3 = (v3.0, v3.1 - thickness, v3.2);
        let b4 = (v4.0, v4.1 - thickness, v4.2);

        let base = (vertices.len() / 3) as u32;

        // Push all 8 vertices.
        for &(vx, vy, vz) in &[v1, v2, v3, v4, b1, b2, b3, b4] {
            vertices.push(vx as f32);
            vertices.push(vy as f32);
            vertices.push(vz as f32);
        }

        // Normals per face — 6 quads, each with 4 vertex normals.
        // But we share vertices across faces, so we store per-vertex normals
        // matching the 8 unique positions.  For a box the dominant normal per
        // vertex is the average of its adjacent face normals; for runtime
        // rendering the Lua path uses flat per-face normals via addQuad.
        // We replicate the flat-face approach: emit *separate* vertices per
        // face so each face gets its own normal (24 verts / segment).

        // Actually, to match the Lua path faithfully (which uses per-quad
        // vertices with flat normals), we need 4 verts per face * 6 faces =
        // 24 vertices per segment.  Let's redo with that approach.
        vertices.clear();
        normals.clear();
        triangles.clear();
        // Restart accumulation with per-face vertices.
        break; // will redo below
    }

    // --- Per-face vertex approach (matches Lua addQuad calls) ---
    vertices.clear();
    normals.clear();
    triangles.clear();

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

        let v1 = (p1.0 - px * half_w, y1, p1.2 - pz * half_w);
        let v2 = (p1.0 + px * half_w, y1, p1.2 + pz * half_w);
        let v3 = (p2.0 + px * half_w, y2, p2.2 + pz * half_w);
        let v4 = (p2.0 - px * half_w, y2, p2.2 - pz * half_w);

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

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_two_point_segment() {
        let points = vec![(0.0, 5.0, 0.0), (10.0, 5.0, 0.0)];
        let strip = build_road_strip(&points, 6.0, 0.15, 0.2);

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
        let strip = build_road_strip(&points, 4.0, 0.1, 0.2);

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
        let strip = build_road_strip(&points, 4.0, 0.1, 0.2);

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
        let strip = build_road_strip(&points, 4.0, 0.1, 0.2);

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
        let strip = build_road_strip(&points, 4.0, 0.15, 0.2);

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
        let strip = build_road_strip(&points, 4.0, 0.1, 0.2);
        assert_eq!(strip.vertex_count(), 0);
        assert_eq!(strip.triangle_count(), 0);
    }

    #[test]
    fn thickness_clamped_to_minimum() {
        let points = vec![(0.0, 0.0, 0.0), (10.0, 0.0, 0.0)];
        let strip = build_road_strip(&points, 4.0, 0.1, 0.01); // below 0.05 minimum

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
        let strip = build_road_strip(&points, 2.0, 0.1, 0.2);

        let mut json = String::new();
        strip.write_json(&mut json, 0);

        // Should be valid-ish JSON with the three arrays.
        assert!(json.contains("\"vertices\""));
        assert!(json.contains("\"triangles\""));
        assert!(json.contains("\"normals\""));
    }
}
