//! Hipped roof — four sloped planes meeting at an inset ridge, with hip edges
//! connecting ridge endpoints to polygon corners.
//!
//! Ported from osm2world's `HippedRoof.java`.
//! Uses `RidgeComputation` with `relative_offset = 1/3` (ridge inset from ends).

use super::ridge::{distance_to_ridge_line, RidgeComputation};
use super::{Point2D, Polygon2D, RoofShape, RoofTags, Segment2D};

pub struct HippedRoof {
    polygon: Polygon2D,
    ridge: RidgeComputation,
}

impl HippedRoof {
    pub fn new(polygon: Polygon2D, tags: &RoofTags) -> Self {
        let ridge = RidgeComputation::compute(
            &polygon,
            1.0 / 3.0, // hipped: ridge inset from ends
            tags.direction,
            tags.orientation.as_deref(),
            tags.height,
            tags.angle,
            3.0, // default height
        );

        Self { polygon, ridge }
    }
}

impl RoofShape for HippedRoof {
    fn polygon(&self) -> &Polygon2D {
        &self.polygon
    }

    fn inner_segments(&self) -> Vec<Segment2D> {
        // The ridge segment + 4 hip edges connecting ridge endpoints to cap corners.
        let r = &self.ridge;
        vec![
            r.ridge,
            // Hip edges from ridge.p1 to cap1 corners.
            Segment2D::new(r.ridge.p1, r.cap1.p1),
            Segment2D::new(r.ridge.p1, r.cap1.p2),
            // Hip edges from ridge.p2 to cap2 corners.
            Segment2D::new(r.ridge.p2, r.cap2.p1),
            Segment2D::new(r.ridge.p2, r.cap2.p2),
        ]
    }

    fn inner_points(&self) -> Vec<Point2D> {
        vec![]
    }

    fn height_at(&self, pos: Point2D) -> Option<f64> {
        let dist = distance_to_ridge_line(pos, self.ridge.ridge);
        let t = (1.0 - dist / self.ridge.max_distance_to_ridge).max(0.0);
        Some(self.ridge.roof_height * t)
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
    fn hipped_inner_segments_has_five() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = HippedRoof::new(poly, &tags);
        let segs = roof.inner_segments();
        assert_eq!(
            segs.len(),
            5,
            "hipped should have 5 inner segments (1 ridge + 4 hips), got {}",
            segs.len()
        );
    }

    #[test]
    fn hipped_ridge_is_inset() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = HippedRoof::new(poly, &tags);

        let ridge_len = roof.ridge.ridge.length();
        // Ridge should be shorter than the full 20-unit span.
        assert!(
            ridge_len < 20.0,
            "hipped ridge should be inset (shorter than 20), got {}",
            ridge_len
        );
        // But should still be substantial.
        assert!(
            ridge_len > 5.0,
            "hipped ridge should be substantial, got {}",
            ridge_len
        );
    }

    #[test]
    fn hipped_height_at_ridge_midpoint() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = HippedRoof::new(poly, &tags);

        let mid = Point2D::new(
            (roof.ridge.ridge.p1.x + roof.ridge.ridge.p2.x) / 2.0,
            (roof.ridge.ridge.p1.z + roof.ridge.ridge.p2.z) / 2.0,
        );
        let h = roof.height_at(mid).unwrap();
        assert!(
            (h - 5.0).abs() < 0.5,
            "height at ridge midpoint should be ~5.0, got {}",
            h
        );
    }

    #[test]
    fn hipped_height_at_corner_is_zero() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = HippedRoof::new(poly, &tags);

        let h = roof.height_at(Point2D::new(0.0, 0.0)).unwrap();
        assert!(h < 0.5, "height at corner should be ~0, got {}", h);
    }

    #[test]
    fn hipped_polygon_unchanged() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = HippedRoof::new(poly, &tags);
        assert_eq!(
            roof.polygon().outer.len(),
            4,
            "hipped should not modify the polygon"
        );
    }

    #[test]
    fn hipped_roof_height_matches_tag() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(8.0),
            ..Default::default()
        };
        let roof = HippedRoof::new(poly, &tags);
        assert!((roof.roof_height() - 8.0).abs() < 1e-9);
    }
}
