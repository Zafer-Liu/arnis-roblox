//! Heightfield triangulation engine — the shared core that ALL roof shapes use.
//!
//! Takes a `RoofShape` trait object and produces a `PrecomputedMesh` by:
//! 1. Collecting the polygon outline vertices + inner segments + inner points
//! 2. Running constrained Delaunay triangulation using `spade`
//! 3. Lifting each 2D vertex to 3D using the roof shape's `height_at()` function
//! 4. When `height_at()` returns `None`, interpolating from nearest segment endpoints
//! 5. Computing per-face normals from triangle winding order
//!
//! Reference: osm2world HeightfieldRoof.getRoofTriangles(baseEle)

use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};

use super::{distance_point_to_segment, Point2D, RoofShape, Segment2D};
use crate::mesh_builder::PrecomputedMesh;

/// Triangulate a roof shape into a PrecomputedMesh.
///
/// The mesh is positioned with Y = base_y + height_at(vertex), using the XZ
/// plane for the footprint. This matches Roblox's coordinate system where Y is
/// up.
pub fn triangulate_roof(shape: &dyn RoofShape, base_y: f64) -> PrecomputedMesh {
    let polygon = shape.polygon();
    let inner_segments = shape.inner_segments();
    let inner_points = shape.inner_points();

    // Phase 1: Collect all unique 2D points and build CDT.
    // We track which spade vertex handle corresponds to which point so we can
    // look up heights after triangulation.

    let mut cdt: ConstrainedDelaunayTriangulation<Point2<f64>> =
        ConstrainedDelaunayTriangulation::new();

    // Insert polygon outer ring as constrained edges (closing the loop).
    // `add_constraint_edge` inserts the vertices internally, so we don't need
    // separate insert calls for polygon vertices.
    let outer = &polygon.outer;
    for i in 0..outer.len() {
        let j = (i + 1) % outer.len();
        let _ = cdt.add_constraint_edge(
            Point2::new(outer[i].x, outer[i].z),
            Point2::new(outer[j].x, outer[j].z),
        );
    }

    // Insert hole vertices and constrain their edges.
    for hole in &polygon.holes {
        for i in 0..hole.len() {
            let j = (i + 1) % hole.len();
            let _ = cdt.add_constraint_edge(
                Point2::new(hole[i].x, hole[i].z),
                Point2::new(hole[j].x, hole[j].z),
            );
        }
    }

    // Insert inner constraint segments (ridge lines, hip edges, etc.).
    for seg in &inner_segments {
        let _ = cdt.add_constraint_edge(
            Point2::new(seg.p1.x, seg.p1.z),
            Point2::new(seg.p2.x, seg.p2.z),
        );
    }

    // Insert inner points (apex, ring points, etc.).
    for p in &inner_points {
        cdt.insert(Point2::new(p.x, p.z))
            .expect("CDT insert failed");
    }

    // Phase 2: Extract triangles, filter to those inside the polygon, lift to 3D.
    //
    // Build a flat vertex array from spade's vertices. Each spade vertex gets an
    // index; we then iterate triangles and emit indices.

    let vertex_positions: Vec<Point2D> = cdt
        .vertices()
        .map(|v| {
            let p = v.position();
            Point2D::new(p.x, p.y)
        })
        .collect();

    // Build a lookup from spade's VertexHandle index to our flat array index.
    // spade vertex handles are dense from 0..n, matching iteration order.
    // We rely on `vertices()` iterating in handle-index order (spade guarantees this).

    // Compute 3D Y for each vertex via height_at or interpolation fallback.
    let all_segments = collect_all_segments(shape);
    let vertex_heights: Vec<f64> = vertex_positions
        .iter()
        .map(|p| {
            let h = shape.height_at(*p);
            match h {
                Some(height) => base_y + height,
                None => base_y + interpolate_height_from_segments(*p, &all_segments, shape),
            }
        })
        .collect();

    // Phase 3: Extract inner triangles and build mesh.
    let mut vertices: Vec<f32> = Vec::new();
    let mut triangles: Vec<u32> = Vec::new();
    let mut normals: Vec<f32> = Vec::new();

    // We need to map spade vertex indices to our output vertex indices.
    // Since we emit per-face vertices (for per-face normals), each triangle gets
    // its own 3 vertices.
    for face in cdt.inner_faces() {
        let [v0, v1, v2] = face.vertices();

        let i0 = v0.index();
        let i1 = v1.index();
        let i2 = v2.index();

        let p0 = vertex_positions[i0];
        let p1 = vertex_positions[i1];
        let p2 = vertex_positions[i2];

        // Filter: only emit triangles whose centroid is inside the polygon.
        let centroid = Point2D::new(
            (p0.x + p1.x + p2.x) / 3.0,
            (p0.z + p1.z + p2.z) / 3.0,
        );
        if !point_in_polygon(centroid, &polygon.outer, &polygon.holes) {
            continue;
        }

        let y0 = vertex_heights[i0];
        let y1 = vertex_heights[i1];
        let y2 = vertex_heights[i2];

        // Emit 3 vertices per face (flat shading with per-face normals).
        let base_idx = (vertices.len() / 3) as u32;

        vertices.extend_from_slice(&[p0.x as f32, y0 as f32, p0.z as f32]);
        vertices.extend_from_slice(&[p1.x as f32, y1 as f32, p1.z as f32]);
        vertices.extend_from_slice(&[p2.x as f32, y2 as f32, p2.z as f32]);

        triangles.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);

        // Compute per-face normal from CCW winding (Y-up).
        let edge1 = [
            p1.x as f32 - p0.x as f32,
            y1 as f32 - y0 as f32,
            p1.z as f32 - p0.z as f32,
        ];
        let edge2 = [
            p2.x as f32 - p0.x as f32,
            y2 as f32 - y0 as f32,
            p2.z as f32 - p0.z as f32,
        ];
        let mut nx = edge1[1] * edge2[2] - edge1[2] * edge2[1];
        let mut ny = edge1[2] * edge2[0] - edge1[0] * edge2[2];
        let mut nz = edge1[0] * edge2[1] - edge1[1] * edge2[0];
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len > 1e-12 {
            nx /= len;
            ny /= len;
            nz /= len;
        } else {
            // Degenerate triangle — default to up.
            nx = 0.0;
            ny = 1.0;
            nz = 0.0;
        }
        // Ensure normal points upward (Y > 0). Flip if it points down.
        if ny < 0.0 {
            nx = -nx;
            ny = -ny;
            nz = -nz;
        }
        // Same normal for all 3 vertices of this face.
        for _ in 0..3 {
            normals.extend_from_slice(&[nx, ny, nz]);
        }
    }

    PrecomputedMesh {
        vertices,
        triangles,
        normals,
    }
}

