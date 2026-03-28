# Play Fidelity And Observability Implementation Plan

Status: Active

**Goal:** Continue from the completed March 28 baseline by tightening roof/interior/wall truth, improving player-local observability, measuring remaining fidelity gaps on `tertiary`, and keeping the docs stack current.

## Execution Status

- 2026-03-28: Docs stack rollover started. The completed canonical-baseline tranche is now marked completed, and this plan/status/spec stack is the active truth surface.
- 2026-03-28: Shared resolved config is already aligned across preview and runtime startup import.
- 2026-03-28: Player-local telemetry now includes support, enclosure, and roof-cover fields.
- 2026-03-28: Roof-closure decks are now marked internal support and verified on `tertiary` with `GabledRoofClosureTruth.spec.lua`.
- 2026-03-28: Shell-mesh play telemetry improved on `tertiary` (`nearbyMergedBuildingMeshParts` moved from `0` to `5` at gameplay-ready spawn), but nearby wall counts are still `0` at the sampled spawn.

## Tasks

- [x] Roll the docs stack forward so the completed baseline tranche is no longer labeled active.
- [x] Mark shaped roof-closure decks as internal support rather than visible roof truth.
- [x] Improve shell-mesh player-local telemetry in `WorldProbe`.
- [ ] Reproduce and classify the remaining wall-gap signal on `tertiary`: actual missing walls vs. spawn/radius/classification artifact.
- [ ] Measure and reduce the next high-signal fidelity gaps: terrain material/detail truth and room/floor/roof overlap.
- [ ] Keep the rolling status file current after each meaningful remote run, then commit and push the tranche.
