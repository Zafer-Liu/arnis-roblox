# Play Fidelity And Observability Status

Date: 2026-03-28
Status: Active

## Purpose

This is the current rolling status and handoff document for the active play-fidelity-and-observability tranche.

Use this file as the active status trail for:

- what has landed after the completed March 28 baseline tranche
- what was verified locally and on `tertiary`
- what still needs measured follow-up for walls, terrain, interiors, and hotspots

The active implementation plan for this tranche is:

- `docs/superpowers/plans/2026-03-28-play-fidelity-and-observability.md`

The active design spec for this tranche is:

- `docs/superpowers/specs/2026-03-28-play-fidelity-and-observability-design.md`

## Current Snapshot

- The March 28 canonical baseline tranche is complete and historical.
- Shared resolved config is aligned across preview and runtime startup import.
- Player-local telemetry now includes support, enclosure, and roof-cover fields.
- Roof-closure decks are marked as internal support instead of visible roof truth.
- On `tertiary`, `GabledRoofClosureTruth.spec.lua` passed after the roof-closure change.
- On `tertiary`, play telemetry now surfaces shell-mesh building evidence at gameplay-ready spawn (`nearbyMergedBuildingMeshParts=5`), where the earlier read showed `0`.
- The remaining wall-gap signal was a `WorldProbe` measurement artifact; shell-wall proximity now uses surface distance instead of merged-mesh centroids, and `WorldProbeGeometry.spec.lua` passed on `tertiary`.
- The first interior-overlap fix is now in place; room-authored `shellMesh` buildings no longer lay down shell terrain fill when interiors are enabled, and `RoomInteriorShellFillTruth.spec.lua` passed on `tertiary`.
- Top-floor room ceilings now clamp to the imported shell top; the strengthened `RoomInteriorShellFillTruth.spec.lua` passed on `tertiary` after proving ceiling tops stay at or below `ArnisImportBuildingTopY` and below the lowest roof bottom.
- Terrain explicit-material truth is now stronger for sub-4-stud source cells; `TerrainQuantizedMaterialTruth.spec.lua` passed on `tertiary` after replacing last-writer-wins overlap writes with voxel-center source-cell ownership.
- Ground-support observability is now stronger at spawn; `WorldProbeSupport.spec.lua` passed on `tertiary` after excluding the hidden runtime spawn, skipping decorative road detail, and trusting explicit road surface roles.
- On `tertiary`, gameplay-ready play telemetry now resolves support as terrain at the sampled spawn (`supportSurfaceRole=terrain`, `groundMaterial=Enum.Material.Grass`, `supportY=5.1`, `terrainY=5.1`) instead of the earlier `unknown` read.
- On `tertiary`, gameplay-ready play telemetry now also shows nearby shell evidence at spawn (`nearbyWallParts=4`, `nearestWallDistanceStuds=2.2`, `overheadRoofParts=2`, `overheadRoofMinClearanceStuds=12.4`), so the old wall/support probe blind spot is no longer the active signal.
- The biggest current measured hotspot is still preview/edit builder cost on `tertiary`: one slow building-heavy chunk around `166ms`, and full preview sync around `17.3s` for the current `80`-chunk bounded scene.
- The manifest audit is now stronger for terrain complaints: it reports area-weighted terrain material coverage, dominant-material ratio, terrain area by `cellSizeStuds`, coarse-granularity findings, and focused local-zone terrain accounting clipped to included terrain cells instead of whole intersecting chunks.
- The scene fidelity audit now carries the same manifest terrain granularity/material context into edit/play artifacts, so terrain complaints can be interpreted next to the same render/audit output used for parity and play-proof debugging.
- Preview telemetry now preserves the current slow chunk as structured state, clears stale `lastSlowChunk` state on `sync_started`, and the preview telemetry summary now prints compact hotspot timing (`last_sync_elapsed_ms`, `slow_chunk`, `slow_chunk_total_ms`, `slow_chunk_buildings_ms`, `slow_chunk_terrain_ms`, `slow_chunk_roads_ms`, `slow_chunk_landuse_terrain_fill_ms`, `slow_chunk_artifacts`) instead of hiding that data in raw Studio logs only.
- The scene fidelity audit now also emits explicit player-local exposure findings (`client_local_support_unknown`, `client_local_enclosure_gap`, `client_local_roof_cover_gap`) and shows nested `localSupport` / `localEnclosure` / `localRoofCover` metrics directly in the HTML report.
- Player-local telemetry now also includes structured `localTerrain` roughness metrics (`status`, sample coverage, center/min/max terrain Y, height range, max step, mean absolute step), and both scene fidelity and parity audits now preserve those fields.
- On `tertiary`, the new pure terrain reducer is edit-verified through `WorldProbeTerrain.spec.lua`, but the current real play-log artifact still truncates long `ARNIS_CLIENT_WORLD(_COMPACT)` lines before `localTerrain` survives into remote log-derived reports.