// ---------------------------------------------------------------------------
// Height interpolation fallback
// ---------------------------------------------------------------------------

/// Segment with known endpoint heights, for interpolation.
struct HeightSegment {
    seg: Segment2D,
    h1: f64,
    h2: f64,
}

/// Collect all segments (polygon edges + inner segments) with their endpoint
/// heights resolved via `height_at`. Used for fallback interpolation.
fn collect_all_segments(shape: &dyn RoofShape) -> Vec<HeightSegment> {
    let mut result = Vec::new();
    let polygon = shape.polygon();

    // Polygon outer ring edges.
    let outer = &polygon.outer;
    for i in 0..outer.len() {
        let j = (i + 1) % outer.len();
        let p1 = outer[i];
        let p2 = outer[j];
        if let (Some(h1), Some(h2)) = (shape.height_at(p1), shape.height_at(p2)) {
            result.push(HeightSegment {
                seg: Segment2D::new(p1, p2),
                h1,
                h2,
            });
        }
    }

    // Hole edges.
    for hole in &polygon.holes {
        for i in 0..hole.len() {
            let j = (i + 1) % hole.len();
            let p1 = hole[i];
            let p2 = hole[j];
            if let (Some(h1), Some(h2)) = (shape.height_at(p1), shape.height_at(p2)) {
                result.push(HeightSegment {
                    seg: Segment2D::new(p1, p2),
                    h1,
                    h2,
                });
            }
        }
    }

    // Inner segments.
    for seg in &shape.inner_segments() {
        if let (Some(h1), Some(h2)) = (shape.height_at(seg.p1), shape.height_at(seg.p2)) {
            result.push(HeightSegment {
                seg: *seg,
                h1,
                h2,
            });
        }
    }

    result
}

