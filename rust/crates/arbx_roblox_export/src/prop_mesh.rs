//! Compile-time prop mesh generation for trees, benches, and lamps.
//!
//! Mirrors the procedural tree geometry from `PropBuilder.lua` so that meshes
//! can be pre-computed at export time and loaded directly by the Lua importer.

use std::fmt::Write as _;

/// Pre-computed prop mesh: flat interleaved arrays matching the Roblox
/// EditableMesh vertex/triangle layout (same layout as `PrecomputedMesh`
/// and `RoadMeshStrip`).
#[derive(Debug, Clone, PartialEq)]
pub struct PropMesh {
    /// Flat [x,y,z, x,y,z, ...] vertex positions.
    pub vertices: Vec<f32>,
    /// Flat [v0,v1,v2, v0,v1,v2, ...] triangle indices (0-based).
    pub triangles: Vec<u32>,
    /// Flat [nx,ny,nz, nx,ny,nz, ...] per-vertex normals.
    pub normals: Vec<f32>,
}

impl PropMesh {
    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / 3
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len() / 3
    }
}

// ---------------------------------------------------------------------------
// Mesh accumulator (local to this module, same pattern as mesh_builder.rs)
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

    /// Add a quad (4 vertices, 2 triangles). Winding: tri1 = {p1, p2, p3},
    /// tri2 = {p1, p3, p4}.
    fn add_quad(&mut self, p1: [f64; 3], p2: [f64; 3], p3: [f64; 3], p4: [f64; 3], normal: [f64; 3]) {
        let v0 = self.push_vertex(p1, normal);
        let v1 = self.push_vertex(p2, normal);
        let v2 = self.push_vertex(p3, normal);
        let v3 = self.push_vertex(p4, normal);
        self.triangles.extend_from_slice(&[v0, v1, v2, v0, v2, v3]);
    }

    /// Add a triangle with a shared flat normal.
    fn add_triangle(&mut self, p1: [f64; 3], p2: [f64; 3], p3: [f64; 3], normal: [f64; 3]) {
        let v0 = self.push_vertex(p1, normal);
        let v1 = self.push_vertex(p2, normal);
        let v2 = self.push_vertex(p3, normal);
        self.triangles.extend_from_slice(&[v0, v1, v2]);
    }

    fn into_prop_mesh(self) -> PropMesh {
        PropMesh {
            vertices: self.vertices,
            triangles: self.triangles,
            normals: self.normals,
        }
    }
}

// ---------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------

use std::f64::consts::PI;

/// Generate a cylinder along the Y axis centered at `center`, with the given
/// `radius`, `height`, and number of radial `segments`.
///
/// The cylinder is capped top and bottom (closed).  This mirrors the
/// `PropBuilder.lua` trunk geometry (Roblox Cylinder parts).
fn add_cylinder(acc: &mut MeshAccum, center: [f64; 3], radius: f64, height: f64, segments: usize) {
    let half_h = height * 0.5;
    let bottom_y = center[1] - half_h;
    let top_y = center[1] + half_h;

    // Generate ring points
    let ring: Vec<(f64, f64)> = (0..segments)
        .map(|i| {
            let angle = (i as f64 / segments as f64) * 2.0 * PI;
            (angle.cos() * radius, angle.sin() * radius)
        })
        .collect();

    // Side faces — one quad per segment
    for i in 0..segments {
        let j = (i + 1) % segments;
        let (rx0, rz0) = ring[i];
        let (rx1, rz1) = ring[j];

        let bl = [center[0] + rx0, bottom_y, center[2] + rz0];
        let br = [center[0] + rx1, bottom_y, center[2] + rz1];
        let tr = [center[0] + rx1, top_y, center[2] + rz1];
        let tl = [center[0] + rx0, top_y, center[2] + rz0];

        // Outward-facing normal at the midpoint of this face
        let mid_nx = (rx0 + rx1) * 0.5;
        let mid_nz = (rz0 + rz1) * 0.5;
        let len = (mid_nx * mid_nx + mid_nz * mid_nz).sqrt();
        let normal = if len > 1e-12 {
            [mid_nx / len, 0.0, mid_nz / len]
        } else {
            [1.0, 0.0, 0.0]
        };

        acc.add_quad(bl, br, tr, tl, normal);
    }

    // Top cap (fan from center)
    let top_center = [center[0], top_y, center[2]];
    let up: [f64; 3] = [0.0, 1.0, 0.0];
    for i in 0..segments {
        let j = (i + 1) % segments;
        let p1 = [center[0] + ring[i].0, top_y, center[2] + ring[i].1];
        let p2 = [center[0] + ring[j].0, top_y, center[2] + ring[j].1];
        acc.add_triangle(top_center, p1, p2, up);
    }

    // Bottom cap (fan from center, reversed winding)
    let bottom_center = [center[0], bottom_y, center[2]];
    let down: [f64; 3] = [0.0, -1.0, 0.0];
    for i in 0..segments {
        let j = (i + 1) % segments;
        let p1 = [center[0] + ring[j].0, bottom_y, center[2] + ring[j].1];
        let p2 = [center[0] + ring[i].0, bottom_y, center[2] + ring[i].1];
        acc.add_triangle(bottom_center, p1, p2, down);
    }
}

