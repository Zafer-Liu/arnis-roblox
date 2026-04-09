//! Gabled roof — two sloped planes meeting at a ridge that touches both walls.
//!
//! Ported from osm2world's `GabledRoof.java`.
//! Uses `RidgeComputation` with `relative_offset = 0.0` (ridge runs wall-to-wall).

use super::ridge::{distance_to_ridge_line, insert_ridge_into_polygon, RidgeComputation};
use super::{Point2D, Polygon2D, RoofShape, RoofTags, Segment2D};

pub struct GabledRoof {
    /// Polygon with ridge endpoints inserted into the outline.
    polygon: Polygon2D,
    ridge: RidgeComputation,
}

impl GabledRoof {
    pub fn new(polygon: Polygon2D, tags: &RoofTags) -> Self {
        let ridge = RidgeComputation::compute(
            &polygon,
            0.0, // gabled: ridge touches walls
            tags.direction,
            tags.orientation.as_deref(),
            tags.height,
            tags.angle,
            3.0, // default height
        );

        // Insert ridge endpoints into the polygon outline so the CDT
        // constrains the ridge properly along the boundary.
        let new_outer = insert_ridge_into_polygon(&polygon.outer, &ridge.ridge);

        let modified_polygon = Polygon2D {
            outer: new_outer,
            holes: polygon.holes.clone(),
        };

        Self {
            polygon: modified_polygon,
            ridge,
        }
    }
}

impl RoofShape for GabledRoof {
    fn polygon(&self) -> &Polygon2D {
        &self.polygon
    }

    fn inner_segments(&self) -> Vec<Segment2D> {
        vec![self.ridge.ridge]
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
    fn gabled_inner_segments_has_one_ridge() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = GabledRoof::new(poly, &tags);
        let segs = roof.inner_segments();
        assert_eq!(segs.len(), 1, "gabled should have exactly 1 inner segment (the ridge)");
    }

    #[test]
    fn gabled_height_at_ridge_midpoint_is_roof_height() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = GabledRoof::new(poly, &tags);

        // Ridge midpoint should be near the centroid.
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
    fn gabled_height_at_edge_is_zero() {
        // 10 wide (X), 20 deep (Z). Ridge along Z at x=5.
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = GabledRoof::new(poly, &tags);

        // A point on the left edge (x=0, z=10) should be at max distance from ridge.
        let h = roof.height_at(Point2D::new(0.0, 10.0)).unwrap();
        assert!(
            h < 0.5,
            "height at polygon edge should be ~0, got {}",
            h
        );
    }

    #[test]
    fn gabled_polygon_has_extra_vertices() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = GabledRoof::new(poly, &tags);

        // Original polygon has 4 vertices, gabled should have 6 (ridge endpoints inserted).
        assert!(
            roof.polygon().outer.len() >= 5,
            "gabled polygon should have extra vertices, got {}",
            roof.polygon().outer.len()
        );
    }

    #[test]
    fn gabled_roof_height_matches_tag() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(7.5),
            ..Default::default()
        };
        let roof = GabledRoof::new(poly, &tags);
        assert!((roof.roof_height() - 7.5).abs() < 1e-9);
    }
}
