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
- 2026-03-28: Terrain remains the next measured builder-fidelity slice; current evidence points to 4-stud write-grid quantization and last-writer-wins material collapse rather than a play-only renderer split.

## Tasks

- [x] Roll the docs stack forward so the completed baseline tranche is no longer labeled active.
- [x] Mark shaped roof-closure decks as internal support rather than visible roof truth.
- [x] Improve shell-mesh player-local telemetry in `WorldProbe`.
- [x] Reproduce and classify the remaining wall-gap signal on `tertiary`: actual missing walls vs. spawn/radius/classification artifact.
- [ ] Measure and reduce the next high-signal fidelity gaps: terrain material/detail truth and the remaining room/floor/roof overlap work after the shell-fill fix.
- [ ] Keep the rolling status file current after each meaningful remote run, then commit and push the tranche.
