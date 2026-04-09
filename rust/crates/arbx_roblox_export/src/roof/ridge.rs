//! Shared ridge computation for ridge-based roof shapes (gabled, hipped,
//! half-hipped, gambrel, mansard).
//!
//! Ported from osm2world's `RoofWithRidge` logic:
//! 1. Determine ridge direction from `roof:direction` tag or min-area bounding
//!    rectangle longest axis (shortest if `orientation=across`).
//! 2. Shoot a ray through the polygon centroid along that direction, intersect
//!    all polygon edges, take the two farthest intersections as raw ridge endpoints.
//! 3. Inset by `relative_offset * min(cap_length, 0.4 * ridge_span)` toward
//!    centroid to produce the final ridge segment.
//! 4. Compute cap segments (the polygon edges at each end of the ridge).
//! 5. Derive `max_distance_to_ridge` and `roof_height`.

use super::{Point2D, Polygon2D, Segment2D};

/// Result of the shared ridge computation.
#[derive(Debug, Clone)]
pub struct RidgeComputation {
    /// The ridge segment (after inset).
    pub ridge: Segment2D,
    /// Cap segment at the p1 end of the ridge (polygon edge where p1 was projected).
    pub cap1: Segment2D,
    /// Cap segment at the p2 end of the ridge.
    pub cap2: Segment2D,
    /// Maximum perpendicular distance from any polygon vertex to the ridge LINE
    /// (infinite extension). Used as the denominator for linear height falloff.
    pub max_distance_to_ridge: f64,
    /// Computed roof height (ridge peak above wall top).
    pub roof_height: f64,
}

impl RidgeComputation {
    /// Compute ridge geometry for a building polygon.
    ///
    /// * `polygon`         - building footprint
    /// * `relative_offset` - how far to inset the ridge endpoints (0 = gabled, 1/3 = hipped)
    /// * `direction_deg`   - optional `roof:direction` tag in compass degrees
    /// * `orientation`     - optional `roof:orientation` ("along" or "across")
    /// * `tag_height`      - optional explicit `roof:height`
    /// * `tag_angle`       - optional `roof:angle` in degrees
    /// * `default_height`  - fallback height if neither tag_height nor tag_angle given
    pub fn compute(
        polygon: &Polygon2D,
        relative_offset: f64,
        direction_deg: Option<f64>,
        orientation: Option<&str>,
        tag_height: Option<f64>,
        tag_angle: Option<f64>,
        default_height: f64,
    ) -> Self {
        let outer = &polygon.outer;
        let centroid = compute_centroid(outer);

        // --- Step 1: ridge direction vector ---
        let ridge_dir = determine_ridge_direction(outer, direction_deg, orientation);

        // --- Step 2: intersect ray through centroid with polygon edges ---
        let (raw_p1, raw_p2, edge_idx1, edge_idx2) =
            ray_polygon_farthest_intersections(centroid, ridge_dir, outer);

        // Cap segments: the polygon edges that the raw ridge endpoints sit on.
        let cap1 = polygon_edge(outer, edge_idx1);
        let cap2 = polygon_edge(outer, edge_idx2);

        // --- Step 3: inset ridge endpoints ---
        let ridge_span = raw_p1.distance_to(raw_p2);
        let cap1_len = cap1.length();
        let cap2_len = cap2.length();
        let max_cap = cap1_len.max(cap2_len);
        let inset = relative_offset * max_cap.min(0.4 * ridge_span);

        let dir_to_center = centroid.subtract(raw_p1).normalize();
        let dir_to_center2 = centroid.subtract(raw_p2).normalize();
        let p1 = raw_p1.add(dir_to_center.scale(inset));
        let p2 = raw_p2.add(dir_to_center2.scale(inset));
        let ridge = Segment2D::new(p1, p2);

        // --- Step 4: max distance to ridge LINE (infinite extension) ---
        let max_distance_to_ridge = outer
            .iter()
            .map(|v| distance_to_infinite_line(*v, ridge))
            .fold(0.0_f64, f64::max);

        // --- Step 5: roof height ---
        let roof_height = if let Some(h) = tag_height {
            h
        } else if let Some(angle_deg) = tag_angle {
            let angle_rad = angle_deg.to_radians();
            angle_rad.tan() * max_distance_to_ridge
        } else {
            default_height
        };

        Self {
            ridge,
            cap1,
            cap2,
            max_distance_to_ridge,
            roof_height,
        }
    }
}

// ---------------------------------------------------------------------------
// Public helpers used by gabled / hipped / etc.
// ---------------------------------------------------------------------------

/// Perpendicular distance from a point to the infinite line defined by a segment.
/// (NOT clamped to the segment endpoints — extends the line infinitely.)
pub fn distance_to_ridge_line(p: Point2D, seg: Segment2D) -> f64 {
    distance_to_infinite_line(p, seg)
}

