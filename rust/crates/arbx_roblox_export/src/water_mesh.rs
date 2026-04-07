//! Compile-time water body mesh generation.
//!
//! Translates the runtime water surface geometry from `WaterBuilder.lua` into
//! pre-computed vertex/triangle/normal arrays that the Lua importer can load
//! directly instead of generating at import time.

use std::fmt::Write as _;

/// Pre-computed water mesh: flat interleaved arrays matching the Roblox
/// EditableMesh vertex/triangle layout (same layout as `PrecomputedMesh`,
/// `RoadMeshStrip`, and `PropMesh`).
#[derive(Debug, Clone, PartialEq)]
pub struct WaterMesh {
    /// Flat [x,y,z, x,y,z, ...] vertex positions.
    pub vertices: Vec<f32>,
    /// Flat [v0,v1,v2, v0,v1,v2, ...] triangle indices (0-based).
    pub triangles: Vec<u32>,
    /// Flat [nx,ny,nz, nx,ny,nz, ...] per-vertex normals.
    pub normals: Vec<f32>,
}

impl WaterMesh {
    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / 3
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len() / 3
    }
}

// ---------------------------------------------------------------------------
// Mesh accumulator (same pattern as mesh_builder.rs / prop_mesh.rs)
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

    fn push_vertex(&mut self, pos: [f64; 3], normal: [f64; 3]) -> u32 {
        let idx = self.vertex_count();
        self.vertices.push(pos[0] as f32);
        self.vertices.push(pos[1] as f32);
        self.vertices.push(pos[2] as f32);
        self.normals.push(normal[0] as f32);
        self.normals.push(normal[1] as f32);
        self.normals.push(normal[2] as f32);
        idx
    }

    /// Add a triangle with a shared flat normal.
    fn add_triangle(&mut self, p1: [f64; 3], p2: [f64; 3], p3: [f64; 3], normal: [f64; 3]) {
        let v0 = self.push_vertex(p1, normal);
        let v1 = self.push_vertex(p2, normal);
        let v2 = self.push_vertex(p3, normal);
        self.triangles.extend_from_slice(&[v0, v1, v2]);
    }

    /// Add a quad (4 vertices, 2 triangles). Winding: tri1 = {p1, p2, p3},
    /// tri2 = {p1, p3, p4}.
    fn add_quad(
        &mut self,
        p1: [f64; 3],
        p2: [f64; 3],
        p3: [f64; 3],
        p4: [f64; 3],
        normal: [f64; 3],
    ) {
        let v0 = self.push_vertex(p1, normal);
        let v1 = self.push_vertex(p2, normal);
        let v2 = self.push_vertex(p3, normal);
        let v3 = self.push_vertex(p4, normal);
        self.triangles.extend_from_slice(&[v0, v1, v2, v0, v2, v3]);
    }

    fn into_mesh(self) -> WaterMesh {
        WaterMesh {
            vertices: self.vertices,
            triangles: self.triangles,
            normals: self.normals,
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Water surface normal — always points straight up.
const UP: [f64; 3] = [0.0, 1.0, 0.0];

/// Build a flat water surface mesh from a polygon footprint using fan
/// triangulation from the centroid.
///
/// All vertices are placed at `surface_y` height. Normals point up (0, 1, 0).
///
/// # Arguments
/// * `footprint` — 2D polygon points as (x, z) pairs in stud space.
/// * `surface_y` — Y coordinate of the water surface.
pub fn build_water_polygon_mesh(footprint: &[(f64, f64)], surface_y: f64) -> WaterMesh {
    let mut acc = MeshAccum::new();
    let n = footprint.len();
    if n < 3 {
        return acc.into_mesh();
    }

    // Compute centroid.
    let mut cx = 0.0;
    let mut cz = 0.0;
    for &(x, z) in footprint {
        cx += x;
        cz += z;
    }
    cx /= n as f64;
    cz /= n as f64;

    let center = [cx, surface_y, cz];

    // Fan triangulation: for each consecutive edge, create a triangle from the
    // centroid to that edge.
    for i in 0..n {
        let j = (i + 1) % n;
        let (ax, az) = footprint[i];
        let (bx, bz) = footprint[j];
        let a = [ax, surface_y, az];
        let b = [bx, surface_y, bz];
        acc.add_triangle(center, a, b, UP);
    }

    acc.into_mesh()
}

/// Build a ribbon-strip water surface mesh along a polyline (rivers, streams).
///
/// Generates only the top face (single-sided) — no bottom or sides, since water
/// surfaces don't need thickness. Uses the same ribbon approach as `road_mesh`
/// but simplified.
///
/// # Arguments
/// * `points` — ordered 3-D polyline of the water centreline (stud space).
/// * `width` — full water width in studs.
/// * `surface_y` — Y coordinate of the water surface. If `Some`, overrides the
///   Y from each point; if `None`, uses the Y coordinate from each polyline
///   point directly.
pub fn build_water_river_mesh(
    points: &[(f64, f64, f64)],
    width: f64,
    surface_y: Option<f64>,
) -> WaterMesh {
    let mut acc = MeshAccum::new();

    if points.len() < 2 || width <= 0.0 {
        return acc.into_mesh();
    }

    let half_w = width * 0.5;

    for window in points.windows(2) {
        let (p1, p2) = (window[0], window[1]);

        let dx = p2.0 - p1.0;
        let dz = p2.2 - p1.2;
        let horiz_len = (dx * dx + dz * dz).sqrt();
        if horiz_len < 0.01 {
            continue;
        }

        // Perpendicular direction (left-hand normal in XZ plane).
        let px = -dz / horiz_len;
        let pz = dx / horiz_len;

        let y1 = surface_y.unwrap_or(p1.1);
        let y2 = surface_y.unwrap_or(p2.1);

        // Four corners of the ribbon segment (top face only).
        let v1 = [p1.0 - px * half_w, y1, p1.2 - pz * half_w];
        let v2 = [p1.0 + px * half_w, y1, p1.2 + pz * half_w];
        let v3 = [p2.0 + px * half_w, y2, p2.2 + pz * half_w];
        let v4 = [p2.0 - px * half_w, y2, p2.2 - pz * half_w];

        acc.add_quad(v1, v2, v3, v4, UP);
    }

    acc.into_mesh()
}

// ---------------------------------------------------------------------------
// JSON serialisation (same pattern as RoadMeshStrip::write_json / PropMesh::write_json)
// ---------------------------------------------------------------------------

impl WaterMesh {
    /// Write this mesh as a JSON object into the manifest serialiser.
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── Polygon mesh tests ──────────────────────────────────────────────

    fn square_footprint() -> Vec<(f64, f64)> {
        vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
        ]
    }

    #[test]
    fn polygon_square_vertex_count() {
        let mesh = build_water_polygon_mesh(&square_footprint(), 5.0);
        // 4 edges × 1 triangle each × 3 verts per tri = 12 vertices
        assert_eq!(mesh.vertex_count(), 12);
    }

    #[test]
    fn polygon_square_triangle_count() {
        let mesh = build_water_polygon_mesh(&square_footprint(), 5.0);
        // 4 fan triangles
        assert_eq!(mesh.triangle_count(), 4);
    }

    #[test]
    fn polygon_all_normals_point_up() {
        let mesh = build_water_polygon_mesh(&square_footprint(), 5.0);
        for i in 0..mesh.vertex_count() {
            let nx = mesh.normals[i * 3];
            let ny = mesh.normals[i * 3 + 1];
            let nz = mesh.normals[i * 3 + 2];
            assert_eq!((nx, ny, nz), (0.0, 1.0, 0.0), "normal at vertex {} must point up", i);
        }
    }

    #[test]
    fn polygon_all_y_at_surface() {
        let surface_y = 7.5;
        let mesh = build_water_polygon_mesh(&square_footprint(), surface_y);
        for i in 0..mesh.vertex_count() {
            let y = mesh.vertices[i * 3 + 1];
            assert!(
                (y - surface_y as f32).abs() < 1e-4,
                "vertex {} Y={} expected {}",
                i,
                y,
                surface_y
            );
        }
    }

    #[test]
    fn polygon_triangles_reference_valid_vertices() {
        let mesh = build_water_polygon_mesh(&square_footprint(), 5.0);
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
    fn polygon_vertices_normals_same_length() {
        let mesh = build_water_polygon_mesh(&square_footprint(), 5.0);
        assert_eq!(mesh.vertices.len(), mesh.normals.len());
    }

    #[test]
    fn polygon_pentagon() {
        let fp = vec![
            (0.0, 0.0),
            (5.0, 0.0),
            (7.0, 4.0),
            (3.5, 7.0),
            (-1.0, 4.0),
        ];
        let mesh = build_water_polygon_mesh(&fp, 3.0);
        // 5 fan triangles × 3 verts = 15 vertices
        assert_eq!(mesh.vertex_count(), 15);
        assert_eq!(mesh.triangle_count(), 5);
    }

    #[test]
    fn polygon_degenerate_returns_empty() {
        // < 3 points
        let mesh = build_water_polygon_mesh(&[(0.0, 0.0), (1.0, 1.0)], 5.0);
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.triangle_count(), 0);
    }

    #[test]
    fn polygon_empty_returns_empty() {
        let mesh = build_water_polygon_mesh(&[], 5.0);
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.triangle_count(), 0);
    }

    // ── River mesh tests ────────────────────────────────────────────────

    fn straight_river_points() -> Vec<(f64, f64, f64)> {
        vec![
            (0.0, 0.0, 0.0),
            (0.0, 0.0, 20.0),
        ]
    }

    #[test]
    fn river_straight_vertex_count() {
        let mesh = build_water_river_mesh(&straight_river_points(), 6.0, Some(5.0));
        // 1 segment × 1 quad × 4 verts = 4 vertices
        assert_eq!(mesh.vertex_count(), 4);
    }

    #[test]
    fn river_straight_triangle_count() {
        let mesh = build_water_river_mesh(&straight_river_points(), 6.0, Some(5.0));
        // 1 quad = 2 triangles
        assert_eq!(mesh.triangle_count(), 2);
    }

    #[test]
    fn river_all_normals_point_up() {
        let mesh = build_water_river_mesh(&straight_river_points(), 6.0, Some(5.0));
        for i in 0..mesh.vertex_count() {
            let nx = mesh.normals[i * 3];
            let ny = mesh.normals[i * 3 + 1];
            let nz = mesh.normals[i * 3 + 2];
            assert_eq!((nx, ny, nz), (0.0, 1.0, 0.0), "normal at vertex {} must point up", i);
        }
    }

    #[test]
    fn river_surface_y_override() {
        let mesh = build_water_river_mesh(&straight_river_points(), 6.0, Some(12.0));
        for i in 0..mesh.vertex_count() {
            let y = mesh.vertices[i * 3 + 1];
            assert!(
                (y - 12.0).abs() < 1e-4,
                "vertex {} Y={} expected 12.0",
                i,
                y
            );
        }
    }

    #[test]
    fn river_uses_point_y_when_no_override() {
        let pts = vec![
            (0.0, 3.0, 0.0),
            (0.0, 5.0, 20.0),
        ];
        let mesh = build_water_river_mesh(&pts, 4.0, None);
        // First two verts from segment start (y=3), next two from segment end (y=5)
        let y0 = mesh.vertices[1]; // first vertex Y
        let y2 = mesh.vertices[7]; // third vertex Y (segment end)
        assert!((y0 - 3.0).abs() < 1e-4);
        assert!((y2 - 5.0).abs() < 1e-4);
    }

    #[test]
    fn river_multi_segment() {
        let pts = vec![
            (0.0, 0.0, 0.0),
            (0.0, 0.0, 10.0),
            (10.0, 0.0, 20.0),
        ];
        let mesh = build_water_river_mesh(&pts, 4.0, Some(2.0));
        // 2 segments × 4 verts = 8 vertices, 2 × 2 = 4 triangles
        assert_eq!(mesh.vertex_count(), 8);
        assert_eq!(mesh.triangle_count(), 4);
    }

    #[test]
    fn river_triangles_reference_valid_vertices() {
        let mesh = build_water_river_mesh(&straight_river_points(), 6.0, Some(5.0));
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
    fn river_vertices_normals_same_length() {
        let mesh = build_water_river_mesh(&straight_river_points(), 6.0, Some(5.0));
        assert_eq!(mesh.vertices.len(), mesh.normals.len());
    }

    #[test]
    fn river_degenerate_single_point() {
        let mesh = build_water_river_mesh(&[(0.0, 0.0, 0.0)], 6.0, Some(5.0));
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.triangle_count(), 0);
    }

    #[test]
    fn river_empty_points() {
        let mesh = build_water_river_mesh(&[], 6.0, Some(5.0));
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.triangle_count(), 0);
    }

    #[test]
    fn river_zero_width_returns_empty() {
        let mesh = build_water_river_mesh(&straight_river_points(), 0.0, Some(5.0));
        assert_eq!(mesh.vertex_count(), 0);
    }

    // ── JSON round-trip tests ───────────────────────────────────────────

    #[test]
    fn polygon_json_round_trip() {
        let mesh = build_water_polygon_mesh(&square_footprint(), 5.0);
        let mut json = String::new();
        mesh.write_json(&mut json, 0);

        assert!(json.contains("\"vertices\""));
        assert!(json.contains("\"triangles\""));
        assert!(json.contains("\"normals\""));

        // Count elements: 12 verts × 3 = 36 floats in vertices array
        let vert_count = count_json_array_elements(&json, "vertices");
        assert_eq!(vert_count, 36, "expected 36 vertex floats");

        let tri_count = count_json_array_elements(&json, "triangles");
        assert_eq!(tri_count, 12, "expected 12 triangle indices");

        let norm_count = count_json_array_elements(&json, "normals");
        assert_eq!(norm_count, 36, "expected 36 normal floats");
    }

    #[test]
    fn river_json_round_trip() {
        let mesh = build_water_river_mesh(&straight_river_points(), 6.0, Some(5.0));
        let mut json = String::new();
        mesh.write_json(&mut json, 0);

        assert!(json.contains("\"vertices\""));
        assert!(json.contains("\"triangles\""));
        assert!(json.contains("\"normals\""));

        // 4 verts × 3 = 12 floats
        let vert_count = count_json_array_elements(&json, "vertices");
        assert_eq!(vert_count, 12, "expected 12 vertex floats");

        // 2 tris × 3 = 6 indices
        let tri_count = count_json_array_elements(&json, "triangles");
        assert_eq!(tri_count, 6, "expected 6 triangle indices");
    }

    /// Helper: count comma-separated elements inside a JSON array for a given key.
    fn count_json_array_elements(json: &str, key: &str) -> usize {
        let key_str = format!("\"{}\"", key);
        let kpos = json.find(&key_str).expect("key not found");
        let arr_open = json[kpos..].find('[').unwrap() + kpos;
        let arr_close = json[arr_open..].find(']').unwrap() + arr_open;
        let inner = json[arr_open + 1..arr_close].trim();
        if inner.is_empty() {
            0
        } else {
            inner.split(',').count()
        }
    }

    // ── Chunk-level JSON round-trip (mirrors mesh_builder test) ─────────

    #[test]
    fn water_mesh_json_round_trip_via_chunk() {
        use crate::manifest::{Chunk, WaterFeature, GroundPoint, Color};
        use arbx_geo::{ChunkId, Vec3};

        let footprint_pts = square_footprint();
        let mesh = build_water_polygon_mesh(&footprint_pts, 5.0);

        let expected_vert_floats = mesh.vertices.len();
        let expected_tri_indices = mesh.triangles.len();
        let expected_norm_floats = mesh.normals.len();

        let water = WaterFeature {
            id: "test-lake".into(),
            kind: "lake".into(),
            material: "Water".into(),
            color: Some(Color::new(0, 100, 200)),
            width_studs: None,
            points: None,
            footprint: Some(footprint_pts.iter().map(|&(x, z)| GroundPoint::new(x, z)).collect()),
            holes: vec![],
            indices: None,
            surface_y: Some(5.0),
            width: None,
            intermittent: None,
            water_type: None,
            water_mesh: Some(mesh),
        };

        let chunk = Chunk {
            id: ChunkId::new(0, 0),
            origin_studs: Vec3::new(0.0, 0.0, 0.0),
            terrain: None,
            terrain_texture_path: None,
            terrain_texture_rgba_path: None,
            roads: vec![],
            rails: vec![],
            buildings: vec![],
            water: vec![water],
            props: vec![],
            landuse: vec![],
            barriers: vec![],
        };

        let json = chunk.to_json_pretty();

        assert!(
            json.contains("\"waterMesh\""),
            "JSON must contain waterMesh key"
        );

        // Extract waterMesh object and verify array sizes.
        let sm_start = json.find("\"waterMesh\"").unwrap();
        let obj_start = json[sm_start..].find('{').unwrap() + sm_start;
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

        let count_elements = |key: &str| -> usize {
            let key_str = format!("\"{}\"", key);
            let kpos = mesh_json.find(&key_str).unwrap_or_else(|| {
                panic!("waterMesh JSON missing key '{}'", key);
            });
            let arr_open = mesh_json[kpos..].find('[').unwrap() + kpos;
            let arr_close = mesh_json[arr_open..].find(']').unwrap() + arr_open;
            let inner = mesh_json[arr_open + 1..arr_close].trim();
            if inner.is_empty() { 0 } else { inner.split(',').count() }
        };

        assert_eq!(count_elements("vertices"), expected_vert_floats);
        assert_eq!(count_elements("triangles"), expected_tri_indices);
        assert_eq!(count_elements("normals"), expected_norm_floats);
    }
}
