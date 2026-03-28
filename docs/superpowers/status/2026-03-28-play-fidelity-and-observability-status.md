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
- The biggest current measured hotspot is still preview/edit builder cost on `tertiary`: one slow building-heavy chunk around `166ms`, and full preview sync around `17.3s` for the current `80`-chunk bounded scene.

## Verification Snapshot

### Local Static

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
- `bash scripts/run_studio_harness.sh --takeover --hard-restart --skip-edit-tests --play-wait 30 --pattern-wait 120`
  - rerun on 2026-03-28 from the remote clone
  - authoritative play proof remained green
  - gameplay-ready client telemetry improved shell-mesh evidence from `nearbyMergedBuildingMeshParts=0` to `5`
  - the remaining `nearbyWallParts=0` read is now understood as a centroid-measurement artifact rather than evidence of missing shell walls

## Residual Gaps

- `supportSurfaceRole` at the sampled gameplay-ready spawn is still `unknown`, even when `groundMaterial` is `Enum.Material.Concrete`.
- Terrain fidelity still needs dedicated work; the current observed issue set includes boxy/default-looking terrain and material/detail questions that this tranche has not yet resolved.
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
