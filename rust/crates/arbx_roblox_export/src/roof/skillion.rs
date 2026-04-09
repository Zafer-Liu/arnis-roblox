//! SkillionRoof — single-slope roof from high edge to low edge.
//!
//! Ported from osm2world's SkillionRoof. The slope direction comes from the
//! `roof:direction` tag (compass degrees) or defaults to the shortest polygon
//! edge's outward normal. Height interpolates linearly along the slope axis.

use super::{Point2D, Polygon2D, RoofShape, RoofTags, Segment2D};

pub struct SkillionRoof {
    polygon: Polygon2D,
    height: f64,
    /// Unit vector in the slope direction (from high side toward low side).
    slope_dir: Point2D,
    /// Minimum projection of polygon vertices onto the slope axis.
    proj_min: f64,
    /// Maximum projection of polygon vertices onto the slope axis.
    proj_max: f64,
}

impl SkillionRoof {
    pub fn new(polygon: Polygon2D, tags: &RoofTags) -> Self {
        let slope_dir = if let Some(deg) = tags.direction {
            // OSM roof:direction is compass degrees (0 = north = +Z, 90 = east = +X).
            let rad = deg.to_radians();
            Point2D::new(rad.sin(), rad.cos()).normalize()
        } else {
            // Fallback: shortest polygon edge direction (outward normal).
            shortest_edge_normal(&polygon.outer)
        };

        let (proj_min, proj_max) = projection_range(&polygon.outer, slope_dir);

        let height = tags.height.unwrap_or_else(|| {
            let max_dim = max_polygon_dimension(&polygon.outer);
            max_dim / 4.0
        });

        Self {
            polygon,
            height,
            slope_dir,
            proj_min,
            proj_max,
        }
    }
}

impl RoofShape for SkillionRoof {
    fn polygon(&self) -> &Polygon2D {
        &self.polygon
    }

    fn inner_segments(&self) -> Vec<Segment2D> {
        vec![]
    }

    fn inner_points(&self) -> Vec<Point2D> {
        vec![]
    }

    fn height_at(&self, pos: Point2D) -> Option<f64> {
        let span = self.proj_max - self.proj_min;
        if span < 1e-12 {
            return Some(self.height);
        }
        let proj = dot(pos, self.slope_dir);
        // High side is at proj_min (the side the slope faces away from),
        // low side is at proj_max (the side the slope faces toward).
        let t = ((proj - self.proj_min) / span).clamp(0.0, 1.0);
        Some(self.height * (1.0 - t))
    }

    fn roof_height(&self) -> f64 {
        self.height
    }
}

fn dot(a: Point2D, b: Point2D) -> f64 {
    a.x * b.x + a.z * b.z
}

fn projection_range(vertices: &[Point2D], dir: Point2D) -> (f64, f64) {
    let mut min = f64::MAX;
    let mut max = f64::MIN;
    for v in vertices {
        let p = dot(*v, dir);
        min = min.min(p);
        max = max.max(p);
    }
    (min, max)
}

/// Outward normal of the shortest polygon edge.
fn shortest_edge_normal(vertices: &[Point2D]) -> Point2D {
    if vertices.len() < 2 {
        return Point2D::new(1.0, 0.0);
    }
    let mut best_len = f64::MAX;
    let mut best_dir = Point2D::new(1.0, 0.0);
    for i in 0..vertices.len() {
        let j = (i + 1) % vertices.len();
        let edge = vertices[j].subtract(vertices[i]);
        let len = (edge.x * edge.x + edge.z * edge.z).sqrt();
        if len < best_len && len > 1e-12 {
            best_len = len;
            // Outward normal (right-hand rule for CW winding, but either
            // direction works — the projection range handles orientation).
            best_dir = edge.right_normal().normalize();
        }
    }
    best_dir
}

fn max_polygon_dimension(vertices: &[Point2D]) -> f64 {
    if vertices.is_empty() {
        return 0.0;
    }
    let (mut min_x, mut max_x) = (f64::MAX, f64::MIN);
    let (mut min_z, mut max_z) = (f64::MAX, f64::MIN);
    for v in vertices {
        min_x = min_x.min(v.x);
        max_x = max_x.max(v.x);
        min_z = min_z.min(v.z);
        max_z = max_z.max(v.z);
    }
    (max_x - min_x).max(max_z - min_z)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect_polygon() -> Polygon2D {
        // 20 wide (X) x 10 deep (Z)
        Polygon2D {
            outer: vec![
                Point2D::new(0.0, 0.0),
                Point2D::new(20.0, 0.0),
                Point2D::new(20.0, 10.0),
                Point2D::new(0.0, 10.0),
            ],
            holes: vec![],
        }
    }

    #[test]
    fn high_side_at_roof_height() {
        // Slope direction = +Z (compass north), so high side is at min-Z (z=0).
        let roof = SkillionRoof::new(rect_polygon(), &RoofTags {
            height: Some(4.0),
            direction: Some(0.0), // north = +Z
            ..Default::default()
        });
        // Points at z=0 should be at max height.
        let h = roof.height_at(Point2D::new(10.0, 0.0)).unwrap();
        assert!((h - 4.0).abs() < 1e-9, "high side should be 4.0, got {h}");
    }

    #[test]
    fn low_side_at_zero() {
        let roof = SkillionRoof::new(rect_polygon(), &RoofTags {
            height: Some(4.0),
            direction: Some(0.0),
            ..Default::default()
        });
        // Points at z=10 should be at 0.
        let h = roof.height_at(Point2D::new(10.0, 10.0)).unwrap();
        assert!(h.abs() < 1e-9, "low side should be 0.0, got {h}");
    }

    #[test]
    fn midpoint_at_half_height() {
        let roof = SkillionRoof::new(rect_polygon(), &RoofTags {
            height: Some(4.0),
            direction: Some(0.0),
            ..Default::default()
        });
        let h = roof.height_at(Point2D::new(10.0, 5.0)).unwrap();
        assert!((h - 2.0).abs() < 1e-9, "midpoint should be 2.0, got {h}");
    }

    #[test]
    fn inner_points_and_segments_empty() {
        let roof = SkillionRoof::new(rect_polygon(), &RoofTags::default());
        assert!(roof.inner_points().is_empty());
        assert!(roof.inner_segments().is_empty());
    }

    #[test]
    fn default_height_one_quarter_max_dim() {
        let roof = SkillionRoof::new(rect_polygon(), &RoofTags::default());
        // Max dimension is 20, default = 20/4 = 5
        assert!((roof.roof_height() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn slope_direction_east() {
        // direction = 90° = east = +X. High side at min-X (x=0), low side at max-X (x=20).
        let roof = SkillionRoof::new(rect_polygon(), &RoofTags {
            height: Some(8.0),
            direction: Some(90.0),
            ..Default::default()
        });
        let h_high = roof.height_at(Point2D::new(0.0, 5.0)).unwrap();
        let h_low = roof.height_at(Point2D::new(20.0, 5.0)).unwrap();
        assert!((h_high - 8.0).abs() < 1e-9, "high side (x=0) should be 8.0, got {h_high}");
        assert!(h_low.abs() < 1e-9, "low side (x=20) should be 0.0, got {h_low}");
    }
}
