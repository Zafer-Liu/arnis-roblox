//! Flat roof — height = 0 everywhere, no inner geometry.

use super::{Point2D, Polygon2D, RoofShape, RoofTags, Segment2D};

pub struct FlatRoof {
    polygon: Polygon2D,
}

impl FlatRoof {
    pub fn new(polygon: Polygon2D, _tags: &RoofTags) -> Self {
        Self { polygon }
    }
}

impl RoofShape for FlatRoof {
    fn polygon(&self) -> &Polygon2D {
        &self.polygon
    }

    fn inner_segments(&self) -> Vec<Segment2D> {
        vec![]
    }

    fn inner_points(&self) -> Vec<Point2D> {
        vec![]
    }

    fn height_at(&self, _pos: Point2D) -> Option<f64> {
        Some(0.0)
    }

    fn roof_height(&self) -> f64 {
        0.0
    }
}
