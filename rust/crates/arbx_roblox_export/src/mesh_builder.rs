//! Compile-time building shell mesh generation.
//!
//! Translates the runtime `addOrientedBox` wall + flat-roof geometry from
//! `BuildingBuilder.lua` into pre-computed vertex/triangle/normal arrays that
//! the Lua importer can load directly instead of generating at import time.

/// Pre-computed mesh data: flat interleaved arrays matching the Roblox
/// EditableMesh vertex/triangle layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PrecomputedMesh {
    /// Flat [x,y,z, x,y,z, ...] vertex positions.
    pub vertices: Vec<f32>,
    /// Flat [v0,v1,v2, v0,v1,v2, ...] triangle indices (0-based).
    pub triangles: Vec<u32>,
    /// Flat [nx,ny,nz, nx,ny,nz, ...] per-vertex normals.
    pub normals: Vec<f32>,
}

impl PrecomputedMesh {
    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / 3
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len() / 3
    }
}

// ---------------------------------------------------------------------------
// Linear-algebra helpers (no external dependency needed for this geometry)
// ---------------------------------------------------------------------------

type V3 = [f64; 3];

fn v3_sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn v3_add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn v3_scale(v: V3, s: f64) -> V3 {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)] // Reserved for future explicit normal computation
fn v3_cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[allow(dead_code)]
fn v3_normalize(v: V3) -> V3 {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        return [0.0, 1.0, 0.0]; // degenerate fallback
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

#[allow(dead_code)]
fn v3_length(v: V3) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

// ---------------------------------------------------------------------------
// Mesh accumulator (mirrors the Lua MeshAccumulator, but writes into flat vecs)
// ---------------------------------------------------------------------------

struct MeshAccum {
    vertices: Vec<f32>,
    normals: Vec<f32>,
    triangles: Vec<u32>,
}

impl MeshAccum {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            triangles: Vec::new(),
        }
    }

    fn vertex_count(&self) -> u32 {
        (self.vertices.len() / 3) as u32
    }

    fn push_vertex(&mut self, pos: V3, normal: V3) -> u32 {
        let idx = self.vertex_count();
        self.vertices.push(pos[0] as f32);
        self.vertices.push(pos[1] as f32);
        self.vertices.push(pos[2] as f32);
        self.normals.push(normal[0] as f32);
        self.normals.push(normal[1] as f32);
        self.normals.push(normal[2] as f32);
        idx
    }

    /// Add a quad (4 vertices, 2 triangles). Winding matches the Lua version:
    /// tri1 = {p1, p2, p3}, tri2 = {p1, p3, p4}.
    fn add_quad(&mut self, p1: V3, p2: V3, p3: V3, p4: V3, normal: V3) {
        let v0 = self.push_vertex(p1, normal);
        let v1 = self.push_vertex(p2, normal);
        let v2 = self.push_vertex(p3, normal);
        let v3 = self.push_vertex(p4, normal);
        self.triangles.extend_from_slice(&[v0, v1, v2, v0, v2, v3]);
    }

    /// Add an oriented box (6 faces, 24 vertices, 12 triangles).
    /// Mirrors `addOrientedBox` in BuildingBuilder.lua exactly.
    fn add_oriented_box(
        &mut self,
        center: V3,
        right_axis: V3,
        up_axis: V3,
        forward_axis: V3,
        half_size: V3, // [half_x, half_y, half_z]
    ) {
        let right = v3_scale(right_axis, half_size[0]);
        let up = v3_scale(up_axis, half_size[1]);
        let forward = v3_scale(forward_axis, half_size[2]);

        // 8 corners
        let lbb = v3_sub(v3_sub(v3_sub(center, right), up), forward);
        let lbf = v3_add(v3_sub(v3_sub(center, right), up), forward);
        let ltb = v3_add(v3_sub(v3_sub(center, right), forward), up);
        let ltf = v3_add(v3_sub(v3_add(center, forward), right), up);
        let rbb = v3_sub(v3_add(v3_sub(center, up), right), forward);
        let rbf = v3_add(v3_add(v3_sub(center, up), right), forward);
        let rtb = v3_sub(v3_add(v3_add(center, right), up), forward);
        let rtf = v3_add(v3_add(v3_add(center, right), up), forward);

        // 6 faces — same winding order as Lua addOrientedBox
        let neg_forward = v3_scale(forward_axis, -1.0);
        let neg_right = v3_scale(right_axis, -1.0);
        let neg_up = v3_scale(up_axis, -1.0);

        self.add_quad(lbf, rbf, rtf, ltf, forward_axis);   // +forward
        self.add_quad(rbb, lbb, ltb, rtb, neg_forward);     // -forward
        self.add_quad(rbf, rbb, rtb, rtf, right_axis);      // +right
        self.add_quad(lbb, lbf, ltf, ltb, neg_right);       // -right
        self.add_quad(ltf, rtf, rtb, ltb, up_axis);         // +up (top)
        self.add_quad(lbb, rbb, rbf, lbf, neg_up);          // -up (bottom)
    }

    fn into_mesh(self) -> PrecomputedMesh {
        PrecomputedMesh {
            vertices: self.vertices,
            triangles: self.triangles,
            normals: self.normals,
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

const MIN_EDGE: f64 = 0.5;
const DEFAULT_WALL_THICKNESS: f64 = 0.6;

/// Build the exterior shell mesh for a building: oriented-box walls for each
/// footprint edge plus a flat roof slab.
///
/// Mirrors the runtime geometry path in `BuildingBuilder.lua`
/// (`addOrientedBox` per wall segment + roof grid).
///
/// # Arguments
/// * `footprint` — 2D polygon points as (x, z) pairs in stud space.
/// * `base_y` — Y coordinate of the building base (ground level).
/// * `height` — Total building height in studs.
/// * `wall_thickness` — Wall thickness in studs (use 0.0 for default 0.6).
pub fn build_shell_mesh(
    footprint: &[(f64, f64)],
    base_y: f64,
    height: f64,
    wall_thickness: f64,
) -> PrecomputedMesh {
    let wall_t = if wall_thickness <= 0.0 {
        DEFAULT_WALL_THICKNESS
    } else {
        wall_thickness
    };

    let mut acc = MeshAccum::new();
    let n = footprint.len();
    if n < 3 || height <= 0.0 {
        return acc.into_mesh();
    }

    // ---- Walls ----
    // For each consecutive edge of the footprint polygon, create an oriented
    // box that represents a wall segment (same as the Lua runtime path).
    let up: V3 = [0.0, 1.0, 0.0];
    for i in 0..n {
        let j = (i + 1) % n;
        let (ax, az) = footprint[i];
        let (bx, bz) = footprint[j];
        let dx = bx - ax;
        let dz = bz - az;
        let edge_len = (dx * dx + dz * dz).sqrt();
        if edge_len < MIN_EDGE {
            continue;
        }

        // Forward = along the edge, Right = perpendicular (outward normal), Up = Y
        let forward: V3 = [dx / edge_len, 0.0, dz / edge_len];
        let right: V3 = [dz / edge_len, 0.0, -dx / edge_len];

        let mid_x = (ax + bx) * 0.5;
        let mid_z = (az + bz) * 0.5;
        let center_y = base_y + height * 0.5;
        let center: V3 = [mid_x, center_y, mid_z];

        // Z-extent includes wall_t overlap on each end to match Lua's
        // addOrientedBox(... Vector3.new(WALL_THICKNESS, height, edgeLen + WALL_THICKNESS))
        let half_size: V3 = [wall_t * 0.5, height * 0.5, (edge_len + wall_t) * 0.5];

        acc.add_oriented_box(center, right, up, forward, half_size);
    }

    // NOTE: Roof geometry is NOT included in the pre-computed shell mesh.
    // Lua `buildRoof()` owns all roof shapes (flat, gabled, hipped, gambrel)
    // with material selection, readability cues, and detail geometry.
    // Including a flat roof slab here would cause double-roof rendering.

    acc.into_mesh()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A simple axis-aligned rectangular building: 10×10 footprint at origin.
    fn rect_footprint() -> Vec<(f64, f64)> {
        vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
        ]
    }

    #[test]
    fn rect_building_vertex_count() {
        let mesh = build_shell_mesh(&rect_footprint(), 0.0, 12.0, 0.6);
        // 4 wall boxes × 24 verts = 96 vertices (no roof — Lua owns roofs)
        assert_eq!(mesh.vertex_count(), 96, "expected 4 wall boxes × 24 verts");
    }

    #[test]
    fn rect_building_triangle_count() {
        let mesh = build_shell_mesh(&rect_footprint(), 0.0, 12.0, 0.6);
        // 4 wall boxes × 12 triangles = 48 triangles (no roof)
        assert_eq!(mesh.triangle_count(), 48, "expected 4 wall boxes × 12 tris");
    }

    #[test]
    fn normals_are_unit_length() {
        let mesh = build_shell_mesh(&rect_footprint(), 0.0, 12.0, 0.6);
        for i in 0..mesh.vertex_count() {
            let nx = mesh.normals[i * 3] as f64;
            let ny = mesh.normals[i * 3 + 1] as f64;
            let nz = mesh.normals[i * 3 + 2] as f64;
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            assert!(
                (len - 1.0).abs() < 1e-4,
                "normal at vertex {} has length {}, expected 1.0",
                i,
                len
            );
        }
    }

    #[test]
    fn triangles_reference_valid_vertices() {
        let mesh = build_shell_mesh(&rect_footprint(), 0.0, 12.0, 0.6);
        let vc = mesh.vertex_count() as u32;
        for (ti, &idx) in mesh.triangles.iter().enumerate() {
            assert!(
                idx < vc,
                "triangle index [{}] = {} exceeds vertex count {}",
                ti,
                idx,
                vc
            );
        }
    }

    #[test]
    fn vertices_and_normals_same_count() {
        let mesh = build_shell_mesh(&rect_footprint(), 0.0, 12.0, 0.6);
        assert_eq!(
            mesh.vertices.len(),
            mesh.normals.len(),
            "vertex and normal arrays must have the same length"
        );
    }

    #[test]
    fn degenerate_footprint_returns_empty() {
        // < 3 points
        let mesh = build_shell_mesh(&[(0.0, 0.0), (1.0, 1.0)], 0.0, 10.0, 0.6);
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.triangle_count(), 0);
    }

    #[test]
    fn zero_height_returns_empty() {
        let mesh = build_shell_mesh(&rect_footprint(), 0.0, 0.0, 0.6);
        assert_eq!(mesh.vertex_count(), 0);
    }

    #[test]
    fn short_edges_are_skipped() {
        // Triangle with one very short edge (< MIN_EDGE = 0.5)
        let fp = vec![
            (0.0, 0.0),
            (0.1, 0.0), // edge length 0.1 — should be skipped
            (5.0, 5.0),
        ];
        let mesh = build_shell_mesh(&fp, 0.0, 10.0, 0.6);
        // 2 valid edges × 24 verts = 48 verts (no roof)
        assert_eq!(mesh.vertex_count(), 48);
    }

    #[test]
    fn default_wall_thickness_applied() {
        let m1 = build_shell_mesh(&rect_footprint(), 0.0, 10.0, 0.0);
        let m2 = build_shell_mesh(&rect_footprint(), 0.0, 10.0, 0.6);
        assert_eq!(m1.vertices, m2.vertices, "wall_thickness=0 should use default 0.6");
    }

    #[test]
    fn base_y_offset_applied() {
        let mesh = build_shell_mesh(&rect_footprint(), 5.0, 10.0, 0.6);
        // All Y coordinates should be >= base_y (5.0) minus half wall thickness
        for i in 0..mesh.vertex_count() {
            let y = mesh.vertices[i * 3 + 1];
            // Wall centers at base_y + height/2 = 10, half_height = 5
            // Lowest vertex is at base_y + height/2 - height/2 = base_y = 5.0
            // Highest vertex is at base_y + height = 15.0 (walls only, no roof)
            assert!(
                y >= 4.99, // tiny float tolerance
                "vertex {} Y={} is below base_y=5.0",
                i,
                y
            );
        }
    }

    #[test]
    fn pentagon_footprint() {
        let fp = vec![
            (0.0, 0.0),
            (5.0, 0.0),
            (7.0, 4.0),
            (3.5, 7.0),
            (-1.0, 4.0),
        ];
        let mesh = build_shell_mesh(&fp, 0.0, 8.0, 0.6);
        // 5 wall edges × 24 verts = 120 verts (no roof)
        assert_eq!(mesh.vertex_count(), 120); // 5 × 24
        assert_eq!(mesh.triangle_count(), 60); // 5 × 12
    }

    /// Build a shell mesh, embed it in a BuildingShell inside a Chunk,
    /// serialise to JSON via `Chunk::to_json_pretty`, and verify the
    /// shellMesh vertices/triangles/normals arrays survive the round-trip
    /// with correct element counts.
    #[test]
    fn shell_mesh_json_round_trip_via_chunk() {
        use crate::manifest::{BuildingShell, Chunk, GroundPoint};
        use arbx_geo::{ChunkId, Vec3};

        let footprint = rect_footprint();
        let mesh = build_shell_mesh(&footprint, 0.0, 12.0, 0.6);

        // Capture expected counts before moving mesh into the shell.
        // 4 wall boxes × 24 verts = 96 verts (no roof — Lua owns roofs)
        let expected_vert_floats = mesh.vertices.len();   // 96 verts × 3 = 288
        let expected_tri_indices = mesh.triangles.len();   // 48 tris × 3  = 144
        let expected_norm_floats = mesh.normals.len();     // same as vertices = 288

        let shell = BuildingShell {
            id: "test-bldg".into(),
            footprint: footprint.iter().map(|&(x, z)| GroundPoint::new(x, z)).collect(),
            holes: vec![],
            indices: None,
            material: "SmoothPlastic".into(),
            wall_color: None,
            roof_color: None,
            roof_shape: None,
            roof_material: None,
            usage: None,
            min_height: None,
            base_y: 0.0,
            height: 12.0,
            height_m: None,
            levels: None,
            roof_levels: None,
            roof: "flat".into(),
            facade_style: None,
            structure_type: None,
            rooms: vec![],
            roof_height: None,
            roof_direction: None,
            roof_angle: None,
            name: None,
            shell_mesh: Some(mesh),
            atlas_uv: None,
        };

        let chunk = Chunk {
            id: ChunkId::new(0, 0),
            origin_studs: Vec3::new(0.0, 0.0, 0.0),
            terrain: None,
            terrain_texture_path: None,
            terrain_texture_rgba_path: None,
            roads: vec![],
            rails: vec![],
            buildings: vec![shell],
            water: vec![],
            props: vec![],
            landuse: vec![],
            barriers: vec![],
            building_atlas: None,
        };

        let json = chunk.to_json_pretty();

        // ── Verify the shellMesh key exists ──
        assert!(
            json.contains("\"shellMesh\""),
            "JSON must contain shellMesh key"
        );

        // ── Extract the shellMesh object (everything between its opening { and
        //    matching closing }) and count array elements. ──
        let sm_start = json.find("\"shellMesh\"").unwrap();
        let obj_start = json[sm_start..].find('{').unwrap() + sm_start;

        // Find the matching closing brace (simple depth counter).
        let mut depth = 0u32;
        let mut obj_end = obj_start;
        for (i, ch) in json[obj_start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        obj_end = obj_start + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        let mesh_json = &json[obj_start..=obj_end];

        // Helper: count comma-separated elements inside a JSON array value.
        let count_elements = |key: &str| -> usize {
            let key_str = format!("\"{}\"", key);
            let kpos = mesh_json.find(&key_str).unwrap_or_else(|| {
                panic!("shellMesh JSON missing key '{}'", key);
            });
            let arr_open = mesh_json[kpos..].find('[').unwrap() + kpos;
            let arr_close = mesh_json[arr_open..].find(']').unwrap() + arr_open;
            let inner = mesh_json[arr_open + 1..arr_close].trim();
            if inner.is_empty() {
                0
            } else {
                inner.split(',').count()
            }
        };

        let vert_count = count_elements("vertices");
        let tri_count = count_elements("triangles");
        let norm_count = count_elements("normals");

        assert_eq!(
            vert_count, expected_vert_floats,
            "vertices array element count mismatch (expected {} floats)",
            expected_vert_floats
        );
        assert_eq!(
            tri_count, expected_tri_indices,
            "triangles array element count mismatch (expected {} indices)",
            expected_tri_indices
        );
        assert_eq!(
            norm_count, expected_norm_floats,
            "normals array element count mismatch (expected {} floats)",
            expected_norm_floats
        );
    }
}
