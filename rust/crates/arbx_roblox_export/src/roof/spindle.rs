//! Spindle roof — rotational solid with a custom height profile. For now, uses
//! the hemisphere profile (same as dome) as default. The spindle shape is a
//! placeholder for more exotic rotationally-symmetric profiles.
//!
//! The key difference from dome: the profile function can be swapped for
//! elongated, tapered, or multi-curve shapes in the future.

use super::{Point2D, Polygon2D, RoofShape, RoofTags, Segment2D};

pub struct SpindleRoof {
    polygon: Polygon2D,
    height: f64,
    centroid: Point2D,
    max_radius: f64,
    ring_points: Vec<Point2D>,
}

impl SpindleRoof {
    pub fn new(polygon: Polygon2D, tags: &RoofTags) -> Self {
        let centroid = compute_centroid(&polygon.outer);
        let max_radius = polygon
            .outer
            .iter()
            .map(|v| centroid.distance_to(*v))
            .fold(0.0_f64, f64::max);

        let height = tags.height.unwrap_or_else(|| max_radius);

        // Generate ring sample points for CDT quality.
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

impl RoofShape for SpindleRoof {
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
        // Hemisphere profile (default spindle shape).
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
    fn spindle_centroid_at_max_height() {
        let roof = SpindleRoof::new(square_polygon(), &RoofTags {
            height: Some(5.0),
            ..Default::default()
        });
        let h = roof.height_at(Point2D::new(5.0, 5.0)).unwrap();
        assert!((h - 5.0).abs() < 1e-9, "centroid should be at roof_height, got {}", h);
    }

    #[test]
    fn spindle_corner_near_zero() {
        let roof = SpindleRoof::new(square_polygon(), &RoofTags {
            height: Some(5.0),
            ..Default::default()
        });
        let h = roof.height_at(Point2D::new(0.0, 0.0)).unwrap();
        assert!(h < 0.1, "corner should be near 0, got {}", h);
    }

    #[test]
    fn spindle_inner_points_include_centroid() {
        let roof = SpindleRoof::new(square_polygon(), &RoofTags {
            height: Some(5.0),
            ..Default::default()
        });
        let pts = roof.inner_points();
        assert!(!pts.is_empty());
        assert!((pts[0].x - 5.0).abs() < 1e-9);
    }
}
