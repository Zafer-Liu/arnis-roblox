# Planetary Realism Sprint Status

Date: 2026-04-06
Status: Active

## Purpose

Rolling status and handoff document for the planetary realism sprint.

The active design spec is:

- `docs/superpowers/specs/2026-04-06-planetary-realism-sprint-design.md`

The active implementation plan is:

- `docs/superpowers/plans/2026-04-06-planetary-realism-sprint.md`

## Current Snapshot

- Inherited from the completed March 30 outdoor fidelity tranche: route-driven streaming proven, selective telemetry working, truth-pack audit infrastructure in place, all local and remote proof lanes verified
- Pipeline compiles rich data from OSM, Overpass, Overture, satellite classification, and DEM elevation that builders do not fully consume
- Key unused manifest fields identified: `facadeStyle`, `roofLevels`, `sidewalk` enum, `layer`, `subkind`, satellite `materials[]`, water `color`
- All existing tests green: Python 212+, Rust 205+, shell syntax, scaffold

## Verification Snapshot

### Local Static

- `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_play_render_truth scripts.tests.test_convergence_guardrails scripts.tests.test_run_studio_harness -v`
  - passed on 2026-04-07 (272 tests, up from 76 at tranche start)
- `cargo test --manifest-path rust/Cargo.toml --workspace`
  - passed on 2026-04-07 (360 tests, all green on primary)
- `git diff --check`
  - passed on 2026-04-07

### Remote `tertiary`

- `cargo test --manifest-path rust/Cargo.toml --workspace`
  - passed on 2026-04-06 (235 tests on tertiary)
- Austin compile with enriched pipeline:
  - `cargo run -p arbx_cli --release -- compile --source data/austin_overpass.json --bbox 30.245,-97.765,30.305,-97.715 --profile high --out out/austin-manifest.json`
  - compiled in 3.5s, 25,322 buildings, 46,374 roads
- Austin compile with satellite tiles:
  - `--satellite-tiles data/esri-tiles` at zoom 17
  - 7,400 chunks textured, 16MB ESRI tiles cached, compiled in 5.5 minutes
- Amsterdam live compile:
  - `--live --bbox 52.370,4.890,52.375,4.895 --world-name Amsterdam`
  - compiled in 8.2s from Overpass, 514 buildings, 619 roads, 160 rails
- `audit-signal` on real Austin manifest:
  - buildings: 45.1% signal rate (25,322 features)
  - roads: 42.9% signal rate (46,374 features)
  - water: 62.9% signal rate (1,462 features)
  - specialized fields (facadeStyle, roofDirection, etc.) 0% — tags don't exist in Austin OSM
- Studio play proof on tertiary:
  - `bootstrapState: "gameplay_ready"` — world loaded and running
  - Roof material diversity confirmed: concrete(16), brick(4), limestone(2), metal(2)
  - 38 readable facade cue parts, 21 wall parts, 6 nearby buildings
  - Route catalog streaming active
  - Screenshot captured Terminal window (not Studio viewport) — known display-capture limitation
- Second Studio proof on tertiary (2026-04-07 00:28 UTC):
  - Recompiled with fixed satellite texture paths (relative to manifest dir)
  - `gameplay_ready` confirmed again with enriched manifest
  - Named buildings flowing: Texas State Capitol, Texas Supreme Court Building, Capitol Extension, Texas Workforce Commission, John H. Reagan Building, Capitol Checkpoint North
  - 50 readable facade cue parts, 48 roof parts, 7 building models
  - ARNIS_CLIENT_PERF not emitting — telemetry family wiring to Workspace needs fix (env var not reaching attribute)
  - Display capture still blocked on tertiary

## Residual Gaps

- Satellite imagery draping on terrain (EditableImage approach) deferred pending MaterialVariant terrain results
- Road text labels and building signage deferred
- Style resolver infrastructure deferred to next tranche
- Regional style packs deferred

## Status Notes

### 2026-04-07: Session 3 — Full Planetary Streaming Pipeline

