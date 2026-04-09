//! Compile-time terrain heightfield mesh generation.
//!
//! Generates a regular-grid mesh from per-cell terrain heights and materials.
//! The mesh can be loaded as a MeshPart heightfield by the Lua importer,
//! providing an alternative rendering path to runtime `FillBlock` calls.

use std::fmt::Write as _;

/// Pre-computed terrain heightfield mesh: flat interleaved arrays matching
/// the Roblox EditableMesh vertex/triangle layout (same layout as
/// `PrecomputedMesh`, `RoadMeshStrip`, and `PropMesh`), plus per-vertex
/// material indices for satellite-derived surface materials.
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainMesh {
    /// Flat [x,y,z, x,y,z, ...] vertex positions.
    pub vertices: Vec<f32>,
    /// Flat [v0,v1,v2, v0,v1,v2, ...] triangle indices (0-based).
    pub triangles: Vec<u32>,
    /// Flat [nx,ny,nz, nx,ny,nz, ...] per-vertex normals.
    pub normals: Vec<f32>,
    /// Per-vertex material index (indexes into the material palette).
    /// Length equals `vertices.len() / 3`.
    pub material_indices: Vec<u32>,
    /// Material palette: unique material names. `material_indices` entries
    /// are offsets into this array.
    pub material_palette: Vec<String>,
}

impl TerrainMesh {
    /// Number of vertices in this mesh.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / 3
    }

    /// Number of triangles in this mesh.
    pub fn triangle_count(&self) -> usize {
        self.triangles.len() / 3
    }
}

// ---------------------------------------------------------------------------
// Linear-algebra helpers
// ---------------------------------------------------------------------------

type V3 = [f64; 3];

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
        return [0.0, 1.0, 0.0]; // degenerate fallback: point up
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Default terrain cell size in studs (WorldConfig.VoxelSize * 4).
pub const DEFAULT_CELL_SIZE: f64 = 4.0;

