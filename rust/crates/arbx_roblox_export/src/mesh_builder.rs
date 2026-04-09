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
    /// Flat [u,v, u,v, ...] per-vertex texture coordinates mapped into the
    /// building's atlas UV rect. Empty when no atlas UV is available.
    pub uvs: Vec<f32>,
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
            uvs: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

const MIN_EDGE: f64 = 0.5;
const DEFAULT_WALL_THICKNESS: f64 = 0.6;

/// Roblox EditableMesh vertex budget — meshes exceeding this are invalid.
const EDITABLE_MESH_VERTEX_BUDGET: usize = 60_000;

/// Merge two `PrecomputedMesh` instances into one, offsetting triangle indices
/// of the second mesh by the vertex count of the first.
///
/// If the merged vertex count would exceed `EDITABLE_MESH_VERTEX_BUDGET`
/// (60 000), the merge is rejected and only mesh `a` (walls) is returned.
/// This prevents invalid meshes from reaching Roblox's EditableMesh limit.
pub fn merge_meshes(a: PrecomputedMesh, b: PrecomputedMesh) -> PrecomputedMesh {
    if b.vertices.is_empty() {
        return a;
    }
    if a.vertices.is_empty() {
        return b;
    }
    let merged_vertex_count = (a.vertices.len() + b.vertices.len()) / 3;
    if merged_vertex_count > EDITABLE_MESH_VERTEX_BUDGET {
        eprintln!(
            "[mesh_builder] WARNING: merged vertex count {} exceeds EditableMesh budget {}; \
             falling back to walls-only mesh ({} verts)",
            merged_vertex_count,
            EDITABLE_MESH_VERTEX_BUDGET,
            a.vertices.len() / 3,
        );
        return a;
    }
    let a_vert_count = a.vertices.len();
    let offset = (a_vert_count / 3) as u32;
    let mut vertices = a.vertices;
    vertices.extend_from_slice(&b.vertices);
    let mut triangles = a.triangles;
    triangles.extend(b.triangles.iter().map(|i| i + offset));
    let mut normals = a.normals;
    normals.extend_from_slice(&b.normals);
    let mut uvs = a.uvs;
    if !b.uvs.is_empty() {
        // If `a` has vertices but no UVs, pad with zeros so the merged array
        // stays aligned (2 floats per vertex).
        if uvs.is_empty() && a_vert_count > 0 {
            uvs.resize((a_vert_count / 3) * 2, 0.0);
        }
        uvs.extend_from_slice(&b.uvs);
    }
    PrecomputedMesh {
        vertices,
        triangles,
        normals,
        uvs,
    }
}

/// Build wall geometry only (oriented boxes for each footprint edge).
fn build_walls(footprint: &[(f64, f64)], base_y: f64, height: f64, wall_t: f64) -> PrecomputedMesh {
    let mut acc = MeshAccum::new();
    let n = footprint.len();
    if n < 3 || height <= 0.0 {
        return acc.into_mesh();
    }

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

        let forward: V3 = [dx / edge_len, 0.0, dz / edge_len];
        let right: V3 = [dz / edge_len, 0.0, -dx / edge_len];

        let mid_x = (ax + bx) * 0.5;
        let mid_z = (az + bz) * 0.5;
        let center_y = base_y + height * 0.5;
        let center: V3 = [mid_x, center_y, mid_z];

        let half_size: V3 = [wall_t * 0.5, height * 0.5, (edge_len + wall_t) * 0.5];

        acc.add_oriented_box(center, right, up, forward, half_size);
    }

    acc.into_mesh()
}

