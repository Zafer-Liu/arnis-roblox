//! Wall surfaces with window and door openings — ported from osm2world's
//! `ExteriorBuildingWall` + `WallSurface` + `WindowImplementation`.
//!
//! Takes a building footprint polygon and vertical parameters, produces a
//! `PrecomputedMesh` with thick walls, punched window openings per floor band,
//! and a door on the ground floor of the longest wall.

use crate::building_atlas::AtlasUv;
use crate::mesh_builder::PrecomputedMesh;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Default window width (metres).
const WINDOW_WIDTH: f64 = 1.2;
/// Default window height (metres).
const WINDOW_HEIGHT: f64 = 1.5;
/// Default door width (metres).
const DOOR_WIDTH: f64 = 1.4;
/// Default door height (metres).
const DOOR_HEIGHT: f64 = 2.2;
/// Vertical position of window bottom within a floor band (fraction of floor_height).
const WINDOW_SILL_FRAC: f64 = 0.30;
/// Horizontal inset from wall edge for first/last window (metres).
const WINDOW_H_MARGIN: f64 = 0.8;

// ---------------------------------------------------------------------------
// Vector helpers (3-component f64)
// ---------------------------------------------------------------------------

type V3 = [f64; 3];

#[allow(dead_code)]
fn v3_sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn v3_add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn v3_scale(v: V3, s: f64) -> V3 {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
fn v3_cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn v3_normalize(v: V3) -> V3 {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn v3_neg(v: V3) -> V3 {
    [-v[0], -v[1], -v[2]]
}

// ---------------------------------------------------------------------------
// Mesh accumulator (local, lightweight)
// ---------------------------------------------------------------------------

struct MeshAccum {
    vertices: Vec<f32>,
    normals: Vec<f32>,
    triangles: Vec<u32>,
    uvs: Vec<f32>,
}

impl MeshAccum {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            triangles: Vec::new(),
            uvs: Vec::new(),
        }
    }

    fn vert_count(&self) -> u32 {
        (self.vertices.len() / 3) as u32
    }

    fn push_vert(&mut self, pos: V3, normal: V3) -> u32 {
        let idx = self.vert_count();
        self.vertices.extend_from_slice(&[pos[0] as f32, pos[1] as f32, pos[2] as f32]);
        self.normals.extend_from_slice(&[normal[0] as f32, normal[1] as f32, normal[2] as f32]);
        idx
    }

    fn push_vert_uv(&mut self, pos: V3, normal: V3, u: f32, v: f32) -> u32 {
        let idx = self.push_vert(pos, normal);
        self.uvs.push(u);
        self.uvs.push(v);
        idx
    }

    /// Emit a quad (two CCW triangles). Caller supplies the shared face normal.
    fn add_quad(&mut self, p0: V3, p1: V3, p2: V3, p3: V3, normal: V3) {
        let v0 = self.push_vert(p0, normal);
        let v1 = self.push_vert(p1, normal);
        let v2 = self.push_vert(p2, normal);
        let v3 = self.push_vert(p3, normal);
        self.triangles.extend_from_slice(&[v0, v1, v2, v0, v2, v3]);
    }

    /// Emit a quad with per-vertex UV coordinates.
    fn add_quad_uv(
        &mut self,
        p0: V3, p1: V3, p2: V3, p3: V3,
        normal: V3,
        uv0: [f32; 2], uv1: [f32; 2], uv2: [f32; 2], uv3: [f32; 2],
    ) {
        let v0 = self.push_vert_uv(p0, normal, uv0[0], uv0[1]);
        let v1 = self.push_vert_uv(p1, normal, uv1[0], uv1[1]);
        let v2 = self.push_vert_uv(p2, normal, uv2[0], uv2[1]);
        let v3 = self.push_vert_uv(p3, normal, uv3[0], uv3[1]);
        self.triangles.extend_from_slice(&[v0, v1, v2, v0, v2, v3]);
    }

    fn to_mesh(self) -> PrecomputedMesh {
        PrecomputedMesh {
            vertices: self.vertices,
            triangles: self.triangles,
            normals: self.normals,
            uvs: self.uvs,
        }
    }
}

// ---------------------------------------------------------------------------
// Opening description
// ---------------------------------------------------------------------------