/// Build a heightfield mesh from a regular grid of terrain cells.
///
/// The grid is `width` columns by `depth` rows.  `heights` is a flat
/// row-major array of `width * depth` cell-centre heights.  The mesh
/// places vertices at cell *corners*, interpolating heights from the four
/// neighbouring cells.  Each cell produces one quad (2 triangles).
///
/// # Arguments
/// * `heights` - row-major `[depth][width]` cell heights.
/// * `width`   - number of columns.
/// * `depth`   - number of rows.
/// * `cell_size` - horizontal size of each cell in studs (0 => default 4.0).
/// * `materials` - optional per-cell material names (same length as `heights`).
///
/// Returns `None` if the grid is empty (width or depth is 0) or `heights`
/// length does not match `width * depth`.
pub fn build_terrain_mesh(
    heights: &[f64],
    width: usize,
    depth: usize,
    cell_size: f64,
    materials: Option<&[String]>,
) -> Option<TerrainMesh> {
    if width == 0 || depth == 0 {
        return None;
    }
    if heights.len() != width * depth {
        return None;
    }

    let cs = if cell_size <= 0.0 { DEFAULT_CELL_SIZE } else { cell_size };

    // -------------------------------------------------------------------
    // 1. Build corner vertex grid: (width+1) x (depth+1) vertices.
    //    Each corner height is the average of the adjacent (up to 4) cells.
    // -------------------------------------------------------------------
    let vw = width + 1; // vertex-grid width
    let vh = depth + 1; // vertex-grid height
    let num_verts = vw * vh;

    let mut corner_heights = vec![0.0_f64; num_verts];
    let mut corner_counts = vec![0u32; num_verts];

    for row in 0..depth {
        for col in 0..width {
            let h = heights[row * width + col];
            // Cell (row, col) touches corners (row,col), (row,col+1),
            // (row+1,col), (row+1,col+1).
            for &(dr, dc) in &[(0, 0), (0, 1), (1, 0), (1, 1)] {
                let vi = (row + dr) * vw + (col + dc);
                corner_heights[vi] += h;
                corner_counts[vi] += 1;
            }
        }
    }
    for i in 0..num_verts {
        if corner_counts[i] > 0 {
            corner_heights[i] /= corner_counts[i] as f64;
        }
    }

    // -------------------------------------------------------------------
    // 2. Compute per-vertex normals from height differences.
    // -------------------------------------------------------------------
    let mut corner_normals: Vec<V3> = vec![[0.0, 1.0, 0.0]; num_verts];
    for vy in 0..vh {
        for vx in 0..vw {
            let vi = vy * vw + vx;
            let h = corner_heights[vi];

            // Finite differences using neighbouring corner heights.
            let hx_left = if vx > 0 { corner_heights[vi - 1] } else { h };
            let hx_right = if vx + 1 < vw { corner_heights[vi + 1] } else { h };
            let hz_up = if vy > 0 { corner_heights[vi - vw] } else { h };
            let hz_down = if vy + 1 < vh { corner_heights[vi + vw] } else { h };

            // Tangent vectors in X and Z directions.
            let dx_span = if vx > 0 && vx + 1 < vw { 2.0 * cs } else { cs };
            let dz_span = if vy > 0 && vy + 1 < vh { 2.0 * cs } else { cs };

            let tangent_x: V3 = [dx_span, hx_right - hx_left, 0.0];
            let tangent_z: V3 = [0.0, hz_down - hz_up, dz_span];

            corner_normals[vi] = v3_normalize(v3_cross(tangent_z, tangent_x));
        }
    }

    // -------------------------------------------------------------------
    // 3. Build material palette and per-cell material index lookup.
    // -------------------------------------------------------------------
    let mut palette: Vec<String> = Vec::new();
    let cell_mat_indices: Vec<u32> = if let Some(mats) = materials {
        mats.iter()
            .map(|m| {
                if let Some(pos) = palette.iter().position(|p| p == m) {
                    pos as u32
                } else {
                    palette.push(m.clone());
                    (palette.len() - 1) as u32
                }
            })
            .collect()
    } else {
        palette.push("Grass".to_string());
        vec![0u32; width * depth]
    };

    // -------------------------------------------------------------------
    // 4. Emit per-cell quads.  Each cell uses *shared* corner vertices
    //    (indexed by corner grid position).
    // -------------------------------------------------------------------
    // Pre-allocate flat buffers.
    let mut vertices: Vec<f32> = Vec::with_capacity(num_verts * 3);
    let mut normals: Vec<f32> = Vec::with_capacity(num_verts * 3);
    let mut vert_material_indices: Vec<u32> = Vec::with_capacity(num_verts);

    // Emit all corner vertices.
    for vy in 0..vh {
        for vx in 0..vw {
            let vi = vy * vw + vx;
            let x = vx as f64 * cs;
            let y = corner_heights[vi];
            let z = vy as f64 * cs;
            vertices.push(x as f32);
            vertices.push(y as f32);
            vertices.push(z as f32);
            let n = corner_normals[vi];
            normals.push(n[0] as f32);
            normals.push(n[1] as f32);
            normals.push(n[2] as f32);

            // Vertex material: pick the material of the most common adjacent
            // cell.  For simplicity use the cell to the upper-left when it
            // exists, else the nearest available cell.
            let cr = if vy > 0 { vy - 1 } else { 0 };
            let cc = if vx > 0 { vx - 1 } else { 0 };
            let cr = cr.min(depth - 1);
            let cc = cc.min(width - 1);
            vert_material_indices.push(cell_mat_indices[cr * width + cc]);
        }
    }

    // Emit triangles: 2 per cell.
    let num_tris = width * depth * 2;
    let mut triangles: Vec<u32> = Vec::with_capacity(num_tris * 3);
    for row in 0..depth {
        for col in 0..width {
            let tl = (row * vw + col) as u32;       // top-left corner
            let tr = tl + 1;                         // top-right
            let bl = ((row + 1) * vw + col) as u32;  // bottom-left
            let br = bl + 1;                         // bottom-right

            // Two triangles per quad, consistent winding (CCW from above).
            triangles.push(tl);
            triangles.push(bl);
            triangles.push(tr);

            triangles.push(tr);
            triangles.push(bl);
            triangles.push(br);
        }
    }

    Some(TerrainMesh {
        vertices,
        triangles,
        normals,
        material_indices: vert_material_indices,
        material_palette: palette,
    })
}