- **All 5 geometry types pre-computed in Rust + consumed in Lua**: buildings (shellMesh), roads (roadMesh + sidewalk/curb bundle), props (propMesh — broadleaf/conifer/palm), terrain (terrainMesh — heightfield grid), water (waterMesh — polygon fan + river ribbon)
- **Per-chunk texture atlas**: 512x512 PNG atlas for building facades with base64 encoding. Lua consumer decodes once per chunk, applies shared SurfaceAppearance. Gated behind `EnableBuildingAtlas`.
- **LOD cascade**: full/reduced/minimal per streaming ring. Re-import on ring transitions with in-flight tracking to prevent import storms.
- **Aircraft streaming prefetch**: Velocity-adaptive lookahead (walking/vehicle/aircraft classes). High-velocity prefetch forces minimal LOD, upgraded via existing re-import.
- **Per-ring memory budgets**: Near ring protected from far ring pressure. Ring-specific admission gates and forced eviction when over budget.
- **Regional style packs**: Austin (limestone), Amsterdam (brick), Tokyo (concrete), SF (stucco). Resolved from bbox centroid with fallback to defaults.
- **ARNIS_CLIENT_PERF root cause fixed**: Bootstrap was ignoring telemetry families when route config wasn't enabled. Fixed to read telemetryFamilies independently of route activation.
- **Senior code review**: 2 critical PropBuilder bugs fixed (double origin offset, double scale). LOD re-import storm fix.
- **Multi-city validation**: Austin 100%, SF 99.7%, Amsterdam 100% mesh coverage.
- **Full pipeline audit**: shellMesh 100%, roadMesh 100%, propMesh 30.5%, waterMesh 100%, atlasUv 27.4%, terrain 48.8% satellite.
- **360 Rust + 272 Python = 632 tests green**.
- **Tertiary cleaned**: 13GB freed (from 690MB).

### 2026-04-07: Session 2 — Hero PBR, Telemetry Fix, Sidewalk/Curb Rust, Austin Proof

- **Hero PBR surfaces merged**: Procedural SurfaceAppearance with EditableImage on named buildings >20 studs. Three facade types (office/stone/metal) with normal maps, roughness, metalness. Budget-gated to 10 per chunk. Session-level feature detection.
- **ARNIS_CLIENT_PERF telemetry fixed**: Harness now defaults `ARNIS_TELEMETRY_FAMILIES=client_perf` so frame profiling emits automatically on every play proof.
- **Sidewalk/curb geometry lowered into Rust**: `RoadMeshBundle` with surface + sidewalkLeft/Right + curbLeft/Right. `SidewalkMode` enum, lateral offset support, 13 new Rust tests. Lua consumer handles both bundle and legacy flat formats.
- **Double-roof bug fixed**: Rust `build_shell_mesh` stripped of flat roof slab — Lua `buildRoof` is sole roof owner.
- **Austin Capitol recompile on tertiary**: 100% shellMesh (401 buildings) and 100% roadMesh (3400 roads) coverage confirmed via audit-signal.
- **269 Rust + 265 Python tests green on both primary and tertiary**.

### 2026-04-07: Session 1 — Rust→Lua Pre-Computed Mesh Consumer Wiring

- **BuildingBuilder pre-computed mesh fast path**: `MeshAccumulator:addPrecomputedMesh()` loads Rust-compiled `shellMesh` (flat vertex/triangle/normal arrays) directly into EditableMesh accumulators, bypassing runtime `addOrientedBox` wall generation. Falls back to runtime generation when shellMesh absent. Coordinate conversion: chunk-local → world-space via originStuds offset. Index conversion: 0-based Rust → 1-based Lua.
- **RoadBuilder pre-computed mesh fast path**: `RoadMeshAccumulator:addPrecomputedMesh()` loads Rust-compiled `roadMesh` directly, skipping per-segment `addRoadStrip` for ground segments. Bridges, tunnels, sidewalks, and curbs still generate at runtime. Pre-computed mesh loaded once per road before segment loop.
- **Expected performance impact**: chunk import time reduction from ~167-232ms to <50ms for building/road geometry, since all trigonometry and oriented-box construction is pre-computed at compile time in Rust.
- **Full end-to-end pipeline**: `arbx_cli compile` → shellMesh/roadMesh in manifest JSON → Lua consumer loads flat arrays → EditableMesh. All 257 Rust + 265 Python tests green.
- **audit-signal updated**: `shellMesh` and `roadMesh` fields added to signal audit field lists for pre-computed mesh coverage tracking.
- **Telemetry counters**: `precomputedMeshCount` / `runtimeMeshCount` in both building and road build stats, wired to chunk profile for Studio telemetry.

