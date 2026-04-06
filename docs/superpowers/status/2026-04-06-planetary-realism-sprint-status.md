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

- `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_play_render_truth scripts.tests.test_convergence_guardrails -v`
  - passed on 2026-04-06 (84 tests, up from 76 at tranche start)
- `cargo test --manifest-path rust/Cargo.toml --workspace`
  - passed on 2026-04-06 (205 tests)
- `git diff --check`
  - passed on 2026-04-06

### Remote `tertiary`

(To be populated as tasks complete)

## Residual Gaps

- Satellite imagery draping on terrain (EditableImage approach) deferred pending MaterialVariant terrain results
- Road text labels and building signage deferred
- Style resolver infrastructure deferred to next tranche
- Regional style packs deferred

## Status Notes

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
