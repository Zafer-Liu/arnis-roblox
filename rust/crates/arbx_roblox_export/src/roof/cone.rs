//! ConeRoof — conical roof with apex at centroid, linear height falloff.
//!
//! Ported from osm2world's ConeRoof. Geometrically identical to PyramidalRoof
//! (single apex, radial falloff). The CDT triangulation over a high-vertex-count
//! polygon naturally produces a smooth cone approximation. The distinction
//! exists for tag fidelity — `roof:shape=cone` on round buildings.

use super::{compute_centroid, max_polygon_dimension, Point2D, Polygon2D, RoofShape, RoofTags, Segment2D};

pub struct ConeRoof {
    polygon: Polygon2D,
    height: f64,
    centroid: Point2D,
    max_dist: f64,
}

impl ConeRoof {
    pub fn new(polygon: Polygon2D, tags: &RoofTags) -> Self {
        let centroid = compute_centroid(&polygon.outer);
        let max_dist = polygon
            .outer
            .iter()
            .map(|v| centroid.distance_to(*v))
            .fold(0.0_f64, f64::max);

        let height = tags.height.unwrap_or_else(|| {
            // Default: 1/3 of max polygon dimension (same as pyramidal).
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

impl RoofShape for ConeRoof {
    fn polygon(&self) -> &Polygon2D {
        &self.polygon
    }

    fn inner_segments(&self) -> Vec<Segment2D> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn hexagon_polygon() -> Polygon2D {
        // Regular hexagon centered at (5, 5) with radius 5.
        let cx = 5.0;
        let cz = 5.0;
        let r = 5.0;
        let verts: Vec<Point2D> = (0..6)
            .map(|i| {
                let angle = (i as f64) * PI / 3.0;
                Point2D::new(cx + r * angle.cos(), cz + r * angle.sin())
            })
            .collect();
        Polygon2D {
            outer: verts,
            holes: vec![],
        }
    }

    #[test]
    fn centroid_at_max_height() {
        let roof = ConeRoof::new(hexagon_polygon(), &RoofTags {
            height: Some(5.0),
            ..Default::default()
        });
        let h = roof.height_at(Point2D::new(5.0, 5.0)).unwrap();
        assert!((h - 5.0).abs() < 1e-9, "centroid should be at roof_height, got {h}");
    }

    #[test]
    fn vertices_at_zero() {
        let poly = hexagon_polygon();
        let roof = ConeRoof::new(poly.clone(), &RoofTags {
            height: Some(5.0),
            ..Default::default()
        });
        for v in &poly.outer {
            let h = roof.height_at(*v).unwrap();
            assert!(h.abs() < 1e-6, "vertex ({},{}) should be ~0, got {h}", v.x, v.z);
        }
    }

    #[test]
    fn inner_points_is_centroid() {
        let roof = ConeRoof::new(hexagon_polygon(), &RoofTags {
            height: Some(5.0),
            ..Default::default()
        });
        let pts = roof.inner_points();
        assert_eq!(pts.len(), 1);
        assert!((pts[0].x - 5.0).abs() < 1e-9);
        assert!((pts[0].z - 5.0).abs() < 1e-9);
    }

    #[test]
    fn inner_segments_empty() {
        let roof = ConeRoof::new(hexagon_polygon(), &RoofTags::default());
        assert!(roof.inner_segments().is_empty());
    }

    #[test]
    fn halfway_point_at_half_height() {
        let roof = ConeRoof::new(hexagon_polygon(), &RoofTags {
            height: Some(10.0),
            ..Default::default()
        });
        // A point halfway between centroid (5,5) and vertex (10,5).
        let h = roof.height_at(Point2D::new(7.5, 5.0)).unwrap();
        assert!((h - 5.0).abs() < 1e-9, "halfway should be 5.0, got {h}");
    }
}
