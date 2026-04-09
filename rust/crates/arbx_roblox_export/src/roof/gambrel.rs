//! Gambrel roof — two slopes per side: steep lower (~60 deg) and shallow upper
//! (~30 deg). Uses ridge with offset=0 (like gabled). The break line between
//! steep and shallow slopes runs parallel to the ridge at 1/3 distance from the
//! ridge on each side.
//!
//! Height profile (from ridge outward):
//! - Ridge to break line (inner 1/3): shallow slope
//! - Break line to edge (outer 2/3): steep slope
//! The two slopes meet at the break line height, which is 2/3 of roof_height.

use super::ridge::{distance_to_ridge_line, insert_ridge_into_polygon, RidgeComputation};
use super::{Point2D, Polygon2D, RoofShape, RoofTags, Segment2D};

pub struct GambrelRoof {
    polygon: Polygon2D,
    ridge: RidgeComputation,
    /// Break lines parallel to ridge (inner constraint segments).
    break_line1: Segment2D,
    break_line2: Segment2D,
}

impl GambrelRoof {
    pub fn new(polygon: Polygon2D, tags: &RoofTags) -> Self {
        let ridge = RidgeComputation::compute(
            &polygon,
            0.0, // gabled-style: ridge touches walls
            tags.direction,
            tags.orientation.as_deref(),
            tags.height,
            tags.angle,
            3.0,
        );

        // Insert ridge endpoints into polygon outline.
        let new_outer = insert_ridge_into_polygon(&polygon.outer, &ridge.ridge);
        let modified_polygon = Polygon2D {
            outer: new_outer,
            holes: polygon.holes.clone(),
        };

        // Break lines at 1/3 of max_distance from ridge on each side.
        let ridge_dir = ridge.ridge.direction();
        let perp = ridge_dir.right_normal();
        let break_offset = ridge.max_distance_to_ridge / 3.0;

        let break_line1 = Segment2D::new(
            ridge.ridge.p1.add(perp.scale(break_offset)),
            ridge.ridge.p2.add(perp.scale(break_offset)),
        );
        let break_line2 = Segment2D::new(
            ridge.ridge.p1.add(perp.scale(-break_offset)),
            ridge.ridge.p2.add(perp.scale(-break_offset)),
        );

        Self {
            polygon: modified_polygon,
            ridge,
            break_line1,
            break_line2,
        }
    }
}

impl RoofShape for GambrelRoof {
    fn polygon(&self) -> &Polygon2D {
        &self.polygon
    }

    fn inner_segments(&self) -> Vec<Segment2D> {
        vec![self.ridge.ridge, self.break_line1, self.break_line2]
    }

    fn inner_points(&self) -> Vec<Point2D> {
        vec![]
    }

    fn height_at(&self, pos: Point2D) -> Option<f64> {
        let dist = distance_to_ridge_line(pos, self.ridge.ridge);
        let max_dist = self.ridge.max_distance_to_ridge;
        if max_dist < 1e-12 {
            return Some(self.ridge.roof_height);
        }

        let break_dist = max_dist / 3.0;
        let rh = self.ridge.roof_height;
        // Break line height: 2/3 of roof height.
        let break_height = rh * 2.0 / 3.0;

        if dist <= break_dist {
            // Shallow upper slope: from ridge (rh) to break line (break_height).
            let t = dist / break_dist;
            Some(rh - t * (rh - break_height))
        } else {
            // Steep lower slope: from break line (break_height) to edge (0).
            let remaining = max_dist - break_dist;
            let t = ((dist - break_dist) / remaining).clamp(0.0, 1.0);
            Some(break_height * (1.0 - t))
        }
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
    fn gambrel_inner_segments_has_three() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = GambrelRoof::new(poly, &tags);
        let segs = roof.inner_segments();
        assert_eq!(segs.len(), 3, "should have 3 inner segments (1 ridge + 2 break lines), got {}", segs.len());
    }

    #[test]
    fn gambrel_ridge_midpoint_at_roof_height() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(6.0),
            ..Default::default()
        };
        let roof = GambrelRoof::new(poly, &tags);
        let mid = Point2D::new(
            (roof.ridge.ridge.p1.x + roof.ridge.ridge.p2.x) / 2.0,
            (roof.ridge.ridge.p1.z + roof.ridge.ridge.p2.z) / 2.0,
        );
        let h = roof.height_at(mid).unwrap();
        assert!((h - 6.0).abs() < 0.5, "ridge midpoint should be ~6.0, got {}", h);
    }

    #[test]
    fn gambrel_edge_at_zero() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(6.0),
            ..Default::default()
        };
        let roof = GambrelRoof::new(poly, &tags);
        let h = roof.height_at(Point2D::new(0.0, 10.0)).unwrap();
        assert!(h < 0.5, "polygon edge should be near 0, got {}", h);
    }

    #[test]
    fn gambrel_break_line_at_two_thirds_height() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(9.0),
            ..Default::default()
        };
        let roof = GambrelRoof::new(poly, &tags);
        // The break line is at 1/3 of max_distance from ridge.
        // For a 10-wide rect with ridge at x=5, max_distance ~= 5.
        // Break distance ~= 5/3 ~= 1.67. Test a point at that distance.
        let break_dist = roof.ridge.max_distance_to_ridge / 3.0;
        // Move perpendicular from ridge midpoint.
        let ridge_mid = Point2D::new(
            (roof.ridge.ridge.p1.x + roof.ridge.ridge.p2.x) / 2.0,
            (roof.ridge.ridge.p1.z + roof.ridge.ridge.p2.z) / 2.0,
        );
        let perp = roof.ridge.ridge.direction().right_normal();
        let break_pt = ridge_mid.add(perp.scale(break_dist));
        let h = roof.height_at(break_pt).unwrap();
        // Break height should be 2/3 of 9 = 6.
        assert!((h - 6.0).abs() < 0.5, "break line height should be ~6.0, got {}", h);
    }
}
