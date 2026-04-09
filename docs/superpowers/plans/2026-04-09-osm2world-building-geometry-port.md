# osm2world Building Geometry Port — Implementation Plan

Date: 2026-04-09
Status: Active

This is the active implementation plan in the repo's sole superpowers truth stack.

Use it with:

- `docs/superpowers/specs/2026-04-09-osm2world-building-geometry-port-design.md`
- `docs/superpowers/status/2026-04-09-osm2world-building-geometry-port-status.md`

Historical superseded tranches are summarized in:

- `docs/superpowers/archive-index.md`

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port osm2world's building geometry system (15 roof shapes, S3DB building parts, wall surfaces with windows, material mapping) into the Rust `arbx_roblox_export` crate, outputting into the existing `PrecomputedMesh` format for consumption by Roblox 2026's EditableMesh + SurfaceAppearance pipeline.

**Architecture:** Heightfield roof system — each roof shape defines constraint segments + a scalar height function; a shared CDT triangulation base lifts 2D polygons to 3D meshes. Wall surfaces are parameterized faces with punched window/door openings. All geometry flows into `PrecomputedMesh` (flat f32/u32 arrays) consumed by the Lua `BuildingBuilder.lua` EditableMesh consumer, with one explicit manifest handshake field: `building.roofIncluded` tells Lua when a precomputed `shellMesh` already contains roof geometry and the runtime roof path must be skipped. Designed for Roblox 2026: EditableMesh for geometry, SurfaceAppearance for PBR materials, MaterialVariant for diversity without unique textures.

**Tech Stack:** Rust, `spade` (CDT), `geo`/`geo-types` (polygon ops), existing `arbx_roblox_export` crate structure.

---

### Task 1: Add geometry dependencies + RoofShape trait

**Files:**
- Modify: `rust/crates/arbx_roblox_export/Cargo.toml`
- Create: `rust/crates/arbx_roblox_export/src/roof/mod.rs`
- Modify: `rust/crates/arbx_roblox_export/src/lib.rs`
- Test: inline `#[cfg(test)]` in `roof/mod.rs`

- [ ] **Step 1: Add spade + geo dependencies to Cargo.toml**

Add after the existing `base64` dependency:

```toml
spade = "2"
geo = "0.28"
geo-types = "0.7"
```

- [ ] **Step 2: Create roof/mod.rs with RoofShape trait + factory**

```rust
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
pub mod heightfield;

use crate::mesh_builder::PrecomputedMesh;

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
```

- [ ] **Step 3: Add `pub mod roof;` to lib.rs**

Add after the existing module declarations:

```rust
pub mod roof;
```

And add to the pub use block:

```rust
pub use roof::{create_roof, RoofShape, RoofTags, Polygon2D, Point2D, Segment2D};
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p arbx_roblox_export`
Expected: All existing tests pass + 3 new tests pass.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/arbx_roblox_export/
git commit -m "feat(roof): RoofShape trait + factory + geometry primitives

Port of osm2world's HeightfieldRoof architecture. Each roof shape
defines constraint segments + height function; shared CDT base
triangulates and lifts to 3D. Factory maps roof:shape tags to
concrete implementations."
```

---

### Task 2: Heightfield triangulation engine

**Files:**
- Create: `rust/crates/arbx_roblox_export/src/roof/heightfield.rs`
- Test: inline `#[cfg(test)]`

This is the shared engine that ALL roof shapes use. It takes a RoofShape trait object and produces a PrecomputedMesh by:
1. Collecting the polygon outline + inner segments + inner points
2. Running constrained Delaunay triangulation (via `spade`)
3. Lifting each 2D vertex to 3D using the height function
4. Computing per-face normals

- [ ] **Step 1: Write the heightfield triangulation module**

The full implementation of `triangulate_roof()` that converts any `RoofShape` into a `PrecomputedMesh`. Uses `spade::ConstrainedDelaunayTriangulation` for CDT. Handles height interpolation fallback (nearest-segment interpolation when `height_at` returns None).

- [ ] **Step 2: Write tests** — unit test with a known square polygon + flat roof → expected triangle count. Test with gabled mock (ridge segment) → verify vertices lifted correctly.

- [ ] **Step 3: Run tests, commit**

---

### Task 3: FlatRoof implementation

**Files:**
- Create: `rust/crates/arbx_roblox_export/src/roof/flat.rs`

Simplest roof: height = 0 everywhere, no inner segments/points. Pure CDT of the outline.

- [ ] **Step 1: Implement FlatRoof** — returns original polygon, empty segments/points, height_at → Some(0.0).
- [ ] **Step 2: Test** — square footprint → 2 triangles, all at height 0.
- [ ] **Step 3: Commit**

---

### Task 4: Ridge computation (RoofWithRidge shared base)

**Files:**
- Create: `rust/crates/arbx_roblox_export/src/roof/ridge.rs`

Shared by gabled, hipped, half-hipped, gambrel, mansard. Computes ridge direction from tags, intersects with polygon to find endpoints, insets by relative offset.

