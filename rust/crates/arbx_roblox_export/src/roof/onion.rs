//! OnionRoof — stub, to be filled by swarm agent.

use super::{Point2D, Polygon2D, RoofShape, RoofTags, Segment2D};

pub struct OnionRoof {
    polygon: Polygon2D,
    _height: f64,
}

impl OnionRoof {
    pub fn new(polygon: Polygon2D, tags: &RoofTags) -> Self {
        Self {
            polygon,
            _height: tags.height.unwrap_or(3.0),
        }
    }
}

impl RoofShape for OnionRoof {
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
        self._height
    }
}
