# Planetary Realism Sprint Implementation Plan

Status: Active

This is the active implementation surface for the April 6 planetary realism sprint.

> **For agentic workers:** Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Ship visible material quality by consuming the rich compiled data the builders currently ignore, adding MaterialVariant diversity, curb geometry, LOD, atmospheric depth, and hero PBR surfaces. Prove every change on `tertiary`.

**Architecture:** No new world-definition paths. All improvements flow through existing builders (TerrainBuilder, BuildingBuilder, RoadBuilder, WaterBuilder). Budget MaterialVariant and SurfaceAppearance per chunk by streaming ring.

**Tech Stack:** Luau builders, Python contract tests, shell harness, `tertiary` Studio proof

---

## File Map

### Builders (Primary Changes)

- Modify: `roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/Builders/RoadBuilder.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/Builders/WaterBuilder.lua`

### Configuration

- Modify: `roblox/src/ReplicatedStorage/Shared/WorldConfig.lua`

### Streaming/LOD

- Modify: `roblox/src/ServerScriptService/ImportService/StreamingService.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/ChunkPriority.lua`

### Atmosphere

- Modify: `roblox/src/StarterPlayer/StarterPlayerScripts/DayNightCycle.lua`

### Tests

- Modify: `scripts/tests/test_austin_runtime_contract.py`
- Modify: `scripts/tests/test_play_render_truth.py`
- Create: `roblox/src/ServerScriptService/Tests/TerrainSatelliteMaterials.spec.lua`
- Create: `roblox/src/ServerScriptService/Tests/RoadSidewalkCurb.spec.lua`
- Create: `roblox/src/ServerScriptService/Tests/BuildingMaterialVariant.spec.lua`

### Docs

- Modify: `docs/superpowers/status/2026-04-06-planetary-realism-sprint-status.md`

## Success Thresholds

- Every compiled manifest field listed in the spec's "Data We Have But Don't Use" is consumed by a builder
- Satellite-derived terrain materials visible from altitude on tertiary
- Road `sidewalk` enum drives curb placement on tertiary
- Building wall variation across 3+ material families visible on tertiary
- Rooftop material variety visible from simulated aircraft altitude on tertiary
- All existing tests remain green (Python 212+, Rust 205+, shell syntax)

## Task 1: Terrain Satellite Material Consumption

**Files:**
- Modify: `roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua`
- Create: `roblox/src/ServerScriptService/Tests/TerrainSatelliteMaterials.spec.lua`
- Modify: `scripts/tests/test_play_render_truth.py`

- [ ] **Step 1: Write failing test that requires satellite material palette usage**

Create `TerrainSatelliteMaterials.spec.lua` and extend `test_play_render_truth.py` to require:
- When `terrainGrid.materials[]` is populated, TerrainBuilder uses satellite material as primary
- Full material palette includes Sand, Mud, Pavement, Limestone, Sandstone, Slate beyond Grass/Rock/Ground
- Fallback to slope-based when satellite data absent

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Update TerrainBuilder to consume satellite materials as primary source**

Update the material classification in `TerrainBuilder.lua` so `terrainGrid.materials[]` is the primary material source. The slope-based Grass/Rock/Ground classification becomes the fallback only when satellite data is absent.

- [ ] **Step 4: Re-run tests**

- [ ] **Step 5: Append status note**

## Task 2: Road Data Consumption (sidewalk, layer, subkind)

**Files:**
- Modify: `roblox/src/ServerScriptService/ImportService/Builders/RoadBuilder.lua`
- Create: `roblox/src/ServerScriptService/Tests/RoadSidewalkCurb.spec.lua`
- Modify: `scripts/tests/test_austin_runtime_contract.py`

- [ ] **Step 1: Write failing tests**

Require:
- `sidewalk` enum (both/left/right/no/separate) drives curb placement geometry
- `layer` field produces vertical offset for overpasses
- `subkind` drives visual material differentiation (highway darker asphalt, residential lighter)

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement sidewalk curb geometry**

Add 0.3-stud curb lip via EditableMesh on the correct side(s) per `sidewalk` enum. Use RoadBuilder's existing mesh accumulator pattern.

- [ ] **Step 4: Implement layer-based vertical offset**

Roads with `layer > 0` get a Y offset proportional to layer. Roads with `layer < 0` render as tunnels (existing `tunnel` flag may already handle this).

- [ ] **Step 5: Implement subkind-based material variation**

Highway/trunk → dark fresh asphalt. Residential/service → lighter weathered asphalt. Track/path → ground material. Use existing `SURFACE_MATERIAL` map, keyed by `subkind` when available.

- [ ] **Step 6: Re-run tests**

- [ ] **Step 7: Append status note**

## Task 3: Building Material Diversity

**Files:**
- Modify: `roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua`
- Create: `roblox/src/ServerScriptService/Tests/BuildingMaterialVariant.spec.lua`
- Modify: `scripts/tests/test_austin_runtime_contract.py`