/// Compute the maximum distance from any footprint point to the centroid,
/// projected onto the shorter bounding-box axis. Used as `max_dist_to_ridge`
/// for `resolve_heights`.
fn footprint_max_dist_to_ridge(footprint: &[(f64, f64)]) -> f64 {
    if footprint.is_empty() {
        return 5.0; // sensible fallback
    }
    let (mut min_x, mut max_x) = (f64::MAX, f64::MIN);
    let (mut min_z, mut max_z) = (f64::MAX, f64::MIN);
    for &(x, z) in footprint {
        if x < min_x { min_x = x; }
        if x > max_x { max_x = x; }
        if z < min_z { min_z = z; }
        if z > max_z { max_z = z; }
    }
    let dx = max_x - min_x;
    let dz = max_z - min_z;
    // Half of the shorter bounding-box dimension ≈ half-width perpendicular to ridge.
    (dx.min(dz) / 2.0).max(1.0)
}

/// Build the complete building mesh: walls + roof geometry merged into a single
/// `PrecomputedMesh`.
///
/// Uses `resolve_heights` to compute wall/roof/floor intervals from OSM tags,
/// then selects wall geometry:
/// - `level_count >= 2`: windowed walls via `generate_wall_mesh` (osm2world port).
/// - `level_count < 2`: simple oriented-box walls (sheds, garages, single-story).
///
/// The roof shape is created from the `roof_shape` tag and triangulated via the
/// `roof::heightfield` engine.
///
/// # Arguments
/// * `footprint` — 2D polygon points as (x, z) pairs in stud space.
/// * `base_y` — Y coordinate of the building base (ground level).
/// * `height` — Total building height in studs (wall + roof).
/// * `wall_thickness` — Wall thickness in studs (use 0.0 for default 0.6).
/// * `roof_shape` — OSM `roof:shape` tag value (e.g. "flat", "gabled", "hipped").
/// * `roof_height` — Explicit roof height in studs, if known.
/// * `roof_direction` — Roof ridge direction in degrees, if known.
/// * `roof_angle` — Roof pitch angle in degrees, if known.
/// * `roof_orientation` — "along" or "across", if known.
/// * `levels` — Number of building levels.
/// * `roof_levels` — Number of roof levels.
/// * `min_height` — Minimum height / ground offset in studs.
/// * `material_tag` — OSM `building:material` tag (stored for future SurfaceAppearance).
/// * `usage` — OSM `building` usage tag (stored for future SurfaceAppearance).
/// * `building_id` — Stable building identifier (stored for future material seeding).
/// * `atlas_uv` — When provided, wall surfaces are UV-mapped into this atlas
///   sub-rect so the Roblox runtime can apply the chunk facade texture directly.
pub fn build_building_mesh(
    footprint: &[(f64, f64)],
    base_y: f64,
    height: f64,
    wall_thickness: f64,
    roof_shape: &str,
    roof_height: Option<f64>,
    roof_direction: Option<f64>,
    roof_angle: Option<f64>,
    roof_orientation: Option<&str>,
    levels: Option<u32>,
    roof_levels: Option<u32>,
    min_height: Option<f64>,
    // Material fields — not yet used for geometry but threaded for future SurfaceAppearance
    _material_tag: Option<&str>,
    _usage: Option<&str>,
    _building_id: &str,
    atlas_uv: Option<&crate::building_atlas::AtlasUv>,
) -> PrecomputedMesh {
    let wall_t = if wall_thickness <= 0.0 {
        DEFAULT_WALL_THICKNESS
    } else {
        wall_thickness
    };

    let n = footprint.len();
    if n < 3 || height <= 0.0 {
        return PrecomputedMesh {
            vertices: Vec::new(),
            triangles: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
        };
    }

    // ---- Resolve heights via level_height module ----
    use crate::level_height::resolve_heights;

    let max_dist = footprint_max_dist_to_ridge(footprint);
    let resolved = resolve_heights(
        base_y,
        Some(height),
        levels,
        roof_height,
        roof_angle,
        roof_levels,
        min_height,
        max_dist,
    );

    let effective_base_y = resolved.base_y;
    let effective_wall_height = resolved.wall_height;
    let effective_roof_height = resolved.roof_height;
    let level_count = resolved.level_count;
    let floor_height = resolved.floor_height;

    // ---- Walls ----
    // Use resolved.base_y (accounts for min_height) instead of the raw base_y.
    // Windowed walls for all multi-story buildings. Fidelity over speed —
    // chunks are larger but the visual upgrade is worth the fetch latency.
    // Cloudflare free tier handles the payload fine; the user accepts
    // longer bootstrap times for extreme detail.
    let wall_mesh = if level_count >= 2 {
        // Multi-story buildings get windowed wall surfaces.
        use crate::wall_surface::generate_wall_mesh_with_atlas;
        generate_wall_mesh_with_atlas(
            footprint,
            effective_base_y,
            effective_wall_height,
            level_count,
            floor_height,
            wall_t,
            atlas_uv,
        )
    } else {
        // Single-story buildings keep the simpler oriented-box walls for performance.
        build_walls(footprint, effective_base_y, effective_wall_height, wall_t)
    };

    // ---- Roof ----
    use crate::roof::{self, Point2D, Polygon2D, RoofTags};
    use crate::roof::heightfield::triangulate_roof;

    let polygon = Polygon2D {
        outer: footprint.iter().map(|&(x, z)| Point2D::new(x, z)).collect(),
        holes: vec![],
    };

    let tags = RoofTags {
        shape: roof_shape.to_string(),
        height: if effective_roof_height > 0.0 { Some(effective_roof_height) } else { roof_height },
        angle: roof_angle,
        direction: roof_direction,
        orientation: roof_orientation.map(|s| s.to_string()),
        levels: levels,
        material: None,
        colour: None,
    };

    let roof_obj = roof::create_roof(roof_shape, polygon, &tags);
    let wall_top_y = effective_base_y + effective_wall_height;
    let roof_mesh = triangulate_roof(roof_obj.as_ref(), wall_top_y);

    // ---- Merge ----
    merge_meshes(wall_mesh, roof_mesh)
}

