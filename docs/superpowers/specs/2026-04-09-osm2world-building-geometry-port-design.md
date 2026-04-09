# osm2world Building Geometry Port — Design Spec

Date: 2026-04-09
Status: Active

This is the active design spec in the repo's sole superpowers truth stack.

Use it with:

- `docs/superpowers/plans/2026-04-09-osm2world-building-geometry-port.md`
- `docs/superpowers/status/2026-04-09-osm2world-building-geometry-port-status.md`

Historical superseded tranches are summarized in:

- `docs/superpowers/archive-index.md`

## Goal

Port osm2world's building geometry system (MIT license, Java) into our Rust `arbx_roblox_export` pipeline at full 1:1 algorithmic parity. Output flows into the existing `PrecomputedMesh` format (vertices/triangles/normals) consumed by the Lua `BuildingBuilder.lua` with one narrow manifest contract extension: `building.roofIncluded` marks when a precomputed `shellMesh` already contains roof geometry.

## Ethos

Maximum fidelity and style diversity. Never trample source signal — if OSM says `roof:shape=hipped`, render hipped. If OSM says nothing, use intelligent defaults (minimum rotated bounding box → ridge along longest axis). No cookie-cutter: every building should look distinct based on its real-world data, not hash-bucketed into 4 archetypes.

## Scope

Full parity with osm2world `modules/building/` package:
- 15 parameterized roof shapes (heightfield architecture)
- S3DB `building:part` support with tag inheritance and vertical stacking
- Wall surfaces with parameterized window/door openings
- Level and height data resolution (`building:levels`, `min_height`, `height`, `roof:height`, `roof:angle`)
- OSM tag → material/color mapping

NOT in scope: roads, terrain, water, street furniture, barriers (we have our own).

## Architecture

### Roof System (osm2world's HeightfieldRoof pattern)

Each roof shape is a scalar height function over the 2D building polygon. Subclasses define:
1. Inner constraint segments (ridge line, hip edges, etc.)
2. Inner constraint points (apex for pyramidal, etc.)
3. Height function: `height_at(point) -> Option<f64>`

A shared base module does:
1. Constrained Delaunay Triangulation of the polygon + inner segments/points (via `spade` crate)
2. Lifts each 2D vertex to 3D: `y = base_ele + height_at(v)`
3. Computes per-face normals from triangle winding

This is elegant because adding a new roof shape is ~50 lines of Rust (define segments + height function), not hundreds.

### Roof Catalog (15 shapes)

| Shape | Ridge offset | Key geometry |
|-------|-------------|--------------|
| flat | n/a | Height = 0 everywhere |
| gabled | 0 | Ridge touches walls, linear falloff |
| hipped | 1/3 | Ridge inset, 4 hip edges to cap corners |
| half-hipped | 1/3 upper, 0 lower | Hybrid: hipped top, gabled bottom |
| gambrel | 0 | Two slopes per side (steep lower, shallow upper) |
| mansard | 1/3 | Like gambrel but with hip ends |
| pyramidal | n/a | Single apex at centroid, all edges slope to center |
| skillion | n/a | Single slope from high edge to low edge |
| dome | n/a | Hemispherical, ring discretization |
| round | n/a | Barrel vault along ridge direction |
| cone | n/a | Like pyramidal but circular |
| onion | n/a | Bulb profile (tangent curve), ring discretization |
| chimney | n/a | Thin spike/cylinder |
| spindle | n/a | Rotational solid from custom profile |
| complex | n/a | Explicit `roof:ridge`/`roof:edge` tagged ways |

### Ridge Computation (RoofWithRidge)

Shared by gabled, hipped, half-hipped, gambrel, mansard:

1. Determine ridge direction from tags (priority order):
   - `roof:direction` → snap to nearest polygon edge within tolerance
   - `roof:ridge:direction` → direct angle
   - `roof:orientation` (along/across) + minimum rotated bounding box → longest axis
2. Shoot ray through centroid along ridge direction
3. Find two farthest polygon intersections → ridge endpoints
4. Inset by `relative_offset * min(cap_length, 0.4 * ridge_span)`
5. Compute `max_distance_to_ridge` for height normalization