/// Generate a UV-sphere at `center` with the given `radius_x`, `radius_y`,
/// and `radius_z`, using `rings` latitude bands and `segments` longitude
/// divisions.  This mirrors the broadleaf canopy shape from
/// `PropBuilder.lua`'s `buildRealisticCanopy` (ellipsoidal Ball parts).
fn add_ellipsoid(
    acc: &mut MeshAccum,
    center: [f64; 3],
    radius_x: f64,
    radius_y: f64,
    radius_z: f64,
    rings: usize,
    segments: usize,
) {
    // Build a grid of points on the sphere surface, then emit quads for
    // each grid cell.  Per-face normals are computed from the ellipsoid
    // gradient (same direction as the position offset from center, scaled
    // by inverse radii squared, but for visual fidelity flat face normals
    // are good enough at this resolution).

    // Generate grid positions: rows 0..=rings, cols 0..=segments
    let grid_point = |ring: usize, seg: usize| -> [f64; 3] {
        let phi = PI * (ring as f64 / rings as f64); // 0 = top, PI = bottom
        let theta = 2.0 * PI * (seg as f64 / segments as f64);
        let sp = phi.sin();
        [
            center[0] + radius_x * sp * theta.cos(),
            center[1] + radius_y * phi.cos(),
            center[2] + radius_z * sp * theta.sin(),
        ]
    };

    let face_normal = |p1: [f64; 3], p2: [f64; 3], p3: [f64; 3]| -> [f64; 3] {
        let ax = p2[0] - p1[0];
        let ay = p2[1] - p1[1];
        let az = p2[2] - p1[2];
        let bx = p3[0] - p1[0];
        let by = p3[1] - p1[1];
        let bz = p3[2] - p1[2];
        let nx = ay * bz - az * by;
        let ny = az * bx - ax * bz;
        let nz = ax * by - ay * bx;
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len < 1e-12 {
            [0.0, 1.0, 0.0]
        } else {
            [nx / len, ny / len, nz / len]
        }
    };

    for r in 0..rings {
        for s in 0..segments {
            let s1 = (s + 1) % segments;
            let p00 = grid_point(r, s);
            let p01 = grid_point(r, s1);
            let p10 = grid_point(r + 1, s);
            let p11 = grid_point(r + 1, s1);

            let n = face_normal(p00, p01, p10);

            if r == 0 {
                // Top cap triangle
                acc.add_triangle(p00, p01, p11, n);
            } else if r == rings - 1 {
                // Bottom cap triangle
                acc.add_triangle(p00, p01, p10, n);
            } else {
                acc.add_quad(p00, p01, p11, p10, n);
            }
        }
    }
}

/// Generate a cone frustum (truncated cone) along the Y axis.
///
/// Used for conifer tier geometry (matching `PropBuilder.lua`'s stacked Ball
/// tiers that approximate a cone).
fn add_cone_frustum(
    acc: &mut MeshAccum,
    center: [f64; 3],
    bottom_radius: f64,
    top_radius: f64,
    height: f64,
    segments: usize,
) {
    let half_h = height * 0.5;
    let bottom_y = center[1] - half_h;
    let top_y = center[1] + half_h;

    let bottom_ring: Vec<(f64, f64)> = (0..segments)
        .map(|i| {
            let angle = (i as f64 / segments as f64) * 2.0 * PI;
            (angle.cos() * bottom_radius, angle.sin() * bottom_radius)
        })
        .collect();

    let top_ring: Vec<(f64, f64)> = (0..segments)
        .map(|i| {
            let angle = (i as f64 / segments as f64) * 2.0 * PI;
            (angle.cos() * top_radius, angle.sin() * top_radius)
        })
        .collect();

    // Slant angle for normals
    let slant_y = (bottom_radius - top_radius) / height;

    // Side faces
    for i in 0..segments {
        let j = (i + 1) % segments;
        let bl = [center[0] + bottom_ring[i].0, bottom_y, center[2] + bottom_ring[i].1];
        let br = [center[0] + bottom_ring[j].0, bottom_y, center[2] + bottom_ring[j].1];
        let tr = [center[0] + top_ring[j].0, top_y, center[2] + top_ring[j].1];
        let tl = [center[0] + top_ring[i].0, top_y, center[2] + top_ring[i].1];

        let mid_nx = (bottom_ring[i].0 + bottom_ring[j].0) * 0.5;
        let mid_nz = (bottom_ring[i].1 + bottom_ring[j].1) * 0.5;
        let horiz_len = (mid_nx * mid_nx + mid_nz * mid_nz).sqrt();
        let normal = if horiz_len > 1e-12 {
            let ny_component = slant_y;
            let len = (1.0 + ny_component * ny_component).sqrt();
            [mid_nx / horiz_len / len, ny_component / len, mid_nz / horiz_len / len]
        } else {
            [1.0, 0.0, 0.0]
        };

        acc.add_quad(bl, br, tr, tl, normal);
    }

    // Top cap
    if top_radius > 1e-6 {
        let top_center = [center[0], top_y, center[2]];
        let up: [f64; 3] = [0.0, 1.0, 0.0];
        for i in 0..segments {
            let j = (i + 1) % segments;
            let p1 = [center[0] + top_ring[i].0, top_y, center[2] + top_ring[i].1];
            let p2 = [center[0] + top_ring[j].0, top_y, center[2] + top_ring[j].1];
            acc.add_triangle(top_center, p1, p2, up);
        }
    }

    // Bottom cap
    let bottom_center = [center[0], bottom_y, center[2]];
    let down: [f64; 3] = [0.0, -1.0, 0.0];
    for i in 0..segments {
        let j = (i + 1) % segments;
        let p1 = [center[0] + bottom_ring[j].0, bottom_y, center[2] + bottom_ring[j].1];
        let p2 = [center[0] + bottom_ring[i].0, bottom_y, center[2] + bottom_ring[i].1];
        acc.add_triangle(bottom_center, p1, p2, down);
    }
}

