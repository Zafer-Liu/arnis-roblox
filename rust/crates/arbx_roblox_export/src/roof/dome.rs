//! Dome roof — hemispherical cap over the building footprint.
//!
//! Height at any point is computed from the distance to the polygon centroid:
//! `h(r) = H * sqrt(1 - (r/R)^2)` where R is the max distance from centroid
//! to any polygon vertex and H is the roof height (dome radius for a true
//! hemisphere). Inner points include ring sample points to improve CDT quality.

use super::{Point2D, Polygon2D, RoofShape, RoofTags, Segment2D};

pub struct DomeRoof {
    polygon: Polygon2D,
    height: f64,
    centroid: Point2D,
    max_radius: f64,
    /// Ring sample points for CDT quality (concentric rings).
    ring_points: Vec<Point2D>,
}

impl DomeRoof {
    pub fn new(polygon: Polygon2D, tags: &RoofTags) -> Self {
        let centroid = compute_centroid(&polygon.outer);
        let max_radius = polygon
            .outer
            .iter()
            .map(|v| centroid.distance_to(*v))
            .fold(0.0_f64, f64::max);

        let height = tags.height.unwrap_or_else(|| {
            // Default: hemisphere, height = radius.
            max_radius
        });

        // Generate ring sample points for CDT.
        let num_rings = 4;
        let num_segments = 16.max(polygon.outer.len());
        let mut ring_points = Vec::new();
        for ring in 1..num_rings {
            let r = max_radius * (ring as f64) / (num_rings as f64);
            for seg in 0..num_segments {
                let angle = 2.0 * std::f64::consts::PI * (seg as f64) / (num_segments as f64);
                let pt = Point2D::new(
                    centroid.x + r * angle.cos(),
                    centroid.z + r * angle.sin(),
                );
                // Only include points roughly inside the polygon.
                if point_roughly_inside_polygon(&pt, &polygon.outer) {
                    ring_points.push(pt);
                }
            }
        }

        Self {
            polygon,
            height,
            centroid,
            max_radius,
            ring_points,
        }
    }
}

impl RoofShape for DomeRoof {
    fn polygon(&self) -> &Polygon2D {
        &self.polygon
    }

    fn inner_segments(&self) -> Vec<Segment2D> {
        vec![]
    }

    fn inner_points(&self) -> Vec<Point2D> {
        let mut pts = vec![self.centroid];
        pts.extend_from_slice(&self.ring_points);
        pts
    }

    fn height_at(&self, pos: Point2D) -> Option<f64> {
        if self.max_radius < 1e-12 {
            return Some(self.height);
        }
        let dist = self.centroid.distance_to(pos);
        let ratio = (dist / self.max_radius).clamp(0.0, 1.0);
        // Hemisphere profile: sqrt(1 - r^2).
        let h = self.height * (1.0 - ratio * ratio).sqrt();
        Some(h)
    }

    fn roof_height(&self) -> f64 {
        self.height
    }
}

fn compute_centroid(vertices: &[Point2D]) -> Point2D {
    let n = vertices.len() as f64;
    if n < 1.0 {
        return Point2D::new(0.0, 0.0);
    }
    let sum_x: f64 = vertices.iter().map(|v| v.x).sum();
    let sum_z: f64 = vertices.iter().map(|v| v.z).sum();
    Point2D::new(sum_x / n, sum_z / n)
}

/// Simple ray-casting point-in-polygon test.
fn point_roughly_inside_polygon(pt: &Point2D, ring: &[Point2D]) -> bool {
    let n = ring.len();
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let vi = ring[i];
        let vj = ring[j];
        if ((vi.z > pt.z) != (vj.z > pt.z))
            && (pt.x < (vj.x - vi.x) * (pt.z - vi.z) / (vj.z - vi.z) + vi.x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
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
    fn dome_centroid_at_max_height() {
        let roof = DomeRoof::new(square_polygon(), &RoofTags {
            height: Some(5.0),
            ..Default::default()
        });
        let h = roof.height_at(Point2D::new(5.0, 5.0)).unwrap();
        assert!((h - 5.0).abs() < 1e-9, "centroid should be at roof_height, got {}", h);
    }

    #[test]
    fn dome_edge_near_zero() {
        let roof = DomeRoof::new(square_polygon(), &RoofTags {
            height: Some(5.0),
            ..Default::default()
        });
        // Corner at (0,0): distance to centroid (5,5) = ~7.07 = max_radius.
        let h = roof.height_at(Point2D::new(0.0, 0.0)).unwrap();
        assert!(h < 0.1, "corner should be ~0, got {}", h);
    }

    #[test]
    fn dome_hemisphere_profile() {
        let roof = DomeRoof::new(square_polygon(), &RoofTags {
            height: Some(5.0),
            ..Default::default()
        });
        // At half-radius, height should be sqrt(1 - 0.25) * 5 = sqrt(0.75) * 5 ~= 4.33
        let half_r = roof.max_radius / 2.0;
        let pt = Point2D::new(5.0 + half_r, 5.0);
        let h = roof.height_at(pt).unwrap();
        let expected = 5.0 * (0.75_f64).sqrt();
        assert!((h - expected).abs() < 0.1, "half-radius height should be ~{}, got {}", expected, h);
    }

    #[test]
    fn dome_inner_points_include_centroid() {
        let roof = DomeRoof::new(square_polygon(), &RoofTags {
            height: Some(5.0),
            ..Default::default()
        });
        let pts = roof.inner_points();
        assert!(!pts.is_empty(), "should have at least the centroid");
        assert!((pts[0].x - 5.0).abs() < 1e-9, "first point should be centroid");
    }
}