/// Interpolate height at `p` from the nearest segment with known endpoint
/// heights. Projects `p` onto the segment and lerps between endpoint heights.
fn interpolate_height_from_segments(
    p: Point2D,
    segments: &[HeightSegment],
    _shape: &dyn RoofShape,
) -> f64 {
    if segments.is_empty() {
        return 0.0;
    }

    let mut best_dist = f64::MAX;
    let mut best_height = 0.0;

    for hs in segments {
        let dist = distance_point_to_segment(p, hs.seg);
        if dist < best_dist {
            best_dist = dist;
            // Project p onto the segment to get interpolation parameter t.
            let dx = hs.seg.p2.x - hs.seg.p1.x;
            let dz = hs.seg.p2.z - hs.seg.p1.z;
            let len_sq = dx * dx + dz * dz;
            let t = if len_sq < 1e-12 {
                0.0
            } else {
                ((p.x - hs.seg.p1.x) * dx + (p.z - hs.seg.p1.z) * dz) / len_sq
            };
            let t = t.clamp(0.0, 1.0);
            best_height = hs.h1 + t * (hs.h2 - hs.h1);
        }
    }

    best_height
}

// ---------------------------------------------------------------------------
// Point-in-polygon test (ray casting)
// ---------------------------------------------------------------------------

/// Test whether a point is inside a polygon (outer ring, minus holes).
fn point_in_polygon(p: Point2D, outer: &[Point2D], holes: &[Vec<Point2D>]) -> bool {
    if !point_in_ring(p, outer) {
        return false;
    }
    for hole in holes {
        if point_in_ring(p, hole) {
            return false;
        }
    }
    true
}

