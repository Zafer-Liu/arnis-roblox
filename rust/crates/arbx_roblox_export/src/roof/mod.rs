//! Roof shape catalog ported from osm2world's HeightfieldRoof architecture.
//!
//! Each roof shape is a scalar height function over the 2D building polygon.
//! Subclasses define inner constraint segments + points; the shared
//! `triangulate_roof` function does constrained Delaunay triangulation and
//! lifts each vertex to 3D.
//!
//! Reference: github.com/tordanik/OSM2World/tree/master/core/.../building/roof/

pub mod flat;
pub mod gabled;
pub mod hipped;
pub mod half_hipped;
pub mod mansard;
pub mod gambrel;
pub mod pyramidal;
pub mod skillion;
pub mod dome;
pub mod round;
pub mod cone;
pub mod onion;
pub mod chimney;
pub mod spindle;
pub mod ridge;
pub mod heightfield;

/// A 2D point in the XZ plane (building footprint space).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: f64,
    pub z: f64,
}

impl Point2D {
    pub fn new(x: f64, z: f64) -> Self {
        Self { x, z }
    }

    pub fn distance_to(&self, other: Point2D) -> f64 {
        let dx = self.x - other.x;
        let dz = self.z - other.z;
        (dx * dx + dz * dz).sqrt()
    }

    pub fn subtract(&self, other: Point2D) -> Point2D {
        Point2D::new(self.x - other.x, self.z - other.z)
    }

    pub fn add(&self, other: Point2D) -> Point2D {
        Point2D::new(self.x + other.x, self.z + other.z)
    }

    pub fn scale(&self, s: f64) -> Point2D {
        Point2D::new(self.x * s, self.z * s)
    }

    pub fn normalize(&self) -> Point2D {
        let len = (self.x * self.x + self.z * self.z).sqrt();
        if len < 1e-12 { *self } else { Point2D::new(self.x / len, self.z / len) }
    }

    /// Angle in radians from positive X axis (counterclockwise).
    pub fn angle(&self) -> f64 {
        self.z.atan2(self.x)
    }

    /// Right-hand normal (rotate 90° clockwise in XZ plane).
    pub fn right_normal(&self) -> Point2D {
        Point2D::new(self.z, -self.x)
    }
}

/// A 2D line segment in footprint space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment2D {
    pub p1: Point2D,
    pub p2: Point2D,
}

impl Segment2D {
    pub fn new(p1: Point2D, p2: Point2D) -> Self {
        Self { p1, p2 }
    }

    pub fn length(&self) -> f64 {
        self.p1.distance_to(self.p2)
    }

    pub fn direction(&self) -> Point2D {
        self.p2.subtract(self.p1).normalize()
    }
}

/// Polygon with optional holes, in the XZ plane.
#[derive(Debug, Clone)]
pub struct Polygon2D {
    pub outer: Vec<Point2D>,
    pub holes: Vec<Vec<Point2D>>,
}

/// Building tags relevant to roof generation, extracted from OSM.
#[derive(Debug, Clone, Default)]
pub struct RoofTags {
    pub shape: String,
    pub height: Option<f64>,
    pub angle: Option<f64>,
    pub direction: Option<f64>,
    pub orientation: Option<String>,
    pub levels: Option<u32>,
    pub material: Option<String>,
    pub colour: Option<String>,
}

/// The trait every roof shape implements. Mirrors osm2world's HeightfieldRoof.
pub trait RoofShape {
    /// The building outline, possibly with extra vertices snapped in
    /// (e.g. ridge endpoints for gabled roofs).
    fn polygon(&self) -> &Polygon2D;

    /// Constraint edges for CDT triangulation (ridge, hip edges, etc.).
    fn inner_segments(&self) -> Vec<Segment2D>;

    /// Constraint points for CDT (apex for pyramidal, ring points for dome).
    fn inner_points(&self) -> Vec<Point2D>;

    /// Roof height at a given XZ position. Returns None if the point
    /// should be interpolated from the nearest segment.
    fn height_at(&self, pos: Point2D) -> Option<f64>;

    /// Total roof height (ridge peak above wall top).
    fn roof_height(&self) -> f64;
}

/// Factory: create the appropriate RoofShape from an OSM roof:shape tag.
/// Mirrors osm2world's Roof.createRoofForShape().
pub fn create_roof(
    shape: &str,
    polygon: Polygon2D,
    tags: &RoofTags,
) -> Box<dyn RoofShape> {
    match shape {
        "gabled" => Box::new(gabled::GabledRoof::new(polygon, tags)),
        "hipped" => Box::new(hipped::HippedRoof::new(polygon, tags)),
        "half-hipped" => Box::new(half_hipped::HalfHippedRoof::new(polygon, tags)),
        "gambrel" => Box::new(gambrel::GambrelRoof::new(polygon, tags)),
        "mansard" => Box::new(mansard::MansardRoof::new(polygon, tags)),
        "pyramidal" => Box::new(pyramidal::PyramidalRoof::new(polygon, tags)),
        "skillion" => Box::new(skillion::SkillionRoof::new(polygon, tags)),
        "dome" => Box::new(dome::DomeRoof::new(polygon, tags)),
        "round" => Box::new(round::RoundRoof::new(polygon, tags)),
        "cone" => Box::new(cone::ConeRoof::new(polygon, tags)),
        "onion" => Box::new(onion::OnionRoof::new(polygon, tags)),
        "chimney" => Box::new(chimney::ChimneyRoof::new(polygon, tags)),
        "spindle" => Box::new(spindle::SpindleRoof::new(polygon, tags)),
        _ => Box::new(flat::FlatRoof::new(polygon, tags)),
    }
}

/// Distance from point to line segment (used by multiple roof shapes).
pub fn distance_point_to_segment(p: Point2D, seg: Segment2D) -> f64 {
    let dx = seg.p2.x - seg.p1.x;
    let dz = seg.p2.z - seg.p1.z;
    let len_sq = dx * dx + dz * dz;
    if len_sq < 1e-12 {
        return p.distance_to(seg.p1);
    }
    let t = ((p.x - seg.p1.x) * dx + (p.z - seg.p1.z) * dz) / len_sq;
    let t = t.clamp(0.0, 1.0);
    let proj = Point2D::new(seg.p1.x + t * dx, seg.p1.z + t * dz);
    p.distance_to(proj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_point_to_segment_on_line() {
        let seg = Segment2D::new(Point2D::new(0.0, 0.0), Point2D::new(10.0, 0.0));
        assert!((distance_point_to_segment(Point2D::new(5.0, 3.0), seg) - 3.0).abs() < 1e-9);
    }

    #[test]
    fn distance_point_to_segment_at_endpoint() {
        let seg = Segment2D::new(Point2D::new(0.0, 0.0), Point2D::new(10.0, 0.0));
        assert!((distance_point_to_segment(Point2D::new(-3.0, 4.0), seg) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn create_roof_returns_flat_for_unknown() {
        let poly = Polygon2D {
            outer: vec![
                Point2D::new(0.0, 0.0),
                Point2D::new(10.0, 0.0),
                Point2D::new(10.0, 10.0),
                Point2D::new(0.0, 10.0),
            ],
            holes: vec![],
        };
        let tags = RoofTags::default();
        let roof = create_roof("unknown_shape", poly, &tags);
        assert_eq!(roof.roof_height(), 0.0);
    }
}
