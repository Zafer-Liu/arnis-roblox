//! Onion dome roof — bulb-shaped profile that widens in the middle before
//! tapering to a point at the top. Same radial approach as dome but with an
//! onion curve profile instead of hemisphere.
//!
//! Profile: `h(r) = H * (1 - (r/R)^2) * (1 + 0.3 * sin(pi * r/R))`
//! This produces the characteristic bulge near 1/3 radius and smooth taper.

use super::{compute_centroid, point_roughly_inside_polygon, Point2D, Polygon2D, RoofShape, RoofTags, Segment2D};

pub struct OnionRoof {
    polygon: Polygon2D,
    height: f64,
    centroid: Point2D,
    max_radius: f64,
    ring_points: Vec<Point2D>,
}

impl OnionRoof {
    pub fn new(polygon: Polygon2D, tags: &RoofTags) -> Self {
        let centroid = compute_centroid(&polygon.outer);
        let max_radius = polygon
            .outer
            .iter()
            .map(|v| centroid.distance_to(*v))
            .fold(0.0_f64, f64::max);

        let height = tags.height.unwrap_or_else(|| max_radius * 1.5);

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

    /// Onion profile: peaks at center, bulges slightly, then drops.
    fn onion_profile(&self, ratio: f64) -> f64 {
        let base = 1.0 - ratio * ratio;
        let bulge = 1.0 + 0.3 * (std::f64::consts::PI * ratio).sin();
        (base * bulge).max(0.0)
    }
}

impl RoofShape for OnionRoof {
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
        Some(self.height * self.onion_profile(ratio))
    }

    fn roof_height(&self) -> f64 {
        self.height
    }
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
    fn onion_centroid_at_max_height() {
        let roof = OnionRoof::new(square_polygon(), &RoofTags {
            height: Some(8.0),
            ..Default::default()
        });
        let h = roof.height_at(Point2D::new(5.0, 5.0)).unwrap();
        assert!((h - 8.0).abs() < 1e-9, "centroid should be at roof_height, got {}", h);
    }

    #[test]
    fn onion_corner_near_zero() {
        let roof = OnionRoof::new(square_polygon(), &RoofTags {
            height: Some(8.0),
            ..Default::default()
        });
        let h = roof.height_at(Point2D::new(0.0, 0.0)).unwrap();
        assert!(h < 0.5, "corner should be near 0, got {}", h);
    }

    #[test]
    fn onion_bulge_higher_than_linear() {
        // At ~1/3 radius the bulge factor should make onion higher than a simple cone.
        let roof = OnionRoof::new(square_polygon(), &RoofTags {
            height: Some(10.0),
            ..Default::default()
        });
        let third_r = roof.max_radius / 3.0;
        let pt = Point2D::new(5.0 + third_r, 5.0);
        let h = roof.height_at(pt).unwrap();
        // Linear (cone) at 1/3 radius: 10 * (1 - 1/3) = 6.67
        // Onion should be higher due to bulge.
        assert!(h > 7.0, "onion at 1/3 radius should bulge above linear, got {}", h);
    }
}
