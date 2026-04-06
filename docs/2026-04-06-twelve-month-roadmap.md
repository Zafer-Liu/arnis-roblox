# Twelve-Month Roadmap

Date: 2026-04-06
Status: Draft

## Vision

A gorgeous, real, immersive 3D world — traversable on foot, in vehicles, with jetpacks, parachutes, and aircraft. Built from free open geodata (OSM, Overpass, Overture, ESRI, DEM). Compiled in Rust for extreme performance. Rendered in Roblox at 60fps. Any city on Earth.

## Where We Are Today

The pipeline is architecturally complete: Rust compiles geodata into chunked manifests, Lua builders render them in Roblox with streaming, LOD, and telemetry. The Planetary Realism Sprint (April 2026) shipped:

- 15-material satellite terrain palette
- Building material diversity (roof hash, window tint, facade style, structure type)
- Road curbs, layer stacking, subkind differentiation, street labels, lane markings
- Water kind differentiation, per-body color
- Rail kind differentiation
- Conifer canopy shapes
- Window pane mesh merge (massive draw call reduction)
- Building LOD, atmospheric depth, ring-based transparency
- Zero-alloc frame profiler
- ESRI satellite tile pipeline (compile-time, fuzz-proven coordinate math)
- Terrain satellite texture overlay infrastructure
- `arbx_cli audit-signal` for quantitative signal preservation measurement
- Pipeline enrichment: road names, building cladding, roof direction/angle, sidewalk surface, facade style, water type, structure type

Test coverage: 235 Rust + 264 Python. Clippy clean. Stylua clean. Fuzz clean (481K runs).

## Phase 1: Prove and Profile (April 2026)

**Goal:** See it running. Measure the cost. Fix what breaks.

- Compile Austin with `--satellite-tiles` on tertiary
- Run `audit-signal` on real manifest — quantify signal preservation
- Run frame profiler — establish fps/instance count baseline
- Screenshot at street level + aerial altitude
- Fix whatever breaks with real data
- Widen proof to step-1, prefetch, retain route slices

**Success:** Screenshot that looks meaningfully better than before. Frame time numbers in hand.

## Phase 2: Satellite Imagery Live (May 2026)

**Goal:** Aerial view transforms from colored voxels to satellite photography.

- End-to-end satellite texture draping proven on tertiary
- Normal map generation from DEM heightfield (free PBR hillshading)
- Budget optimization: 512x512 near-ring, 256x256 mid-ring, material-only far-ring
- EditableImage recycling on chunk stream-out
- Performance verification: satellite overlay must cost <2ms per chunk import

**Success:** Flying over Austin in a jetpack and seeing real satellite ground texture.

## Phase 3: Multi-City Validation (June 2026)

**Goal:** Prove the pipeline is truly worldwide.

- Compile Tokyo (dense grid, different building patterns)
- Compile Amsterdam (canals, narrow streets, low-rise)
- Compile Dubai (high-rise, desert terrain, coastal)
- Run audit-signal on each — compare signal rates to Austin
- Fix city-specific builder assumptions
- Document per-city quality notes

**Success:** Three cities render correctly without code changes.

## Phase 4: Performance Optimization (July 2026)

**Goal:** 60fps on target hardware with full visual quality.

- Profile each builder: import time, instance count, memory per chunk
- Merge remaining Part instances into EditableMesh (road markings, facade cues)
- Implement SIMD-accelerated tile compositing in Rust (parallel pixel mapping)
- Pre-compiled mid-ring LOD variants (4-8x triangle reduction)
- Chunk import pipeline parallelism (builder independence analysis)
- MicroProfiler analysis on tertiary with real gameplay (jetpack flight, vehicle driving)

**Success:** p99 frame time under 16.67ms (60fps) during jetpack flight over Austin.

## Phase 5: Style Resolver (August–September 2026)

**Goal:** Formalize the four-layer rendering pipeline.

- Layer 3 (`StyleResolver.lua`): canonical feature → style selector → render recipe
- Migrate builder heuristics into resolver lookups
- Regional style registry: `us-sunbelt`, `jp-urban`, `uk-terraced`, `de-suburban`
- Per-style material packs (brick families, roof families, road surface families)
- Style audit: parity tests verifying resolver produces same output as current hardcoded paths
- Custom mesh plug-in surface (per-style-region bundles)