## Verification Snapshot

### Local Static

- `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - passed on 2026-03-28
  - verifies the new terrain quantization contract, shared support helper wiring, and updated player-local observability contract
- `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_scene_parity_audit scripts.tests.test_scene_fidelity_audit -v`
  - passed on 2026-03-28
  - verifies shared resolved config, roof-closure support truth, and richer client-world observability contracts
- `python3 -m unittest scripts.tests.test_manifest_quality_audit scripts.tests.test_scene_fidelity_audit -v`
  - passed on 2026-03-28
  - verifies area-weighted terrain audit fields/findings and scene-fidelity propagation of manifest terrain granularity/material context
- `python3 -m unittest scripts.tests.test_manifest_quality_audit scripts.tests.test_scene_fidelity_audit scripts.tests.test_preview_telemetry_summary -v`
  - passed on 2026-03-28
  - verifies area-weighted terrain audit fields/findings, scene-fidelity propagation of manifest terrain context, and compact preview slow-chunk summary formatting
- `python3 -m unittest scripts.tests.test_scene_fidelity_audit.SceneFidelityAuditTests.test_report_surfaces_player_local_exposure_findings_and_html_metrics scripts.tests.test_scene_fidelity_audit.SceneFidelityAuditTests.test_report_preserves_structured_local_support_and_enclosure_client_world_fields -v`
  - passed on 2026-03-28
  - verifies explicit player-local exposure findings plus nested local support/enclosure/roof metrics in the scene-fidelity HTML surface
- `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_scene_fidelity_audit scripts.tests.test_scene_parity_audit scripts.tests.test_play_render_truth -v`
  - passed on 2026-03-28
  - verifies the new `localTerrain` runtime contract, scene-fidelity carry-through, parity normalization, and existing play-render truth coverage
- `git diff --check`
  - passed on 2026-03-28

### Remote `tertiary`

- `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_scene_parity_audit scripts.tests.test_scene_fidelity_audit -v`
  - passed on 2026-03-28 in `~/.codex-remote-studio/arnis-roblox`
- `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_scene_fidelity_audit scripts.tests.test_scene_parity_audit -v`
  - passed on 2026-03-28 in `~/.codex-remote-studio/arnis-roblox`
  - verifies the `localTerrain` runtime contract plus fidelity/parity carry-through from the remote clone
- `bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter WorldProbeTerrain.spec.lua --edit-wait 30 --pattern-wait 120`
  - passed on 2026-03-28 in the remote clone
  - `PASS WorldProbeTerrain.spec`
  - `ARNIS_MCP_EDIT_ACTION` reported `total=1 passed=1 failed=0`
  - verifies the pure local-terrain roughness reducer on the real `tertiary` Studio lane after taking the sparse-sample case red first
- `bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter GabledRoofClosureTruth.spec.lua --edit-wait 30 --pattern-wait 120`
  - passed on 2026-03-28 in the remote clone
  - `PASS GabledRoofClosureTruth.spec`
  - `ARNIS_MCP_EDIT_ACTION` reported `total=1 passed=1 failed=0`
- `bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter WorldProbeGeometry.spec.lua --edit-wait 30 --pattern-wait 120`
  - passed on 2026-03-28 in the remote clone
  - `PASS WorldProbeGeometry.spec`
  - `ARNIS_MCP_EDIT_ACTION` reported `total=1 passed=1 failed=0`
- `bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter RoomInteriorShellFillTruth.spec.lua --edit-wait 30 --pattern-wait 120`
  - passed on 2026-03-28 in the remote clone
  - `PASS RoomInteriorShellFillTruth.spec`
  - `ARNIS_MCP_EDIT_ACTION` reported `total=1 passed=1 failed=0`
- `bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter TerrainQuantizedMaterialTruth.spec.lua --edit-wait 30 --pattern-wait 120`
  - passed on 2026-03-28 in the remote clone
  - `PASS TerrainQuantizedMaterialTruth.spec`
  - `ARNIS_MCP_EDIT_ACTION` reported `total=1 passed=1 failed=0`
- `bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter WorldProbeSupport.spec.lua --edit-wait 30 --pattern-wait 120`
  - passed on 2026-03-28 in the remote clone
  - `PASS WorldProbeSupport.spec`
  - `ARNIS_MCP_EDIT_ACTION` reported `total=1 passed=1 failed=0`
- `bash scripts/run_studio_harness_remote.sh --remote-profile tertiary --remote-host tertiary -- --no-play --edit-tests --spec-filter AustinPreviewTelemetry.spec.lua --edit-wait 30 --pattern-wait 120`
  - passed on 2026-03-28 against `tertiary`
  - remote edit action reported `total=1 passed=1 failed=0`
  - verifies both that a populated `lastSlowChunk` survives the compact workspace JSON flush and that `sync_started` clears stale hotspot state before the next flush
  - the wrapper again lingered after success, so cleanup was completed directly over SSH and `tertiary` was left clean afterward
- `bash scripts/run_studio_harness.sh --takeover --hard-restart --skip-edit-tests --play-wait 30 --pattern-wait 120`
  - rerun on 2026-03-28 from the remote clone
  - authoritative play proof remained green
  - gameplay-ready client telemetry improved shell-mesh evidence from `nearbyMergedBuildingMeshParts=0` to `5`
  - gameplay-ready client telemetry now resolves support as terrain at the sampled spawn (`supportSurfaceRole=terrain`, `groundMaterial=Enum.Material.Grass`, `supportY=5.1`, `terrainY=5.1`)
  - gameplay-ready client telemetry now shows nearby shell walls and roof cover (`nearbyWallParts=4`, `nearestWallDistanceStuds=2.2`, `overheadRoofParts=2`, `overheadRoofMinClearanceStuds=12.4`)
  - the same run reached `gameplay_ready` and emitted the normal client-world markers plus `ARNIS_MCP_PLAY`, but the raw Studio log still truncates long `ARNIS_CLIENT_WORLD(_COMPACT)` lines before the new nested `localTerrain` block is visible in the remote artifact

## Residual Gaps

- Terrain fidelity still needs dedicated work; explicit material collapse is fixed, but the current observed issue set still includes inherent 4-stud write-grid boxiness and broader detail questions that this tranche has not yet resolved.
- Terrain observability is better, and runtime/player-local terrain roughness is now emitted locally, but the current remote play-log artifact still drops that nested `localTerrain` block under long-line truncation and still lacks runtime voxel-material coverage from play.
- Interior work still needs a dedicated follow-up pass; shell terrain fill and top-floor ceiling overshoot are fixed, but richer traversal/interior detail and any remaining multi-level ceiling/roof edge cases are still open.
- Preview/edit hotspot export is better, but still incomplete; the last slow chunk is now structured, yet it is not joined into scene parity/fidelity outputs or chunk-level hotspot comparison reports yet.
- Player-local observability is stronger, but still incomplete; the audit now surfaces local support/enclosure/roof-cover signals, yet it still lacks local terrain roughness/step metrics and explicit interior-presence quantification.
- Player-local observability is stronger, but still incomplete; the audit now surfaces local support/enclosure/roof-cover plus nested `localTerrain` signals, yet it still lacks reliable remote artifact capture for that terrain block and still lacks explicit interior-presence quantification.
- Remote screenshot capture on `tertiary` is still best-effort only.

## Status Notes

### 2026-03-28: Roof/Internal Support And Shell-Mesh Observability

- Added failing contract checks for roof-closure support handling and shell-mesh wall observability.
- Marked roof-closure decks as internal support in `BuildingBuilder`.
- Updated `WorldProbe` to count shell-mesh descendants under building `Shell` folders and to ignore roof-closure decks as visible roof truth.
- Verified locally with Python contract/audit tests and remotely on `tertiary` with `GabledRoofClosureTruth.spec.lua`.
- Re-ran the play-only proof on `tertiary` and confirmed improved shell-mesh telemetry (`nearbyMergedBuildingMeshParts=5`) with the remaining wall-count gap still open.

### 2026-03-28: Wall Probe Surface Distance

- Reproduced the remaining wall-gap signal as a `WorldProbe` measurement artifact: shell meshes were present, but the probe was using `descendant.Position` (merged-mesh centroids) instead of nearest shell-surface distance.
- Added `ReplicatedStorage.Shared.WorldProbeGeometry` and switched shell-wall proximity checks in `WorldProbe.client.lua` to use `GetClosestPointOnSurface(...)` with centroid fallback.
- Added `WorldProbeGeometry.spec.lua`, took it red on `tertiary`, then verified it green on `tertiary`.
- Result: the repo now has measured evidence that the earlier `nearbyWallParts=0` read did not prove missing wall rendering.

### 2026-03-28: Roomed Shell Buildings Skip Terrain Fill

- Added `RoomInteriorShellFillTruth.spec.lua` to prove that authored room interiors should not keep shell terrain occupancy at room-floor height.
- Took that spec red on `tertiary`, where it failed with terrain occupancy still present inside the authored room footprint.
- Updated `BuildingBuilder.MeshBuildAll` to skip shell terrain fill for room-authored `shellMesh` buildings when interiors are enabled.
- Re-ran `RoomInteriorShellFillTruth.spec.lua` on `tertiary`; it passed.
- The remaining interior fidelity work is now narrower: top-floor ceiling/roof overlap and richer interior traversal, not shell terrain occupying authored rooms.

### 2026-03-28: Top-Floor Ceiling Clamp

- Strengthened `RoomInteriorShellFillTruth.spec.lua` so it no longer only checks room air volume; it now also asserts that the highest room ceiling top stays at or below `ArnisImportBuildingTopY` and below the lowest roof or roof-closure bottom.
- Took that spec red on `tertiary`, where the new assertion failed with the top-floor ceiling extending above the imported building top.
- Updated `RoomBuilder` to clamp ceiling center placement against the imported shell top (`ArnisImportBuildingTopY`) before batching ceiling slabs.
- Added a focused static contract check in `scripts/tests/test_play_render_truth.py` for the ceiling clamp wiring.
- Re-ran `RoomInteriorShellFillTruth.spec.lua` on `tertiary`; it passed.
- Result: the known single-level top-floor ceiling/roof overlap is fixed without changing roof geometry or closure-deck truth.

### 2026-03-28: Terrain Quantized Material Truth

- Added `TerrainQuantizedMaterialTruth.spec.lua` to prove that sub-4-stud explicit terrain materials should quantize by the 4-stud write voxel's center-owning source cell rather than by overlap iteration order.
- Took that spec red on `tertiary`, where voxel A picked the wrong explicit material under the old overlap-write path.
- Updated `TerrainBuilder` to precompute owning source cells for X/Z voxel centers and to let only that source cell write occupancy/material for each 4-stud voxel.
- Extended `TerrainQuantizedMaterialTruth.spec.lua` with a misaligned negative-origin `cellSizeStuds=3` case so the contract is no longer limited to one positive-origin `cellSizeStuds=2` layout.
- Re-ran `TerrainQuantizedMaterialTruth.spec.lua` on `tertiary`; it passed.
- Result: explicit terrain material truth is better aligned, but terrain geometry still remains constrained by Roblox's 4-stud write resolution.

### 2026-03-28: Ground Support Probe Truth

- Added `WorldProbeSupport.spec.lua` to lock the remaining support-surface observability gap.
- Took that spec red on `tertiary`, where the missing shared helper surfaced the old name-only probe behavior.
- Added `ReplicatedStorage.Shared.WorldProbeSupport` and updated `WorldProbe.client.lua` to skip the hidden runtime `SpawnLocation`, ignore decorative `Roads/Detail` hits, ignore non-world helper hits, and trust `ArnisRoadSurfaceRole` before falling back to name/ancestor heuristics.
- Expanded `WorldProbeSupport.spec.lua` to exercise the real retrying raycast loop against hidden spawn, decorative road detail, explicit road surface, and terrain fallback.
- Re-ran `WorldProbeSupport.spec.lua` on `tertiary`; it passed.
- Re-ran the play-only proof on `tertiary`; gameplay-ready telemetry now resolves `supportSurfaceRole=terrain` with `groundMaterial=Enum.Material.Grass`, `nearbyWallParts=4`, and nonzero roof cover at the sampled spawn.
- Result: the earlier gameplay-ready `supportSurfaceRole = "unknown"` symptom is corrected at the probe layer, and the active measured play truth is terrain-backed support rather than an unclassified hit.

### 2026-03-28: Area-Weighted Terrain Audit And Scene Carry-Through

- Strengthened `scripts/manifest_quality_audit.py` so terrain complaints are no longer evaluated only by chunk-level unique-material monotony.
- Added area-weighted terrain summary fields: dominant material ratio, terrain material area distribution, terrain area by `cellSizeStuds`, and coarse-granularity chunk ratio.
- Added terrain-specific findings for dominant-material collapse and coarse authored terrain granularity.
- Tightened focused local-zone terrain accounting so reports only credit included terrain cells instead of whole intersecting chunks.
- Carried the same terrain context into `scripts/scene_fidelity_audit.py`, so edit/play scene audit artifacts now expose manifest-side terrain granularity/material truth directly.
- Verified locally with `python3 -m unittest scripts.tests.test_manifest_quality_audit scripts.tests.test_scene_fidelity_audit -v`.
- Result: the audit surface is more useful for the current “boxy/default terrain” complaints, and the next missing telemetry is now clearer: local terrain roughness/coverage metrics in play plus structured preview slow-chunk hotspot export.

### 2026-03-28: Structured Preview Slow-Chunk Telemetry

- Updated `AustinPreviewTelemetry.lua` so `slow_chunk` events persist as `lastSlowChunk` in the compact preview snapshot alongside `lastSync`, and `sync_started` clears stale slow-chunk state before the next run.
- Updated `AustinPreviewBuilder.lua` so the slow-chunk path records a structured telemetry event instead of only logging text.
- Updated `preview_telemetry_summary.py` so operators see `last_sync_elapsed_ms` plus the current slow-chunk timing breakdown directly in the summary line.
- Verified locally with `python3 -m unittest scripts.tests.test_preview_telemetry_summary -v`.
- Verified remotely on `tertiary` with `AustinPreviewTelemetry.spec.lua`, which now passes through the edit-only harness lane while covering both the populated hotspot flush path and the stale-hotspot reset path.
- Result: preview hotspot timing is now available as compact structured state, but it still needs to be joined into the higher-level edit/play audit reports.

### 2026-03-28: Player-Local Exposure Audit Findings

- Added a red-phase Python audit test proving the scene-fidelity report was not yet surfacing high-signal player-local exposure states from the existing `clientWorld` payload.
- Updated `scripts/scene_fidelity_audit.py` so `localSupport`, `localEnclosure`, and `localRoofCover` contribute explicit findings when the client-world payload says the player is locally unsupported, unenclosed, or uncovered.
- Updated the HTML report so nested local support/enclosure/roof metrics are visible directly in the metric strip instead of requiring raw JSON inspection.
- Verified locally with `python3 -m unittest scripts.tests.test_scene_fidelity_audit.SceneFidelityAuditTests.test_report_surfaces_player_local_exposure_findings_and_html_metrics scripts.tests.test_scene_fidelity_audit.SceneFidelityAuditTests.test_report_preserves_structured_local_support_and_enclosure_client_world_fields -v`.
- Result: the audit is more useful for player-experience regressions, and the next missing local runtime signals are terrain roughness/step metrics plus explicit interior-presence quantification.

### 2026-03-28: Local Terrain Roughness Metrics

- Added a new shared `ReplicatedStorage.Shared.WorldProbeTerrain` reducer and a focused `WorldProbeTerrain.spec.lua` contract for flat, stepped, and sparse terrain samples.
- Extended `WorldProbe.client.lua` so the client probe now samples a small terrain cross around the player and publishes nested `localTerrain` metrics: `status`, `samplePattern`, `sampleRadiusStuds`, `sampleCount`, `missingSampleCount`, `centerTerrainY`, `minTerrainY`, `maxTerrainY`, `heightRangeStuds`, `maxStepStuds`, and `meanAbsStepStuds`.
- Extended `scene_fidelity_audit.py` and `scene_parity_audit.py` so those nested `localTerrain` fields survive into structured reports and parity normalization instead of remaining invisible runtime-only data.
- Took the new reducer red on `tertiary`, where a sparse-sample case exposed an incorrect holey-array sample-count bug; fixed the reducer/probe to use dense sample slots and re-ran `WorldProbeTerrain.spec.lua` green on `tertiary`.
- Verified locally with `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_scene_fidelity_audit scripts.tests.test_scene_parity_audit scripts.tests.test_play_render_truth -v` and remotely in the remote clone with `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_scene_fidelity_audit scripts.tests.test_scene_parity_audit -v`.
- Re-ran the play-only proof on `tertiary`; it still reached `gameplay_ready`, but the raw Studio log truncates long `ARNIS_CLIENT_WORLD(_COMPACT)` lines before the new nested `localTerrain` block is visible in remote log-derived artifacts.
- Result: local terrain roughness is now part of the canonical client-world contract and audit schema, but the next observability fix is reliable remote artifact capture for that local block rather than more schema churn.
