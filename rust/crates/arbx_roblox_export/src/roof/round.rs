//! Round (barrel vault) roof — semi-circular cross section perpendicular to the
//! ridge direction. Uses RidgeComputation for ridge direction determination.
//!
//! Height at any point: `H * cos(pi/2 * dist_to_ridge / max_dist)` giving a
//! semicircular profile that peaks at the ridge and drops to 0 at the edges.

use super::ridge::{distance_to_ridge_line, insert_ridge_into_polygon, RidgeComputation};
use super::{point_roughly_inside_polygon, Point2D, Polygon2D, RoofShape, RoofTags, Segment2D};

pub struct RoundRoof {
    polygon: Polygon2D,
    ridge: RidgeComputation,
    /// Cross-ridge sample points that give the CDT enough vertices to
    /// approximate the cosine curve instead of producing a tent shape.
    cross_ridge_points: Vec<Point2D>,
}

impl RoundRoof {
    pub fn new(polygon: Polygon2D, tags: &RoofTags) -> Self {
        let ridge = RidgeComputation::compute(
            &polygon,
            0.0, // ridge touches walls (like gabled)
            tags.direction,
            tags.orientation.as_deref(),
            tags.height,
            tags.angle,
            3.0,
        );

        let new_outer = insert_ridge_into_polygon(&polygon.outer, &ridge.ridge);
        let modified_polygon = Polygon2D {
            outer: new_outer,
            holes: polygon.holes.clone(),
        };

        // Generate cross-ridge sample points at fractional distances from the
        // ridge. For each offset line parallel to the ridge, add intersection
        // points that lie inside the polygon.
        let perp = ridge.ridge.direction().right_normal();
        let max_dist = ridge.max_distance_to_ridge;
        let fractions = [0.25, 0.5, 0.75];
        let ridge_mid = Point2D::new(
            (ridge.ridge.p1.x + ridge.ridge.p2.x) / 2.0,
            (ridge.ridge.p1.z + ridge.ridge.p2.z) / 2.0,
        );
        let ridge_dir = ridge.ridge.direction();
        // Sample along the ridge direction at multiple positions
        let ridge_len = ridge.ridge.length();
        let num_along = 8usize;

        let mut cross_ridge_points = Vec::new();
        for &frac in &fractions {
            let offset_dist = frac * max_dist;
            for side in &[1.0_f64, -1.0] {
                let offset = perp.scale(offset_dist * side);
                // Place points along the offset line at evenly spaced positions
                for i in 0..=num_along {
                    let t = (i as f64) / (num_along as f64);
                    let along = ridge_dir.scale(ridge_len * (t - 0.5));
                    let pt = ridge_mid.add(along).add(offset);
                    if point_roughly_inside_polygon(&pt, &polygon.outer) {
                        cross_ridge_points.push(pt);
                    }
                }
            }
        }

        Self {
            polygon: modified_polygon,
            ridge,
            cross_ridge_points,
        }
    }
}

impl RoofShape for RoundRoof {
    fn polygon(&self) -> &Polygon2D {
        &self.polygon
    }

    fn inner_segments(&self) -> Vec<Segment2D> {
        vec![self.ridge.ridge]
    }

    fn inner_points(&self) -> Vec<Point2D> {
        self.cross_ridge_points.clone()
    }

    fn height_at(&self, pos: Point2D) -> Option<f64> {
        let dist = distance_to_ridge_line(pos, self.ridge.ridge);
        let max_dist = self.ridge.max_distance_to_ridge;
        if max_dist < 1e-12 {
            return Some(self.ridge.roof_height);
        }
        let ratio = (dist / max_dist).clamp(0.0, 1.0);
        // Semicircular (cosine) profile.
        let h = self.ridge.roof_height * (std::f64::consts::FRAC_PI_2 * ratio).cos();
        Some(h)
    }

    fn roof_height(&self) -> f64 {
        self.ridge.roof_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect_polygon(w: f64, h: f64) -> Polygon2D {
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
    fn round_ridge_midpoint_at_roof_height() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = RoundRoof::new(poly, &tags);
        let mid = Point2D::new(
            (roof.ridge.ridge.p1.x + roof.ridge.ridge.p2.x) / 2.0,
            (roof.ridge.ridge.p1.z + roof.ridge.ridge.p2.z) / 2.0,
        );
        let h = roof.height_at(mid).unwrap();
        assert!((h - 5.0).abs() < 0.5, "ridge midpoint should be ~5.0, got {}", h);
    }

    #[test]
    fn round_edge_near_zero() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = RoundRoof::new(poly, &tags);
        // Edge at x=0, z=10 should be at max distance from ridge.
        let h = roof.height_at(Point2D::new(0.0, 10.0)).unwrap();
        assert!(h < 0.5, "edge should be near 0, got {}", h);
    }

    #[test]
    fn round_profile_is_curved() {
        // The round profile should be higher than linear (gabled) at midpoints.
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(10.0),
            ..Default::default()
        };
        let roof = RoundRoof::new(poly, &tags);
        // At half-distance from ridge, cosine profile gives cos(pi/4) ~= 0.707
        // while linear would give 0.5. So round should be notably higher.
        let half_dist_pt = Point2D::new(
            roof.ridge.ridge.p1.x + roof.ridge.max_distance_to_ridge * 0.5,
            (roof.ridge.ridge.p1.z + roof.ridge.ridge.p2.z) / 2.0,
        );
        let h = roof.height_at(half_dist_pt).unwrap();
        // Should be > 5.0 (linear) due to curved profile.
        assert!(h > 5.0, "round profile at half-dist should be > 5.0 (linear), got {}", h);
    }

    #[test]
    fn round_inner_segments_has_one_ridge() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = RoundRoof::new(poly, &tags);
        assert_eq!(roof.inner_segments().len(), 1);
    }
}