- [ ] **Step 1: Implement RidgeComputation struct** with:
  - `ridge_direction_from_tags()` — priority: roof:direction > roof:ridge:direction > roof:orientation + min rotated bbox
  - `compute_ridge()` — ray through centroid, find farthest polygon intersections, inset
  - `snap_direction()` — snap compass/degree direction to nearest polygon edge
  - `minimum_rotated_bounding_box()` — rotating calipers for default ridge direction
- [ ] **Step 2: Test** — rectangular polygon → ridge along longest axis. Tagged direction → snapped to nearest edge.
- [ ] **Step 3: Commit**

---

### Task 5: GabledRoof + HippedRoof

**Files:**
- Create: `rust/crates/arbx_roblox_export/src/roof/gabled.rs`
- Create: `rust/crates/arbx_roblox_export/src/roof/hipped.rs`

These are the two most common non-flat roofs and prove the ridge system works.

- [ ] **Step 1: GabledRoof** — ridge_offset=0, inner_segments=[ridge], height = linear falloff from ridge to edges.
- [ ] **Step 2: HippedRoof** — ridge_offset=1/3, inner_segments=[ridge + 4 hip edges to cap corners], same height function.
- [ ] **Step 3: Test both** — verify triangle counts, verify ridge vertices are at max height, verify edge vertices are at 0.
- [ ] **Step 4: Commit**

---

### Task 6: Remaining ridge-based roofs (half-hipped, gambrel, mansard)

**Files:**
- Create: `rust/crates/arbx_roblox_export/src/roof/half_hipped.rs`
- Create: `rust/crates/arbx_roblox_export/src/roof/gambrel.rs`
- Create: `rust/crates/arbx_roblox_export/src/roof/mansard.rs`

- [ ] **Step 1: HalfHippedRoof** — hybrid: hipped top portion, gabled lower. Two height zones.
- [ ] **Step 2: GambrelRoof** — two slopes per side (steep lower, shallow upper), no hip ends.
- [ ] **Step 3: MansardRoof** — like gambrel but with hip ends (inset ridge + two-slope height).
- [ ] **Step 4: Test all three, commit**

---

### Task 7: Non-ridge roofs (pyramidal, skillion, cone)

**Files:**
- Create: `rust/crates/arbx_roblox_export/src/roof/pyramidal.rs`
- Create: `rust/crates/arbx_roblox_export/src/roof/skillion.rs`
- Create: `rust/crates/arbx_roblox_export/src/roof/cone.rs`

- [ ] **Step 1: PyramidalRoof** — single apex at centroid, all edges slope to center. inner_points=[centroid], height = linear from centroid (max) to edges (0).
- [ ] **Step 2: SkillionRoof** — single slope from high edge to low edge. No inner geometry, height = linear interpolation across the polygon.
- [ ] **Step 3: ConeRoof** — circular profile with apex, discretized into radial triangles.
- [ ] **Step 4: Test, commit**

---

### Task 8: Curved roofs (dome, round, onion, chimney, spindle)

**Files:**
- Create: `rust/crates/arbx_roblox_export/src/roof/dome.rs`
- Create: `rust/crates/arbx_roblox_export/src/roof/round.rs`
- Create: `rust/crates/arbx_roblox_export/src/roof/onion.rs`
- Create: `rust/crates/arbx_roblox_export/src/roof/chimney.rs`
- Create: `rust/crates/arbx_roblox_export/src/roof/spindle.rs`

These use ring discretization (concentric circles of inner points) rather than the heightfield approach. They generate explicit triangle rings.

- [ ] **Step 1: DomeRoof** — hemispherical profile, N latitude rings × M longitude segments. inner_points = ring vertices at each latitude.
- [ ] **Step 2: RoundRoof** — barrel vault along ridge direction, semi-circular cross-section.
- [ ] **Step 3: OnionRoof** — bulb profile (tangent curve), ring discretization like dome but with inward curve at base.
- [ ] **Step 4: ChimneyRoof** — thin cylinder/spike, minimal geometry.
- [ ] **Step 5: SpindleRoof** — rotational solid from custom profile curve.
- [ ] **Step 6: Test all, commit**

---

### Task 9: Wire roof generation into build_shell_mesh

**Files:**
- Modify: `rust/crates/arbx_roblox_export/src/mesh_builder.rs`
- Modify: `rust/crates/arbx_roblox_export/src/chunker.rs`

Replace the current walls-only `build_shell_mesh` with a new `build_building_mesh` that generates walls + roof.

- [ ] **Step 1: Create `build_building_mesh()` function** that:
  - Generates wall geometry (existing oriented-box code)
  - Creates RoofTags from BuildingShell fields
  - Calls `create_roof()` to get the right shape
  - Calls `heightfield::triangulate_roof()` to generate roof mesh
  - Merges wall mesh + roof mesh into single PrecomputedMesh
- [ ] **Step 2: Update chunker.rs** to call `build_building_mesh()` instead of `build_shell_mesh()`, passing the full tag set.
- [ ] **Step 3: Keep `build_shell_mesh()` as a backwards-compat wrapper** (calls `build_building_mesh` with flat roof default).
- [ ] **Step 4: Run ALL existing tests** — must pass unchanged.
- [ ] **Step 5: Commit**