### Building Parts (S3DB)

When OSM has `building:part=yes` relations inside a `building=*` outline:
- Each part inherits tags from the parent building (unless overridden)
- Parts define their own `min_height`, `height`, `building:levels`, `roof:shape`
- Multiple parts stack vertically (base tower + setback upper + penthouse)
- Parent outline renders only where no part covers it

This is the single biggest visual win for skylines — tiered setbacks, podium+tower, mixed-use bases.

### Wall Surfaces

Each building face (from `splitIntoWalls()`) becomes a parameterized surface:
- Divided vertically by `building:levels` into floor bands
- Each floor band gets window openings based on `window` tag or defaults
- Window geometry: inset rectangle (frame depth from wall thickness)
- Door openings at ground level
- UV coordinates snap to floor/corner boundaries (texture alignment)

### Material Mapping

OSM tags → material properties:
- `building:material` / `building:facade:material` → wall material
- `building:colour` / `building:facade:colour` → wall color
- `roof:material` → roof material
- `roof:colour` → roof color
- Defaults by `building=*` type (residential=brick, commercial=glass, industrial=metal)
- No cookie-cutter: if the tag exists, use it. If not, use type-based defaults with variation seeded from building ID.

## New Rust Modules

```
rust/crates/arbx_roblox_export/src/
  roof/
    mod.rs            — RoofShape trait + factory + shared utilities
    heightfield.rs    — CDT triangulation + vertex lifting
    flat.rs
    gabled.rs
    hipped.rs
    half_hipped.rs
    mansard.rs
    gambrel.rs
    pyramidal.rs
    skillion.rs
    dome.rs
    round.rs
    cone.rs
    onion.rs
    complex.rs
    chimney.rs
    spindle.rs
  building_part.rs    — S3DB building:part with tag inheritance
  wall_surface.rs     — Parameterized walls with window/door openings
  level_height.rs     — Level/height tag resolution
  material_map.rs     — OSM tag → material/color
```

## Dependencies

New Cargo.toml additions:
- `spade = "2"` — constrained Delaunay triangulation
- `geo = "0.28"` — polygon ops, minimum rotated bounding box
- `geo-types = "0.7"` — shared geometry types

## Data Flow

```
OSM Feature (building tags + footprint polygon)
    ↓
level_height::resolve(tags) → base_y, wall_height, roof_height
    ↓
building_part::split(feature, related_parts) → Vec<BuildingPart>
    ↓
For each BuildingPart:
    ↓
    material_map::resolve(tags) → wall_material, roof_material
    ↓
    wall_surface::generate(footprint, levels, windows) → wall PrecomputedMesh
    ↓
    roof::create(shape_tag, footprint, tags) → roof PrecomputedMesh
    ↓
    merge(wall_mesh + roof_mesh) → combined PrecomputedMesh
    ↓
BuildingShell.shell_mesh = Some(combined)
```

## Backwards Compatibility

- `PrecomputedMesh` struct unchanged
- Manifest JSON schema extends only by an optional `building.roofIncluded: boolean`
- Lua `BuildingBuilder.lua` still loads the same `shellMesh` field and now uses `roofIncluded` as the explicit contract for skipping duplicate runtime roof generation
- When `shell_mesh` is absent (old manifests), Lua falls back to runtime generation

## Testing

- Per-shape unit tests: known polygon + tags → expected triangle count + bounding box
- Ridge computation tests: various polygon shapes + tag combinations → correct ridge direction
- S3DB tests: parent + parts → correct vertical stacking
- Wall surface tests: levels + windows → correct opening geometry
- Integration: compile Austin → verify buildings have roofs in shell_mesh
- Regression: all 360 existing Rust tests must pass unchanged

## Success Criteria

- Austin recompile shows visually distinct roofs (gabled, hipped, flat) matching OSM tags
- Buildings with `building:part` relations render as tiered structures
- Walls show floor-band divisions and window openings
- No existing test failures
- Compile time for Austin 2km² balanced profile stays under 30s