/// Ray-casting point-in-ring test.
fn point_in_ring(p: Point2D, ring: &[Point2D]) -> bool {
    let n = ring.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let pi = ring[i];
        let pj = ring[j];
        if ((pi.z > p.z) != (pj.z > p.z))
            && (p.x < (pj.x - pi.x) * (p.z - pi.z) / (pj.z - pi.z) + pi.x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A simple flat roof: height = 0 everywhere.
    struct FlatTestRoof {
        polygon: Polygon2D,
    }

    use super::super::Polygon2D;

    impl RoofShape for FlatTestRoof {
        fn polygon(&self) -> &Polygon2D {
            &self.polygon
        }
        fn inner_segments(&self) -> Vec<Segment2D> {
            vec![]
        }
        fn inner_points(&self) -> Vec<Point2D> {
            vec![]
        }
        fn height_at(&self, _pos: Point2D) -> Option<f64> {
            Some(0.0)
        }
        fn roof_height(&self) -> f64 {
            0.0
        }
    }

    #[test]
    fn flat_square_produces_two_triangles() {
        let roof = FlatTestRoof {
            polygon: Polygon2D {
                outer: vec![
                    Point2D::new(0.0, 0.0),
                    Point2D::new(10.0, 0.0),
                    Point2D::new(10.0, 10.0),
                    Point2D::new(0.0, 10.0),
                ],
                holes: vec![],
            },
        };

        let mesh = triangulate_roof(&roof, 5.0);

        // A square with CDT should produce exactly 2 triangles.
        assert_eq!(
            mesh.triangle_count(),
            2,
            "square should produce 2 triangles, got {}",
            mesh.triangle_count()
        );

        // All Y values should be base_y (5.0) since height_at returns 0.
        for i in 0..mesh.vertex_count() {
            let y = mesh.vertices[i * 3 + 1];
            assert!(
                (y - 5.0).abs() < 1e-6,
                "vertex {} y should be 5.0, got {}",
                i,
                y
            );
        }

        // Should have 6 vertices (3 per triangle, per-face duplication).
        assert_eq!(mesh.vertices.len(), 18); // 6 vertices * 3 components
        assert_eq!(mesh.normals.len(), 18);
        assert_eq!(mesh.triangles.len(), 6); // 2 triangles * 3 indices

        // All normals should point up (0, 1, 0) for a flat roof.
        for i in 0..6 {
            let nx = mesh.normals[i * 3];
            let ny = mesh.normals[i * 3 + 1];
            let nz = mesh.normals[i * 3 + 2];
            assert!(
                nx.abs() < 1e-6 && (ny - 1.0).abs() < 1e-6 && nz.abs() < 1e-6,
                "normal {} should be (0,1,0), got ({},{},{})",
                i,
                nx,
                ny,
                nz
            );
        }
    }

    /// A gabled mock roof: ridge segment through the middle, linear height.
    /// Ridge runs along X axis at z=5, from x=0 to x=10.
    /// Height = ridge_height * (1 - |z - 5| / 5).
    struct GabledTestRoof {
        polygon: Polygon2D,
        ridge_height: f64,
    }

    impl RoofShape for GabledTestRoof {
        fn polygon(&self) -> &Polygon2D {
            &self.polygon
        }
        fn inner_segments(&self) -> Vec<Segment2D> {
            // Ridge runs along z=5 from x=0 to x=10.
            vec![Segment2D::new(
                Point2D::new(0.0, 5.0),
                Point2D::new(10.0, 5.0),
            )]
        }
        fn inner_points(&self) -> Vec<Point2D> {
            vec![]
        }
        fn height_at(&self, pos: Point2D) -> Option<f64> {
            // Linear falloff from ridge (z=5) to edges (z=0 and z=10).
            let dist_from_ridge = (pos.z - 5.0).abs();
            let t = 1.0 - (dist_from_ridge / 5.0).min(1.0);
            Some(self.ridge_height * t)
        }
        fn roof_height(&self) -> f64 {
            self.ridge_height
        }
    }

    #[test]
    fn gabled_ridge_vertices_at_max_height() {
        let ridge_height = 4.0;
        let base_y = 10.0;
        let roof = GabledTestRoof {
            polygon: Polygon2D {
                outer: vec![
                    Point2D::new(0.0, 0.0),
                    Point2D::new(10.0, 0.0),
                    Point2D::new(10.0, 10.0),
                    Point2D::new(0.0, 10.0),
                ],
                holes: vec![],
            },
            ridge_height,
        };

        let mesh = triangulate_roof(&roof, base_y);

        // Should have more than 2 triangles (the ridge segment splits the square).
        assert!(
            mesh.triangle_count() >= 4,
            "gabled roof should have at least 4 triangles, got {}",
            mesh.triangle_count()
        );

        // Check that vertices on the ridge (z ~= 5) have Y = base_y + ridge_height.
        // And vertices on edges (z ~= 0 or z ~= 10) have Y = base_y.
        let mut found_ridge = false;
        let mut found_edge = false;
        for i in 0..mesh.vertex_count() {
            let x = mesh.vertices[i * 3] as f64;
            let y = mesh.vertices[i * 3 + 1] as f64;
            let z = mesh.vertices[i * 3 + 2] as f64;

            if (z - 5.0).abs() < 0.01 && x >= -0.01 && x <= 10.01 {
                // Ridge vertex.
                assert!(
                    (y - (base_y + ridge_height)).abs() < 0.1,
                    "ridge vertex at ({},{},{}) should have y={}, got {}",
                    x,
                    y,
                    z,
                    base_y + ridge_height,
                    y
                );
                found_ridge = true;
            }
            if (z.abs() < 0.01 || (z - 10.0).abs() < 0.01) && x >= -0.01 && x <= 10.01 {
                // Edge vertex.
                assert!(
                    (y - base_y).abs() < 0.1,
                    "edge vertex at ({},{},{}) should have y={}, got {}",
                    x,
                    y,
                    z,
                    base_y,
                    y
                );
                found_edge = true;
            }
        }
        assert!(found_ridge, "should have found at least one ridge vertex");
        assert!(found_edge, "should have found at least one edge vertex");
    }

    #[test]
    fn height_interpolation_fallback() {
        // A roof where only segment endpoints have known heights, and interior
        // points must be interpolated.
        struct InterpolatingRoof {
            polygon: Polygon2D,
        }

        impl RoofShape for InterpolatingRoof {
            fn polygon(&self) -> &Polygon2D {
                &self.polygon
            }
            fn inner_segments(&self) -> Vec<Segment2D> {
                // A segment from (0,5) to (10,5) — the "ridge".
                vec![Segment2D::new(
                    Point2D::new(0.0, 5.0),
                    Point2D::new(10.0, 5.0),
                )]
            }
            fn inner_points(&self) -> Vec<Point2D> {
                vec![]
            }
            fn height_at(&self, pos: Point2D) -> Option<f64> {
                // Only return heights for polygon corner vertices and ridge endpoints.
                let eps = 0.01;
                let corners = [
                    (0.0, 0.0, 0.0),
                    (10.0, 0.0, 0.0),
                    (10.0, 10.0, 0.0),
                    (0.0, 10.0, 0.0),
                    (0.0, 5.0, 3.0),
                    (10.0, 5.0, 3.0),
                ];
                for (cx, cz, h) in corners {
                    if (pos.x - cx).abs() < eps && (pos.z - cz).abs() < eps {
                        return Some(h);
                    }
                }
                // Unknown — must be interpolated.
                None
            }
            fn roof_height(&self) -> f64 {
                3.0
            }
        }

        let roof = InterpolatingRoof {
            polygon: Polygon2D {
                outer: vec![
                    Point2D::new(0.0, 0.0),
                    Point2D::new(10.0, 0.0),
                    Point2D::new(10.0, 10.0),
                    Point2D::new(0.0, 10.0),
                ],
                holes: vec![],
            },
        };

        let mesh = triangulate_roof(&roof, 0.0);

        // The mesh should have been produced without panics.
        assert!(mesh.triangle_count() >= 2);
        // Ridge endpoints should be at height 3.0.
        let mut found_ridge_at_3 = false;
        for i in 0..mesh.vertex_count() {
            let y = mesh.vertices[i * 3 + 1] as f64;
            let z = mesh.vertices[i * 3 + 2] as f64;
            if (z - 5.0).abs() < 0.01 && (y - 3.0).abs() < 0.1 {
                found_ridge_at_3 = true;
            }
        }
        assert!(
            found_ridge_at_3,
            "should find ridge vertices interpolated/set at height 3.0"
        );
    }

    #[test]
    fn point_in_polygon_basic() {
        let outer = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(10.0, 10.0),
            Point2D::new(0.0, 10.0),
        ];
        assert!(point_in_polygon(Point2D::new(5.0, 5.0), &outer, &[]));
        assert!(!point_in_polygon(Point2D::new(15.0, 5.0), &outer, &[]));
    }

    #[test]
    fn point_in_polygon_with_hole() {
        let outer = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(10.0, 10.0),
            Point2D::new(0.0, 10.0),
        ];
        let hole = vec![
            Point2D::new(3.0, 3.0),
            Point2D::new(7.0, 3.0),
            Point2D::new(7.0, 7.0),
            Point2D::new(3.0, 7.0),
        ];
        // Inside outer but inside hole -> outside.
        assert!(!point_in_polygon(
            Point2D::new(5.0, 5.0),
            &outer,
            &[hole.clone()]
        ));
        // Inside outer, outside hole -> inside.
        assert!(point_in_polygon(
            Point2D::new(1.0, 1.0),
            &outer,
            &[hole]
        ));
    }
}