---

### Task 10: Wall surfaces with window openings

**Files:**
- Create: `rust/crates/arbx_roblox_export/src/wall_surface.rs`
- Modify: `rust/crates/arbx_roblox_export/src/mesh_builder.rs`

Port osm2world's ExteriorBuildingWall + WallSurface + WindowImplementation.

- [ ] **Step 1: Create wall_surface.rs** with:
  - `split_into_walls()` — divide building outline at corner + explicit wall breaks
  - `generate_wall_surface()` — parameterized wall face divided into floor bands by `building:levels`
  - `punch_window_openings()` — inset rectangles per floor band, frame geometry
  - UV coordinate generation with texture snapping to floor/corner boundaries
- [ ] **Step 2: Wire into mesh_builder** — `build_building_mesh()` uses `wall_surface` instead of oriented boxes when levels > 0.
- [ ] **Step 3: Test** — 2-story building → wall mesh has window openings, correct floor divisions.
- [ ] **Step 4: Commit**

---

### Task 11: Level and height resolution

**Files:**
- Create: `rust/crates/arbx_roblox_export/src/level_height.rs`

Port osm2world's LevelAndHeightData — resolves the complex interplay of building:levels, min_height, height, roof:height, roof:angle into concrete vertical intervals.

- [ ] **Step 1: Implement `resolve_heights()`** — takes all height-related tags, returns `ResolvedHeights { base_y, wall_height, roof_height, floor_height, level_count }`.
- [ ] **Step 2: Test** — various tag combinations → correct resolved heights. Edge cases: levels without height, height without levels, roof:angle → roof:height conversion.
- [ ] **Step 3: Commit**

---

### Task 12: Material mapping

**Files:**
- Create: `rust/crates/arbx_roblox_export/src/material_map.rs`

OSM tag → Roblox material/color. Designed for Roblox 2026's MaterialVariant system.

- [ ] **Step 1: Implement `resolve_building_material()`** — maps `building:material`, `building:facade:material`, `building:cladding` → Roblox material string + default color.
- [ ] **Step 2: Implement `resolve_roof_material()`** — maps `roof:material` → Roblox material. Defaults by roof shape (flat→concrete, gabled→slate, etc.).
- [ ] **Step 3: No cookie-cutter rule** — if OSM tag exists, use it verbatim. If not, vary defaults by building ID hash + usage type so adjacent buildings look distinct.
- [ ] **Step 4: Test, commit**

---

### Task 13: S3DB building:part support

**Files:**
- Create: `rust/crates/arbx_roblox_export/src/building_part.rs`
- Modify: `rust/crates/arbx_pipeline/src/lib.rs` (tag parsing)
- Modify: `rust/crates/arbx_roblox_export/src/chunker.rs`

Port osm2world's Building + BuildingPart with tag inheritance and vertical stacking.

- [ ] **Step 1: Parse `building:part=yes` relations** — in the pipeline's OSM parser, group building parts under their parent building by spatial containment (point-in-polygon of part centroid within building outline).
- [ ] **Step 2: Implement tag inheritance** — parts inherit parent building's tags unless overridden. A part with its own `roof:shape` uses that; without, it inherits.
- [ ] **Step 3: Implement vertical stacking** — each part has its own min_height + height. Parent outline renders only where no part covers it.
- [ ] **Step 4: Modify chunker** — when a building has parts, generate one PrecomputedMesh per part (each with its own walls + roof), skip the parent outline where covered.
- [ ] **Step 5: Test** — parent building with 2 parts (base + tower) → 2 meshes with correct vertical positions.
- [ ] **Step 6: Run ALL tests, commit**

---

### Task 14: Integration test — Austin recompile + visual verification

**Files:**
- Create: `rust/crates/arbx_roblox_export/tests/roof_integration.rs`

- [ ] **Step 1: Write integration test** — compile a small Austin bbox, verify buildings have roof geometry in shell_mesh (vertex count > walls-only baseline).
- [ ] **Step 2: Run full test suite** — `cargo test --workspace` must be all green.
- [ ] **Step 3: Recompile Austin 2km² balanced** — `arbx_cli compile --source ... --bbox 30.265,-97.755,30.285,-97.735 --profile balanced --out /tmp/austin-roofed.json`
- [ ] **Step 4: Split + upload to R2** — use the admin upload endpoint.
- [ ] **Step 5: Run auto_loop** — verify the new geometry loads and renders in Studio.
- [ ] **Step 6: Commit**

---

### Task 15: Lua consumer backwards compatibility

**Files:**
- Modify: `roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua`

- [ ] **Step 1: Detect roof-included shell_mesh** — when `shell_mesh` vertex count exceeds the walls-only baseline (calculated from footprint edge count), skip Lua `buildRoof()`.
- [ ] **Step 2: Fallback** — when `shell_mesh` is absent or walls-only, Lua `buildRoof()` runs as before.
- [ ] **Step 3: Test via harness** — verify buildings render with Rust-generated roofs in Play mode.
- [ ] **Step 4: Commit**