/// Build the exterior shell mesh for a building: oriented-box walls for each
/// footprint edge. Backwards-compatible wrapper that produces walls-only geometry.
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

    build_walls(footprint, base_y, height, wall_t)
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

    // -----------------------------------------------------------------------
    // build_building_mesh tests
    // -----------------------------------------------------------------------

    #[test]
    fn building_mesh_flat_single_story_uses_oriented_box_walls() {
        let fp = rect_footprint();
        let shell = build_shell_mesh(&fp, 0.0, 5.0, 0.6);
        // levels=1 forces level_count=1 → oriented-box walls (same as shell).
        let building = build_building_mesh(
            &fp, 0.0, 5.0, 0.6, "flat", None, None, None, None,
            Some(1), None, None, None, None, "test", None,
        );
        // Flat roof adds a cap on top; building mesh has at least as many verts.
        assert!(
            building.vertex_count() >= shell.vertex_count(),
            "building mesh ({} verts) should have >= shell mesh ({} verts)",
            building.vertex_count(),
            shell.vertex_count()
        );
        // The first N vertices (wall geometry) should be identical to shell.
        assert_eq!(
            &building.vertices[..shell.vertices.len()],
            &shell.vertices[..],
            "single-story wall vertices should be identical between shell and building mesh"
        );
    }

    #[test]
    fn building_mesh_gabled_more_verts_than_flat() {
        let fp = rect_footprint();
        // Use levels=1 to isolate roof geometry comparison (both use oriented-box walls).
        let flat = build_building_mesh(
            &fp, 0.0, 12.0, 0.6, "flat", None, None, None, None,
            Some(1), None, None, None, None, "test", None,
        );
        let gabled = build_building_mesh(
            &fp, 0.0, 12.0, 0.6, "gabled", Some(4.0), None, None, None,
            Some(1), None, None, None, None, "test", None,
        );
        assert!(
            gabled.vertex_count() > flat.vertex_count(),
            "gabled ({} verts) should have more vertices than flat ({} verts)",
            gabled.vertex_count(),
            flat.vertex_count()
        );
    }

    #[test]
    fn building_mesh_hipped_more_verts_than_flat() {
        let fp = rect_footprint();
        // Use levels=1 to isolate roof geometry comparison (both use oriented-box walls).
        let flat = build_building_mesh(
            &fp, 0.0, 12.0, 0.6, "flat", None, None, None, None,
            Some(1), None, None, None, None, "test", None,
        );
        let hipped = build_building_mesh(
            &fp, 0.0, 12.0, 0.6, "hipped", Some(4.0), None, None, None,
            Some(1), None, None, None, None, "test", None,
        );
        assert!(
            hipped.vertex_count() > flat.vertex_count(),
            "hipped ({} verts) should have more vertices than flat ({} verts)",
            hipped.vertex_count(),
            flat.vertex_count()
        );
    }

    #[test]
    fn building_mesh_normals_unit_length() {
        let fp = rect_footprint();
        let mesh = build_building_mesh(
            &fp, 0.0, 12.0, 0.6, "gabled", Some(4.0), None, None, None, None, None, None, None, None, "test", None,
        );
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
    fn building_mesh_triangles_valid() {
        let fp = rect_footprint();
        let mesh = build_building_mesh(
            &fp, 0.0, 12.0, 0.6, "hipped", Some(3.0), None, None, None, None, None, None, None, None, "test", None,
        );
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
    fn merge_meshes_empty() {
        let empty = PrecomputedMesh {
            vertices: vec![],
            triangles: vec![],
            normals: vec![],
            uvs: vec![],
        };
        let nonempty = PrecomputedMesh {
            vertices: vec![1.0, 2.0, 3.0],
            triangles: vec![0],
            normals: vec![0.0, 1.0, 0.0],
            uvs: vec![],
        };
        // merge(empty, nonempty) == nonempty
        let m = merge_meshes(empty.clone(), nonempty.clone());
        assert_eq!(m.vertices, nonempty.vertices);
        // merge(nonempty, empty) == nonempty
        let m2 = merge_meshes(nonempty.clone(), empty);
        assert_eq!(m2.vertices, nonempty.vertices);
    }

    #[test]
    fn merge_meshes_vertex_budget_guard() {
        // Create two meshes that together exceed EDITABLE_MESH_VERTEX_BUDGET.
        let big = |n: usize| -> PrecomputedMesh {
            PrecomputedMesh {
                vertices: vec![0.0f32; n * 3],
                triangles: (0..n as u32).collect(),
                normals: vec![0.0f32; n * 3],
                uvs: vec![],
            }
        };
        let a = big(40_000);
        let b = big(30_000);
        // 40k + 30k = 70k > 60k budget → should fall back to `a` only.
        let merged = merge_meshes(a.clone(), b);
        assert_eq!(
            merged.vertex_count(),
            40_000,
            "vertex budget guard should return walls-only mesh"
        );
        // Under budget: should merge normally.
        let a2 = big(30_000);
        let b2 = big(20_000);
        let merged2 = merge_meshes(a2, b2);
        assert_eq!(merged2.vertex_count(), 50_000);
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
            roof_included: true,
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

        // Verify B64 fields exist (legacy JSON arrays stripped).
        let has_b64_key = |key: &str| -> bool {
            let key_str = format!("\"{}\"", key);
            mesh_json.contains(&key_str)
        };
        assert!(has_b64_key("verticesB64"), "shellMesh JSON missing verticesB64");
        assert!(has_b64_key("trianglesB64"), "shellMesh JSON missing trianglesB64");
        assert!(has_b64_key("normalsB64"), "shellMesh JSON missing normalsB64");

        // Verify counts match expectations.
        let extract_int = |key: &str| -> usize {
            let key_str = format!("\"{}\"", key);
            let kpos = mesh_json.find(&key_str).expect("missing count key");
            // Skip past "key": and any whitespace to find the integer value.
            let after_key = kpos + key_str.len();
            let colon_pos = mesh_json[after_key..].find(':').unwrap() + after_key + 1;
            let val_str = mesh_json[colon_pos..].trim_start();
            let val_end = val_str.find(|c: char| !c.is_ascii_digit()).unwrap_or(val_str.len());
            val_str[..val_end].parse::<usize>().unwrap_or(0)
        };
        let vert_count = extract_int("vertexCount") * 3; // vertexCount is vertex count, not float count
        let tri_count = extract_int("triangleCount") * 3;

        assert_eq!(
            vert_count, expected_vert_floats,
            "vertexCount mismatch (expected {} floats = {} vertices)",
            expected_vert_floats, expected_vert_floats / 3
        );
        assert_eq!(
            tri_count, expected_tri_indices,
            "triangleCount mismatch (expected {} indices = {} triangles)",
            expected_tri_indices, expected_tri_indices / 3
        );
    }

    // -----------------------------------------------------------------------
    // Integration tests: wall_surface + level_height wiring
    // -----------------------------------------------------------------------

    #[test]
    fn three_story_building_has_more_vertices_than_single_story() {
        let fp = rect_footprint();
        // 1-story: oriented-box walls (simple)
        let one_story = build_building_mesh(
            &fp, 0.0, 5.0, 0.6, "flat", None, None, None, None,
            Some(1), None, None, None, None, "bldg-1", None,
        );
        // 3-story: windowed wall surfaces (more geometry)
        let three_story = build_building_mesh(
            &fp, 0.0, 12.0, 0.6, "flat", None, None, None, None,
            Some(3), None, None, None, None, "bldg-3", None,
        );
        assert!(
            three_story.vertex_count() > one_story.vertex_count(),
            "3-story ({} verts) should have more vertices than 1-story ({} verts) \
             due to windowed wall geometry",
            three_story.vertex_count(),
            one_story.vertex_count()
        );
    }

    #[test]
    fn single_story_uses_oriented_box_walls_same_vertex_count() {
        let fp = rect_footprint();
        // Shell mesh (oriented-box walls only, no roof).
        let shell = build_shell_mesh(&fp, 0.0, 5.0, 0.6);
        // 1-story building mesh with flat roof — wall portion should use oriented-box.
        let building = build_building_mesh(
            &fp, 0.0, 5.0, 0.6, "flat", None, None, None, None,
            Some(1), None, None, None, None, "bldg-1story", None,
        );
        // Wall vertex count should match shell exactly.
        // Building has shell vertices + roof vertices, so total >= shell.
        assert!(
            building.vertex_count() >= shell.vertex_count(),
            "1-story building ({}) should have >= shell verts ({})",
            building.vertex_count(),
            shell.vertex_count()
        );
        // First N verts should be identical oriented-box walls.
        assert_eq!(
            &building.vertices[..shell.vertices.len()],
            &shell.vertices[..],
            "1-story building wall vertices must match shell oriented-box walls"
        );
    }

    #[test]
    fn resolve_heights_integration_levels_3_no_explicit_height() {
        // levels=3 with no explicit height → height computed as 3*3.5+2.0 = 12.5
        // resolve_heights should compute wall_height=12.5, level_count=3, floor_height≈4.17
        use crate::level_height::resolve_heights;
        let r = resolve_heights(0.0, None, Some(3), None, None, None, None, 5.0);
        assert_eq!(r.level_count, 3);
        assert!((r.wall_height - 12.5).abs() < 0.1,
            "wall_height should be ~12.5, got {}", r.wall_height);
        assert!((r.floor_height - 4.167).abs() < 0.1,
            "floor_height should be ~4.17, got {}", r.floor_height);

        // Now build a mesh with those resolved values and confirm it uses windowed walls
        let fp = rect_footprint();
        let total_h = r.wall_height + r.roof_height;
        let mesh = build_building_mesh(
            &fp, 0.0, total_h, 0.6, "flat", None, None, None, None,
            Some(3), None, None, None, None, "bldg-resolve-test", None,
        );
        // Windowed walls for level_count=3 should produce more geometry than
        // 4 oriented-box walls (4×24=96 verts).
        assert!(
            mesh.vertex_count() > 96,
            "3-level resolved mesh should have > 96 wall verts (got {})",
            mesh.vertex_count()
        );
    }
}