// ---------------------------------------------------------------------------
// Public API — tree mesh builders
// ---------------------------------------------------------------------------

/// Radial segments for cylinder / sphere geometry.  8 is a good balance of
/// visual quality and vertex count for props viewed from typical game camera
/// distances.
const CYLINDER_SEGMENTS: usize = 8;
const SPHERE_RINGS: usize = 6;
const SPHERE_SEGMENTS: usize = 8;
const CONE_SEGMENTS: usize = 8;

/// Meters-to-studs conversion factor (same as PropBuilder.lua).
const METERS_TO_STUDS: f64 = 1.0 / 0.3;

/// Build a broadleaf canopy mesh: a single ellipsoid centered at (0, center_y, 0).
///
/// Mirrors `buildRealisticCanopy` in PropBuilder.lua — the main canopy lobe
/// (the secondary lobes are a runtime Lua detail for visual variety, but the
/// pre-computed mesh provides the primary shape).
///
/// The ellipsoid has an aspect ratio of ~0.75 (wider than tall), matching the
/// Lua default `aspectY` range midpoint (~0.8).
pub fn build_broadleaf_canopy(radius: f64, height: f64) -> PropMesh {
    let mut acc = MeshAccum::new();
    if radius <= 0.0 || height <= 0.0 {
        return acc.into_prop_mesh();
    }
    let aspect_y = 0.75; // midpoint of Lua's 0.55..1.1 range
    let ry = radius * aspect_y;
    let center_y = height + ry * 0.5; // canopy sits atop the trunk
    add_ellipsoid(
        &mut acc,
        [0.0, center_y, 0.0],
        radius,
        ry,
        radius,
        SPHERE_RINGS,
        SPHERE_SEGMENTS,
    );
    acc.into_prop_mesh()
}

/// Build a conifer canopy mesh: 3 stacked cone-frustum tiers.
///
/// Mirrors the `needleleaved` branch in `PropBuilder.lua`'s `buildTree`:
/// 3 tiers of progressively smaller Ball parts approximating a Christmas-tree
/// shape.  Here we use actual cone frustums for better silhouette.
pub fn build_conifer_canopy(radius: f64, height: f64) -> PropMesh {
    let mut acc = MeshAccum::new();
    if radius <= 0.0 || height <= 0.0 {
        return acc.into_prop_mesh();
    }

    let tiers = 3;
    let total_cone_h = radius * 2.8; // same ratio as Lua: canopyR * 2.8
    let tier_h = total_cone_h / tiers as f64;

    for tier in 0..tiers {
        let t = tier as f64 / (tiers - 1) as f64; // 0 = bottom, 1 = top
        let tier_radius = radius * (1.0 - t * 0.55); // same as Lua
        let top_radius = if tier == tiers - 1 {
            0.0 // pointed top
        } else {
            radius * (1.0 - ((tier + 1) as f64 / (tiers - 1) as f64) * 0.55) * 0.7
        };
        let y_off = tier as f64 * tier_h * 0.75; // slight overlap
        let center_y = height + y_off + tier_h * 0.4;

        add_cone_frustum(
            &mut acc,
            [0.0, center_y, 0.0],
            tier_radius,
            top_radius,
            tier_h * 1.1, // same 1.1× factor as Lua
            CONE_SEGMENTS,
        );
    }
    acc.into_prop_mesh()
}

