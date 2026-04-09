//! PyramidalRoof — single apex at polygon centroid, linear height falloff.
//!
//! Ported from osm2world's PyramidalRoof. The apex sits at the centroid of
//! the building footprint; height falls linearly to 0 at every polygon edge.

use super::{Point2D, Polygon2D, RoofShape, RoofTags, Segment2D};

pub struct PyramidalRoof {
    polygon: Polygon2D,
    height: f64,
    centroid: Point2D,
    max_dist: f64,
}

impl PyramidalRoof {
    pub fn new(polygon: Polygon2D, tags: &RoofTags) -> Self {
        let centroid = compute_centroid(&polygon.outer);
        let max_dist = polygon
            .outer
            .iter()
            .map(|v| centroid.distance_to(*v))
            .fold(0.0_f64, f64::max);

        let height = tags.height.unwrap_or_else(|| {
            // Default: 1/3 of max polygon dimension (bounding box diagonal proxy).
            let max_dim = max_polygon_dimension(&polygon.outer);
            max_dim / 3.0
        });

        Self {
            polygon,
            height,
            centroid,
            max_dist,
        }
    }
}

impl RoofShape for PyramidalRoof {
    fn polygon(&self) -> &Polygon2D {
        &self.polygon
    }

    fn inner_segments(&self) -> Vec<Segment2D> {
        // CDT will naturally connect the centroid to all polygon vertices.
        vec![]
    }

    fn inner_points(&self) -> Vec<Point2D> {
        vec![self.centroid]
    }

    fn height_at(&self, pos: Point2D) -> Option<f64> {
        if self.max_dist < 1e-12 {
            return Some(self.height);
        }
        let dist = self.centroid.distance_to(pos);
        let ratio = (dist / self.max_dist).clamp(0.0, 1.0);
        Some(self.height * (1.0 - ratio))
    }

    fn roof_height(&self) -> f64 {
        self.height
    }
}

/// Simple centroid (average of vertices). Works for convex and mildly concave
/// footprints — matches osm2world's approach.
fn compute_centroid(vertices: &[Point2D]) -> Point2D {
    let n = vertices.len() as f64;
    if n < 1.0 {
        return Point2D::new(0.0, 0.0);
    }
    let sum_x: f64 = vertices.iter().map(|v| v.x).sum();
    let sum_z: f64 = vertices.iter().map(|v| v.z).sum();
    Point2D::new(sum_x / n, sum_z / n)
}

/// Maximum dimension of a polygon's axis-aligned bounding box.
fn max_polygon_dimension(vertices: &[Point2D]) -> f64 {
    if vertices.is_empty() {
        return 0.0;
    }
    let (mut min_x, mut max_x) = (f64::MAX, f64::MIN);
    let (mut min_z, mut max_z) = (f64::MAX, f64::MIN);
    for v in vertices {
        min_x = min_x.min(v.x);
        max_x = max_x.max(v.x);
        min_z = min_z.min(v.z);
        max_z = max_z.max(v.z);
    }
    (max_x - min_x).max(max_z - min_z)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_polygon() -> Polygon2D {
        Polygon2D {
            outer: vec![
                Point2D::new(0.0, 0.0),
                Point2D::new(10.0, 0.0),
                Point2D::new(10.0, 10.0),
                Point2D::new(0.0, 10.0),
            ],
            holes: vec![],
        }
    }

    #[test]
    fn centroid_at_max_height() {
        let roof = PyramidalRoof::new(square_polygon(), &RoofTags {
            height: Some(6.0),
            ..Default::default()
        });
        let h = roof.height_at(Point2D::new(5.0, 5.0)).unwrap();
        assert!((h - 6.0).abs() < 1e-9, "centroid should be at roof_height, got {h}");
    }

    #[test]
    fn corners_at_zero() {
        let roof = PyramidalRoof::new(square_polygon(), &RoofTags {
            height: Some(6.0),
            ..Default::default()
        });
        for corner in &[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)] {
            let h = roof.height_at(Point2D::new(corner.0, corner.1)).unwrap();
            assert!(h.abs() < 1e-6, "corner ({},{}) should be ~0, got {h}", corner.0, corner.1);
        }
    }

    #[test]
    fn edge_midpoints_between_zero_and_peak() {
        let roof = PyramidalRoof::new(square_polygon(), &RoofTags {
            height: Some(6.0),
            ..Default::default()
        });
        // Midpoint of bottom edge (5, 0) — distance to centroid = 5, max_dist = ~7.07
        let h = roof.height_at(Point2D::new(5.0, 0.0)).unwrap();
        assert!(h > 0.0 && h < 6.0, "edge midpoint should be between 0 and peak, got {h}");
    }

    #[test]
    fn inner_points_is_centroid() {
        let roof = PyramidalRoof::new(square_polygon(), &RoofTags {
            height: Some(6.0),
            ..Default::default()
        });
        let pts = roof.inner_points();
        assert_eq!(pts.len(), 1);
        assert!((pts[0].x - 5.0).abs() < 1e-9);
        assert!((pts[0].z - 5.0).abs() < 1e-9);
    }

    #[test]
    fn inner_segments_empty() {
        let roof = PyramidalRoof::new(square_polygon(), &RoofTags::default());
        assert!(roof.inner_segments().is_empty());
    }

    #[test]
    fn default_height_is_one_third_max_dim() {
        let roof = PyramidalRoof::new(square_polygon(), &RoofTags::default());
        // Square is 10x10, max dim = 10, default = 10/3
        assert!((roof.roof_height() - 10.0 / 3.0).abs() < 1e-9);
    }
}
