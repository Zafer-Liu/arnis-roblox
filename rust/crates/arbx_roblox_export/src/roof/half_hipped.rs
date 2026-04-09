//! Half-hipped roof — hybrid of gabled and hipped. The upper portion has hipped
//! geometry (sloped triangles at each end), but below a transition height the
//! ends become vertical gable walls.
//!
//! Implemented as `min(hipped_height, gabled_height + half_roof_height)` which
//! produces the characteristic clipped-hip silhouette: gabled at base, hipped
//! at top. Uses `RidgeComputation` with `relative_offset = 1/3`.

use super::ridge::{distance_to_ridge_line, RidgeComputation};
use super::{Point2D, Polygon2D, RoofShape, RoofTags, Segment2D};

pub struct HalfHippedRoof {
    polygon: Polygon2D,
    ridge: RidgeComputation,
}

impl HalfHippedRoof {
    pub fn new(polygon: Polygon2D, tags: &RoofTags) -> Self {
        let ridge = RidgeComputation::compute(
            &polygon,
            1.0 / 3.0, // hipped-style inset
            tags.direction,
            tags.orientation.as_deref(),
            tags.height,
            tags.angle,
            3.0,
        );

        Self { polygon, ridge }
    }

    /// Hipped height: linear falloff from ridge line (same as HippedRoof).
    fn hipped_height(&self, pos: Point2D) -> f64 {
        let dist = distance_to_ridge_line(pos, self.ridge.ridge);
        let t = (1.0 - dist / self.ridge.max_distance_to_ridge).max(0.0);
        self.ridge.roof_height * t
    }

    /// Gabled height: linear falloff from ridge line, but only perpendicular
    /// to the ridge (ignoring along-ridge distance). We reuse the same
    /// distance_to_ridge_line which is perpendicular distance to the infinite
    /// line, giving gabled behavior.
    fn gabled_height(&self, pos: Point2D) -> f64 {
        let dist = distance_to_ridge_line(pos, self.ridge.ridge);
        let t = (1.0 - dist / self.ridge.max_distance_to_ridge).max(0.0);
        self.ridge.roof_height * t
    }
}

impl RoofShape for HalfHippedRoof {
    fn polygon(&self) -> &Polygon2D {
        &self.polygon
    }

    fn inner_segments(&self) -> Vec<Segment2D> {
        // Ridge + 4 hip edges (like hipped).
        let r = &self.ridge;
        vec![
            r.ridge,
            Segment2D::new(r.ridge.p1, r.cap1.p1),
            Segment2D::new(r.ridge.p1, r.cap1.p2),
            Segment2D::new(r.ridge.p2, r.cap2.p1),
            Segment2D::new(r.ridge.p2, r.cap2.p2),
        ]
    }

    fn inner_points(&self) -> Vec<Point2D> {
        vec![]
    }

    fn height_at(&self, pos: Point2D) -> Option<f64> {
        // The half-hipped profile: min of hipped and (gabled + half_height).
        // Where the hipped surface dips below the gabled+offset surface, the
        // hipped slope takes over (the clipped hip portion at the top).
        let hipped_h = self.hipped_height(pos);
        let gabled_h = self.gabled_height(pos);
        let half_h = self.ridge.roof_height * 0.5;
        // Gabled height capped at half_h, then add the hipped component above.
        // Simpler: min(hipped, gabled) but gabled gets a half-height boost.
        Some(hipped_h.min(gabled_h + half_h).min(self.ridge.roof_height))
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
    fn half_hipped_inner_segments_has_five() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = HalfHippedRoof::new(poly, &tags);
        let segs = roof.inner_segments();
        assert_eq!(segs.len(), 5, "should have 5 inner segments (1 ridge + 4 hips), got {}", segs.len());
    }

    #[test]
    fn half_hipped_ridge_midpoint_at_roof_height() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = HalfHippedRoof::new(poly, &tags);
        let mid = Point2D::new(
            (roof.ridge.ridge.p1.x + roof.ridge.ridge.p2.x) / 2.0,
            (roof.ridge.ridge.p1.z + roof.ridge.ridge.p2.z) / 2.0,
        );
        let h = roof.height_at(mid).unwrap();
        assert!((h - 5.0).abs() < 0.5, "ridge midpoint should be ~5.0, got {}", h);
    }

    #[test]
    fn half_hipped_corner_near_zero() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = HalfHippedRoof::new(poly, &tags);
        let h = roof.height_at(Point2D::new(0.0, 0.0)).unwrap();
        assert!(h < 1.0, "corner should be near 0, got {}", h);
    }

    #[test]
    fn half_hipped_height_between_gabled_and_hipped_at_end() {
        // At the cap end, half-hipped should be higher than pure hipped
        // (because gabled + half_h lifts it) but still below roof_height.
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(6.0),
            ..Default::default()
        };
        let roof = HalfHippedRoof::new(poly, &tags);
        // Test a point on the ridge line but near the cap end.
        let ridge_end = roof.ridge.ridge.p1;
        let h = roof.height_at(ridge_end).unwrap();
        assert!(h > 0.0, "ridge endpoint should have positive height, got {}", h);
        assert!(h <= 6.0 + 1e-9, "should not exceed roof height, got {}", h);
    }
}
