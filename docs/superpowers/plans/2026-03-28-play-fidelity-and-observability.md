# Play Fidelity And Observability Implementation Plan

Status: Completed

**Historical Goal:** Continue from the completed March 28 baseline by tightening roof/interior/wall truth, improving player-local observability, measuring remaining fidelity gaps on `tertiary`, and keeping the docs stack current.

## Execution Status

- 2026-03-28: Docs stack rollover completed. This plan is now historical context for the finished tranche, not the repo's current execution source.
- 2026-03-28: Shared resolved config is already aligned across preview and runtime startup import.
- 2026-03-28: Player-local telemetry now includes support, enclosure, and roof-cover fields.
- 2026-03-28: Roof-closure decks are now marked internal support and verified on `tertiary` with `GabledRoofClosureTruth.spec.lua`.
- 2026-03-28: Shell-mesh play telemetry improved on `tertiary` (`nearbyMergedBuildingMeshParts` moved from `0` to `5` at gameplay-ready spawn); the earlier `nearbyWallParts=0` read was later traced to probe centroid bias rather than missing shell walls.
- 2026-03-28: The remaining wall-gap signal was reproduced as a `WorldProbe` centroid artifact, not a shell-generation failure; `WorldProbeGeometry.spec.lua` now passes on `tertiary` after switching shell-wall proximity to surface-distance checks.
- 2026-03-28: The first room-overlap fix landed; `RoomInteriorShellFillTruth.spec.lua` now passes on `tertiary` after skipping shell terrain fill for room-authored `shellMesh` buildings when interiors are enabled.
- 2026-03-28: Top-floor room ceilings now clamp to `ArnisImportBuildingTopY`; the strengthened `RoomInteriorShellFillTruth.spec.lua` passes on `tertiary` after proving ceilings stay below the imported shell top and roof bottom.
- 2026-03-28: Terrain explicit-material truth now quantizes by voxel-center source-cell ownership instead of last-writer-wins overlap writes; `TerrainQuantizedMaterialTruth.spec.lua` passes on `tertiary`.
- 2026-03-28: Ground-support observability now skips the hidden runtime spawn, skips decorative road detail, and trusts explicit road surface roles; `WorldProbeSupport.spec.lua` passes on `tertiary`.
- 2026-03-28: Gameplay-ready play telemetry on `tertiary` now resolves to `supportSurfaceRole="terrain"` with `groundMaterial=Enum.Material.Grass`, `nearbyWallParts=4`, and nonzero roof cover, so the old `unknown` support signal is no longer the active truth.
- 2026-03-28: `manifest_quality_audit.py` now quantifies area-weighted terrain material dominance and terrain cell-size/granularity, and focused local-zone reports now clip terrain accounting to included terrain cells instead of crediting whole intersecting chunks.
- 2026-03-28: `scene_fidelity_audit.py` now carries manifest terrain granularity/material context into edit/play audit artifacts, so terrain complaints can be joined to the same report surface used for render drift.
- 2026-03-28: Preview telemetry now preserves the current slow chunk as structured snapshot state, clears stale `lastSlowChunk` state on `sync_started`, and `preview_telemetry_summary.py` now surfaces `last_sync_elapsed_ms` plus slow-chunk timing breakdown in a compact operator summary.
- 2026-03-28: `scene_fidelity_audit.py` now emits explicit player-local support/enclosure/roof-cover findings from `clientWorld` and surfaces nested local metrics directly in the HTML report instead of leaving them buried only in raw JSON.
- 2026-03-28: Player-local telemetry now includes structured `localTerrain` roughness metrics (`status`, sample coverage, height range, max step, mean absolute step), and the scene fidelity/parity audits now preserve and normalize those fields.
- 2026-03-28: The new `localTerrain` reducer is verified in edit mode on `tertiary` with `WorldProbeTerrain.spec.lua`, but the current real play-log artifact still truncates long `ARNIS_CLIENT_WORLD(_COMPACT)` lines before `localTerrain` survives into remote log-derived reports.
- 2026-03-28: A dedicated `ARNIS_CLIENT_LOCAL_EXPERIENCE` marker now carries the local player block separately, the Python audit render step now re-enriches prebuilt JSON from the raw Studio log, and `tertiary` raw-log proof now shows that block surviving through `gameplay_ready`.
- 2026-03-28: The remaining remote observability gap is no longer marker loss; it is staged-clone operations. The synced remote stage intentionally lacks ignored `rust/out` manifest-summary outputs, so offline scene-fidelity artifact regeneration there still needs a seeded summary or a bounded regenerate step, and the play harness still needs cleaner post-proof exit behavior.

## Historical Outcomes

- Rolled the docs stack forward so the completed baseline tranche was no longer labeled active.
- Marked shaped roof-closure decks as internal support rather than visible roof truth.
- Improved shell-mesh player-local telemetry in `WorldProbe`.
- Reproduced and classified the remaining wall-gap signal on `tertiary` as a probe artifact rather than missing shell walls.

## Follow-On Work

- Terrain geometry/detail limits, richer interior traversal, and later observability/audit expansion moved into subsequent tranches after this plan completed.