/// Build a palm canopy mesh: 4 frond-like wedges radiating from the trunk top.
///
/// Mirrors the palm branch in `PropBuilder.lua`'s `buildTree`: 4 rectangular
/// Part fronds angled downward at 30 degrees.  Here we use flat quads.
pub fn build_palm_canopy(radius: f64, height: f64) -> PropMesh {
    let mut acc = MeshAccum::new();
    if radius <= 0.0 || height <= 0.0 {
        return acc.into_prop_mesh();
    }

    let frond_length = radius * 1.5; // same as Lua: canopyR * 1.5
    let frond_width = 1.0; // 1 stud wide
    let frond_thickness = 0.5; // 0.5 stud thick
    let droop_angle: f64 = 30.0_f64.to_radians();

    for i in 0..4 {
        let angle = (i as f64) * PI * 0.5; // 0, 90, 180, 270 degrees
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // Frond center is offset from trunk top along the frond direction,
        // drooping downward
        let frond_center_dist = frond_length * 0.5;
        let cx = cos_a * frond_center_dist * droop_angle.cos();
        let cy = height - frond_center_dist * droop_angle.sin();
        let cz = sin_a * frond_center_dist * droop_angle.cos();

        // The frond is a thin box.  We approximate it as a top and bottom quad.
        // Forward direction (along the frond)
        let fx = cos_a * droop_angle.cos();
        let fy = -droop_angle.sin();
        let fz = sin_a * droop_angle.cos();

        // Right direction (perpendicular to frond, horizontal)
        let rx = -sin_a;
        let rz = cos_a;

        let half_l = frond_length * 0.5;
        let half_w = frond_width * 0.5;
        let half_t = frond_thickness * 0.5;

        // 4 corners of the top face
        let t1 = [cx - fx * half_l - rx * half_w, cy - fy * half_l + half_t, cz - fz * half_l - rz * half_w];
        let t2 = [cx - fx * half_l + rx * half_w, cy - fy * half_l + half_t, cz - fz * half_l + rz * half_w];
        let t3 = [cx + fx * half_l + rx * half_w, cy + fy * half_l + half_t, cz + fz * half_l + rz * half_w];
        let t4 = [cx + fx * half_l - rx * half_w, cy + fy * half_l + half_t, cz + fz * half_l - rz * half_w];

        // Top face normal (pointing up from the frond)
        let up_x = fy * rz;
        let up_y = fz * rx - fx * rz;
        let up_z = -fy * rx;
        let up_len = (up_x * up_x + up_y * up_y + up_z * up_z).sqrt();
        let top_normal = if up_len > 1e-12 {
            // Cross product of forward x right
            let nx = fy * rz - 0.0 * 0.0;
            let ny = 0.0 * (-sin_a) - fx * rz;
            let nz = fx * 0.0 - fy * (-sin_a);
            let _ = (nx, ny, nz); // use simplified approach
            // Simplified: normal is roughly upward for a drooping frond
            let n = cross([fx, fy, fz], [rx, 0.0, rz]);
            let nl = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if nl > 1e-12 { [n[0] / nl, n[1] / nl, n[2] / nl] } else { [0.0, 1.0, 0.0] }
        } else {
            [0.0, 1.0, 0.0]
        };

        acc.add_quad(t1, t2, t3, t4, top_normal);

        // Bottom face (reversed winding)
        let b1 = [t1[0], t1[1] - frond_thickness, t1[2]];
        let b2 = [t2[0], t2[1] - frond_thickness, t2[2]];
        let b3 = [t3[0], t3[1] - frond_thickness, t3[2]];
        let b4 = [t4[0], t4[1] - frond_thickness, t4[2]];
        let bottom_normal = [-top_normal[0], -top_normal[1], -top_normal[2]];
        acc.add_quad(b4, b3, b2, b1, bottom_normal);
    }

    acc.into_prop_mesh()
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Known conifer species patterns (mirrors PropBuilder.lua's
/// `CONIFER_SPECIES_PATTERNS`).
const CONIFER_SPECIES: &[&str] = &[
    "pinus", "picea", "abies", "spruce", "fir", "cedar", "juniper",
    "cypress", "larch", "hemlock", "yew", "redwood", "sequoia", "conifer",
    "juniperus",
];

/// Determine the effective leaf type for a prop, applying the same conifer
/// species inference as PropBuilder.lua.
pub fn resolve_leaf_type(leaf_type: Option<&str>, species: Option<&str>) -> &'static str {
    if let Some(lt) = leaf_type {
        if !lt.is_empty() {
            return match lt {
                "needleleaved" => "needleleaved",
                "tropical" | "palm" => "palm",
                _ => "broadleaved",
            };
        }
    }

    // Check species for palm or conifer patterns
    if let Some(sp) = species {
        let lower = sp.to_lowercase();
        if lower.contains("palm") {
            return "palm";
        }
        for pattern in CONIFER_SPECIES {
            if lower.contains(pattern) {
                return "needleleaved";
            }
        }
    }

    "broadleaved"
}

/// Build a complete tree mesh (trunk + canopy) from manifest attributes.
///
/// Mirrors the `buildTree` function in `PropBuilder.lua`.  The mesh is
/// generated in local space centered at the origin — the Lua importer
/// positions it using the prop's world coordinates.
///
/// # Parameters
/// * `height` — tree height in meters from the manifest (converted to studs
///   internally).  Defaults to 6m (~20 studs) if zero or negative.
/// * `canopy_radius` — canopy radius in studs.  If zero, derived from height.
/// * `trunk_radius` — trunk radius in studs.  If zero, derived from height.
/// * `leaf_type` — one of "broadleaved", "needleleaved", "palm", or empty
///   (defaults to broadleaved).
pub fn build_tree_mesh(
    height_m: f64,
    canopy_radius: f64,
    trunk_radius: f64,
    leaf_type: &str,
) -> PropMesh {
    // Convert height from meters to studs, matching Lua's METERS_TO_STUDS.
    let height_studs = if height_m > 0.0 {
        height_m * METERS_TO_STUDS
    } else {
        20.0 // default tree height in studs
    };

    // Scale factor relative to the default 20-stud tree
    let scale = (height_studs / 20.0).clamp(0.5, 3.0);

    let trunk_h = 7.0 * scale;
    let trunk_r = if trunk_radius > 0.0 {
        trunk_radius
    } else {
        0.5 * scale
    };
    let canopy_r = if canopy_radius > 0.0 {
        canopy_radius
    } else {
        5.0 * scale // midpoint of Lua's 3.5..8.0 range
    };

    let mut acc = MeshAccum::new();

    // Trunk: cylinder centered at (0, trunk_h/2, 0)
    add_cylinder(&mut acc, [0.0, trunk_h * 0.5, 0.0], trunk_r, trunk_h, CYLINDER_SEGMENTS);

    // Canopy: shape depends on leaf_type
    let canopy = match leaf_type {
        "needleleaved" => build_conifer_canopy(canopy_r, trunk_h),
        "palm" | "tropical" => build_palm_canopy(canopy_r, trunk_h),
        _ => build_broadleaf_canopy(canopy_r, trunk_h),
    };

    // Merge canopy into accumulator
    let base_idx = acc.vertex_count();
    acc.vertices.extend_from_slice(&canopy.vertices);
    acc.normals.extend_from_slice(&canopy.normals);
    for &idx in &canopy.triangles {
        acc.triangles.push(idx + base_idx);
    }

    acc.into_prop_mesh()
}

/// Build a bench mesh: seat plank + backrest + 2 legs.
/// All geometry in prop-local space with base at Y=0.
pub fn build_bench_mesh() -> PropMesh {
    let mut acc = MeshAccum::new();
    // Seat: 5×0.3×1.5 at y=1.5
    add_box(&mut acc, [0.0, 1.5, 0.0], [5.0, 0.3, 1.5]);
    // Backrest: 5×1.2×0.2 at y=2.3, offset z=-0.65
    add_box(&mut acc, [0.0, 2.3, -0.65], [5.0, 1.2, 0.2]);
    // Left leg: 0.3×1.5×1.5 at y=0.75, x=-2
    add_box(&mut acc, [-2.0, 0.75, 0.0], [0.3, 1.5, 1.5]);
    // Right leg
    add_box(&mut acc, [2.0, 0.75, 0.0], [0.3, 1.5, 1.5]);
    acc.into_prop_mesh()
}

/// Build a waste basket mesh: small box.
pub fn build_waste_basket_mesh() -> PropMesh {
    let mut acc = MeshAccum::new();
    add_box(&mut acc, [0.0, 0.6, 0.0], [0.8, 1.2, 0.8]);
    acc.into_prop_mesh()
}

/// Build a fire hydrant mesh: small box.
pub fn build_fire_hydrant_mesh() -> PropMesh {
    let mut acc = MeshAccum::new();
    add_box(&mut acc, [0.0, 0.5, 0.0], [0.6, 1.0, 0.6]);
    acc.into_prop_mesh()
}

/// Build a vending machine mesh: tall box.
pub fn build_vending_machine_mesh() -> PropMesh {
    let mut acc = MeshAccum::new();
    add_box(&mut acc, [0.0, 3.0, 0.0], [3.0, 6.0, 2.5]);
    acc.into_prop_mesh()
}

/// Build a bus stop mesh: pole + sign.
pub fn build_bus_stop_mesh() -> PropMesh {
    let mut acc = MeshAccum::new();
    add_cylinder(&mut acc, [0.0, 2.5, 0.0], 0.1, 5.0, 8);
    add_box(&mut acc, [0.0, 5.2, 0.0], [0.8, 0.5, 0.1]);
    acc.into_prop_mesh()
}

/// Build a traffic signal mesh: tall pole + signal housing.
pub fn build_traffic_signal_mesh() -> PropMesh {
    let mut acc = MeshAccum::new();
    add_cylinder(&mut acc, [0.0, 7.5, 0.0], 0.2, 15.0, 8);
    add_box(&mut acc, [0.0, 13.0, 0.0], [1.5, 4.0, 1.0]);
    acc.into_prop_mesh()
}

/// Build a telephone booth mesh: tall glass box.
pub fn build_telephone_mesh() -> PropMesh {
    let mut acc = MeshAccum::new();
    add_box(&mut acc, [0.0, 3.5, 0.0], [3.0, 7.0, 3.0]);
    acc.into_prop_mesh()
}

/// Build a post box mesh: box on short base.
pub fn build_post_box_mesh() -> PropMesh {
    let mut acc = MeshAccum::new();
    add_cylinder(&mut acc, [0.0, 0.75, 0.0], 0.2, 1.5, 8);
    add_box(&mut acc, [0.0, 2.0, 0.0], [1.2, 1.0, 0.8]);
    acc.into_prop_mesh()
}

/// Build a street lamp mesh: pole + lamp head.
/// All geometry in prop-local space with base at Y=0.
pub fn build_street_lamp_mesh() -> PropMesh {
    let mut acc = MeshAccum::new();
    // Pole: cylinder r=0.15, h=12 at y=6
    add_cylinder(&mut acc, [0.0, 6.0, 0.0], 0.15, 12.0, 8);
    // Lamp head: box 1.5×0.5×1.5 at top
    add_box(&mut acc, [0.0, 12.5, 0.0], [1.5, 0.5, 1.5]);
    acc.into_prop_mesh()
}

/// Build a bollard mesh: short cylinder.
/// All geometry in prop-local space with base at Y=0.
pub fn build_bollard_mesh() -> PropMesh {
    let mut acc = MeshAccum::new();
    // Cylinder: r=0.75, h=3 at y=1.5
    add_cylinder(&mut acc, [0.0, 1.5, 0.0], 0.75, 3.0, 8);
    acc.into_prop_mesh()
}

/// Simple axis-aligned box helper for furniture props.
fn add_box(acc: &mut MeshAccum, center: [f64; 3], size: [f64; 3]) {
    let hx = size[0] * 0.5;
    let hy = size[1] * 0.5;
    let hz = size[2] * 0.5;
    let cx = center[0];
    let cy = center[1];
    let cz = center[2];

    // 8 corners
    let v = [
        [cx - hx, cy - hy, cz - hz], // 0: left-bottom-back
        [cx + hx, cy - hy, cz - hz], // 1: right-bottom-back
        [cx + hx, cy + hy, cz - hz], // 2: right-top-back
        [cx - hx, cy + hy, cz - hz], // 3: left-top-back
        [cx - hx, cy - hy, cz + hz], // 4: left-bottom-front
        [cx + hx, cy - hy, cz + hz], // 5: right-bottom-front
        [cx + hx, cy + hy, cz + hz], // 6: right-top-front
        [cx - hx, cy + hy, cz + hz], // 7: left-top-front
    ];
    // 6 faces (2 tris each), per-face vertices for flat normals
    let faces: [([usize; 4], [f64; 3]); 6] = [
        ([4, 5, 6, 7], [0.0, 0.0, 1.0]),   // front (+Z)
        ([1, 0, 3, 2], [0.0, 0.0, -1.0]),   // back (-Z)
        ([5, 1, 2, 6], [1.0, 0.0, 0.0]),    // right (+X)
        ([0, 4, 7, 3], [-1.0, 0.0, 0.0]),   // left (-X)
        ([7, 6, 2, 3], [0.0, 1.0, 0.0]),    // top (+Y)
        ([0, 1, 5, 4], [0.0, -1.0, 0.0]),   // bottom (-Y)
    ];
    for (indices, normal) in &faces {
        let v0 = acc.push_vertex(v[indices[0]], *normal);
        let v1 = acc.push_vertex(v[indices[1]], *normal);
        let v2 = acc.push_vertex(v[indices[2]], *normal);
        let v3 = acc.push_vertex(v[indices[3]], *normal);
        acc.triangles.extend_from_slice(&[v0, v1, v2, v0, v2, v3]);
    }
}

// ---------------------------------------------------------------------------
// JSON serialisation (same pattern as RoadMeshStrip::write_json)
// ---------------------------------------------------------------------------

impl PropMesh {
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

    // ── Helper: validate mesh invariants ──────────────────────────────────

    fn assert_mesh_valid(mesh: &PropMesh, label: &str) {
        // Vertices and normals must be the same length
        assert_eq!(
            mesh.vertices.len(),
            mesh.normals.len(),
            "{}: vertex and normal arrays must match in length",
            label
        );

        // Both must be multiples of 3
        assert_eq!(mesh.vertices.len() % 3, 0, "{}: vertices not multiple of 3", label);
        assert_eq!(mesh.triangles.len() % 3, 0, "{}: triangles not multiple of 3", label);

        // Triangle indices must reference valid vertices
        let vc = mesh.vertex_count() as u32;
        for (ti, &idx) in mesh.triangles.iter().enumerate() {
            assert!(
                idx < vc,
                "{}: triangle index [{}] = {} exceeds vertex count {}",
                label, ti, idx, vc
            );
        }

        // Normals must be approximately unit length
        for i in 0..mesh.vertex_count() {
            let nx = mesh.normals[i * 3] as f64;
            let ny = mesh.normals[i * 3 + 1] as f64;
            let nz = mesh.normals[i * 3 + 2] as f64;
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            assert!(
                (len - 1.0).abs() < 0.02,
                "{}: normal at vertex {} has length {}, expected ~1.0",
                label, i, len
            );
        }
    }

    // ── Broadleaf canopy ─────────────────────────────────────────────────

    #[test]
    fn broadleaf_canopy_produces_geometry() {
        let mesh = build_broadleaf_canopy(5.0, 7.0);
        assert!(mesh.vertex_count() > 0, "broadleaf canopy must produce vertices");
        assert!(mesh.triangle_count() > 0, "broadleaf canopy must produce triangles");
        assert_mesh_valid(&mesh, "broadleaf_canopy");
    }

    #[test]
    fn broadleaf_canopy_zero_radius_empty() {
        let mesh = build_broadleaf_canopy(0.0, 7.0);
        assert_eq!(mesh.vertex_count(), 0);
    }

    #[test]
    fn broadleaf_canopy_zero_height_empty() {
        let mesh = build_broadleaf_canopy(5.0, 0.0);
        assert_eq!(mesh.vertex_count(), 0);
    }

    // ── Conifer canopy ───────────────────────────────────────────────────

    #[test]
    fn conifer_canopy_produces_geometry() {
        let mesh = build_conifer_canopy(4.0, 7.0);
        assert!(mesh.vertex_count() > 0, "conifer canopy must produce vertices");
        assert!(mesh.triangle_count() > 0, "conifer canopy must produce triangles");
        assert_mesh_valid(&mesh, "conifer_canopy");
    }

    #[test]
    fn conifer_canopy_zero_radius_empty() {
        let mesh = build_conifer_canopy(0.0, 7.0);
        assert_eq!(mesh.vertex_count(), 0);
    }

    #[test]
    fn conifer_has_three_tiers() {
        // With 8 segments per tier: each cone frustum has
        // 8 side quads (4v each) + 8 top cap tris (3v each) + 8 bottom cap tris (3v each)
        // = 32 + 24 + 24 = 80 vertices per tier
        // But the top tier has top_radius=0 so no top cap: 32 + 0 + 24 = 56
        // Total: 80 + 80 + 56 = 216... but let's just verify it's more than
        // a single tier would produce.
        let one_tier = build_conifer_canopy(4.0, 7.0);
        // The mesh should have geometry from 3 tiers
        assert!(
            one_tier.vertex_count() > 100,
            "conifer should have substantial geometry from 3 tiers, got {} verts",
            one_tier.vertex_count()
        );
    }

    // ── Palm canopy ──────────────────────────────────────────────────────

    #[test]
    fn palm_canopy_produces_geometry() {
        let mesh = build_palm_canopy(4.0, 7.0);
        assert!(mesh.vertex_count() > 0, "palm canopy must produce vertices");
        assert!(mesh.triangle_count() > 0, "palm canopy must produce triangles");
        assert_mesh_valid(&mesh, "palm_canopy");
    }

    #[test]
    fn palm_canopy_has_four_fronds() {
        let mesh = build_palm_canopy(4.0, 7.0);
        // 4 fronds × 2 quads (top + bottom) × 4 verts = 32 vertices
        assert_eq!(mesh.vertex_count(), 32, "palm should have 4 fronds × 2 faces × 4 verts");
    }

    #[test]
    fn palm_canopy_zero_radius_empty() {
        let mesh = build_palm_canopy(0.0, 7.0);
        assert_eq!(mesh.vertex_count(), 0);
    }

    // ── Full tree mesh ───────────────────────────────────────────────────

    #[test]
    fn tree_mesh_broadleaf_produces_geometry() {
        let mesh = build_tree_mesh(6.0, 0.0, 0.0, "broadleaved");
        assert!(mesh.vertex_count() > 0, "broadleaf tree must produce vertices");
        assert_mesh_valid(&mesh, "tree_broadleaf");
    }

    #[test]
    fn tree_mesh_needleleaved_produces_geometry() {
        let mesh = build_tree_mesh(8.0, 0.0, 0.0, "needleleaved");
        assert!(mesh.vertex_count() > 0, "conifer tree must produce vertices");
        assert_mesh_valid(&mesh, "tree_needleleaved");
    }

    #[test]
    fn tree_mesh_palm_produces_geometry() {
        let mesh = build_tree_mesh(10.0, 0.0, 0.0, "palm");
        assert!(mesh.vertex_count() > 0, "palm tree must produce vertices");
        assert_mesh_valid(&mesh, "tree_palm");
    }

    #[test]
    fn tree_mesh_default_height() {
        // height_m = 0 should use default 20-stud tree
        let mesh = build_tree_mesh(0.0, 0.0, 0.0, "broadleaved");
        assert!(mesh.vertex_count() > 0, "default height tree must produce vertices");
        assert_mesh_valid(&mesh, "tree_default_height");
    }

    #[test]
    fn tree_mesh_custom_radii() {
        let mesh = build_tree_mesh(6.0, 3.0, 0.8, "broadleaved");
        assert!(mesh.vertex_count() > 0);
        assert_mesh_valid(&mesh, "tree_custom_radii");
    }

    #[test]
    fn tree_mesh_has_trunk_and_canopy() {
        // A trunk cylinder with 8 segments has:
        //   8 side quads (32v) + 8 top tris (24v) + 8 bottom tris (24v) = 80 verts
        // The canopy adds more on top.
        let trunk_only_verts = 80;
        let mesh = build_tree_mesh(6.0, 0.0, 0.0, "broadleaved");
        assert!(
            mesh.vertex_count() > trunk_only_verts,
            "tree should have trunk + canopy, got {} verts (trunk alone = {})",
            mesh.vertex_count(),
            trunk_only_verts
        );
    }

    // ── resolve_leaf_type ────────────────────────────────────────────────

    #[test]
    fn resolve_leaf_type_explicit() {
        assert_eq!(resolve_leaf_type(Some("needleleaved"), None), "needleleaved");
        assert_eq!(resolve_leaf_type(Some("tropical"), None), "palm");
        assert_eq!(resolve_leaf_type(Some("palm"), None), "palm");
        assert_eq!(resolve_leaf_type(Some("broadleaved"), None), "broadleaved");
    }

    #[test]
    fn resolve_leaf_type_from_species() {
        assert_eq!(resolve_leaf_type(None, Some("Pinus sylvestris")), "needleleaved");
        assert_eq!(resolve_leaf_type(None, Some("palm")), "palm");
        assert_eq!(resolve_leaf_type(None, Some("Quercus")), "broadleaved");
    }

    #[test]
    fn resolve_leaf_type_default() {
        assert_eq!(resolve_leaf_type(None, None), "broadleaved");
        assert_eq!(resolve_leaf_type(Some(""), None), "broadleaved");
    }

    // ── JSON round-trip ──────────────────────────────────────────────────

    #[test]
    fn prop_mesh_json_contains_expected_keys() {
        let mesh = build_tree_mesh(6.0, 0.0, 0.0, "broadleaved");
        let mut json = String::new();
        mesh.write_json(&mut json, 0);
        assert!(json.contains("\"vertices\""), "JSON must contain vertices key");
        assert!(json.contains("\"triangles\""), "JSON must contain triangles key");
        assert!(json.contains("\"normals\""), "JSON must contain normals key");
    }

    #[test]
    fn prop_mesh_json_array_counts_match() {
        let mesh = build_tree_mesh(6.0, 0.0, 0.0, "broadleaved");
        let expected_verts = mesh.vertices.len();
        let expected_tris = mesh.triangles.len();
        let expected_norms = mesh.normals.len();

        let mut json = String::new();
        mesh.write_json(&mut json, 0);

        let count_elements = |key: &str| -> usize {
            let key_str = format!("\"{}\"", key);
            let kpos = json.find(&key_str).unwrap();
            let arr_open = json[kpos..].find('[').unwrap() + kpos;
            let arr_close = json[arr_open..].find(']').unwrap() + arr_open;
            let inner = json[arr_open + 1..arr_close].trim();
            if inner.is_empty() { 0 } else { inner.split(',').count() }
        };

        assert_eq!(count_elements("vertices"), expected_verts);
        assert_eq!(count_elements("triangles"), expected_tris);
        assert_eq!(count_elements("normals"), expected_norms);
    }

    // ── Cylinder geometry ────────────────────────────────────────────────

    #[test]
    fn cylinder_vertex_count() {
        let mut acc = MeshAccum::new();
        add_cylinder(&mut acc, [0.0, 0.0, 0.0], 1.0, 2.0, 8);
        let mesh = acc.into_prop_mesh();
        // 8 side quads (32v) + 8 top tris (24v) + 8 bottom tris (24v) = 80
        assert_eq!(mesh.vertex_count(), 80);
        assert_mesh_valid(&mesh, "cylinder");
    }

    // ── Ellipsoid geometry ───────────────────────────────────────────────

    #[test]
    fn ellipsoid_vertex_count() {
        let mut acc = MeshAccum::new();
        add_ellipsoid(&mut acc, [0.0, 5.0, 0.0], 3.0, 2.0, 3.0, 6, 8);
        let mesh = acc.into_prop_mesh();
        // Rings=6, Segments=8:
        // Top cap: 8 triangles (24v)
        // Bottom cap: 8 triangles (24v)
        // Middle 4 rings: 4 × 8 quads (4v each) = 128v
        // Total: 24 + 24 + 128 = 176
        assert_eq!(mesh.vertex_count(), 176);
        assert_mesh_valid(&mesh, "ellipsoid");
    }

    // ── Cone frustum geometry ────────────────────────────────────────────

    #[test]
    fn cone_frustum_vertex_count() {
        let mut acc = MeshAccum::new();
        add_cone_frustum(&mut acc, [0.0, 5.0, 0.0], 3.0, 1.0, 4.0, 8);
        let mesh = acc.into_prop_mesh();
        // 8 side quads (32v) + 8 top tris (24v) + 8 bottom tris (24v) = 80
        assert_eq!(mesh.vertex_count(), 80);
        assert_mesh_valid(&mesh, "cone_frustum");
    }

    #[test]
    fn cone_frustum_pointed_top() {
        let mut acc = MeshAccum::new();
        add_cone_frustum(&mut acc, [0.0, 5.0, 0.0], 3.0, 0.0, 4.0, 8);
        let mesh = acc.into_prop_mesh();
        // top_radius = 0 → no top cap
        // 8 side quads (32v) + 0 top + 8 bottom tris (24v) = 56
        assert_eq!(mesh.vertex_count(), 56);
        assert_mesh_valid(&mesh, "cone_pointed");
    }
}