### 2026-04-06: Satellite Imagery Pipeline + Full Builder Enrichment

- **Satellite tile pipeline in Rust**: `arbx_cli compile --satellite-tiles` fetches ESRI World Imagery tiles at zoom 17, composites per-chunk 512x512 PNG textures, writes `terrainTexturePath` to manifest. Tile coordinate math + fetcher with disk cache + 200ms rate limit.
- **Terrain satellite overlay in Lua**: `TerrainBuilder.BuildSatelliteOverlay` creates EditableMesh heightfield + EditableImage + SurfaceAppearance. Budget-gated to 10 chunks. pcall-wrapped. Ready for integration.
- **Window pane mesh merge**: ~200 Part instances per building → 3-4 EditableMesh instances per chunk. Gated behind `WorldConfig.MergeWindowsIntoMesh`.
- **Rust pipeline: facadeStyle + waterType + structureType** extracted from OSM. 7 new tests.
- **Lua builders consume all new fields**: facadeStyle now live, waterType preferred over kind, structureType as material hint.
- **Review pass 3**: Fixed table.sort stale data bug (wrong p99), instance count cache defeat, building.material type guard, equipment seed diversity, ureq version pin.
- 233 Rust + 112 Python tests green.
- Texture data flow gap identified: PNG file path needs to become embeddable buffer data for Roblox. Agent working on fix.

### 2026-04-06: Data Fidelity Pass + Profiling Infrastructure

- **Water kind differentiation**: rivers lighter/transparent, lakes deeper/reflective, ponds greener, wetlands murky. Per-body color always takes priority (no trampling).
- **Rail kind differentiation**: heavy rail=Metal thick, light rail=Metal thin, subway=Concrete wide. Previously all Cobblestone.
- **Prop leafType**: conifer species list (15 genera) triggers needleleaved canopy. Previously only palms got distinct geometry.
- **WallColor confirmed faithful**: grey rejection is exact (170,170,170) placeholder check only, no broad trampling.
- **Frame time profiling**: zero-alloc ring buffer in WorldProbe, ARNIS_CLIENT_PERF marker with avgFrameTime/p99/fps/instanceCount. Harness extraction ready.
- **Building LOD**: Model.LevelOfDetail=Automatic on all shells for skyline preservation.
- **Atmosphere depth**: WorldConfig knobs for density/offset/haze, additive on phase presets.
- **Ring transparency**: additive fade per ring (far=0.15, mid=0.05) with authored-value snapshot.
- **Rooftop gameplay**: parapets on flat roofs, equipment variety (AC/antenna/vent), threshold lowered to 3 levels.
- **Senior code review**: 3 passes, all issues resolved (forward reference crash, hashId caching, usage-based roof fallback, LOD tagging).
- 236 Python + 205 Rust tests all green.

Pipeline enrichment research complete — top Rust additions identified:
1. roof:direction + roof:angle (oriented gabled roofs)
2. sidewalk:surface + sidewalk:width (distinct sidewalk materials)
3. building:cladding + building:structure (precise facade materials)
4. road name (one-line Rust addition)
5. Overture roof_shape (available but not extracted)
6. arbx_cli audit-signal subcommand (designed, implementation lost to worktree cleanup — ready for next session)

### 2026-04-07: Mesh Rendering Root Cause + Automated Capture + Road/Building Polish