- [ ] **Step 1: Write failing tests**

Require:
- `facadeStyle` consumed when available for window pattern variation
- `roofLevels` consumed for multi-level roof complexity
- Roof material hash diversification (Slate/Metal/Asphalt/Tile) using `roofMaterial` + building ID hash
- Wall MaterialVariant: 3 brick variants, 2 concrete variants via building ID hash
- Window tint varies by `usage`: office blue-grey, residential warm

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement roof material diversification**

Hash building ID to select roof material from [Slate, Metal, Asphalt, Tile], weighted by compiled `roofMaterial` when present. Stop inheriting wall material for roofs.

- [ ] **Step 4: Implement wall MaterialVariant selection**

Add MaterialVariant creation for brick (red/tan/dark) and concrete (smooth/aged). Selection via deterministic building ID hash + `usage`.

- [ ] **Step 5: Implement window tint variation**

Office → blue-grey glass (Color3 tint), Residential → warm glass, Warehouse → opaque. Some windows dark (night/empty) via position hash.

- [ ] **Step 6: Consume facadeStyle and roofLevels**

When `facadeStyle` is present, use it to influence facade detail spacing and window pattern. When `roofLevels` is present, generate multi-tier roof geometry.

- [ ] **Step 7: Re-run tests**

- [ ] **Step 8: Append status note**

## Task 4: Water Color and LOD

**Files:**
- Modify: `roblox/src/ServerScriptService/ImportService/Builders/WaterBuilder.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/StreamingService.lua`
- Modify: `roblox/src/ReplicatedStorage/Shared/WorldConfig.lua`
- Modify: `scripts/tests/test_austin_runtime_contract.py`

- [ ] **Step 1: Write failing tests**

Require:
- WaterBuilder consumes `color` field for per-body water color
- `Model.LevelOfDetail = Automatic` set on building shell models
- WorldConfig exposes atmosphere/fog tuning knobs

- [ ] **Step 2: Implement water color consumption**

WaterBuilder reads `color` RGB from manifest, falls back to current hardcoded blue.

- [ ] **Step 3: Implement building LOD tagging**

Set `Model.LevelOfDetail = Enum.ModelLevelOfDetail.Automatic` on building shell models in BuildingBuilder.

- [ ] **Step 4: Add atmosphere configuration**

Add fog/haze parameters to WorldConfig. Update DayNightCycle to read atmosphere depth settings.

- [ ] **Step 5: Re-run tests**

- [ ] **Step 6: Append status note**

## Task 5: Hero PBR Surfaces

**Files:**
- Modify: `roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua`
- Modify: `scripts/tests/test_austin_runtime_contract.py`

- [ ] **Step 1: Write failing tests**

Require:
- SurfaceAppearance creation gated by streaming ring (near only)
- SurfaceAppearance gated by building landmark threshold (named buildings above size threshold)
- EditableImage used for procedural normal/roughness map generation

- [ ] **Step 2: Implement ring-gated SurfaceAppearance**

Near-ring landmark buildings get SurfaceAppearance with procedural EditableImage textures. Mid/far-ring buildings get MaterialVariant only.

- [ ] **Step 3: Implement procedural PBR texture generation**

Generate 256x256 EditableImage for:
- Office glass: panel joint normal map, high metalness
- Stone facade: rough normal map, weathering in roughness
- Metal rooftop: industrial normal map, corrosion pattern

- [ ] **Step 4: Re-run tests**

- [ ] **Step 5: Append status note**

## Task 6: Rooftop Gameplay Surfaces

**Files:**
- Modify: `roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua`
- Modify: `scripts/tests/test_austin_runtime_contract.py`

- [ ] **Step 1: Write failing tests**

Require:
- HVAC equipment on buildings with 3+ levels (lowered from 5+)
- Rooftop edge lip (parapet) on flat-roof buildings
- Rooftop material distinct from wall material

- [ ] **Step 2: Extend rooftop equipment to more buildings**

Lower the HVAC generation threshold from 5 levels to 3 levels. Add more variety: antenna, vent, water tank.

- [ ] **Step 3: Add parapet geometry**

Small lip around flat roof edges via existing mesh accumulator.

- [ ] **Step 4: Re-run tests**

- [ ] **Step 5: Append status note**

## Task 7: Full Verification and Tertiary Proof

- [ ] **Step 1: Run full local-safe verification**

```bash
python3 -m unittest discover -s scripts/tests -v
cargo test --manifest-path rust/Cargo.toml --workspace
bash -n scripts/run_studio_harness.sh scripts/run_studio_harness_remote.sh
python3 scripts/check_scaffold.py
git diff --check
```

- [ ] **Step 2: Tertiary edit proof**

Run harness on tertiary with edit-only mode. Capture screenshots at street level and aerial altitude.

- [ ] **Step 3: Tertiary play proof**

Run harness on tertiary with play mode. Jetpack traversal for altitude validation.

- [ ] **Step 4: Update status doc with proof results**

- [ ] **Step 5: Commit**
