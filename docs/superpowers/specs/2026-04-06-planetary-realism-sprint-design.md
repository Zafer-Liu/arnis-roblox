# Planetary Realism Sprint

Date: 2026-04-06
Status: Active

## Purpose

Make the world feel like a real place you can walk, drive, fly, and parachute through — with the visual fidelity bar set by Cesium 3D, Google Earth, and Mapbox Vector. Players traverse this world on foot, in vehicles, with jetpacks, with parachutes, and in aircraft. Every surface the player can see, land on, or fly over needs to hold up.

The outdoor fidelity tranche proved the pipeline and proof infrastructure. This tranche uses that infrastructure to ship visible quality by leveraging the rich data already compiled from OSM, Overpass, Overture, satellite classification, and DEM elevation.

## Data We Have But Don't Use

The Rust pipeline compiles rich semantic data from multiple sources that the Lua builders largely ignore:

**Buildings:**
- `facadeStyle` — compiled, never read. Could drive window patterns, architectural diversity
- `roofLevels` — compiled, never read. Could improve multi-level roof complexity
- `usage` — read but under-leveraged. 41 usage types mapped to ~6 visual treatments

**Roads:**
- `sidewalk` — compiled as enum (both/left/right/no/separate), never read. Could drive precise curb placement instead of boolean `hasSidewalk`
- `layer` — compiled as integer for vertical stacking, never read. Critical for correct overpass/underpass rendering
- `subkind` — compiled (residential, primary, tertiary, etc.), exported to attributes but never used for visual differentiation
- `maxspeed`, `lanes`, `lit`, `oneway` — compiled but not visually expressed

**Terrain:**
- `materials[]` — satellite-derived per-cell ground classification compiled for every cell, but TerrainBuilder mostly falls back to slope-based Grass/Rock/Ground. The full satellite palette (Sand, Mud, Pavement, Limestone, etc.) is available but barely consumed.

**Water:**
- `color` — compiled RGB per water body, ignored. Hardcoded to single blue.

**Rails:**
- `color`, `lanes` — compiled, ignored.

## What The Player Sees Today (Problems)

- **Streets:** Flat grey `Enum.Material.Asphalt` everywhere. No distinction between highway, residential street, and alley despite `subkind` data being available. Sidewalks invisible — `sidewalk=both/left/right` is compiled but ignored. No curb. Road overpasses render at the same layer despite `layer` field being compiled.
- **Buildings:** Same flat `Brick` on every residential building despite `facadeStyle` being compiled. Roofs inherit wall material so the skyline is monochrome despite `roofLevels` being available. Windows identical with no tint variation. From aircraft altitude, rooftops are flat untextured rectangles.
- **Terrain:** Three materials total from slope thresholds. Satellite-derived per-cell materials sit in the manifest unused. From altitude, terrain looks like a green quilt instead of showing parking lots, paths, different ground cover.
- **Water:** Every water body is the same blue glass despite `color` data being compiled per body.
- **Distance:** Buildings pop out of existence at the streaming boundary. No LOD, no silhouette preservation, no atmospheric depth.

## What The Player Should See (Goals)

**Street level (80% of gameplay):**
- Road surface varies by `subkind`: highways are fresh dark asphalt, residential streets are lighter and weathered, alleys are cracked
- Curbs on the correct side using the `sidewalk` enum (both/left/right), not just boolean
- Road overpasses render at correct vertical layer
- `maxspeed` and `lanes` visually expressed: wider lanes for highways, narrower for residential
- Street lights driven by the `lit` field
- Building facades vary by `facadeStyle` when available
- Window tint varies by `usage`: office blue-grey, residential warm, warehouse opaque
- Ground-floor retail vs upper residential distinguishable

**Aerial (jetpack, parachute, aircraft):**
- Rooftops have material variety (Slate, Metal, Asphalt, Tile) using hash + `roofMaterial` + `roofLevels`
- Terrain shows satellite-derived ground cover: parks green, parking lots grey, bare earth brown, sand near water
- Water bodies have per-body color from compiled `color` field
- Skyline silhouette visible from 2x current distance via `Model.LevelOfDetail`
- Atmospheric haze sells world scale

**Hero moments (landmarks, close inspection):**
- Named landmark buildings get `SurfaceAppearance` with procedural PBR: normal maps for panel joints on glass facades, weathering on stone
- Rooftop HVAC/equipment on more buildings (players land here)

## Architecture Constraints