/// A rectangular opening punched into a wall face.
struct Opening {
    /// Horizontal centre along the wall edge (metres from edge start).
    h_center: f64,
    /// Width of the opening.
    width: f64,
    /// Bottom of opening (Y, relative to base_y).
    bottom_y: f64,
    /// Height of the opening.
    height: f64,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Atlas UV mapping helper
// ---------------------------------------------------------------------------

/// Per-edge UV mapping context: maps a wall edge's local (h, v) coordinates
/// into the building's atlas sub-rect. `u_start`/`u_end` are the fraction of
/// the total perimeter that this edge spans (both in 0..1 range).
struct EdgeUv<'a> {
    u_start: f64,
    u_end: f64,
    atlas: &'a AtlasUv,
}

impl EdgeUv<'_> {
    /// Convert local wall coordinates (h along edge in metres, v = height from
    /// base in metres) into atlas UV coordinates.
    fn map(&self, h_frac: f64, v_frac: f64) -> [f32; 2] {
        // Interpolate within the edge's perimeter fraction.
        let u_norm = self.u_start + (self.u_end - self.u_start) * h_frac;
        // Remap [0,1] normalised coords into the atlas sub-rect.
        let u = self.atlas.uv_x + u_norm as f32 * self.atlas.uv_width;
        let v = self.atlas.uv_y + v_frac as f32 * self.atlas.uv_height;
        [u, v]
    }
}

/// Generate wall geometry for a building with window/door openings.
///
/// `footprint` is a closed polygon in XZ (Y-up), wound counter-clockwise when
/// viewed from above. The last point must NOT duplicate the first.
///
/// When `atlas_uv` is provided, the outer wall surfaces are UV-mapped into the
/// building's atlas rect. The U axis wraps around the perimeter (0..1 over the
/// full perimeter), and V goes from bottom (0) to top (1) of the wall. Both
/// are then remapped into the atlas sub-rect so the Roblox runtime can apply
/// the chunk atlas texture directly.
///
/// Returns a `PrecomputedMesh` with thick walls (extruded inward by
/// `wall_thickness`), window openings per floor band, and one door on the
/// ground floor of the longest wall.
pub fn generate_wall_mesh(
    footprint: &[(f64, f64)],
    base_y: f64,
    wall_height: f64,
    level_count: u32,
    floor_height: f64,
    wall_thickness: f64,
) -> PrecomputedMesh {
    generate_wall_mesh_with_atlas(footprint, base_y, wall_height, level_count, floor_height, wall_thickness, None)
}

