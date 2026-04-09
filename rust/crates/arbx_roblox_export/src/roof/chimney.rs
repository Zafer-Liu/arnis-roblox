//! Chimney roof — tall narrow spike/cylinder. Essentially a pyramidal roof with
//! a much taller height multiplier (default height = full building width).
//! The tag `roof:shape=chimney` is used for church steeples, factory chimneys,
//! and similar tall narrow structures.

use super::{Point2D, Polygon2D, RoofShape, RoofTags, Segment2D};

pub struct ChimneyRoof {
    polygon: Polygon2D,
    height: f64,
    centroid: Point2D,
    max_dist: f64,
}

impl ChimneyRoof {
    pub fn new(polygon: Polygon2D, tags: &RoofTags) -> Self {
        let centroid = compute_centroid(&polygon.outer);
        let max_dist = polygon
            .outer
            .iter()
            .map(|v| centroid.distance_to(*v))
            .fold(0.0_f64, f64::max);

        let height = tags.height.unwrap_or_else(|| {
            // Default: full polygon width (tall spike).
            let max_dim = max_polygon_dimension(&polygon.outer);
            max_dim
        });

        Self {
            polygon,
            height,
            centroid,
            max_dist,
        }
    }
}

impl RoofShape for ChimneyRoof {
    fn polygon(&self) -> &Polygon2D {
        &self.polygon
    }

    fn inner_segments(&self) -> Vec<Segment2D> {
        vec![]
    }

    fn inner_points(&self) -> Vec<Point2D> {
        vec![self.centroid]
    }

    fn height_at(&self, pos: Point2D) -> Option<f64> {
        if self.max_dist < 1e-12 {
            return Some(self.height);
        }
        let dist = self.centroid.distance_to(pos);
        let ratio = (dist / self.max_dist).clamp(0.0, 1.0);
        Some(self.height * (1.0 - ratio))
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
    fn chimney_centroid_at_max_height() {
        let roof = ChimneyRoof::new(square_polygon(), &RoofTags {
            height: Some(20.0),
            ..Default::default()
        });
        let h = roof.height_at(Point2D::new(5.0, 5.0)).unwrap();
        assert!((h - 20.0).abs() < 1e-9, "centroid should be at roof_height, got {}", h);
    }

    #[test]
    fn chimney_default_height_is_full_width() {
        let roof = ChimneyRoof::new(square_polygon(), &RoofTags::default());
        // 10x10 square, max dimension = 10.
        assert!((roof.roof_height() - 10.0).abs() < 1e-9, "default height should be max dimension (10), got {}", roof.roof_height());
    }

    #[test]
    fn chimney_corner_at_zero() {
        let roof = ChimneyRoof::new(square_polygon(), &RoofTags {
            height: Some(20.0),
            ..Default::default()
        });
        let h = roof.height_at(Point2D::new(0.0, 0.0)).unwrap();
        assert!(h.abs() < 1e-6, "corner should be ~0, got {}", h);
    }
}