1. **Single-source runtime rule** (inherited): All fidelity improvements land in shared builders. No play-only visual patches.
2. **Runtime budget rule** (inherited): `estimatedMemoryCost` remains authoritative. MaterialVariant and SurfaceAppearance have memory cost — budget explicitly.
3. **Proof rule** (inherited): All visual changes proved on `tertiary`. Local machine = static/Python/Rust tests only.
4. **No new world-definition paths**: Improvements go through existing builders.
5. **Bounded memory**: SurfaceAppearance/EditableImage textures budget-gated per chunk by streaming ring. Far = material-only. Near = full PBR.
6. **Deterministic**: Same manifest + config = same visual output. Hash-based variation, not random.
7. **Use the data**: Every compiled manifest field that improves visual fidelity should be consumed by a builder. No more compiling data and ignoring it.

## Delivery

### Track 1: Use The Data We Already Have

The single highest-ROI work: make builders consume fields they currently ignore.

**TerrainBuilder:**
- Consume `terrainGrid.materials[]` satellite classification as primary material source when available
- Full material palette: Grass, Rock, Ground, Sand, Mud, Pavement, Limestone, Sandstone, Slate, etc.
- Fallback to slope-based classification only when satellite data absent
- Result: aerial view transforms from green quilt to readable ground cover

**RoadBuilder:**
- Consume `sidewalk` enum for precise curb placement (both/left/right/no/separate)
- Consume `layer` for overpass/underpass vertical stacking
- Consume `subkind` for visual differentiation: highway vs residential vs service road
- Consume `lit` to gate street light placement (already in PropBuilder, needs road-side wiring)
- Consume `lanes` for visual lane width scaling on highways

**BuildingBuilder:**
- Consume `facadeStyle` for window pattern and detail variation when available
- Consume `roofLevels` for multi-level roof geometry complexity
- Roof material: hash-based selection from Slate/Metal/Asphalt/Tile, weighted by `roofMaterial` when present

**WaterBuilder:**
- Consume `color` for per-body water color instead of hardcoded blue

### Track 2: Material Diversity via MaterialVariant

**Road surfaces (3-5 variants):**
- Asphalt: fresh (dark, smooth), standard (medium grey), weathered (light, rough)
- Concrete: clean, stained
- Variant selection: deterministic hash of road segment ID + `subkind`

**Building walls (5+ variants):**
- Brick: red, tan, dark
- Concrete: smooth, aged
- Selection: deterministic hash of building ID + `usage`

**Building windows:**
- Office: blue-grey tinted glass
- Residential: warm-tinted glass
- Night/empty: dark opaque
- Selection: building `usage` + position hash

**Rooftops (4 families):**
- Slate (grey-blue), Metal (silver-grey), Asphalt (dark), Tile (terracotta)
- Selection: building ID hash weighted by compiled `roofMaterial`

### Track 3: Curb Geometry and Street Detail

**Curb lip via EditableMesh:**
- 0.3-stud height, concrete material, placed per `sidewalk` enum
- Uses RoadBuilder's existing EditableMesh accumulator pattern
- Both/left/right placement from the compiled `sidewalk` field

**Road markings (stretch goal):**
- Crosswalk geometry at intersections (thin white stripe mesh above road)
- Lane line geometry on multi-lane roads (from `lanes` field)
- Approach: thin EditableMesh strips, not textures (simpler, no EditableImage dependency)

### Track 4: Distance, LOD, and Atmosphere

**Building LOD:**
- `Model.LevelOfDetail = Automatic` on all shell building models
- Extends visible skyline to 2x current streaming distance
- Preserves building silhouette during aircraft approach instead of pop-in

**Atmospheric depth:**
- Tune `Atmosphere` properties (Density, Offset, Decay, Haze) to sell world scale
- Distance haze that increases with altitude (aircraft/jetpack gameplay reads as "big world")
- Fog parameters exposed in WorldConfig for per-scene tuning

**Ring-based fidelity:**
- Near ring: full MaterialVariant + SurfaceAppearance where available
- Mid ring: MaterialVariant only (no PBR textures)
- Far ring: base Enum.Material only (minimum memory)

### Track 5: Hero PBR Surfaces

**SurfaceAppearance via EditableImage:**
- Procedural generation of normal/roughness maps at runtime
- 2-3 hero surface types:
  - Office glass facade: panel joint normal map, high metalness, reflective
  - Stone/concrete landmark: rough normal map, low metalness, weathering in roughness
  - Metal rooftop: industrial normal map, corrosion patterns
- Budget: near-ring only, landmark buildings only (above size or `name` threshold)
- EditableImage resolution: 256x256 per surface (bounded)
- Fallback: MaterialVariant-only for non-landmark or distant buildings

### Track 6: Rooftop Gameplay Surfaces

Players land on rooftops. They need to feel like real surfaces.

