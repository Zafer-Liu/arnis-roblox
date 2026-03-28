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
- Terrain explicit-material truth is now stronger for sub-4-stud source cells; `TerrainQuantizedMaterialTruth.spec.lua` passed on `tertiary` after replacing last-writer-wins overlap writes with voxel-center source-cell ownership.
- Ground-support observability is now stronger at spawn; `WorldProbeSupport.spec.lua` passed on `tertiary` after excluding the hidden runtime spawn, skipping decorative road detail, and trusting explicit road surface roles.
- On `tertiary`, gameplay-ready play telemetry now resolves support as terrain at the sampled spawn (`supportSurfaceRole=terrain`, `groundMaterial=Enum.Material.Grass`, `supportY=5.1`, `terrainY=5.1`) instead of the earlier `unknown` read.
- On `tertiary`, gameplay-ready play telemetry now also shows nearby shell evidence at spawn (`nearbyWallParts=4`, `nearestWallDistanceStuds=2.2`, `overheadRoofParts=2`, `overheadRoofMinClearanceStuds=12.4`), so the old wall/support probe blind spot is no longer the active signal.
- The biggest current measured hotspot is still preview/edit builder cost on `tertiary`: one slow building-heavy chunk around `166ms`, and full preview sync around `17.3s` for the current `80`-chunk bounded scene.

## Verification Snapshot

### Local Static

- `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - passed on 2026-03-28
  - verifies the new terrain quantization contract, shared support helper wiring, and updated player-local observability contract
- `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_scene_parity_audit scripts.tests.test_scene_fidelity_audit -v`
  - passed on 2026-03-28
  - verifies shared resolved config, roof-closure support truth, and richer client-world observability contracts
- `git diff --check`
  - passed on 2026-03-28

### Remote `tertiary`

- `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_scene_parity_audit scripts.tests.test_scene_fidelity_audit -v`
  - passed on 2026-03-28 in `~/.codex-remote-studio/arnis-roblox`
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
- `bash scripts/run_studio_harness.sh --takeover --hard-restart --skip-edit-tests --play-wait 30 --pattern-wait 120`
  - rerun on 2026-03-28 from the remote clone
  - authoritative play proof remained green
  - gameplay-ready client telemetry improved shell-mesh evidence from `nearbyMergedBuildingMeshParts=0` to `5`
  - gameplay-ready client telemetry now resolves support as terrain at the sampled spawn (`supportSurfaceRole=terrain`, `groundMaterial=Enum.Material.Grass`, `supportY=5.1`, `terrainY=5.1`)
  - gameplay-ready client telemetry now shows nearby shell walls and roof cover (`nearbyWallParts=4`, `nearestWallDistanceStuds=2.2`, `overheadRoofParts=2`, `overheadRoofMinClearanceStuds=12.4`)

## Residual Gaps

- Terrain fidelity still needs dedicated work; explicit material collapse is fixed, but the current observed issue set still includes inherent 4-stud write-grid boxiness and broader detail questions that this tranche has not yet resolved.
- Interior work still needs a dedicated follow-up pass; the shell-fill overlap is fixed, but top-floor ceiling versus roof/closure-deck overlap is still open.
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