/// Like `generate_wall_mesh` but accepts an optional atlas UV rect for
/// texture-mapped facades.
pub fn generate_wall_mesh_with_atlas(
    footprint: &[(f64, f64)],
    base_y: f64,
    wall_height: f64,
    level_count: u32,
    floor_height: f64,
    wall_thickness: f64,
    atlas_uv: Option<&AtlasUv>,
) -> PrecomputedMesh {
    if footprint.len() < 3 || wall_height <= 0.0 || level_count == 0 {
        return PrecomputedMesh {
            vertices: Vec::new(),
            triangles: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
        };
    }

    // Compute total perimeter for UV mapping.
    let perimeter: f64 = {
        let n = footprint.len();
        (0..n)
            .map(|i| {
                let j = (i + 1) % n;
                let dx = footprint[j].0 - footprint[i].0;
                let dz = footprint[j].1 - footprint[i].1;
                (dx * dx + dz * dz).sqrt()
            })
            .sum()
    };

    let mut accum = MeshAccum::new();

    // Centroid for inward-normal computation.
    let (cx, cz) = centroid(footprint);

    // Find longest edge index (for door placement).
    let longest_idx = longest_edge_index(footprint);

    let n = footprint.len();
    let frame_depth = (wall_thickness / 4.0).max(0.02);

    let mut cumulative_perim = 0.0f64;

    for i in 0..n {
        let j = (i + 1) % n;
        let (ax, az) = footprint[i];
        let (bx, bz) = footprint[j];

        let edge_dx = bx - ax;
        let edge_dz = bz - az;
        let edge_len = (edge_dx * edge_dx + edge_dz * edge_dz).sqrt();
        if edge_len < 1e-6 {
            continue;
        }

        // Tangent along edge (XZ plane, Y=0).
        let tangent = v3_normalize([edge_dx, 0.0, edge_dz]);
        // Outward normal: rotate tangent 90 deg CW in XZ.
        let mut outward = [tangent[2], 0.0, -tangent[0]];
        // Make sure it actually points away from centroid.
        // `to_mid` is the vector from centroid to edge midpoint (outward direction).
        // If `outward` dot `to_mid` is NEGATIVE, outward points inward — flip it.
        let to_mid = [(ax + bx) / 2.0 - cx, 0.0, (az + bz) / 2.0 - cz];
        if dot3(outward, to_mid) < 0.0 {
            outward = v3_neg(outward);
        }
        let inward = v3_neg(outward);

        // Outer and inner base corners (bottom).
        let outer_bl = [ax, base_y, az];
        let outer_br = [bx, base_y, bz];
        let inner_bl = v3_add(outer_bl, v3_scale(inward, wall_thickness));
        let inner_br = v3_add(outer_br, v3_scale(inward, wall_thickness));

        // Compute openings for this wall face.
        let is_door_wall = i == longest_idx;
        let openings = compute_openings(edge_len, level_count, floor_height, is_door_wall);

        // UV mapping parameters for this edge (fraction of perimeter).
        let edge_uv = atlas_uv.map(|uv| EdgeUv {
            u_start: cumulative_perim / perimeter,
            u_end: (cumulative_perim + edge_len) / perimeter,
            atlas: uv,
        });
        cumulative_perim += edge_len;

        // --- Outer face with openings ---
        emit_wall_face_with_openings(
            &mut accum,
            outer_bl,
            tangent,
            outward,
            edge_len,
            wall_height,
            &openings,
            frame_depth,
            inward,
            edge_uv.as_ref(),
        );

        // --- Inner face (flat, no openings — interior walls are plain) ---
        {
            let inner_tl = v3_add(inner_bl, [0.0, wall_height, 0.0]);
            let inner_tr = v3_add(inner_br, [0.0, wall_height, 0.0]);
            accum.add_quad(inner_br, inner_bl, inner_tl, inner_tr, v3_neg(inward));
        }

        // --- Top cap (connects outer top to inner top) ---
        {
            let outer_tl = v3_add(outer_bl, [0.0, wall_height, 0.0]);
            let outer_tr = v3_add(outer_br, [0.0, wall_height, 0.0]);
            let inner_tl = v3_add(inner_bl, [0.0, wall_height, 0.0]);
            let inner_tr = v3_add(inner_br, [0.0, wall_height, 0.0]);
            let up = [0.0, 1.0, 0.0];
            accum.add_quad(outer_tl, outer_tr, inner_tr, inner_tl, up);
        }

        // --- Bottom cap ---
        {
            let down = [0.0, -1.0, 0.0];
            accum.add_quad(outer_br, outer_bl, inner_bl, inner_br, down);
        }
    }

    accum.to_mesh()
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn centroid(pts: &[(f64, f64)]) -> (f64, f64) {
    let n = pts.len() as f64;
    let sx: f64 = pts.iter().map(|p| p.0).sum();
    let sz: f64 = pts.iter().map(|p| p.1).sum();
    (sx / n, sz / n)
}

fn longest_edge_index(pts: &[(f64, f64)]) -> usize {
    let n = pts.len();
    let mut best = 0;
    let mut best_len = 0.0f64;
    for i in 0..n {
        let j = (i + 1) % n;
        let dx = pts[j].0 - pts[i].0;
        let dz = pts[j].1 - pts[i].1;
        let l = (dx * dx + dz * dz).sqrt();
        if l > best_len {
            best_len = l;
            best = i;
        }
    }
    best
}

fn dot3(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Compute openings (windows + optional door) for one wall face.
fn compute_openings(
    edge_len: f64,
    level_count: u32,
    floor_height: f64,
    is_door_wall: bool,
) -> Vec<Opening> {
    let mut out = Vec::new();

    for lvl in 0..level_count {
        let band_bottom = lvl as f64 * floor_height;
        let is_ground = lvl == 0;

        if is_ground && is_door_wall {
            // Door centred on wall.
            if edge_len >= DOOR_WIDTH + 2.0 * WINDOW_H_MARGIN {
                out.push(Opening {
                    h_center: edge_len / 2.0,
                    width: DOOR_WIDTH,
                    bottom_y: band_bottom,
                    height: DOOR_HEIGHT.min(floor_height * 0.9),
                });
            }
            // Windows on either side of door on ground floor.
            let remaining_left = edge_len / 2.0 - DOOR_WIDTH / 2.0;
            let remaining_right = remaining_left;
            add_windows_in_span(
                &mut out,
                WINDOW_H_MARGIN,
                remaining_left - WINDOW_H_MARGIN,
                band_bottom,
                floor_height,
            );
            add_windows_in_span(
                &mut out,
                edge_len / 2.0 + DOOR_WIDTH / 2.0 + WINDOW_H_MARGIN,
                edge_len - WINDOW_H_MARGIN,
                band_bottom,
                floor_height,
            );
            let _ = remaining_right; // symmetry handled above
        } else {
            // Regular windows across the wall.
            add_windows_in_span(
                &mut out,
                WINDOW_H_MARGIN,
                edge_len - WINDOW_H_MARGIN,
                band_bottom,
                floor_height,
            );
        }
    }

    out
}

/// Place windows evenly within a horizontal span [h_start, h_end].
fn add_windows_in_span(
    out: &mut Vec<Opening>,
    h_start: f64,
    h_end: f64,
    band_bottom: f64,
    floor_height: f64,
) {
    let span = h_end - h_start;
    if span < WINDOW_WIDTH {
        return;
    }
    // How many windows fit? Spacing ~ window_width * 2.
    let spacing = WINDOW_WIDTH * 2.0;
    let count = ((span + spacing - WINDOW_WIDTH) / spacing).floor().max(1.0) as u32;
    let actual_spacing = span / count as f64;

    let sill_y = band_bottom + floor_height * WINDOW_SILL_FRAC;
    let win_h = WINDOW_HEIGHT.min(floor_height * 0.55);

    for k in 0..count {
        let center = h_start + actual_spacing * (k as f64 + 0.5);
        out.push(Opening {
            h_center: center,
            width: WINDOW_WIDTH,
            bottom_y: sill_y,
            height: win_h,
        });
    }
}

/// Emit the outer wall face for one edge, with rectangular openings punched
/// out and frame-depth inset panes.
///
/// The wall face spans from `origin` along `tangent` for `edge_len`, upward
/// for `wall_height`. Openings are expressed in local (h, v) coordinates:
/// h along tangent, v = Y from origin.
///
/// When `edge_uv` is provided, outer wall strips and recessed panes emit
/// per-vertex UV coordinates mapped into the atlas sub-rect. Inner faces
/// (frame jambs, caps) use `add_quad` without UVs — they are not textured.
fn emit_wall_face_with_openings(
    accum: &mut MeshAccum,
    origin: V3,   // bottom-left of wall face (outer surface)
    tangent: V3,  // unit direction along edge
    outward: V3,  // outward-facing normal
    edge_len: f64,
    wall_height: f64,
    openings: &[Opening],
    frame_depth: f64,
    inward: V3,
    edge_uv: Option<&EdgeUv>,
) {
    // Strategy: subdivide the outer wall face into border strips around each
    // opening so the outer surface never overlaps a window/door pane.  This
    // eliminates z-fighting that the previous full-quad approach caused.
    //
    // We sort openings left-to-right, then emit solid strips:
    //   - Horizontal bands above, below, and between openings.
    //   - Vertical strips left of the first, between each pair, and right of
    //     the last opening within each horizontal band.
    //
    // For each opening we still emit the recessed pane + four frame jambs.

    // Collect opening left/right/top/bottom edges in local (h, v) coords.
    struct Rect {
        left: f64,
        right: f64,
        bottom: f64,
        top: f64,
    }
    let mut rects: Vec<Rect> = openings
        .iter()
        .map(|op| {
            let half_w = op.width / 2.0;
            Rect {
                left: op.h_center - half_w,
                right: op.h_center + half_w,
                bottom: op.bottom_y,
                top: op.bottom_y + op.height,
            }
        })
        .collect();
    rects.sort_by(|a, b| a.left.partial_cmp(&b.left).unwrap());

    // Helper: emit an outer-surface quad given local (h0..h1, v0..v1).
    // When atlas UVs are available, emits UV-mapped quads; otherwise plain.
    let emit_strip = |acc: &mut MeshAccum, h0: f64, h1: f64, v0: f64, v1: f64| {
        if h1 - h0 < 1e-6 || v1 - v0 < 1e-6 {
            return;
        }
        let p_bl = v3_add(v3_add(origin, v3_scale(tangent, h0)), [0.0, v0, 0.0]);
        let p_br = v3_add(v3_add(origin, v3_scale(tangent, h1)), [0.0, v0, 0.0]);
        let p_tl = v3_add(v3_add(origin, v3_scale(tangent, h0)), [0.0, v1, 0.0]);
        let p_tr = v3_add(v3_add(origin, v3_scale(tangent, h1)), [0.0, v1, 0.0]);
        if let Some(euv) = edge_uv {
            let h0f = h0 / edge_len;
            let h1f = h1 / edge_len;
            let v0f = v0 / wall_height;
            let v1f = v1 / wall_height;
            acc.add_quad_uv(
                p_bl, p_br, p_tr, p_tl, outward,
                euv.map(h0f, v0f), euv.map(h1f, v0f),
                euv.map(h1f, v1f), euv.map(h0f, v1f),
            );
        } else {
            acc.add_quad(p_bl, p_br, p_tr, p_tl, outward);
        }
    };

    if rects.is_empty() {
        // No openings — emit the full wall quad.
        emit_strip(&mut *accum, 0.0, edge_len, 0.0, wall_height);
    } else {
        // Collect unique horizontal bands (v-coords) across all openings.
        let mut v_cuts: Vec<f64> = vec![0.0, wall_height];
        for r in &rects {
            v_cuts.push(r.bottom);
            v_cuts.push(r.top);
        }
        v_cuts.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v_cuts.dedup_by(|a, b| (*a - *b).abs() < 1e-6);

        // For each horizontal band, emit solid strips around the openings
        // that intersect this band.
        for vi in 0..v_cuts.len() - 1 {
            let v0 = v_cuts[vi];
            let v1 = v_cuts[vi + 1];

            // Openings that overlap this vertical band.
            let band_openings: Vec<&Rect> = rects
                .iter()
                .filter(|r| r.bottom < v1 - 1e-6 && r.top > v0 + 1e-6)
                .collect();

            if band_openings.is_empty() {
                // Full-width solid strip for this band.
                emit_strip(&mut *accum, 0.0, edge_len, v0, v1);
            } else {
                // Strips between openings.
                let mut cursor = 0.0;
                for op in &band_openings {
                    if op.left > cursor + 1e-6 {
                        emit_strip(&mut *accum, cursor, op.left, v0, v1);
                    }
                    cursor = op.right;
                }
                if cursor < edge_len - 1e-6 {
                    emit_strip(&mut *accum, cursor, edge_len, v0, v1);
                }
            }
        }
    }

    // Per-opening: recessed pane + frame geometry.
    let up = [0.0, 1.0, 0.0];
    let down = [0.0, -1.0, 0.0];
    for op in openings {
        let half_w = op.width / 2.0;
        let left_h = op.h_center - half_w;
        let right_h = op.h_center + half_w;
        let bot_v = op.bottom_y;
        let top_v = op.bottom_y + op.height;

        // Outer rectangle corners (on outer surface).
        let o_bl = v3_add(v3_add(origin, v3_scale(tangent, left_h)), [0.0, bot_v, 0.0]);
        let o_br = v3_add(v3_add(origin, v3_scale(tangent, right_h)), [0.0, bot_v, 0.0]);
        let o_tl = v3_add(v3_add(origin, v3_scale(tangent, left_h)), [0.0, top_v, 0.0]);
        let o_tr = v3_add(v3_add(origin, v3_scale(tangent, right_h)), [0.0, top_v, 0.0]);

        let offset = v3_scale(inward, frame_depth);

        // Recessed pane (set back into wall).
        let i_bl = v3_add(o_bl, offset);
        let i_br = v3_add(o_br, offset);
        let i_tl = v3_add(o_tl, offset);
        let i_tr = v3_add(o_tr, offset);

        // Pane face (faces outward) — UV-mapped to the same atlas region as
        // the outer wall so the texture stays consistent through the reveal.
        if let Some(euv) = edge_uv {
            let lf = left_h / edge_len;
            let rf = right_h / edge_len;
            let bf = bot_v / wall_height;
            let tf = top_v / wall_height;
            accum.add_quad_uv(
                i_bl, i_br, i_tr, i_tl, outward,
                euv.map(lf, bf), euv.map(rf, bf),
                euv.map(rf, tf), euv.map(lf, tf),
            );
        } else {
            accum.add_quad(i_bl, i_br, i_tr, i_tl, outward);
        }

        // Frame jambs — four quads connecting outer edge to recessed pane.
        // These are non-textured (inner geometry).
        // Bottom sill (faces down into opening).
        accum.add_quad(o_bl, o_br, i_br, i_bl, down);
        // Top lintel (faces up into opening).
        accum.add_quad(o_tr, o_tl, i_tl, i_tr, up);
        // Left jamb (faces right into opening).
        let left_normal = v3_neg(tangent);
        accum.add_quad(o_tl, o_bl, i_bl, i_tl, left_normal);
        // Right jamb (faces left into opening).
        accum.add_quad(o_br, o_tr, i_tr, i_br, tangent);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple square footprint (10m x 10m) centered at origin.
    fn square_footprint() -> Vec<(f64, f64)> {
        vec![
            (-5.0, -5.0),
            (5.0, -5.0),
            (5.0, 5.0),
            (-5.0, 5.0),
        ]
    }

    #[test]
    fn test_generate_wall_mesh_basic() {
        let mesh = generate_wall_mesh(
            &square_footprint(),
            0.0,
            10.5, // wall height
            3,    // 3 levels
            3.5,
            0.3, // wall thickness
        );
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
        // Every vertex should have a matching normal.
        assert_eq!(mesh.vertices.len(), mesh.normals.len());
        // Triangle indices must be in range.
        let vc = mesh.vertex_count() as u32;
        for &idx in &mesh.triangles {
            assert!(idx < vc, "triangle index {idx} out of range (vc={vc})");
        }
    }

    #[test]
    fn test_vertex_count_exceeds_walls_only_baseline() {
        // Walls-only baseline: 4 edges x (outer + inner + top cap + bottom cap)
        // = 4 x 4 quads = 16 quads x 4 verts = 64 verts.
        let walls_only_baseline = 64;

        let mesh = generate_wall_mesh(&square_footprint(), 0.0, 10.5, 3, 3.5, 0.3);
        assert!(
            mesh.vertex_count() > walls_only_baseline,
            "vertex count {} should exceed walls-only baseline {}",
            mesh.vertex_count(),
            walls_only_baseline,
        );
    }

    #[test]
    fn test_degenerate_inputs() {
        // Too few points.
        let mesh = generate_wall_mesh(&[(0.0, 0.0), (1.0, 0.0)], 0.0, 10.0, 3, 3.5, 0.3);
        assert_eq!(mesh.vertex_count(), 0);

        // Zero wall height.
        let mesh = generate_wall_mesh(&square_footprint(), 0.0, 0.0, 3, 3.5, 0.3);
        assert_eq!(mesh.vertex_count(), 0);

        // Zero levels.
        let mesh = generate_wall_mesh(&square_footprint(), 0.0, 10.0, 0, 3.5, 0.3);
        assert_eq!(mesh.vertex_count(), 0);
    }

    #[test]
    fn test_triangle_footprint() {
        let tri = vec![(0.0, 0.0), (10.0, 0.0), (5.0, 8.66)];
        let mesh = generate_wall_mesh(&tri, 0.0, 7.0, 2, 3.5, 0.25);
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_single_level_no_windows_on_short_wall() {
        // Very short edge — no room for windows.
        let tiny = vec![(0.0, 0.0), (0.5, 0.0), (0.5, 0.5), (0.0, 0.5)];
        let mesh = generate_wall_mesh(&tiny, 0.0, 3.5, 1, 3.5, 0.1);
        // Should still produce wall geometry (just no openings).
        assert!(mesh.vertex_count() > 0);
    }

    #[test]
    fn test_atlas_uv_produces_uvs() {
        let uv = AtlasUv {
            uv_x: 0.0,
            uv_y: 0.0,
            uv_width: 0.25,
            uv_height: 0.25,
        };
        let mesh = generate_wall_mesh_with_atlas(
            &square_footprint(),
            0.0,
            10.5,
            3,
            3.5,
            0.3,
            Some(&uv),
        );
        assert!(mesh.vertex_count() > 0);
        // UVs should be present for UV-mapped vertices (outer walls + panes).
        assert!(!mesh.uvs.is_empty(), "atlas UV should produce non-empty uvs");
        // UVs come in pairs.
        assert_eq!(mesh.uvs.len() % 2, 0);
        // All UV values should be within the atlas rect bounds (with small epsilon).
        for (i, &val) in mesh.uvs.iter().enumerate() {
            assert!(
                val >= -0.001 && val <= 1.001,
                "UV[{i}] = {val} out of [0,1] range"
            );
        }
    }

    #[test]
    fn test_no_atlas_produces_empty_uvs() {
        let mesh = generate_wall_mesh(
            &square_footprint(),
            0.0,
            10.5,
            3,
            3.5,
            0.3,
        );
        assert!(mesh.uvs.is_empty(), "no atlas should produce empty uvs");
    }
}