- Extend HVAC/equipment generation to buildings with 3+ levels (currently 5+)
- Rooftop edge lip geometry (parapet) on flat-roof buildings
- Rooftop material texture distinct from wall material
- Rooftop surface physics: slightly different friction than walls (more grip for jetpack landing)

## Success Thresholds

- Every compiled manifest field listed in "Data We Have But Don't Use" is consumed by a builder
- Satellite-derived terrain materials visible from altitude on tertiary
- Road `sidewalk` enum drives curb placement on tertiary
- Road `layer` produces correct overpass stacking on tertiary
- Building wall variation across 3+ material families visible on tertiary
- Rooftop material variety visible from simulated aircraft altitude on tertiary
- Street-level curb geometry visible in play mode on tertiary
- Skyline readable from 2x current streaming distance via LOD on tertiary
- Water bodies with different `color` values render differently on tertiary
- Deterministic: same manifest + config = identical visual output
- MaterialVariant and SurfaceAppearance costs budgeted per chunk within Roblox limits
- All existing tests remain green (Python 212+, Rust 205+, shell syntax)

## Roblox 2026 Platform Budget Constraints

These hard limits shape every track's design:

- **EditableImage**: max 1024x1024 per image, **32 MB total budget** per experience (all images combined)
  - 512x512 RGBA = 1 MB each → ~32 images, or ~16 with normal maps
  - 256x256 RGBA = 0.25 MB each → ~128 images
- **EditableMesh**: 20,000 triangles per mesh, ~60,000 vertices per mesh
  - `CreateMeshPartAsync` bakes and frees the editable budget (current approach is correct)
- **SurfaceAppearance**: no hard instance limit, but each EditableImage-backed texture costs EditableImage budget
  - Pre-uploaded asset-ID-backed textures have no EditableImage budget cost (only GPU memory, Roblox-managed)
- **Atmosphere**: single global instance, no per-ring density control
  - Use per-part Transparency ramping for ring-based depth
- **BillboardGui labels**: ~50-100 visible simultaneously before frame time impact

## Satellite Imagery Pipeline (Research Validated)

Satellite imagery draping on terrain is viable as of January 2026 via `AssetService:CreateSurfaceAppearanceAsync()`. Architecture:

1. **Offline compilation** (Rust pipeline): fetch satellite tiles from Mapbox Satellite (`mapbox.satellite-v9`, zoom 17-18, 0.3-2m/px global) or ESRI World Imagery during `arbx_cli compile`. Bake per-chunk 512x512 PNG textures into manifest artifacts.
2. **Runtime import** (Lua): generate flat or height-following EditableMesh grid per chunk (64x64 quads = 8,192 tris at 4-stud grid), assign UVs, create EditableImage from baked PNG via `WritePixelsBuffer()`, bind as SurfaceAppearance ColorMap.
3. **Normal map generation**: compute per-cell surface normals from DEM heightfield during compilation, encode into EditableImage NormalMap for free PBR hillshading.
4. **Budget management**: near-ring chunks get satellite overlay (512x512 = 1 MB color + 1 MB normal = 2 MB per chunk, ~8-10 chunks in 32 MB budget). Mid-ring gets MaterialVariant terrain only. Far-ring gets base Enum.Material. Recycle EditableImage on stream-out.
5. **Hybrid**: keep Terrain voxels underneath for physics/water/collision. EditableMesh overlay is visual only, 0.05 studs above terrain surface.

## Road Marking Architecture

Road markings (crosswalks, lane lines) via thin EditableMesh overlay geometry:
- White quads 0.02 studs above road surface
- Crosswalks at `highway=crossing` nodes with `crossing:markings` tags from OSM
- Center lane lines from `lanes` field (dashed for passing, solid for no-passing)
- Uses existing MeshAccumulator — a crosswalk costs ~8-16 triangles
- No EditableImage budget consumed (geometry approach, not texture)

## Label Architecture

Road and building names via BillboardGui:
- Road names: BillboardGui at road midpoint, `MaxDistance = 150`, semi-transparent background
- Building names: POI/landmark only (OSM `name=*` tag), `MaxDistance = 80`
- Street signs: small Part + SurfaceGui at intersections where two named roads meet
- Budget: ~20-30 BillboardGuis per near-ring chunk, 0 in mid/far

## What This Does Not Include

- Style resolver infrastructure (the canonical-feature-style-contract is the next tranche after visual quality is proven)
- Regional style packs
- Interior generation
- Traffic simulation or NPC systems
- New vehicle or aircraft types (those are gameplay, not rendering)
- Multi-city compilation (pipeline already supports it; this tranche proves Austin quality first)
- Quadtree multi-resolution chunk subdivision (pipeline change for future tranche)
- Pre-compiled mid-ring LOD variants (future tranche — use StreamingMesh for now)
