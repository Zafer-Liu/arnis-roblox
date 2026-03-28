# Play Fidelity And Observability Implementation Plan

Status: Active

**Goal:** Continue from the completed March 28 baseline by tightening roof/interior/wall truth, improving player-local observability, measuring remaining fidelity gaps on `tertiary`, and keeping the docs stack current.

## Execution Status

- 2026-03-28: Docs stack rollover started. The completed canonical-baseline tranche is now marked completed, and this plan/status/spec stack is the active truth surface.
- 2026-03-28: Shared resolved config is already aligned across preview and runtime startup import.
- 2026-03-28: Player-local telemetry now includes support, enclosure, and roof-cover fields.
- 2026-03-28: Roof-closure decks are now marked internal support and verified on `tertiary` with `GabledRoofClosureTruth.spec.lua`.
- 2026-03-28: Shell-mesh play telemetry improved on `tertiary` (`nearbyMergedBuildingMeshParts` moved from `0` to `5` at gameplay-ready spawn), but nearby wall counts are still `0` at the sampled spawn.
- 2026-03-28: The remaining wall-gap signal was reproduced as a `WorldProbe` centroid artifact, not a shell-generation failure; `WorldProbeGeometry.spec.lua` now passes on `tertiary` after switching shell-wall proximity to surface-distance checks.
- 2026-03-28: The first room-overlap fix landed; `RoomInteriorShellFillTruth.spec.lua` now passes on `tertiary` after skipping shell terrain fill for room-authored `shellMesh` buildings when interiors are enabled.
- 2026-03-28: Top-floor room ceilings now clamp to `ArnisImportBuildingTopY`; the strengthened `RoomInteriorShellFillTruth.spec.lua` passes on `tertiary` after proving ceilings stay below the imported shell top and roof bottom.
- 2026-03-28: Terrain explicit-material truth now quantizes by voxel-center source-cell ownership instead of last-writer-wins overlap writes; `TerrainQuantizedMaterialTruth.spec.lua` passes on `tertiary`.
- 2026-03-28: Ground-support observability now skips the hidden runtime spawn, skips decorative road detail, and trusts explicit road surface roles; `WorldProbeSupport.spec.lua` passes on `tertiary`.
- 2026-03-28: Gameplay-ready play telemetry on `tertiary` now resolves to `supportSurfaceRole="terrain"` with `groundMaterial=Enum.Material.Grass`, `nearbyWallParts=4`, and nonzero roof cover, so the old `unknown` support signal is no longer the active truth.
- 2026-03-28: `manifest_quality_audit.py` now quantifies area-weighted terrain material dominance and terrain cell-size/granularity instead of relying only on per-chunk unique-material monotony.
- 2026-03-28: `scene_fidelity_audit.py` now carries manifest terrain granularity/material context into edit/play audit artifacts, so terrain complaints can be joined to the same report surface used for render drift.
- 2026-03-28: Preview telemetry now preserves the last slow chunk as structured snapshot state, and `preview_telemetry_summary.py` now surfaces `last_sync_elapsed_ms` plus slow-chunk timing breakdown in a compact operator summary.

## Tasks

- [x] Roll the docs stack forward so the completed baseline tranche is no longer labeled active.
- [x] Mark shaped roof-closure decks as internal support rather than visible roof truth.
- [x] Improve shell-mesh player-local telemetry in `WorldProbe`.
- [x] Reproduce and classify the remaining wall-gap signal on `tertiary`: actual missing walls vs. spawn/radius/classification artifact.
- [ ] Measure and reduce the next high-signal fidelity gaps: remaining terrain geometry/detail limits and the remaining richer interior traversal/ceiling-roof edge cases after the top-floor clamp.
- [ ] Extend player-local observability with coverage-style wall/roof metrics, local terrain roughness/step metrics, and local interior presence metrics.
- [ ] Promote preview/edit slow-chunk hotspot data into structured telemetry and audit surfaces.
- [ ] Keep the rolling status file current after each meaningful remote run, then commit and push the tranche.