- **Root cause: SetVertexNormal not available on EditableMesh** — Roblox EditableMesh does not expose `SetVertexNormal`. Auto-normals derived from triangle winding order are the only rendering path. This is not a bug — it is the API surface. All mesh geometry must rely on correct winding for correct shading.
- **EditableMesh walls confirmed working in play mode** with original winding order. The mesh path is viable for buildings and roads without manual normals.
- **Rust mesh pre-computation foundation**: `mesh_builder.rs` (buildings) and `road_mesh.rs` (roads) landed in the Rust pipeline. These modules generate triangle meshes on the compile side, preparing for chunked mesh streaming to Lua.
- **Multi-angle automated capture pipeline**: 4-angle capture per run (front, left, back, right) using ScreenCaptureKit from SSH. Automated screenshot proof now covers multiple viewpoints per proof cycle.
- **Congress Ave compiled with 48 satellite texture modules** — satellite imagery pipeline proven at route scale.
- **Road contrast improved**: darker asphalt base colors, raised mesh lift above terrain to eliminate z-fighting.
- **Debug building color mode added** — toggleable false-color rendering for diagnosing builder output.
- **ScreenCaptureKit capture tool working from SSH** — replaces fragile `screencapture` path with a programmatic CoreGraphics capture that works in headless/remote sessions.
- **Lighting Migration dialog permanently fixed** — XML patching of the place file strips the migration flag before Studio opens, eliminating the modal dialog that blocked automation.
- **HarnessRouteConfig empty file bug found and fixed** — the untracked empty Lua file was silently breaking route config loading; now has proper content.
- **Senior engineer review completed**, all issues resolved across both sessions.
- **254 Rust + 115 Python tests green**.
- **65+ commits across two sessions** (Session 1: ~24 commits, Session 2: ~41 commits).
- **Ten-tranche plan for planetary streaming** designed and documented: covers mesh chunker wiring, LOD cascade, texture atlas, terrain mesh, water mesh, prop instancing, lighting/atmosphere, streaming integration, multi-city validation, and performance optimization.

### 2026-04-06: Tasks 1-4 Landed (Parallel Swarm)

- **Task 1: Terrain satellite materials** — TerrainBuilder now uses satellite-derived per-cell materials as PRIMARY source. Full palette: Sand, Mud, Pavement, Limestone, Sandstone, Slate, Asphalt, Concrete, Snow, Ice, Glacier, LeafyGrass. Slope-based Grass/Rock/Ground is fallback only. Transforms aerial view from green quilt to readable ground cover.
- **Task 2: Road data consumption** — RoadBuilder now reads `sidewalk` enum (both/left/right/no/separate) for curb placement, `layer` integer for overpass/underpass vertical separation (5 studs/layer, negative for tunnels), and `subkind` for visual material differentiation (motorway=dark asphalt, residential=weathered, track=earth). Curb geometry via EditableMesh accumulator.
- **Task 3: Building material diversity** — Roof material hash diversification from Slate/Metal/Asphalt/Brick palette (breaks monochrome skyline). Window tint by usage class (office=blue-grey, residential=warm, industrial=opaque dark, 20% night/vacancy darkening). facadeStyle consumed for detail spacing. roofLevels consumed for stepped roof geometry.
- **Task 4 (partial): Water color** — WaterBuilder now reads `color` RGB from manifest for per-body water color. Falls back to hardcoded blue when absent.
- Executed via 3 parallel worktree agents + main thread. All merged cleanly. 84 tests passing.

### 2026-04-06: Tranche Opened

- Closed the March 30 outdoor fidelity tranche (all tasks complete, route-driven proof end-to-end)
- Fixed pre-existing Rust test failures in route catalog module path resolution (common_path_root fix)
- Fixed Python test assertions for refactored TerrainBuilder neighbor sampling
- All 205 Rust tests, 212+ Python tests passing
- Pushed 51 commits to origin/main as sole source of truth
- Research completed: material system audit, multi-city feasibility, manifest data gap analysis
- Key finding: satellite-derived per-cell terrain materials are compiled but barely used by TerrainBuilder — single highest-ROI fix for aerial view
- Key finding: pipeline is already worldwide (Tokyo and London in CLI examples) — Austin is naming only