// ---------------------------------------------------------------------------
// JSON serialisation (same pattern as PropMesh::write_json / RoadMeshStrip)
// ---------------------------------------------------------------------------

impl TerrainMesh {
    /// Write this mesh as a JSON object into the manifest serialiser.
    /// Emits both base64 binary (compact) and legacy JSON arrays (backwards compat).
    pub fn write_json(&self, out: &mut String, indent: usize) {
        use base64::Engine;

        write_indent(out, indent);
        out.push_str("{\n");

        // Base64 binary encoding (60% smaller than JSON arrays).
        let vert_bytes: Vec<u8> = self.vertices.iter().flat_map(|v| v.to_le_bytes()).collect();
        write_indent(out, indent + 2);
        out.push_str("\"verticesB64\": \"");
        out.push_str(&base64::engine::general_purpose::STANDARD.encode(&vert_bytes));
        out.push_str("\",\n");

        let tri_bytes: Vec<u8> = self.triangles.iter().flat_map(|v| v.to_le_bytes()).collect();
        write_indent(out, indent + 2);
        out.push_str("\"trianglesB64\": \"");
        out.push_str(&base64::engine::general_purpose::STANDARD.encode(&tri_bytes));
        out.push_str("\",\n");

        let norm_bytes: Vec<u8> = self.normals.iter().flat_map(|v| v.to_le_bytes()).collect();
        write_indent(out, indent + 2);
        out.push_str("\"normalsB64\": \"");
        out.push_str(&base64::engine::general_purpose::STANDARD.encode(&norm_bytes));
        out.push_str("\",\n");

        // Legacy JSON arrays (backwards compat — remove once all consumers use B64).
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
        out.push_str("],\n");

        write_indent(out, indent + 2);
        out.push_str("\"materialIndices\": [");
        for (i, m) in self.material_indices.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            write!(out, "{}", m).unwrap();
        }
        out.push_str("],\n");

        write_indent(out, indent + 2);
        out.push_str("\"materialPalette\": [");
        for (i, name) in self.material_palette.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            write_json_string(out, name);
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

fn write_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_terrain_vertex_count() {
        // 4x4 grid => (4+1)*(4+1) = 25 vertices
        let heights = vec![5.0; 16];
        let mesh = build_terrain_mesh(&heights, 4, 4, 4.0, None).unwrap();
        assert_eq!(mesh.vertex_count(), 25);
        assert_eq!(mesh.triangle_count(), 4 * 4 * 2); // 32 triangles
    }

    #[test]
    fn flat_terrain_all_same_height() {
        let heights = vec![10.0; 9]; // 3x3
        let mesh = build_terrain_mesh(&heights, 3, 3, 4.0, None).unwrap();
        // All vertices should have Y = 10.0 since every cell is the same height.
        for i in 0..mesh.vertex_count() {
            let y = mesh.vertices[i * 3 + 1];
            assert!(
                (y - 10.0).abs() < 1e-4,
                "vertex {} Y={}, expected 10.0",
                i,
                y
            );
        }
    }

    #[test]
    fn flat_terrain_normals_point_up() {
        let heights = vec![5.0; 16]; // 4x4
        let mesh = build_terrain_mesh(&heights, 4, 4, 4.0, None).unwrap();
        for i in 0..mesh.vertex_count() {
            let nx = mesh.normals[i * 3] as f64;
            let ny = mesh.normals[i * 3 + 1] as f64;
            let nz = mesh.normals[i * 3 + 2] as f64;
            assert!(
                (nx.abs()) < 1e-4 && (ny - 1.0).abs() < 1e-4 && (nz.abs()) < 1e-4,
                "vertex {} normal=({},{},{}), expected (0,1,0)",
                i,
                nx,
                ny,
                nz
            );
        }
    }