/// Insert the two ridge endpoints into a polygon's outer ring, splitting the
/// edges they lie on. Returns a new ring with the extra vertices.
///
/// Used by gabled roofs where the ridge touches the polygon boundary.
pub fn insert_ridge_into_polygon(outer: &[Point2D], ridge: &Segment2D) -> Vec<Point2D> {
    let mut result = Vec::with_capacity(outer.len() + 2);

    for i in 0..outer.len() {
        let a = outer[i];
        let b = outer[(i + 1) % outer.len()];
        result.push(a);

        // Check if either ridge endpoint lies on this edge (within tolerance).
        let mut insertions: Vec<(f64, Point2D)> = Vec::new();
        for &rp in &[ridge.p1, ridge.p2] {
            if let Some(t) = point_on_segment_parameter(rp, a, b, 1e-6) {
                insertions.push((t, rp));
            }
        }
        // Sort by parameter so they appear in edge order.
        insertions.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        for (_, pt) in insertions {
            // Don't duplicate if it's essentially the same as an existing vertex.
            if let Some(last) = result.last() {
                if last.distance_to(pt) > 1e-9 {
                    result.push(pt);
                }
            } else {
                result.push(pt);
            }
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Internal geometry helpers
// ---------------------------------------------------------------------------

/// Centroid of a simple polygon ring (average of vertices).
fn compute_centroid(ring: &[Point2D]) -> Point2D {
    let n = ring.len() as f64;
    let sx: f64 = ring.iter().map(|p| p.x).sum();
    let sz: f64 = ring.iter().map(|p| p.z).sum();
    Point2D::new(sx / n, sz / n)
}

/// Determine ridge direction as a unit vector in XZ space.
///
/// Priority:
/// 1. Explicit `roof:direction` tag (compass degrees, 0=north=+Z, 90=east=+X).
/// 2. Min-area bounding rectangle longest axis (or shortest if orientation="across").
fn determine_ridge_direction(
    outer: &[Point2D],
    direction_deg: Option<f64>,
    orientation: Option<&str>,
) -> Point2D {
    if let Some(deg) = direction_deg {
        // Compass: 0=north(+Z), 90=east(+X). Convert to XZ vector.
        let rad = deg.to_radians();
        return Point2D::new(rad.sin(), rad.cos()).normalize();
    }

    // Rotating calipers approximation: scan all edge angles, pick the one that
    // gives the minimum-area bounding box, then take the longest axis.
    let use_shortest = orientation == Some("across");
    min_area_bbox_axis(outer, use_shortest)
}

/// Approximate min-area bounding rectangle via edge-angle scan.
/// Returns the longest (or shortest) axis direction as a unit vector.
fn min_area_bbox_axis(outer: &[Point2D], use_shortest: bool) -> Point2D {
    let n = outer.len();
    if n < 2 {
        return Point2D::new(1.0, 0.0);
    }

    let mut best_area = f64::MAX;
    let mut best_long_dir = Point2D::new(1.0, 0.0);
    let mut best_short_dir = Point2D::new(0.0, 1.0);

    for i in 0..n {
        let j = (i + 1) % n;
        let edge = outer[j].subtract(outer[i]);
        let len = (edge.x * edge.x + edge.z * edge.z).sqrt();
        if len < 1e-12 {
            continue;
        }
        let dir = Point2D::new(edge.x / len, edge.z / len);
        let perp = Point2D::new(-dir.z, dir.x);

        // Project all vertices onto dir and perp.
        let mut min_d = f64::MAX;
        let mut max_d = f64::MIN;
        let mut min_p = f64::MAX;
        let mut max_p = f64::MIN;
        for v in outer {
            let d = v.x * dir.x + v.z * dir.z;
            let p = v.x * perp.x + v.z * perp.z;
            min_d = min_d.min(d);
            max_d = max_d.max(d);
            min_p = min_p.min(p);
            max_p = max_p.max(p);
        }

        let extent_d = max_d - min_d;
        let extent_p = max_p - min_p;
        let area = extent_d * extent_p;

        if area < best_area {
            best_area = area;
            if extent_d >= extent_p {
                best_long_dir = dir;
                best_short_dir = perp;
            } else {
                best_long_dir = perp;
                best_short_dir = dir;
            }
        }
    }

    if use_shortest {
        best_short_dir
    } else {
        best_long_dir
    }
}

/// Shoot ray from `origin` in direction `dir` (and its opposite), intersect
/// with all edges of `ring`, return the two farthest intersection points and
/// their edge indices.
fn ray_polygon_farthest_intersections(
    origin: Point2D,
    dir: Point2D,
    ring: &[Point2D],
) -> (Point2D, Point2D, usize, usize) {
    let n = ring.len();
    let mut hits: Vec<(f64, Point2D, usize)> = Vec::new();

    for i in 0..n {
        let j = (i + 1) % n;
        if let Some((t, _u)) = ray_segment_intersection(origin, dir, ring[i], ring[j]) {
            let pt = Point2D::new(origin.x + t * dir.x, origin.z + t * dir.z);
            hits.push((t, pt, i));
        }
    }

    if hits.len() < 2 {
        // Degenerate: just use centroid +/- small offset along dir.
        let p1 = origin.add(dir.scale(-0.5));
        let p2 = origin.add(dir.scale(0.5));
        return (p1, p2, 0, 0);
    }

    // Sort by ray parameter t.
    hits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let first = hits.first().unwrap();
    let last = hits.last().unwrap();

    (first.1, last.1, first.2, last.2)
}

/// Ray-segment intersection. Ray: origin + t*dir, Segment: a + u*(b-a).
/// Returns (t, u) if intersection exists with u in [0,1].
fn ray_segment_intersection(
    origin: Point2D,
    dir: Point2D,
    a: Point2D,
    b: Point2D,
) -> Option<(f64, f64)> {
    let dx = b.x - a.x;
    let dz = b.z - a.z;
    let denom = dir.x * dz - dir.z * dx;
    if denom.abs() < 1e-12 {
        return None; // Parallel.
    }
    let ox = a.x - origin.x;
    let oz = a.z - origin.z;
    let t = (ox * dz - oz * dx) / denom;
    let u = (ox * dir.z - oz * dir.x) / denom;
    if u >= -1e-9 && u <= 1.0 + 1e-9 {
        Some((t, u.clamp(0.0, 1.0)))
    } else {
        None
    }
}

/// Get the i-th edge of a polygon ring as a Segment2D.
fn polygon_edge(ring: &[Point2D], i: usize) -> Segment2D {
    let j = (i + 1) % ring.len();
    Segment2D::new(ring[i], ring[j])
}

/// Perpendicular distance from point to the infinite line defined by a segment.
fn distance_to_infinite_line(p: Point2D, seg: Segment2D) -> f64 {
    let dx = seg.p2.x - seg.p1.x;
    let dz = seg.p2.z - seg.p1.z;
    let len = (dx * dx + dz * dz).sqrt();
    if len < 1e-12 {
        return p.distance_to(seg.p1);
    }
    // |cross product| / length = perpendicular distance.
    ((p.x - seg.p1.x) * dz - (p.z - seg.p1.z) * dx).abs() / len
}

/// If point `p` lies on segment `a->b` (within tolerance), return the
/// parameter t in [0,1]. Otherwise None.
fn point_on_segment_parameter(p: Point2D, a: Point2D, b: Point2D, tol: f64) -> Option<f64> {
    let dx = b.x - a.x;
    let dz = b.z - a.z;
    let len_sq = dx * dx + dz * dz;
    if len_sq < 1e-24 {
        return if p.distance_to(a) < tol { Some(0.0) } else { None };
    }
    let t = ((p.x - a.x) * dx + (p.z - a.z) * dz) / len_sq;
    if t < -tol || t > 1.0 + tol {
        return None;
    }
    let t = t.clamp(0.0, 1.0);
    let proj = Point2D::new(a.x + t * dx, a.z + t * dz);
    if p.distance_to(proj) < tol {
        Some(t)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(w: f64, h: f64) -> Polygon2D {
        Polygon2D {
            outer: vec![
                Point2D::new(0.0, 0.0),
                Point2D::new(w, 0.0),
                Point2D::new(w, h),
                Point2D::new(0.0, h),
            ],
            holes: vec![],
        }
    }

    #[test]
    fn ridge_direction_from_tag() {
        // 0 degrees = north = +Z direction.
        let dir = determine_ridge_direction(&rect(10.0, 20.0).outer, Some(0.0), None);
        assert!(dir.z.abs() > 0.9, "0 deg should point along Z, got {:?}", dir);

        // 90 degrees = east = +X direction.
        let dir = determine_ridge_direction(&rect(10.0, 20.0).outer, Some(90.0), None);
        assert!(dir.x.abs() > 0.9, "90 deg should point along X, got {:?}", dir);
    }

    #[test]
    fn ridge_direction_from_bbox_long_axis() {
        // 10 wide, 30 tall => longest axis is Z.
        let poly = rect(10.0, 30.0);
        let dir = determine_ridge_direction(&poly.outer, None, None);
        assert!(
            dir.z.abs() > dir.x.abs(),
            "longest axis of 10x30 rect should be Z, got {:?}",
            dir
        );
    }

    #[test]
    fn ridge_direction_across_uses_short_axis() {
        // 10 wide, 30 tall => shortest axis is X.
        let poly = rect(10.0, 30.0);
        let dir = determine_ridge_direction(&poly.outer, None, Some("across"));
        assert!(
            dir.x.abs() > dir.z.abs(),
            "across orientation should pick short axis (X), got {:?}",
            dir
        );
    }

    #[test]
    fn ridge_gabled_offset_zero_touches_walls() {
        let poly = rect(10.0, 20.0);
        let rc = RidgeComputation::compute(&poly, 0.0, None, None, Some(5.0), None, 3.0);

        // With offset 0, ridge endpoints should lie on (or very near) the polygon boundary.
        let ridge_len = rc.ridge.length();
        // For a 10x20 rect with ridge along Z, endpoints should be near z=0 and z=20.
        assert!(
            ridge_len > 15.0,
            "gabled ridge (offset=0) should span most of the polygon, got {}",
            ridge_len
        );
    }

    #[test]
    fn ridge_hipped_offset_third_is_inset() {
        let poly = rect(10.0, 20.0);
        let rc = RidgeComputation::compute(&poly, 1.0 / 3.0, None, None, Some(5.0), None, 3.0);

        let ridge_len = rc.ridge.length();
        assert!(
            ridge_len < 18.0,
            "hipped ridge should be inset, got length {}",
            ridge_len
        );
        assert!(
            ridge_len > 5.0,
            "hipped ridge should still be substantial, got length {}",
            ridge_len
        );
    }

    #[test]
    fn max_distance_to_ridge_positive() {
        let poly = rect(10.0, 20.0);
        let rc = RidgeComputation::compute(&poly, 0.0, None, None, Some(5.0), None, 3.0);
        assert!(
            rc.max_distance_to_ridge > 0.0,
            "max_distance should be positive"
        );
    }

    #[test]
    fn roof_height_from_tag() {
        let poly = rect(10.0, 20.0);
        let rc = RidgeComputation::compute(&poly, 0.0, None, None, Some(7.0), None, 3.0);
        assert!((rc.roof_height - 7.0).abs() < 1e-9);
    }

    #[test]
    fn roof_height_from_angle() {
        let poly = rect(10.0, 20.0);
        let rc = RidgeComputation::compute(&poly, 0.0, None, None, None, Some(45.0), 3.0);
        // tan(45) = 1.0, so height = max_distance_to_ridge.
        assert!(
            (rc.roof_height - rc.max_distance_to_ridge).abs() < 0.5,
            "45 deg angle: height {} should ~= max_dist {}",
            rc.roof_height,
            rc.max_distance_to_ridge
        );
    }

    #[test]
    fn roof_height_default_fallback() {
        let poly = rect(10.0, 20.0);
        let rc = RidgeComputation::compute(&poly, 0.0, None, None, None, None, 3.0);
        assert!((rc.roof_height - 3.0).abs() < 1e-9);
    }

    #[test]
    fn distance_to_ridge_line_perpendicular() {
        let seg = Segment2D::new(Point2D::new(0.0, 5.0), Point2D::new(10.0, 5.0));
        // Point at (5, 0) should be 5 units from the line z=5.
        let d = distance_to_ridge_line(Point2D::new(5.0, 0.0), seg);
        assert!((d - 5.0).abs() < 1e-9);
    }

    #[test]
    fn distance_to_ridge_line_extends_beyond_segment() {
        let seg = Segment2D::new(Point2D::new(0.0, 5.0), Point2D::new(10.0, 5.0));
        // Point at (20, 8) — beyond segment endpoint but perpendicular distance is still 3.
        let d = distance_to_ridge_line(Point2D::new(20.0, 8.0), seg);
        assert!((d - 3.0).abs() < 1e-9);
    }

    #[test]
    fn insert_ridge_into_polygon_adds_vertices() {
        // Square: (0,0)-(10,0)-(10,10)-(0,10).
        // Ridge from (5,0) to (5,10) should insert two points.
        let outer = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(10.0, 10.0),
            Point2D::new(0.0, 10.0),
        ];
        let ridge = Segment2D::new(Point2D::new(5.0, 0.0), Point2D::new(5.0, 10.0));
        let result = insert_ridge_into_polygon(&outer, &ridge);
        assert!(
            result.len() >= 6,
            "should have at least 6 vertices (4 + 2 ridge points), got {}",
            result.len()
        );
    }

    #[test]
    fn cap_segments_exist() {
        let poly = rect(10.0, 20.0);
        let rc = RidgeComputation::compute(&poly, 1.0 / 3.0, None, None, Some(5.0), None, 3.0);
        assert!(rc.cap1.length() > 0.0, "cap1 should have nonzero length");
        assert!(rc.cap2.length() > 0.0, "cap2 should have nonzero length");
    }
}
