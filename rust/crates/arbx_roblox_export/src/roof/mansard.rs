//! Mansard roof — like gambrel but with hipped ends. Uses ridge with offset=1/3
//! (like hipped). The dual-slope profile is the same as gambrel: steep lower
//! slope, shallow upper slope, with a break line at 1/3 distance from ridge.
//!
//! Inner segments: ridge + 4 hip edges + 2 break-line segments.

use super::ridge::{distance_to_ridge_line, RidgeComputation};
use super::{clip_segment_to_polygon, Point2D, Polygon2D, RoofShape, RoofTags, Segment2D};

pub struct MansardRoof {
    polygon: Polygon2D,
    ridge: RidgeComputation,
    /// Break lines parallel to ridge, clipped to polygon boundary.
    break_lines: Vec<Segment2D>,
}

impl MansardRoof {
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

        // Break lines at 1/3 of max_distance from ridge on each side.
        // Clip to polygon boundary so endpoints don't extend outside.
        let ridge_dir = ridge.ridge.direction();
        let perp = ridge_dir.right_normal();
        let break_offset = ridge.max_distance_to_ridge / 3.0;

        let raw1 = Segment2D::new(
            ridge.ridge.p1.add(perp.scale(break_offset)),
            ridge.ridge.p2.add(perp.scale(break_offset)),
        );
        let raw2 = Segment2D::new(
            ridge.ridge.p1.add(perp.scale(-break_offset)),
            ridge.ridge.p2.add(perp.scale(-break_offset)),
        );

        let mut break_lines = Vec::new();
        if let Some(clipped) = clip_segment_to_polygon(raw1, &polygon.outer) {
            break_lines.push(clipped);
        }
        if let Some(clipped) = clip_segment_to_polygon(raw2, &polygon.outer) {
            break_lines.push(clipped);
        }

        Self {
            polygon,
            ridge,
            break_lines,
        }
    }
}

impl RoofShape for MansardRoof {
    fn polygon(&self) -> &Polygon2D {
        &self.polygon
    }

    fn inner_segments(&self) -> Vec<Segment2D> {
        let r = &self.ridge;
        let mut segs = vec![
            r.ridge,
            // 4 hip edges
            Segment2D::new(r.ridge.p1, r.cap1.p1),
            Segment2D::new(r.ridge.p1, r.cap1.p2),
            Segment2D::new(r.ridge.p2, r.cap2.p1),
            Segment2D::new(r.ridge.p2, r.cap2.p2),
        ];
        // Clipped break lines
        segs.extend_from_slice(&self.break_lines);
        segs
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
        let break_height = rh * 2.0 / 3.0;

        if dist <= break_dist {
            // Shallow upper slope: ridge (rh) to break line (break_height).
            let t = dist / break_dist;
            Some(rh - t * (rh - break_height))
        } else {
            // Steep lower slope: break line (break_height) to edge (0).
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
    fn mansard_inner_segments_has_ridge_hips_and_break_lines() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = MansardRoof::new(poly, &tags);
        let segs = roof.inner_segments();
        // 1 ridge + 4 hip edges + 0-2 clipped break lines.
        assert!(segs.len() >= 5 && segs.len() <= 7,
            "should have 5-7 inner segments (1 ridge + 4 hips + 0-2 clipped break lines), got {}", segs.len());
    }

    #[test]
    fn mansard_ridge_midpoint_at_roof_height() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(6.0),
            ..Default::default()
        };
        let roof = MansardRoof::new(poly, &tags);
        let mid = Point2D::new(
            (roof.ridge.ridge.p1.x + roof.ridge.ridge.p2.x) / 2.0,
            (roof.ridge.ridge.p1.z + roof.ridge.ridge.p2.z) / 2.0,
        );
        let h = roof.height_at(mid).unwrap();
        assert!((h - 6.0).abs() < 0.5, "ridge midpoint should be ~6.0, got {}", h);
    }

    #[test]
    fn mansard_corner_near_zero() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(5.0),
            ..Default::default()
        };
        let roof = MansardRoof::new(poly, &tags);
        let h = roof.height_at(Point2D::new(0.0, 0.0)).unwrap();
        assert!(h < 0.5, "corner should be near 0, got {}", h);
    }

    #[test]
    fn mansard_dual_slope_profile() {
        let poly = rect_polygon(10.0, 20.0);
        let tags = RoofTags {
            height: Some(9.0),
            ..Default::default()
        };
        let roof = MansardRoof::new(poly, &tags);
        let break_dist = roof.ridge.max_distance_to_ridge / 3.0;
        let ridge_mid = Point2D::new(
            (roof.ridge.ridge.p1.x + roof.ridge.ridge.p2.x) / 2.0,
            (roof.ridge.ridge.p1.z + roof.ridge.ridge.p2.z) / 2.0,
        );
        let perp = roof.ridge.ridge.direction().right_normal();
        let break_pt = ridge_mid.add(perp.scale(break_dist));
        let h = roof.height_at(break_pt).unwrap();
        assert!((h - 6.0).abs() < 0.5, "break line should be ~6.0 (2/3 of 9), got {}", h);
    }
}