    #[test]
    fn sloped_terrain_produces_non_zero_normals() {
        // 3x3 grid with a slope in the X direction.
        let heights = vec![
            0.0, 5.0, 10.0, // row 0
            0.0, 5.0, 10.0, // row 1
            0.0, 5.0, 10.0, // row 2
        ];
        let mesh = build_terrain_mesh(&heights, 3, 3, 4.0, None).unwrap();
        // Interior vertices should have non-trivial X component in their normal.
        let mut found_nonzero_nx = false;
        for i in 0..mesh.vertex_count() {
            let nx = mesh.normals[i * 3] as f64;
            if nx.abs() > 0.01 {
                found_nonzero_nx = true;
                break;
            }
        }
        assert!(
            found_nonzero_nx,
            "expected at least one vertex with non-zero NX on a sloped grid"
        );
    }

    #[test]
    fn normals_are_unit_length() {
        let heights = vec![
            0.0, 2.0, 4.0,
            1.0, 3.0, 5.0,
            2.0, 4.0, 6.0,
        ];
        let mesh = build_terrain_mesh(&heights, 3, 3, 4.0, None).unwrap();
        for i in 0..mesh.vertex_count() {
            let nx = mesh.normals[i * 3] as f64;
            let ny = mesh.normals[i * 3 + 1] as f64;
            let nz = mesh.normals[i * 3 + 2] as f64;
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            assert!(
                (len - 1.0).abs() < 1e-3,
                "normal at vertex {} has length {}, expected 1.0",
                i,
                len
            );
        }
    }

    #[test]
    fn material_indices_map_correctly() {
        let heights = vec![1.0; 4]; // 2x2
        let materials: Vec<String> = vec![
            "Grass".into(),
            "Sand".into(),
            "Sand".into(),
            "Rock".into(),
        ];
        let mesh = build_terrain_mesh(&heights, 2, 2, 4.0, Some(&materials)).unwrap();

        // Palette should have 3 entries: Grass, Sand, Rock (in order of first occurrence).
        assert_eq!(mesh.material_palette.len(), 3);
        assert_eq!(mesh.material_palette[0], "Grass");
        assert_eq!(mesh.material_palette[1], "Sand");
        assert_eq!(mesh.material_palette[2], "Rock");

        // Every vertex material index should be valid.
        for (i, &idx) in mesh.material_indices.iter().enumerate() {
            assert!(
                (idx as usize) < mesh.material_palette.len(),
                "vertex {} has material index {} but palette has only {} entries",
                i,
                idx,
                mesh.material_palette.len()
            );
        }

        // material_indices length matches vertex count.
        assert_eq!(mesh.material_indices.len(), mesh.vertex_count());
    }

