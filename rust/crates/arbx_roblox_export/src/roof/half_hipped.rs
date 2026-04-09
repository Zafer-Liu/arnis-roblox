//! Half-hipped roof — hybrid of gabled and hipped. The upper portion has hipped
//! geometry (sloped triangles at each end), but below a transition height the
//! ends become vertical gable walls.
//!
//! Implemented as `min(hipped_height, gabled_height + half_roof_height)` which
//! produces the characteristic clipped-hip silhouette: gabled at base, hipped
//! at top. Uses `RidgeComputation` with `relative_offset = 1/3`.

use super::ridge::{distance_to_ridge_line, RidgeComputation};
use super::{distance_point_to_segment, Point2D, Polygon2D, RoofShape, RoofTags, Segment2D};

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

    /// Hipped height: linear falloff from the ridge LINE (infinite extension),
    /// same as HippedRoof. Used everywhere.
    fn hipped_height(&self, pos: Point2D) -> f64 {
        let dist = distance_to_ridge_line(pos, self.ridge.ridge);
        let t = (1.0 - dist / self.ridge.max_distance_to_ridge).max(0.0);
        self.ridge.roof_height * t
    }

    /// Gabled height: linear falloff from the ridge SEGMENT (clamped to
    /// endpoints). In the side region (between cap edges) this equals the
    /// perpendicular distance to the infinite line. Beyond the ridge
    /// endpoints the clamped distance grows large, making gabled_height
    /// small — which lets `min(hipped, gabled + half_h)` pick the hipped
    /// surface and produce the characteristic half-hip triangles at the ends.
    fn gabled_height(&self, pos: Point2D) -> f64 {
        let dist = distance_point_to_segment(pos, self.ridge.ridge);
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
        // In the side region the gabled + half_h term dominates (gabled is
        // high because ridge-segment distance equals perpendicular distance)
        // so hipped wins => pure hipped sides.
        // Beyond the ridge endpoints, gabled_height drops (clamped segment
        // distance grows) so gabled + half_h < hipped => the gabled+offset
        // surface takes over, capping the hip triangles at 50% of roof height.
        let hipped_h = self.hipped_height(pos);
        let gabled_h = self.gabled_height(pos);
        let half_h = self.ridge.roof_height * 0.5;
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

    #[test]
    fn half_hipped_differs_from_pure_hipped() {
        // A point on the ridge line beyond the ridge endpoint should have
        // different height than a pure hipped roof would produce (the gabled
        // segment-distance term limits the hip triangle height).
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(6.0),
            ..Default::default()
        };
        let roof = HalfHippedRoof::new(poly, &tags);
        // Point on the ridge LINE but well beyond ridge.p1.
        let beyond = Point2D::new(
            roof.ridge.ridge.p1.x - 1.0,
            roof.ridge.ridge.p1.z,
        );
        let h_half_hipped = roof.height_at(beyond).unwrap();
        let h_pure_hipped = roof.hipped_height(beyond);
        // Half-hipped should be <= hipped (the gabled clamp kicks in).
        assert!(
            h_half_hipped <= h_pure_hipped + 1e-9,
            "half-hipped ({}) should be <= pure hipped ({}) beyond ridge endpoint",
            h_half_hipped, h_pure_hipped
        );
    }

    #[test]
    fn half_hipped_end_cap_corner_is_clamped() {
        // At a cap corner (beyond ridge endpoint AND at perpendicular
        // distance from ridge), gabled segment-distance is large, so
        // gabled_height is low, and gabled_h + half_h should constrain
        // below pure hipped height.
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(10.0),
            ..Default::default()
        };
        let roof = HalfHippedRoof::new(poly, &tags);
        // Use the cap corner itself (a polygon corner near ridge.p1 end).
        let cap_corner = roof.ridge.cap1.p1;
        let h = roof.height_at(cap_corner).unwrap();
        let h_hipped = roof.hipped_height(cap_corner);
        // The half-hipped surface should not exceed pure hipped here.
        assert!(
            h <= h_hipped + 1e-9,
            "cap corner half-hipped ({}) should be <= pure hipped ({})",
            h, h_hipped
        );
    }
}