**Success:** `WorldConfig.StyleRegion = "jp-urban"` produces visually distinct Tokyo vs Austin.

## Phase 6: Hero PBR via Rust (September–October 2026)

**Goal:** Real PBR textures from real data, not procedural invention.

- Satellite facade color sampling at building centroids
- Normal map baking from building geometry (window grid spacing, material grain)
- Roughness/metalness from material classification
- Per-building PBR texture pack in compile output
- Lua loads pre-baked textures as SurfaceAppearance (no runtime generation)
- Budget: near-ring landmarks only, pre-uploaded asset IDs (no EditableImage cost)

**Success:** Close-range inspection of office building shows real panel joints and weathering.

## Phase 7: Road Detail and Signage (October–November 2026)

**Goal:** Streets feel real at eye level.

- Road name BillboardGui with MaxDistance culling (already scaffolded)
- Street signs at intersections (Part + SurfaceGui from road name data)
- Crosswalk geometry from OSM `highway=crossing` nodes
- Parking lane rendering from OSM `parking:lane` tags
- Turn lane arrows from `turn:lanes` tag
- Traffic signal props at OSM `traffic_signal` nodes

**Success:** Walking down a street and seeing real street names, crosswalks, and lane markings.

## Phase 8: Gameplay Integration (November–December 2026)

**Goal:** The world is alive, not just a static scene.

- Vehicle physics tuned per road surface (26 friction types already compiled)
- Jetpack/parachute landing detection using rooftop parapets and equipment
- Aircraft altitude-based LOD (velocity-scaled prefetch radius)
- Ambient sound from water proximity, road type, vegetation density
- Day/night reactive details (street lights from `lit` field, window tint darkening)

**Success:** Driving a car down a real street with real friction, then jetpacking to a rooftop.

## Phase 9: Interior Generation (Q1 2027)

**Goal:** Enter buildings.

- Room generation from building footprint subdivision
- Floor/wall material from building usage
- Window/door placement from facade geometry
- Furniture generation from room type
- Seamless transition from exterior to interior streaming

**Success:** Walk through a door and see an interior that makes sense for the building type.

## Phase 10: Online Companion and Scale (Q2 2027)

**Goal:** Scale beyond single-player, add online tools.

- Open Cloud REST API integration for route sessions, proof artifacts, fidelity summaries
- Multi-player chunk streaming (MemoryStore for cross-server coordination)
- Web companion for browsing compiled cities, viewing audit reports, managing style packs
- CI/CD pipeline: auto-compile on manifest push, auto-deploy to Roblox
- Planetary-scale quadtree LOD (multi-resolution chunk hierarchy)

**Success:** Two players in the same Tokyo instance, with a web dashboard showing live metrics.

## Non-Negotiable Principles (Entire Roadmap)

1. **Free data only** — ESRI, OSM, Overpass, Overture, AWS Terrain, Copernicus. No paid APIs.
2. **Rust-first performance** — heavy computation in compile step, not runtime.
3. **Signal preservation** — every source field faithfully expressed. No invented style when data exists.
4. **60fps or die** — profile before shipping. Cut features that cost frames.
5. **Deterministic** — same input = same output. Hash-based diversity, no randomness.
6. **Measure everything** — audit-signal, frame profiler, instance counts. Evidence before assertions.
7. **Prove on tertiary** — visual proof required. Code that passes tests but looks wrong is wrong.
8. **Commit before cleanup** — never lose work to worktree destruction.
9. **What would Gabe Newell do** — ship the frame, not the feature.

## Key Metrics to Track

| Metric | Current | Phase 1 Target | Phase 4 Target | Phase 10 Target |
|--------|---------|----------------|----------------|-----------------|
| Rust tests | 235 | 250+ | 300+ | 400+ |
| Python tests | 264 | 280+ | 350+ | 500+ |
| Signal preservation (audit-signal) | Unknown | Measured | >90% | >95% |
| Frame time (p99) | Unknown | Measured | <16.67ms | <16.67ms |
| Instance count per chunk | Unknown | Measured | <5,000 | <3,000 |
| Cities compiled | 1 (Austin) | 1 | 4+ | 10+ |
| Satellite tile coverage | 0 | Austin downtown | 4 cities | Global z17 cache |