    #[test]
    fn triangles_reference_valid_vertices() {
        let heights = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]; // 3x2
        let mesh = build_terrain_mesh(&heights, 3, 2, 4.0, None).unwrap();
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
        let heights = vec![1.0; 6]; // 3x2
        let mesh = build_terrain_mesh(&heights, 3, 2, 4.0, None).unwrap();
        assert_eq!(mesh.vertices.len(), mesh.normals.len());
    }

    #[test]
    fn json_round_trip() {
        let heights = vec![1.0, 2.0, 3.0, 4.0]; // 2x2
        let materials: Vec<String> = vec![
            "Grass".into(),
            "Sand".into(),
            "Rock".into(),
            "Grass".into(),
        ];
        let mesh = build_terrain_mesh(&heights, 2, 2, 4.0, Some(&materials)).unwrap();

        let mut json = String::new();
        mesh.write_json(&mut json, 0);

        // Verify all expected keys are present.
        assert!(json.contains("\"vertices\""), "missing vertices key");
        assert!(json.contains("\"triangles\""), "missing triangles key");
        assert!(json.contains("\"normals\""), "missing normals key");
        assert!(
            json.contains("\"materialIndices\""),
            "missing materialIndices key"
        );
        assert!(
            json.contains("\"materialPalette\""),
            "missing materialPalette key"
        );

        // Verify palette values appear in the JSON.
        assert!(json.contains("\"Grass\""));
        assert!(json.contains("\"Sand\""));
        assert!(json.contains("\"Rock\""));

        // Verify element counts by counting commas + 1 in each array.
        let count_elements = |key: &str| -> usize {
            let kpos = json.find(&format!("\"{}\"", key)).expect(key);
            let arr_open = json[kpos..].find('[').unwrap() + kpos;
            let arr_close = json[arr_open..].find(']').unwrap() + arr_open;
            let inner = json[arr_open + 1..arr_close].trim();
            if inner.is_empty() {
                0
            } else {
                inner.split(',').count()
            }
        };

        assert_eq!(count_elements("vertices"), mesh.vertices.len());
        assert_eq!(count_elements("triangles"), mesh.triangles.len());
        assert_eq!(count_elements("normals"), mesh.normals.len());
        assert_eq!(
            count_elements("materialIndices"),
            mesh.material_indices.len()
        );
        assert_eq!(
            count_elements("materialPalette"),
            mesh.material_palette.len()
        );
    }

    #[test]
    fn one_by_one_grid() {
        let heights = vec![7.0]; // 1x1
        let mesh = build_terrain_mesh(&heights, 1, 1, 4.0, None).unwrap();
        // (1+1)*(1+1) = 4 vertices, 2 triangles
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.triangle_count(), 2);
    }

    #[test]
    fn empty_grid_returns_none() {
        assert!(build_terrain_mesh(&[], 0, 0, 4.0, None).is_none());
        assert!(build_terrain_mesh(&[], 5, 0, 4.0, None).is_none());
        assert!(build_terrain_mesh(&[], 0, 5, 4.0, None).is_none());
    }

    #[test]
    fn mismatched_heights_returns_none() {
        // 3x3 grid but only 5 heights provided.
        assert!(build_terrain_mesh(&[1.0; 5], 3, 3, 4.0, None).is_none());
    }

    #[test]
    fn default_cell_size_applied() {
        let heights = vec![1.0; 4]; // 2x2
        let m1 = build_terrain_mesh(&heights, 2, 2, 0.0, None).unwrap();
        let m2 = build_terrain_mesh(&heights, 2, 2, DEFAULT_CELL_SIZE, None).unwrap();
        assert_eq!(
            m1.vertices, m2.vertices,
            "cell_size=0 should use default"
        );
    }

    #[test]
    fn cell_size_affects_vertex_positions() {
        let heights = vec![0.0; 4]; // 2x2
        let m1 = build_terrain_mesh(&heights, 2, 2, 4.0, None).unwrap();
        let m2 = build_terrain_mesh(&heights, 2, 2, 8.0, None).unwrap();
        // With cell_size=4, max X = 2*4 = 8
        // With cell_size=8, max X = 2*8 = 16
        let max_x_1 = m1
            .vertices
            .iter()
            .step_by(3)
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        let max_x_2 = m2
            .vertices
            .iter()
            .step_by(3)
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            (max_x_1 - 8.0).abs() < 1e-4,
            "expected max X=8 for cell_size=4, got {}",
            max_x_1
        );
        assert!(
            (max_x_2 - 16.0).abs() < 1e-4,
            "expected max X=16 for cell_size=8, got {}",
            max_x_2
        );
    }

    #[test]
    fn no_materials_defaults_to_grass() {
        let heights = vec![1.0; 4]; // 2x2
        let mesh = build_terrain_mesh(&heights, 2, 2, 4.0, None).unwrap();
        assert_eq!(mesh.material_palette.len(), 1);
        assert_eq!(mesh.material_palette[0], "Grass");
        // All indices should be 0.
        for &idx in &mesh.material_indices {
            assert_eq!(idx, 0);
        }
    }
}
