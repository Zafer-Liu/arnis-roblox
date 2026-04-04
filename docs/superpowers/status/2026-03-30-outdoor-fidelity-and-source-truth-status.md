# Outdoor Fidelity And Source-Truth Status

Date: 2026-03-30
Status: Active

## Purpose

This is the rolling status and handoff document for the active outdoor fidelity and source-truth tranche.

It is also the only active rolling status file in `docs/superpowers/`. All other superpowers status docs are archived context only.

The active design spec is:

- `docs/superpowers/specs/2026-03-30-outdoor-fidelity-and-source-truth-design.md`

The active implementation plan is:

- `docs/superpowers/plans/2026-03-30-outdoor-fidelity-and-source-truth.md`

The compact historical archive index is:

- `docs/superpowers/archive-index.md`

## Current Snapshot

- Inherited from the March 28 baseline and archived play-fidelity tranche: edit/play parity, canonical runtime ownership, and player-local observability are already proven.
- The compatibility purge is in place: the repo treats `0.4.0` as the supported manifest schema and does not keep older manifest compatibility active.
- The active tranche is now the March 30 outdoor fidelity and source-truth stack.
- The earlier March 28 play-fidelity tranche and other deleted tactical planning docs are summarized only in the archive index.
- Guardrail tests enforce one active repo-wide superpowers truth stack: one active spec, one active plan, and this active rolling status file.

## Verification Snapshot

### Local Static

- `rg -n "^Status: Active$" docs/superpowers/specs docs/superpowers/plans docs/superpowers/status`
  - passed on 2026-03-30
  - verified that the March 30 spec/plan/status stack was the only set of docs carrying active markers after the rollover
- `python3 -m unittest scripts.tests.test_run_studio_harness_remote scripts.tests.test_scene_fidelity_audit -v`
  - passed on 2026-04-01
  - verified the merged integration branch kept the remote harness contract and scene-audit carry-through green
- `python3 -m unittest scripts.tests.test_source_truth_pack scripts.tests.test_source_truth_pack_audit scripts.tests.test_manifest_quality_audit scripts.tests.test_play_render_truth -v`
  - passed on 2026-04-01
  - verified the bounded truth-pack and outdoor-fidelity local-safe lane on the integrated baseline
- `python3 scripts/check_scaffold.py`
  - passed on 2026-04-01
- `python3 scripts/verify_generated_austin_assets.py`
  - passed on 2026-04-01
- `cargo test --manifest-path rust/Cargo.toml --workspace`
  - passed on 2026-04-01
  - verified the merged baseline stayed green across the Rust workspace
- `git diff --check`
  - passed on 2026-03-30 and 2026-04-01
  - verified both the initial doc rollover and the later merged local-safe tranche stayed text-clean
- `python3 -m unittest scripts.tests.test_convergence_guardrails scripts.tests.test_run_studio_harness_remote -v`
  - passed on 2026-04-01
  - verified every superpowers spec/plan/status file now has a supported top-level status, the March 30 stack is the only active repo-wide truth surface, and the remote operator doc still matches the harness contract
- `python3 -m unittest scripts.tests.test_source_truth_pack.SourceTruthPackHelperTests scripts.tests.test_source_truth_pack_audit.SourceTruthPackAuditTests scripts.tests.test_austin_fidelity scripts.tests.test_play_render_truth scripts.tests.test_preview_telemetry_summary -v`
  - passed on 2026-04-01
  - verified the new shared Austin export default, compact semantic-lineage audit surface, and building hotspot subphase reporting without running Studio locally
- `python3 -m py_compile scripts/source_truth_pack.py scripts/source_truth_pack_audit.py scripts/preview_telemetry_summary.py scripts/tests/test_source_truth_pack.py scripts/tests/test_source_truth_pack_audit.py scripts/tests/test_austin_fidelity.py scripts/tests/test_play_render_truth.py scripts/tests/test_preview_telemetry_summary.py`
  - passed on 2026-04-01
- `cargo test --manifest-path rust/Cargo.toml -p arbx_pipeline overture_gap_fill_backfills_missing_osm_semantics_from_collapsed_overture -- --nocapture`
  - passed on 2026-04-01
  - verified retained OSM buildings now backfill missing Overture structure semantics without keeping the collapsed Overture duplicate
- `bash -n scripts/run_studio_harness.sh`
  - passed on 2026-04-01
- `bash -n scripts/run_studio_harness_remote.sh`
  - passed on 2026-04-01

### Remote `tertiary`

- `ssh tertiary 'cd ~/Projects/arnis-roblox-main && ARNIS_TELEMETRY_FAMILIES=terrain,roads,water,vegetation,structures,hotspots,player_local VSYNC_REPO_DIR=$HOME/.codex-remote-studio/vertigo-sync bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter WorldProbeTelemetryFlags.spec.lua --edit-wait 30 --pattern-wait 120'`
  - passed on 2026-04-01
  - proved `WorldProbeTelemetryFlags.spec.lua`
  - emitted `ARNIS_MCP_READY` and `ARNIS_MCP_EDIT_ACTION` with `total=1 passed=1 failed=0`
- `ssh tertiary 'cd ~/Projects/arnis-roblox-main && ARNIS_TELEMETRY_FAMILIES=terrain,roads,water,vegetation,structures,hotspots,player_local VSYNC_REPO_DIR=$HOME/.codex-remote-studio/vertigo-sync bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter WorldProbeTerrain.spec.lua --edit-wait 30 --pattern-wait 120'`
  - passed on 2026-04-01
- `ssh tertiary 'cd ~/Projects/arnis-roblox-main && ARNIS_TELEMETRY_FAMILIES=terrain,roads,water,vegetation,structures,hotspots,player_local VSYNC_REPO_DIR=$HOME/.codex-remote-studio/vertigo-sync bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter TerrainOutdoorFidelity.spec.lua --edit-wait 30 --pattern-wait 120'`
  - passed on 2026-04-01 after correcting the spec to model two 2-stud source cells inside one 4-stud write voxel
- `scp tertiary:/tmp/arnis-preview-plugin-state.json /tmp/arnis-preview-plugin-state.json`
  - passed on 2026-04-01
- `scp tertiary:/tmp/arnis-preview-telemetry-summary.txt /tmp/arnis-preview-telemetry-summary.txt`
  - passed on 2026-04-01
  - synced the preview hotspot artifacts used for the current Task 5/Task 6 target selection
- `ssh tertiary 'cd ~/Projects/arnis-roblox-main && ARNIS_TELEMETRY_FAMILIES=terrain,roads,water,vegetation,structures,hotspots,player_local ARNIS_SCENE_AUDIT_DIR=/tmp/arnis-outdoor-audit-play VSYNC_REPO_DIR=$HOME/.codex-remote-studio/vertigo-sync bash scripts/run_studio_harness.sh --takeover --hard-restart --skip-edit-tests --play-wait 30 --pattern-wait 120'`
  - passed on 2026-04-01
  - reached `gameplay_ready`
  - emitted `ARNIS_MCP_PLAY`, `ARNIS_CLIENT_WORLD_COMPACT`, and `ARNIS_CLIENT_LOCAL_EXPERIENCE` with live `localTerrain` metrics
- Current remote teardown caveat:
  - the successful proof lanes can still hang after `quit_studio requesting graceful quit`
  - `tertiary` was manually force-cleaned afterward so no Studio, harness, MCP, Vertigo Sync, or lock residue remained
- Current remote operator note:
  - the staged clone under `~/.codex-remote-studio/arnis-roblox` is not the active proof surface for `arnis-roblox` right now because its `scripts/` completeness is still unproven
  - the active direct proof lane is the git-backed clone at `~/Projects/arnis-roblox-main`

## Residual Gaps

- Outdoor fidelity still needs dedicated work on terrain detail, shell nuance, and player-visible exterior realism.
- Outdoor hotspots still need tighter measurement so preview/edit cost can be traced at chunk scope instead of only at the whole-run level.
- Source-truth preservation still needs explicit proof across upstream source union, canonical collapse, and downstream audits.
- Harness work is now in wrap-up mode: the direct git-backed `tertiary` proof clone is the only proof lane, and remaining harness work should be limited to teardown/cleanup hygiene instead of new harness feature surfaces.

## Status Notes

### 2026-04-03 13:34 CDT: Local-Safe Swarm Tightened Footprint Admission, Sparse Cliffs, And Street-Level Building Cues

- Landed another product tranche on `main` without local Studio:
  - `StreamingService.lua` now indexes chunk candidates across every spatial-index cell touched by a chunk footprint, then dedupes candidate collection so overlapping footprints are admitted early instead of waiting for the player to move closer to the chunk center.
  - `TerrainBuilder.lua` now applies a bounded `sparseCliffOccupancyScale` so sparse steep cliff columns start from a lower occupancy baseline instead of reading like solid vertical planes.
  - `BuildingBuilder.lua` now emits a deterministic `MergedShellDoorCue` on the primary readable facade edge so merged `shellMesh` buildings keep one stronger street-facing entrance cue near player eye level.
- Added/updated local-safe coverage in:
  - `StreamingFootprintResidency.spec.lua`
  - `TerrainOutdoorFidelity.spec.lua`
  - `BuildingShellMeshReadableCues.spec.lua`
  - `scripts/tests/test_streaming_residency_footprint_contract.py`
  - `scripts/tests/test_terrain_sparse_cliff_occupancy_shaping_truth.py`
  - `scripts/tests/test_building_shell_mesh_readability_contract.py`
- Local-safe verification passed on 2026-04-03:
  - `python3 -m unittest scripts.tests.test_streaming_residency_footprint_contract scripts.tests.test_streaming_dual_focus_priority_contract scripts.tests.test_streaming_lod_footprint_contract scripts.tests.test_streaming_lod_live_root_focus_contract scripts.tests.test_building_shell_mesh_readability_contract scripts.tests.test_building_shell_mesh_wall_presence_contract scripts.tests.test_terrain_sparse_peak_surface_damping_truth scripts.tests.test_terrain_sparse_cliff_occupancy_shaping_truth scripts.tests.test_terrain_chunk_edge_truth scripts.tests.test_terrain_steep_mixed_fill_depth_truth scripts.tests.test_terrain_column_occupancy_shaping_truth scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/Tests/BuildingShellMeshReadableCues.spec.lua roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua roblox/src/ServerScriptService/Tests/StreamingFootprintResidency.spec.lua`
  - `git diff --check`
- Next `tertiary`-only proof question is whether the overlapping-footprint admission fix reduces “invisible until you walk into it” behavior, while the terrain/building cue changes should be checked against sparse peak planes and street-level facade legibility in live play.

### 2026-04-03 13:26 CDT: Local-Safe Play-Fidelity Swarm Tightened Terrain, Facade Readability, And Footprint Residency

- Landed a three-slice local-safe product tranche on `main` without running Studio locally:
  - `TerrainBuilder.lua` now applies extra damping to very sparse steep peaks so isolated highs stay closer to the surrounding surface instead of turning into broad false planes.
  - `BuildingBuilder.lua` now emits bounded `MergedShellStreetFacadeCue` geometry for merged `shellMesh` buildings so street-level facades stay more legible in play without abandoning the mesh path.
  - `ChunkPriority.lua` and `StreamingService.lua` now use chunk footprint distance more consistently for residency/ring accounting instead of falling back to chunk-center behavior when footprint metadata exists.
- Added/updated local contract coverage in:
  - `TerrainOutdoorFidelity.spec.lua`
  - `BuildingShellMeshReadableCues.spec.lua`
  - `scripts/tests/test_terrain_sparse_peak_surface_damping_truth.py`
  - `scripts/tests/test_building_shell_mesh_readability_contract.py`
  - `scripts/tests/test_streaming_residency_footprint_contract.py`
  - `scripts/tests/test_austin_runtime_contract.py`
- Local-safe verification passed on 2026-04-03:
  - `python3 -m unittest scripts.tests.test_streaming_residency_footprint_contract scripts.tests.test_streaming_dual_focus_priority_contract scripts.tests.test_streaming_lod_footprint_contract scripts.tests.test_streaming_lod_live_root_focus_contract scripts.tests.test_building_shell_mesh_readability_contract scripts.tests.test_building_shell_mesh_wall_presence_contract scripts.tests.test_terrain_sparse_peak_surface_damping_truth scripts.tests.test_terrain_chunk_edge_truth scripts.tests.test_terrain_steep_mixed_fill_depth_truth scripts.tests.test_terrain_column_occupancy_shaping_truth scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/ImportService/ChunkPriority.lua roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/Tests/BuildingShellMeshReadableCues.spec.lua roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua`
  - `git diff --check`
- Next step remains `tertiary`-only proof to measure whether this tranche materially reduces street-level facade emptiness, sparse peak false planes, and chunks that feel missing until the player crosses their footprint.

### 2026-04-03: Play-Probe False Positives No Longer Generate Bogus Play Scene Audits

- Continued the `tertiary`-only play-focused Austin proof lane after the earlier seeded-manifest-summary fix.
- Root-cause update:
  - the current remaining play-scene blocker is not staged manifest input anymore
  - the staged clone still picked up the precomputed manifest summary successfully:
    - `using precomputed manifest scene index: /Users/adpena/.codex-remote-studio/arnis-roblox/rust/out/austin-manifest.scene-index.json`
  - the live failure remains the current MCP server contract: `run_code` still resolves against edit context instead of the live play DataModel during this proof lane
- Hardened the harness so this failure is now honest and non-misleading:
  - `run_play_probe_via_mcp()` now emits `ARNIS_SCENE_PLAY` only when the sampled world root actually exists
  - the harness now logs `ARNIS_MCP_PLAY_SCENE_VALIDATED` only after the returned MCP payload passes live-play validation
  - offline play scene-audit generation now requires both `ARNIS_SCENE_PLAY` and `ARNIS_MCP_PLAY_SCENE_VALIDATED`; raw scene lines alone are no longer enough to produce `/tmp/arnis-scene-fidelity-play.*`
- Remote `tertiary` proof on 2026-04-03 from the staged wrapper lane confirmed the new behavior:
  - Austin still reached authoritative client `gameplay_ready`
  - the MCP play probe still failed with:
    - `RuntimeError('run_code resolved against edit context instead of the live play session')`
  - the patched harness no longer emitted `ARNIS_SCENE_PLAY` in that false-positive run, and the play scene-audit write path stayed dark instead of generating a misleading play report
- Residual remote issues are unchanged:
  - screenshot capture still fails with `could not create image from display`
  - wrapper cleanup still tends to hang after `quit_studio requesting graceful quit`, so the successful proof signal was captured before interrupting the late quit tail
- Local-safe verification passed on 2026-04-03:
  - `python3 -m unittest scripts.tests.test_run_studio_harness scripts.tests.test_run_studio_harness_remote scripts.tests.test_studio_mcp_proxy_lib -v`
  - `bash -n scripts/run_studio_harness.sh scripts/run_studio_harness_remote.sh`
  - `git diff --check`
- Next follow-up is no longer "seed the manifest summary"; it is to establish a real live-play scene-audit lane on `tertiary` that does not depend on the current edit-context `run_code` limitation.

### 2026-04-03: Session-Status Probes Now Use A Lightweight UI Snapshot Fallback

- Investigated the current `tertiary` play-focused Austin proof lane after the `e9bc6977` fresh-template handoff fix.
- Root-cause update:
  - the current live blocker is no longer the older Austin bootstrap question alone
  - remote evidence from the active `/Volumes/APDataStore/arnis-roblox-proof` run on `e9bc6977` showed Studio could sit with only `State: Qt::ApplicationActive` plus repeated `http://localhost:44755/request` `HttpError: ConnectFail` noise while short bounded `studio_ui_control.py get-session-status` / `get-state` probes timed out
  - this narrowed the failure boundary to the UI-control/session-status lane itself: the full accessibility snapshot was too heavy/unreliable to remain the only source for harness session classification on this remote surface
- Implemented a minimal bounded fix in `scripts/studio_ui_control.py`:
  - added `capture_fast_session_snapshot()` with a lighter front-window/menu/status probe instead of the full all-windows/all-buttons dump
  - added `capture_session_snapshot()` so `get-session-status` and `get-session-status-value` can fall back to the lighter probe when the full snapshot times out
  - kept the heavier `get-state` / `dump-ui` path intact for richer debugging surfaces
- Added unit coverage in `scripts/tests/test_studio_ui_control.py` to prove:
  - session snapshot falls back to the fast probe after a full-snapshot timeout
  - `get_session_status_value("ready_for_harness")` still succeeds from the fast probe in the timeout scenario
- Local-safe verification passed on 2026-04-03:
  - `python3 -m unittest scripts.tests.test_studio_ui_control.StudioUiControlTests.test_capture_session_snapshot_falls_back_to_fast_probe_after_full_timeout scripts.tests.test_studio_ui_control.StudioUiControlTests.test_get_session_status_value_uses_fast_probe_when_full_snapshot_times_out -v`
  - `python3 -m unittest scripts.tests.test_run_studio_harness scripts.tests.test_studio_ui_control -v`
  - `python3 -m py_compile scripts/studio_ui_control.py scripts/tests/test_studio_ui_control.py`
  - `git diff --check`
- Next proof step is a fresh `tertiary` rerun of the play-focused Austin screenshot lane from this patched main checkout to confirm the harness now escapes the session-status stall and either reaches Austin/play or exposes the next blocker explicitly.

### 2026-04-02: Runtime Streaming Contract Now Publishes Scheduler State And Ring Budgets

- Strengthened the shared runtime streaming contract in `StreamingService.lua` so the scheduler publishes its own plan, not just its outcome.
- Added runtime telemetry for:
  - `ArnisStreamingSchedulerState`
  - per-ring configured budgets:
    - `ArnisStreamingRingNearBudgetBytes`
    - `ArnisStreamingRingMidBudgetBytes`
    - `ArnisStreamingRingFarBudgetBytes`
  - per-ring configured chunk caps:
    - `ArnisStreamingRingNearMaxChunkCount`
    - `ArnisStreamingRingMidMaxChunkCount`
    - `ArnisStreamingRingFarMaxChunkCount`
  - per-ring desired residency plan:
    - `ArnisStreamingRingNearDesiredChunkCount`
    - `ArnisStreamingRingMidDesiredChunkCount`
    - `ArnisStreamingRingFarDesiredChunkCount`
    - `ArnisStreamingRingNearDesiredEstimatedCost`
    - `ArnisStreamingRingMidDesiredEstimatedCost`
    - `ArnisStreamingRingFarDesiredEstimatedCost`
- The scheduler now reports `planning`, `guardrail_paused`, or `steady_state` explicitly instead of leaving operators to infer state from chunk counters alone.
- Updated the runtime contract test surface and the focused Luau streaming-priority proof so the movement-lookahead sample also proves that the authoritative ring budget/guardrail contract is exposed to runtime telemetry.
- Local-safe verification passed:
  - `python3 -m unittest scripts.tests.test_austin_runtime_contract -v`
  - `git diff --check`
- This is product-side progress toward planetary streaming because runtime residency policy is now directly inspectable in the live world contract without adding a second streaming path.

### 2026-04-02: SSD-Backed Austin Proof Finished And Austin Wrapper Now Supports Bounded Derivative Emission

- Completed the SSD-backed non-satellite Austin proof run on `tertiary` from `/Volumes/APDataStore/arnis-roblox-proof` after updating the proof clone to `origin/main`.
- Measured current bounded Austin export cost on `tertiary`:
  - Rust compile step finished in about `33.39s`
  - SQLite manifest/truth-pack compile/store finished in about `20.67s`
  - full end-to-end `bash scripts/export_austin_to_lua.sh` finished in `563.71s`
  - peak resident set was about `2.32 GB`
- The full run completed cleanly with:
  - `7400` runtime shard modules
  - `906` preview shards
  - `906` canonical bounded shards
  - `143` runtime harness shards
  - generated-asset verification passed
- The measured bottleneck is no longer the Rust compile itself; it is the eager downstream derivative fanout after the canonical SQLite compile.
- Added bounded derivative-emission controls to `scripts/export_austin_to_lua.sh`:
  - `--emit runtime`
  - `--emit runtime,preview`
  - default remains `all`
- This keeps the canonical compile path and scene truth unchanged while letting proof/deploy loops skip unneeded derivative refreshes, which is a direct step toward chunked, demand-driven planetary workflows instead of always materializing every downstream family.
- Extended the same no-regression shard-bounding work into canonical runtime emission:
  - extracted the preview/harness fragment contract into `scripts/chunk_fragmentation.py`
  - `scripts/json_manifest_to_sharded_lua.py` now fragments canonical runtime chunks under `--max-bytes` using the same terrain/list-field splitting semantics preview and harness already rely on
  - `scripts/export_austin_to_lua.sh` now passes `--max-bytes "$AUSTIN_RUNTIME_SHARD_MAX_BYTES"` for runtime shard generation, defaulting to the same bounded Lua-module budget used by the preview path
- This is the next real streaming/DX primitive after SQLite-first compile: canonical runtime output no longer has to stay at whole-chunk shard granularity when a bounded byte-cap is requested.
- Local-safe verification for the wrapper slice passed:
  - `python3 -m unittest scripts.tests.test_austin_fidelity -v`
  - `bash -n scripts/export_austin_to_lua.sh`
  - `git diff --check`

### 2026-04-01: Play-Mode Building Wall Gap Is Now Measured Explicitly

- Added visible shell-wall evidence to the shared scene-audit path:
  - `SceneAudit.lua` now distinguishes direct shell existence from visible shell-wall evidence.
  - `scene_fidelity_audit.py` now emits a high-severity `building_visible_wall_gap` finding when play/edit scene summaries report building models without visible shell walls.
  - `scene_parity_audit.py` now compares `buildingModelsWithoutVisibleShellWalls` directly instead of letting that regression hide behind aggregate shell counts.
- Local-safe verification passed:
  - `python3 -m unittest scripts.tests.test_scene_fidelity_audit scripts.tests.test_scene_parity_audit -v`
- Remote `tertiary` verification passed:
  - `python3 -m unittest scripts.tests.test_scene_fidelity_audit scripts.tests.test_scene_parity_audit -v`
  - `SceneAudit.spec.lua` passed through the direct git-backed `~/Projects/arnis-roblox-main` proof lane.
- A fresh serialized `tertiary` play run reached `PlaySoloSuccess`, `ARNIS_MCP_READY`, `ARNIS_SCENE_PLAY`, and `ARNIS_MCP_PLAY` with the usual bootstrap trace through `world_ready,streaming_ready,minimap_ready,gameplay_ready`.
- Rebuilding the play fidelity report offline from the raw `tertiary` Studio log against `rust/out/austin-manifest.json` now shows the user-visible building complaint is real and bounded:
  - `buildingModelCount=92`
  - `buildingModelsWithDirectShell=92`
  - `buildingModelsWithVisibleShellWalls=87`
  - `buildingModelsWithoutVisibleShellWalls=5`
  - `buildingVisibleWallGapCount=5`
  - fidelity findings now include `building_visible_wall_gap`
- The follow-up attribution changed the diagnosis materially:
  - the fresh `ARNIS_SCENE_PLAY` payload exposed `buildingVisibleWallGapDetails` for the five flagged structures
  - all five source ids were `usage="roof"` structures rather than wall-bearing building shells
  - the wall-gap severity was therefore over-reporting roof-only canopies as missing-wall building regressions
  - `SceneAudit.lua` now excludes `usage="roof"` structures from visible-wall gap counts while still tracking their roof evidence normally
  - the next shared follow-up is no longer "restore five missing walls"; it is proving the corrected wall-gap count stays clean on `tertiary` and then tightening per-building facade visibility for real wall-bearing shells if needed
- That corrected proof is now in hand from the direct git-backed `tertiary` lane:
  - `SceneAudit.spec.lua` passed after aligning its roof-only canopy expectations with the modeled scene
  - `AustinSpawn.spec.lua` passed with the tightened runtime look-target logic
  - a fresh serialized play run still reached `PlaySoloSuccess`, `gameplay_ready`, `ARNIS_SCENE_PLAY`, `ARNIS_MCP_PLAY`, and `ARNIS_MCP_PLAY_LATE`
  - rebuilding the play fidelity report locally from the synced `/tmp/playproof-harness.log` against local `rust/out/austin-manifest.json` and `rust/out/austin.truth-pack.sqlite` now reports:
    - `buildingVisibleWallGapCount=0`
    - `buildingModelsWithoutVisibleShellWalls=0`
    - `buildingModelsWithVisibleShellWalls=87`
  - this means the current "buildings look terrible in play" complaint is no longer explained by missing wall-bearing shell geometry in the measured play scene
  - the remaining likely causes are now player-facing quality issues around facade/detail richness, camera framing, or other visual simplification in the shared building path
- Runtime framing around spawn was also tightened in the shared path:
  - `AustinSpawn.getPreferredLookTarget(...)` now treats the trivial canonical Austin `{0,0,1}` look-direction hint as a fallback, not a hard override
  - when the heuristic focus point is meaningful, runtime reuses it
  - otherwise Austin runtime can now bias toward the nearest non-roof building centroid instead of pointing through a roof-only canopy or the trivial forward vector
- Remote proof caveat remains narrower and explicit:
  - the same successful play run still ended with a post-proof `json.decoder.JSONDecodeError` while post-processing a truncated long log line
  - that failure happened after `gameplay_ready`, `ARNIS_SCENE_PLAY`, `ARNIS_MCP_PLAY`, and `ARNIS_CLIENT_LOCAL_EXPERIENCE` were already emitted, so it is a harness log-compaction issue rather than a world-truth failure
- Screenshot/capture status remains blocked but better understood:
  - direct SSH `screencapture` still fails with `could not create image from display`
  - GUI-session `.command` execution on `tertiary` now does create screenshots when launched through the logged-in desktop session
  - blind-timed captures are still noisy: early images caught only blue pre-world frames, and one later image captured Terminal instead of Studio because the one-shot script left Terminal frontmost
  - one frontmost-Studio capture succeeded late enough to prove the lane works, but it landed during teardown with a save-changes modal over a blurred scene
  - the screenshot lane is therefore viable, but it still needs a gameplay-ready trigger rather than fixed sleeps if it is going to become a trustworthy proof artifact

### 2026-04-01: Single Active Truth Stack Is Now Repo-Enforced

- Added guardrail coverage in `scripts/tests/test_convergence_guardrails.py` to enforce:
  - exactly one active spec in `docs/superpowers/specs/`
  - exactly one active plan in `docs/superpowers/plans/`
  - exactly one active rolling status file in `docs/superpowers/status/`
  - top-level `Status:` markers on every superpowers doc, restricted to `Active`, `Historical`, or `Completed`
  - active-status links to the active plan and active spec
- Normalized the older superpowers plan/spec backlog so every file now declares `Status: Historical` or `Status: Completed` instead of leaving status implicit or using `Draft`.
- Updated `AGENTS.md`, `CLAUDE.md`, and `docs/remote-studio-development.md` so the single-active-stack rule is written policy instead of only a test.
- This March 30 outdoor/source-truth stack remains the sole active superpowers truth surface for the repo.

### 2026-04-01: Historical Plan Backlog Consolidated Into A Compact Archive

- Added `docs/superpowers/archive-index.md` as the only historical navigation surface for deleted superseded tranches.
- Retained only:
  - the March 30 active spec/plan/status stack
  - `docs/superpowers/status/2026-03-28-canonical-baseline-status.md` as the completed baseline handoff
- Deleted the older tactical plan/spec/status backlog after folding their replacement pointers into the archive index and retained docs.
- The repo no longer keeps a long tail of superseded plan/spec files in-tree just to mark them historical.

### 2026-04-01: Plan Reconciliation And Audit Observability Tranche Landed Locally

- Reconciled the active March 30 plan so it no longer understates the already-landed Task 2, Task 3a, Task 3b, Task 4 local slice, and the first bounded Task 6 local slice.
- `scripts/preview_telemetry_summary.py` now exposes a bounded structured summary helper for preview/plugin telemetry instead of only emitting a compact text line.
- `scripts/scene_fidelity_audit.py` now accepts an optional preview plugin-state seam and carries a compact `previewTelemetry` block into the JSON/HTML report without dumping raw plugin-state payloads or local file paths.
- `scripts/source_truth_pack_audit.py` now exposes compact grouped breakdowns for dropped semantics and collapse kinds by outdoor family, which carry through into manifest and scene audits and participate in parity comparison when present.
- Local-safe verification for this tranche passed:
  - `python3 -m unittest scripts.tests.test_preview_telemetry_summary.PreviewTelemetrySummaryTests.test_build_plugin_state_summary_returns_compact_structured_blocks scripts.tests.test_scene_fidelity_audit.SceneFidelityAuditTests.test_preview_plugin_state_carries_compact_hotspot_summary_into_json_and_html -v`
  - `python3 -m unittest scripts.tests.test_source_truth_pack_audit.SourceTruthPackAuditTests.test_truth_pack_audit_reports_compact_outdoor_findings scripts.tests.test_manifest_quality_audit.ManifestQualityAuditTests.test_truth_pack_findings_carry_through_into_manifest_quality_report scripts.tests.test_scene_fidelity_audit.SceneFidelityAuditTests.test_truth_pack_carries_through_compact_summary_into_json_and_html scripts.tests.test_scene_parity_audit.SceneParityAuditTests.test_truth_pack_mismatch_is_not_subset_allowed -v`
- No Studio run was performed on this machine for this tranche.

### 2026-04-01: Manual Integration Branch Merged The Active Tranches By Hand

- Created `codex/manual-main-integration` in a clean worktree and merged `codex/breaking-compatibility-purge` plus `codex/outdoor-fidelity-source-truth` by hand instead of relying on the dirty root checkout.
- Pushed the hand-merged tip to `origin/codex/manual-main-integration` as a safety branch and then advanced `origin/main` from that same verified integration worktree.
- Conflict resolution was manual in the active docs, `rust/crates/arbx_cli/src/main.rs`, `scripts/scene_fidelity_audit.py`, and `scripts/tests/test_run_studio_harness_remote.py`; the merged result keeps the `0.4.0` hard break, the truth-pack CLI help surface, the bounded truth-pack scene-audit carry-through, and the worktree-safe remote harness path resolution.
- Fresh local-safe verification on the integration branch passed:
  - `python3 -m unittest scripts.tests.test_run_studio_harness_remote scripts.tests.test_scene_fidelity_audit -v`
  - `cargo test --manifest-path rust/Cargo.toml -p arbx_cli --quiet`
  - `python3 -m unittest scripts.tests.test_source_truth_pack scripts.tests.test_source_truth_pack_audit scripts.tests.test_manifest_quality_audit scripts.tests.test_play_render_truth -v`
  - `python3 scripts/check_scaffold.py`
  - `python3 scripts/verify_generated_austin_assets.py`
  - `cargo test --manifest-path rust/Cargo.toml --workspace`
  - `git diff --check`
- No Studio run was performed on this machine for the integration slice; `tertiary` remains the only Studio proof lane.

### 2026-04-01: Workstation Process-Hygiene Root Cause Was Orphaned Tool Helpers, Not Roblox Runtime

- The repeated local session instability was traced to orphaned Codex helper processes on this workstation, not to `arnis-roblox` runtime code and not to a new `run_studio_harness.sh` regression.
- The concrete failure signature in local session logs was repeated `Too many open files (os error 24)` while stale `chrome-devtools-mcp`, Node helper, and Chrome profile processes remained orphaned under `PPID 1`.
- The exact truncated plugin-cache message was not located in repo sources, so it should not be treated as a confirmed Roblox/plugin root-cause string.
- Current operator guardrails for this workstation are:
  - avoid browser/devtools helper usage in this repo session
  - keep process-backed verification serial instead of broad local swarms
  - perform explicit orphan-helper cleanup before and after heavy local verification tranches
  - keep all Studio proof on `tertiary`
- Treat this as workstation/tooling hygiene, not as evidence that the active outdoor/source-truth tranche introduced a runtime regression.

### 2026-04-01: Task 6 Slice Landed Explicit Hotspot Status And Terrain Richness

- The first bounded Task 6 slice now makes preview hotspot availability explicit in the compact summary output, distinguishing `present`, `absent`, `missing_snapshot`, and `sync_error` instead of silently dropping slow-chunk context.
- Slow terrain-chunk telemetry now carries truthful terrain-material richness from the terrain build plan through import-time chunk profiling into preview telemetry and the summary artifact.
- The selected truth targets were the currently blind local preview summary path and the monolithic terrain-material chunk case; `BuildingBuilder.lua` and `AustinPreviewTelemetry.lua` did not require changes for this slice.
- Local-safe verification passed on 2026-04-01:
  - `python3 -m unittest scripts.tests.test_preview_telemetry_summary scripts.tests.test_play_render_truth -v`
  - `git diff --check`
- No Studio or remote `tertiary` run was required for this slice.

### 2026-04-01: Tasks 4, 5, And 6 Are Now Consolidated On One Current Proof Narrative

- Task 4 proof is now anchored to the direct git-backed `tertiary` proof clone at `~/Projects/arnis-roblox-main`, not the staged clone:
  - `WorldProbeTelemetryFlags.spec.lua` passed on `tertiary`
  - preview telemetry artifacts were synced back from `tertiary` to `/tmp/arnis-preview-plugin-state.json` and `/tmp/arnis-preview-telemetry-summary.txt`
  - the synced summary currently reports `imported=80`, `last_sync_elapsed_ms=18689`, `slow_chunk=-1_0`, `slow_chunk_total_ms=154`, `slow_chunk_buildings_ms=153`, `slow_chunk_building_features=5`, `slow_chunk_dominant_cost_center=buildings`, and `slow_chunk_terrain_signal_status=not_authored`
- Task 5 remains on the `no schema change required` path after a fresh local truth-pack review:
  - the bounded audit over `rust/out/austin.truth-pack.sqlite` still shows the real pressure in truth-pack/audit surfaces, not missing canonical manifest fields
  - the strongest current findings remain structure overlap/collapse and dropped-structure semantics (`23672` overlap collapses, `23107` dropped semantics)
- Task 6 is now proven on `tertiary` for the current bounded terrain/hotspot slice:
  - `WorldProbeTerrain.spec.lua` passed on `tertiary`
  - `TerrainOutdoorFidelity.spec.lua` failed first for a bad test model, then passed after the spec was corrected to model two 2-stud source cells inside one 4-stud Roblox write voxel
  - the focused play proof on `tertiary` reached `gameplay_ready` and emitted both `ARNIS_MCP_PLAY` and `ARNIS_CLIENT_LOCAL_EXPERIENCE`
  - live play now carries the requested telemetry families plus the new local terrain block:
    - `localTerrain.status="ok"`
    - `localTerrain.materialKindCount=2`
    - `localTerrain.dominantMaterial="Grass"`
    - `localTerrain.nonGrassSampleCount=1`
    - `localTerrain.maxStepStuds=1.3`
    - `localSupport.surfaceRole="terrain"`
    - `localEnclosure.nearbyWallParts=4`
- The play proof also exposed and closed a real harness bug:
  - `ARNIS_TELEMETRY_FAMILIES` was only being propagated into the edit-MCP path, so play runs still emitted `playerLocalTelemetryEnabled=false`
  - `scripts/run_studio_harness.sh` now threads the requested telemetry families into the play-MCP Luau payload too
- The staged-clone proof path is still not trustworthy for `arnis-roblox/scripts/` completeness:
  - it previously regressed into `ModuleNotFoundError: studio_mcp_proxy_lib`
  - an experimental directory-skeleton sync patch was not kept because it was not a proven root-cause fix
  - until a real staged-sync root cause is fixed, direct proof on the persistent `tertiary` tree remains the operator truth
- `tertiary` was force-cleaned after each proof run; no lingering harness, Studio, MCP, or `vsync serve` processes remain

### 2026-04-01: Task 2 Narrowed To The First Honest Truth-Pack Slice

- A focused codebase audit confirmed that the current pipeline only retains truthful pre-canonical source-union data inside the Overpass/live adapter path plus the Overture building merge seam.
- The original Task 2 wording overclaimed generic compile-wide truth-pack coverage.
- The active plan now narrows the first slice to:
  - `OverpassAdapter`
  - `LiveOverpassAdapter`
  - Overpass-derived retained features
  - Overture building candidates
  - Overture-to-OSM collapse rows
- `FileSourceAdapter`, synthetic adapters, and post-canonical export-only seams remain out of scope for this first truth-pack slice until raw lineage is preserved there.

### 2026-04-01: Task 2 Review Fix Locked Collapse Rows Back To Overture-To-OSM

- Spec review found one real seam bug and one doc-drift bug in the first Task 2 landing.
- The merge seam now only records collapse rows against overlapping OSM buildings, so the truth-pack no longer emits `overture->overture` collapse rows.
- The active plan file map no longer lists `rust/crates/arbx_roblox_export/` under truth-pack ownership.
- Local-safe verification after the fix passed:
  - `cargo test --manifest-path rust/Cargo.toml -p arbx_pipeline overture_gap_fill_does_not_collapse_against_previously_retained_overture`
  - `python3 -m unittest scripts.tests.test_source_truth_pack -v`
  - `git diff --check`

### 2026-04-01: Task 3a Landed As Manifest-Quality-Only Truth-Pack Carry-Through

- Added `scripts/source_truth_pack_audit.py` as a bounded reader/auditor over the emitted truth-pack SQLite plus compact summary JSON.
- Integrated truth-pack-backed overlap-loss, dropped-semantic, retained-semantic, and per-family outdoor source coverage findings into `scripts/manifest_quality_audit.py` without forking a second manifest audit path.
- The manifest-quality JSON/HTML carry-through stays compact by default: family counts, capped samples, and an optional `--truth-pack` seam instead of raw table dumps.
- This slice intentionally did not touch `scene_fidelity_audit.py` or `scene_parity_audit.py`; those remain follow-on Task 3 work.
- Local-safe verification passed:
  - `python3 -m unittest scripts.tests.test_source_truth_pack_audit scripts.tests.test_manifest_quality_audit -v`

### 2026-04-01: Task 3a Review Tightened Boundedness And Outdoor-Only Scope

- Spec review found two real issues in the first Task 3a landing: the truth-pack auditor was materializing full SQLite tables, and its headline retained/dropped/overlap counts were not explicitly limited to outdoor families.
- The auditor now uses bounded aggregate queries plus capped sample queries, and the top-line findings ignore non-outdoor rows.
- A `rail` fixture row now guards against non-outdoor truth-pack data silently inflating the outdoor headline metrics.
- Local-safe verification after the fix passed:
  - `python3 -m unittest scripts.tests.test_source_truth_pack_audit scripts.tests.test_manifest_quality_audit -v`
  - `git diff --check`

### 2026-04-01: Task 3 Plan Drift Fixed After The Task 3a Review Gate

- A follow-up spec review correctly noted that the active Task 3 file list and step text still overclaimed `scene_fidelity_audit.py` and `scene_parity_audit.py` as part of the Task 3a manifest-quality-only slice.
- The active plan now splits Task 3 into:
  - Task 3a files and steps for truth-pack audit plus manifest-quality carry-through
  - later Task 3 files for scene-fidelity and scene-parity carry-through
- This keeps the plan aligned with the actual execution order before Task 3b starts.

### 2026-04-01: Task 3b Added Compact Truth-Pack Carry-Through To Scene Audits

- `scripts/scene_fidelity_audit.py` now accepts an optional `--truth-pack` argument and reuses the existing bounded `scripts/source_truth_pack_audit.py` reader instead of inventing a second truth-pack parsing path.
- The scene-fidelity JSON/HTML carry-through stays compact: `summary.truthPack` contains family counts, coverage, capped samples, and compact finding rows, without raw SQLite dumps or path-heavy payloads.
- `scripts/scene_parity_audit.py` now compares that compact truth-pack surface directly.
- Bounded-preview subset allowances remain limited to scene geometry metrics; truth-pack source-truth mismatches are treated as real parity mismatches.
- Hotspot carry-through was intentionally left out of this slice.
- Local-safe verification passed:
  - `python3 -m unittest scripts.tests.test_scene_fidelity_audit scripts.tests.test_scene_parity_audit -v`

### 2026-04-01: Task 3 Plan Drift Fixed Again After The Task 3b Review Gate

- A follow-up spec review correctly noted that the active Task 3a steps still listed scene-audit and hotspot coverage after Task 3a had already been narrowed.
- The active plan now separates:
  - Task 3a manifest-quality truth-pack audit work
  - Task 3b scene-fidelity and scene-parity truth-pack carry-through
- Hotspot telemetry remains explicitly deferred to a later slice.

### 2026-04-01: Task 4a1 Landed The Harness/Operator Contract Slice

- `ARNIS_TELEMETRY_FAMILIES` is now a first-class `scripts/run_studio_harness.sh` contract and is exported into the preview telemetry summary step.
- `scripts.preview_telemetry_summary` now surfaces a requested family subset compactly and stably as a `telemetry_families=` token without changing the default compact summary shape.
- This slice is intentionally limited to the harness/operator contract and summary presentation; Luau/runtime gating remains deferred to the later Task 4 implementation steps.

### 2026-04-01: Task 4a2 Landed The Runtime Flag Seam

- Added `roblox/src/ReplicatedStorage/Shared/WorldProbeTelemetryFlags.lua` as the shared parser/enable-check seam for the explicit outdoor family list.
- The family membership set is now derived from the ordered family list in one place, so the supported vocabulary stays explicit without a second source of truth.
- `scripts/run_studio_harness.sh` now mirrors `ARNIS_TELEMETRY_FAMILIES` into `Workspace:SetAttribute("ArnisTelemetryFamilies", ...)` so the Studio-side probe reads the same contract surface.
- `roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua` now keeps the bootstrap/core markers intact while gating the local terrain and player-local payload slices, plus structure details, behind the shared family flags, and annotates emitted markers with the canonical `telemetryFamilies` subset when one is requested.
- `player_local` now emits a deterministic local-experience tombstone payload when disabled so stale runtime state cannot linger as the latest authoritative marker in downstream readers.
- The preview-summary and Austin preview telemetry modules did not need follow-up edits for this slice; 4a1 remains the only harness/operator summary change.
- Local-safe verification passed:
  - `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_run_studio_harness scripts.tests.test_preview_telemetry_summary -v`

### 2026-04-01: Remote Harness Hygiene Tightened, But Graceful Quit Still Blocks Final Exit

- `scripts/run_studio_harness.sh` now:
  - reaps orphan harness shells before taking the remote lock
  - self-terminates when its SSH parent disappears
  - bounds all `studio_ui_control.py` reads and actions through the same shell timeout helper
  - kills timed-out UI helper child processes, not just the Python launcher
  - uses bounded TERM->KILL helpers for the background MCP sidecar, Vertigo Sync server, memory monitor, and log tail
  - emits explicit cleanup and `quit_studio` phase logs so teardown is no longer a black box
- Focused harness regression coverage was extended in `scripts/tests/test_run_studio_harness.py` and remains green locally and on the staged clone on `tertiary`.
- Remote `tertiary` proof signal is materially better:
  - the edit/preview body succeeds serially with fresh preview telemetry, `ARNIS_MCP_READY`, `ARNIS_MCP_EDIT_ACTION`, and a passing Vertigo Sync plugin smoke check
  - teardown now reaches `main harness flow complete`, `cleanup starting`, `cleanup policy`, `cleanup invoking quit_studio`, and `quit_studio starting`
  - the earlier raw `studio_mcp_direct_lib` fallback failure is gone, and the raw untimed `studio_ui_control.py` action path is gone
- The remaining blocker is narrower and explicit:
  - the remote run still does not emit `quit_studio finished`, `cleanup finished`, or `HARNESS_EXIT`
  - the last observed hang point is after `quit_studio requesting graceful quit`
  - `tertiary` was force-cleaned after the debugging runs; no harness, Studio, MCP, Vertigo Sync, AppleScript, or lock state was left resident
- Local-safe verification for this slice passed:
  - `python3 -m unittest scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_ui_status_probes_are_bounded_by_shell_timeout_helper scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_cleanup_uses_bounded_term_then_kill_for_background_helpers scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_quit_loop_dismisses_dialogs_without_polling_session_status_each_iteration -v`
  - `bash -n scripts/run_studio_harness.sh`
  - `git diff --check`

### 2026-04-01: Plugin-Smoke Live-Log Hang Removed, UI Helper Timeouts Moved Into `studio_ui_control.py`

- `scripts/studio_ui_control.py` now enforces its own bounded `osascript` timeout and returns exit code `124` on timeout instead of relying on the shell wrapper to kill the Python launcher from the outside.
- `scripts/run_studio_harness.sh` now kills timed-out UI helper child processes explicitly, not just the top-level Python helper, and no longer polls `studio_session_status_value` inside every `quit_studio` loop iteration before dismissing save/startup dialogs.
- `run_plugin_smoke_check()` no longer points `vsync plugin-smoke-log` at the live `LOG_SLICE_FILE`; it snapshots the slice to a temporary file first, which removed the observed post-smoke stall before `main harness flow complete`.
- `stop_parent_watchdog()` now uses the same bounded TERM->KILL helper as the other background-process shutdown paths.
- Focused regression coverage was extended locally and on the staged `tertiary` clone:
  - `scripts/tests/test_studio_ui_control.py` now locks the timeout exit-code behavior
  - `scripts/tests/test_run_studio_harness.py` now asserts the plugin-smoke snapshot path and the bounded watchdog shutdown path
- Remote `tertiary` evidence after these fixes:
  - the serial no-play preview lane still produces fresh preview telemetry and `ARNIS_MCP_EDIT_ACTION`
  - the live-log plugin-smoke hang is gone; the run now logs `main harness flow complete; exiting`
  - the cleanup phase now at least begins from the fixed branch, instead of stalling inside plugin smoke or a raw `osascript` child
  - direct SSH transport remained flaky in one subsequent attempt (`exec_command` returned `255`), but the staged clone was left clean afterward with no harness, Studio, MCP, Vertigo Sync, AppleScript, or lock residue
- The remaining uncertainty is narrower than before:
  - the strongest repo-side teardown stalls were removed
  - the next verification slice should focus on proving a stable `HARNESS_EXIT:0` / `cleanup finished` transcript on `tertiary` without a transport drop
- Local-safe verification passed:
  - `python3 -m unittest scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_plugin_smoke_uses_snapshot_of_live_log_slice scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_ui_status_probes_are_bounded_by_shell_timeout_helper scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_cleanup_uses_bounded_term_then_kill_for_background_helpers scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_quit_loop_dismisses_dialogs_without_polling_session_status_each_iteration scripts.tests.test_studio_ui_control -v`
  - `bash -n scripts/run_studio_harness.sh`
  - remote staged-clone focused tests for the same harness/UI-control assertions passed on `tertiary`

### 2026-04-01: Task 4 Remote Outdoor Telemetry Proof Captured Fresh `tertiary` Evidence

- Ran the narrow edit/preview proof directly on `tertiary` from the staged clone with:
  - `ARNIS_TELEMETRY_FAMILIES=terrain,roads,water,vegetation,structures,hotspots,player_local`
  - `bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter WorldProbeTelemetryFlags.spec.lua --edit-wait 30 --pattern-wait 120`
- The remote proof passed the intended telemetry gate:
  - `WorldProbeTelemetryFlags.spec.lua` passed
  - `ARNIS_MCP_READY` emitted
  - `ARNIS_MCP_EDIT_ACTION` reported `total=1 passed=1 failed=0`
- Fresh preview telemetry artifacts were produced on `tertiary` and synced back locally:
  - `/tmp/arnis-preview-plugin-state.json`
  - `/tmp/arnis-preview-telemetry-summary.txt`
- The synced summary established the current outdoor hotspot baseline:
  - `imported=80`
  - `hotspot_status=present`
  - `last_sync_elapsed_ms=17442`
  - `slow_chunk=-1_0`
  - `slow_chunk_total_ms=155`
  - `slow_chunk_buildings_ms=153`
  - `slow_chunk_terrain_ms=0`
  - `telemetry_families=terrain,roads,water,vegetation,structures,hotspots,player_local`
- The remaining harness hygiene gap is unchanged but now tightly bounded:
  - the run reached `main harness flow complete; exiting`
  - teardown reached `cleanup starting` and `quit_studio requesting graceful quit`
  - the run did not emit `quit_studio finished`, `cleanup finished`, or `HARNESS_EXIT:0`
  - `tertiary` was force-cleaned after the run so no harness, Studio, MCP, Vertigo Sync, or lock state remained resident

### 2026-04-01: Task 5 Stayed On The No-Schema-Expansion Path

- Regenerated the local `austin` compile outputs from the clean active worktree:
  - `rust/out/austin-manifest.json`
  - `rust/out/austin-manifest.sqlite`
  - `rust/out/austin.truth-pack.sqlite`
  - `rust/out/austin.truth-pack.summary.json`
- Re-ran `python3 scripts/source_truth_pack_audit.py rust/out/austin.truth-pack.sqlite --json-out /tmp/arnis-outdoor-truth-pack-report.json`.
- The fresh truth-pack result did not justify a canonical manifest/schema change for this tranche:
  - `feature_count=83503`
  - `retained_semantic_count=68939`
  - `dropped_semantic_count=23107`
  - `collapse_count=23672`
  - outdoor coverage stayed complete for `landuse`, `roads`, `vegetation`, and `water`
  - current loss pressure remains concentrated in `structures` (`overture->osm|cross_source_overlap` and dropped structure semantics), which the truth-pack already captures without new manifest fields
- Decision:
  - no `0.4.0` manifest/schema expansion was required for the first outdoor tranche
  - the next pressure remains in truth-pack/audit surfaces and downstream render/telemetry fidelity, not the canonical manifest contract

### 2026-04-01: Task 6 Local Outdoor Tranche Landed Player-Local Terrain Material Richness And Stronger Building Hotspot Context

- The first measured terrain/material/detail target was upgraded from roughness-only to player-local material richness:
  - `WorldProbe.client.lua` now emits `terrainMaterial` per local terrain ray hit
  - `WorldProbeTerrain.lua` now summarizes `materialKindCount`, `dominantMaterial`, `dominantMaterialSampleCount`, and `nonGrassSampleCount`
  - `scene_fidelity_audit.py` and `scene_parity_audit.py` now preserve and compare those fields so edit/play reports can quantify “default grass / textureless” complaints near the player instead of only reporting height roughness
- The first measured hotspot target now exposes the building-heavy slow-chunk reality compactly:
  - `AustinPreviewBuilder.lua` now records `buildingMeshCreateMs`, `buildingMeshPartCount`, `buildingRoofMeshPartCount`, and `buildingMeshTriangleCount` into the compact `slow_chunk` telemetry event
  - `preview_telemetry_summary.py` now derives `terrainSignalStatus`, `dominantCostCenter`, and `dominantCostRatio`, and carries `terrainCellCount`, `terrainSubsampleCount`, and `buildingFeatureCount`
  - This turns the current `-1_0` hotspot from a coarse `buildingsMs` clue into a compact, queryable breakdown suitable for fast iteration and log-safe remote proof
- New/updated focused coverage is green locally:
  - `scripts/tests/test_play_render_truth.py`
  - `scripts/tests/test_preview_telemetry_summary.py`
  - `scripts/tests/test_scene_fidelity_audit.py`
  - `scripts/tests/test_scene_parity_audit.py`
  - `roblox/src/ServerScriptService/Tests/WorldProbeTerrain.spec.lua`
  - `roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua`
- Local-safe verification passed after the tranche landed:
  - `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_preview_telemetry_summary -v`
  - `python3 -m unittest scripts.tests.test_scene_fidelity_audit scripts.tests.test_scene_parity_audit -v`
- Remaining next proof:
  - sync the current worktree snapshot to `tertiary`
  - run `TerrainOutdoorFidelity.spec.lua`
  - rerun the narrow play proof with `ARNIS_TELEMETRY_FAMILIES=terrain,roads,water,vegetation,structures,hotspots,player_local`
  - confirm the remote markers/artifacts preserve the new local terrain material fields and the stronger slow-chunk building breakdown

### 2026-04-01: Task 4 Remote Telemetry-Flags Proof Passed On `tertiary`, With Teardown Still Blocking Final Exit

- Direct staged-clone proof on `tertiary` passed the focused edit spec:
  - `PASS WorldProbeTelemetryFlags.spec`
  - `ARNIS_MCP_EDIT_ACTION` reported `total=1 passed=1 failed=0`
- The same run also produced fresh preview telemetry artifacts and they were synced back locally:
  - remote `/tmp/arnis-preview-plugin-state.json`
  - remote `/tmp/arnis-preview-telemetry-summary.txt`
  - local `/tmp/arnis-preview-plugin-state.json`
  - local `/tmp/arnis-preview-telemetry-summary.txt`
- The synced compact preview summary recorded the current hotspot baseline used for the first outdoor target selection:
  - `imported=80`
  - `hotspot_status=present`
  - `last_sync_elapsed_ms=17442`
  - `slow_chunk=-1_0`
  - `slow_chunk_phase=foreground`
  - `slow_chunk_total_ms=155`
  - `slow_chunk_buildings_ms=153`
  - `slow_chunk_terrain_ms=0`
  - `slow_chunk_artifacts=28`
  - `telemetry_families=terrain,roads,water,vegetation,structures,hotspots,player_local`
- The remaining Task 4 blocker is still teardown only:
  - the run reached `main harness flow complete; exiting`
  - then `cleanup starting`
  - then `quit_studio requesting graceful quit`
  - but it still did not emit `quit_studio finished`, `cleanup finished`, or `HARNESS_EXIT:0`
- `tertiary` was force-cleaned after the run; no harness, Studio, MCP, Vertigo Sync, or lock state was left resident.

### 2026-04-01: Task 5 Stayed On The No-Schema Path After Fresh Truth-Pack Review

- A fresh local truth-pack export and bounded audit confirmed that the first outdoor tranche still does not need canonical manifest/schema expansion.
- The current upstream pressure is real but remains fully expressible in truth-pack/audit surfaces rather than missing `0.4.0` manifest fields:
  - `truth_pack_collapse_count = 23672`
  - `truth_pack_dropped_semantic_count = 23107`
  - all current outdoor overlap-loss and dropped-semantic pressure is concentrated in `structures`
  - the dominant collapse bucket remains `overture->osm|cross_source_overlap`
  - the dominant dropped semantic fields remain `height_m`, `name`, and `levels`
- No terrain/landuse/roads/water/vegetation schema gap was proven by the fresh report, so the next tranche stays on shared runtime/audit improvements rather than contract churn.

### 2026-04-01: Task 6 Local Slice Landed Shared Terrain Richness And Hotspot Shape Context

- The first local-safe outdoor fidelity/hotspot slice is now materially richer on the shared edit/play path:
  - `WorldProbeTerrain.lua` and `WorldProbe.client.lua` now carry near-player terrain material richness, not just roughness
  - `scene_fidelity_audit.py` and `scene_parity_audit.py` now preserve and compare `localTerrain` material richness fields
  - `preview_telemetry_summary.py` now surfaces existing building hotspot breakdown plus chunk-shape context and dominant-cost interpretation
- The new/downstream metrics now include:
  - player-local: `materialKindCount`, `dominantMaterial`, `dominantMaterialSampleCount`, `nonGrassSampleCount`
  - preview hotspot: `buildingMeshCreateMs`, `buildingMeshPartCount`, `buildingRoofMeshPartCount`, `buildingMeshTriangleCount`, `terrainCellCount`, `terrainSubsampleCount`, `buildingFeatureCount`, `dominantCostCenter`, `dominantCostMs`, `dominantCostRatio`, `terrainSignalStatus`
- This means the current `-1_0` hotspot is no longer just “buildings are slow”; the compact summary can now distinguish:
  - whether terrain signal was not authored vs missing
  - how much of the chunk shape was terrain-driven
  - how much of the cost center is building-dominated
- Local-safe verification passed after the slice:
  - `python3 -m unittest scripts.tests.test_preview_telemetry_summary scripts.tests.test_scene_fidelity_audit scripts.tests.test_scene_parity_audit -v`
  - `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract scripts.tests.test_preview_telemetry_summary scripts.tests.test_scene_fidelity_audit scripts.tests.test_scene_parity_audit -v`
- The next open proof step is `tertiary` only:
  - `WorldProbeTerrain.spec.lua`
  - `TerrainOutdoorFidelity.spec.lua`
  - refreshed edit/play preview telemetry from the staged clone

### 2026-04-01: Task 6 Continued With Compact Building Hotspot Split And Truth-Pack Headline

- `scripts/source_truth_pack_audit.py` now emits a compact `summary.headline` block so the largest outdoor-family coverage gap, dropped-semantics family, and overlap-loss family are visible without reading the full per-family tables.
- `roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua` now separates shell/detail time from mesh-creation time with `shellDetailMs`, and `roblox/src/ServerScriptService/ImportService/init.lua` now tracks `buildingShellDetailMs` plus `buildingInteriorMs` in the chunk profile.
- `roblox/src/ServerScriptService/StudioPreview/AustinPreviewBuilder.lua` now preserves those new building split fields in the compact slow-chunk telemetry event and perf attributes, so preview hotspot evidence is no longer collapsed into one `buildingsMs` bucket.
- `scripts/preview_telemetry_summary.py` now computes compact derived building metrics:
  - `buildingResidualMs`
  - `buildingMeshCreateRatio`
  - `buildingResidualRatio`
  - `buildingMeshPartsPerFeature`
  - `buildingMeshTrianglesPerFeature`
- Local-safe verification passed for the touched lane:
  - `python3 -m unittest scripts.tests.test_preview_telemetry_summary scripts.tests.test_scene_fidelity_audit scripts.tests.test_source_truth_pack_audit scripts.tests.test_play_render_truth -v`
  - `bash -n scripts/run_studio_harness.sh`
  - `bash -n scripts/run_studio_harness_remote.sh`
  - `cargo test --manifest-path rust/Cargo.toml --workspace`
  - `git diff --check`
- Direct `tertiary` proof against the git-backed proof clone produced fresh live preview evidence for the current hotspot:
  - `AustinPreviewTelemetry.spec.lua` passed with `ARNIS_MCP_EDIT_ACTION total=1 passed=1 failed=0`
  - `AustinPreviewBuilder` emitted slow-chunk lines for `chunkId=-1_0` with `buildingsMs=150`, `buildingMeshCreateMs=1`, `buildingShellDetailMs=149`, `buildingInteriorMs=0`, `buildingMeshVertexCount=6112`, and `buildingMeshTriangleCount=3056`
  - a follow-on `ImportService.spec.lua` run also emitted the new split fields on the same building-dominant chunk before teardown stalled
- Harness stance for this tranche is now explicit:
  - direct proof on `~/Projects/arnis-roblox-main` on `tertiary` remains the operator truth
  - the staged clone is still untrusted for `scripts/` completeness
  - no new harness feature work should be opened from this status note; only bounded teardown/cleanup hygiene remains in scope
- `tertiary` was force-cleaned after the debugging runs; no Studio, harness, MCP, `vsync serve`, or lock residue was intentionally left behind.

### 2026-04-01: Shared Austin Fidelity Default, Semantic Lineage, And Hotspot Subphases Landed Locally

- The shared Austin export path is now deliberately higher fidelity:
  - `scripts/export_austin_from_osm.sh` defaults to `high`
  - `scripts/export_austin_to_lua.sh` now documents that higher shared default explicitly
  - the plan remains to prove the visual impact on `tertiary`, not to treat the script change itself as the proof
- The truth-pack now records field-level structure resolution instead of only coarse collapse/drop counts:
  - `rust/crates/arbx_pipeline/src/lib.rs` now merges missing Overture building semantics into retained overlapping OSM buildings for the current mapped structure fields
  - `rust/crates/arbx_pipeline/src/truth_pack.rs` now writes `semantic_lineage` rows
  - `scripts/source_truth_pack.py` and `scripts/source_truth_pack_audit.py` now surface merged/conflict lineage compactly
  - this means the dominant structure pressure can now distinguish:
    - values merged from collapsed Overture features
    - values identical across retained/collapsed features
    - values lost to retained canonical OSM features
- Building-heavy preview hotspots are now materially more actionable:
  - `BuildingBuilder.lua`, `ImportService/init.lua`, `AustinPreviewBuilder.lua`, and `scripts/preview_telemetry_summary.py` now split shell-detail time into:
    - `buildingRoofBuildMs`
    - `buildingFacadeDetailMs`
    - `buildingPerimeterDetailMs`
    - `buildingTerrainFillMs`
    - `buildingRooftopDetailMs`
    - `buildingNameLabelMs`
  - the compact summary now emits `buildingShellDominantDetailPhase` and `buildingShellDominantDetailMs`
  - the old blanket `buildingShellDetailMs` helper path was removed from the Rust source-truth seam where it was no longer the truthful unit of analysis
- Local-safe verification for this continuation slice passed:
  - `python3 -m unittest scripts.tests.test_source_truth_pack.SourceTruthPackHelperTests scripts.tests.test_source_truth_pack_audit.SourceTruthPackAuditTests scripts.tests.test_austin_fidelity scripts.tests.test_play_render_truth scripts.tests.test_preview_telemetry_summary -v`
  - `python3 -m py_compile scripts/source_truth_pack.py scripts/source_truth_pack_audit.py scripts/preview_telemetry_summary.py scripts/tests/test_source_truth_pack.py scripts/tests/test_source_truth_pack_audit.py scripts/tests/test_austin_fidelity.py scripts/tests/test_play_render_truth.py scripts/tests/test_preview_telemetry_summary.py`
  - `cargo test --manifest-path rust/Cargo.toml -p arbx_pipeline overture_gap_fill_backfills_missing_osm_semantics_from_collapsed_overture -- --nocapture`
  - `bash -n scripts/run_studio_harness.sh`
  - `bash -n scripts/run_studio_harness_remote.sh`
- No Studio ran on this machine for this slice.
- The next proof step is still `tertiary` only:
  - rerun `AustinPreviewTelemetry.spec.lua`
  - rerun `ImportService.spec.lua`
  - capture a fresh preview hotspot artifact from the git-backed proof clone after the higher-fidelity export default lands there

### 2026-04-01: `tertiary` Proof Lane Moved To A Real Git Clone And Verified The New Hotspot Fields

- The older `~/Projects/arnis-roblox` folder on `tertiary` is not a git repository, so it should not be treated as the canonical remote proof surface anymore.
- A new shallow git-backed proof clone was seeded at `~/Projects/arnis-roblox-main` directly from `origin/main` at commit `16b6124`.
- Remote static verification on that clone passed:
  - `python3 -m unittest scripts.tests.test_source_truth_pack.SourceTruthPackHelperTests scripts.tests.test_source_truth_pack_audit.SourceTruthPackAuditTests scripts.tests.test_preview_telemetry_summary -v`
- Focused Studio edit proof on that clone surfaced the new fields in the live log:
  - `PASS AustinPreviewTelemetry.spec`
  - `PASS ImportService.spec`
  - the preview slow-chunk line for `chunkId=-1_0` now includes the new shell-detail subphases:
    - `buildingRoofBuildMs=20`
    - `buildingFacadeDetailMs=0`
    - `buildingPerimeterDetailMs=0`
    - `buildingTerrainFillMs=116`
    - `buildingRooftopDetailMs=0`
    - `buildingNameLabelMs=0`
    - alongside `buildingShellDetailMs=149` and `buildingMeshCreateMs=1`
- The first attempt launched two edit proofs in parallel and produced avoidable log interleaving on the shared remote Studio lane. The proof findings are still usable, but future `tertiary` runs should stay serialized unless the proof surface is explicitly split.
- The harness still reached:
  - `main harness flow complete; exiting`
  - `cleanup starting exit_code=0`
  - `cleanup policy ... should_close=true reason=success`
  - `quit_studio starting`
  - `quit_studio requesting graceful quit`
- After the interleaved proof run, `tertiary` was force-cleaned again. No `run_studio_harness.sh`, `rbx-studio-mcp`, or `vsync serve` process remains, and Studio was explicitly killed to keep the machine polite after the run.

### 2026-04-01: Serialized `tertiary` Play Debug Narrowed The Building Problem To Street-Level Wall/Façade Fidelity, Not Missing Runtime Import

- Ran a fresh serialized play-only proof directly on `tertiary` from `~/Projects/arnis-roblox-main` with:
  - `ARNIS_TELEMETRY_FAMILIES=terrain,roads,water,vegetation,structures,hotspots,player_local`
  - `ARNIS_SCENE_AUDIT_DIR=/tmp/arnis-play-debug-buildings`
  - `VSYNC_REPO_DIR=$HOME/.codex-remote-studio/vertigo-sync bash scripts/run_studio_harness.sh --takeover --hard-restart --skip-edit-tests --play-wait 30 --pattern-wait 150`
- The runtime world itself is loading in play:
  - `RunAustin` loaded `AustinCanonicalManifestIndex`
  - `[ImportManifest]` reported `worldRoot=Workspace.GeneratedWorld_Austin totalInstances=2447 chunksImported=80`
  - `RunAustin` reported `roads=1046 buildings=92 props=575`
  - bootstrap advanced through `world_ready,streaming_ready,minimap_ready,gameplay_ready`
- Fresh player-local play telemetry at `gameplay_ready` shows nearby building content exists around spawn:
  - `nearbyBuildingModels=7`
  - `nearbyMergedBuildingMeshParts=5`
  - `nearbyWallParts=4`
  - `nearestWallDistanceStuds=2.2`
  - `nearbyRoofParts=14`
  - nearby source IDs include `osm_952130555`, `osm_93135618`, `osm_269078411`, `osm_269078413`, `osm_93135675`, and `osm_93135773`
- Fresh scene-play audit markers also show this is not a wholesale building-loss bug:
  - `buildingModelsMissingDirectShell=0`
  - `buildingModelsWithRoofClosureDeck=15`
  - `buildingModelsWithNoRoofEvidence=2`
  - wall/roof material buckets and roof-shape buckets were emitted normally for `GeneratedWorld_Austin`
- Current diagnosis changed:
  - the user-visible complaint is now best explained as a street-level wall/facade visibility or building simplification problem inside the shared building path, not a play-mode failure to import buildings at all
  - the strongest local suspects are the merged `shellMesh` wall presentation and the current `shouldPreferSimpleShellDetail(...)` path for low-rise residential/apartment buildings near spawn
  - the current audit stack is still too aggregate to prove “walls are visually not there” per nearby building even when global shell/roof counts remain green
- The failed remote screenshot attempt (`screencapture ... could not create image from display`) means this slice still lacks a committed visual artifact; the next tranche should add explicit near-player building façade visibility diagnostics before changing builder behavior.
- `tertiary` was force-cleaned after the run; lingering `RobloxStudio` and crash-handler processes were killed and the harness lock directory was removed manually.

### 2026-04-01: Simple Residential Readability And Terrain-Fill Batching Went Green On `tertiary`

- The next shared builder tranche was driven red-first on `tertiary`, then proven green on the same git-backed proof clone at `~/Projects/arnis-roblox-main`.
- Two focused Luau specs now pass remotely after the shared `BuildingBuilder.lua` changes:
  - `PASS SimpleResidentialShell.spec`
  - `PASS BuildingTerrainFillBatching.spec`
- What changed in shared runtime/edit code:
  - simple low-rise residential/apartment shells now retain a cheap street-level perimeter cue instead of dropping all readability detail
  - shell terrain fill now batches contiguous interior row spans through a hookable `_fillTerrainBlock` seam instead of issuing one `FillBlock` per surviving cell
- The new terrain-fill batching proof is explicit:
  - the rectangular apartment fixture now collapses to one fill call per interior row
  - the batching logic still honors the existing edge-clearance and point-in-polygon gates before any merged write is emitted
- The same remote proof runs also refreshed the preview hotspot summary for the real outdoor-heavy offender `chunkId=-1_0`:
  - before this tranche, the remote proof surface was reading `buildingTerrainFillMs=220`
  - after the shared batching change, the same slice read `buildingTerrainFillMs=193` on the `SimpleResidentialShell.spec` pass and `buildingTerrainFillMs=191` on the `BuildingTerrainFillBatching.spec` pass
  - `buildingRoofBuildMs` stayed flat at about `20ms`, and no new façade-band cost was introduced (`buildingFacadeDetailMs=0`)
- This corrects the next-step diagnosis again:
  - the builder no longer needs a speculative “missing walls” fix
  - the highest-value remaining building work is richer façade/readability detail for the shared simple-shell path and further reduction of the still-dominant `buildingTerrainFillMs` cost on `-1_0`
- Verification for this slice:
  - local-safe: `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_preview_telemetry_summary -v`
  - local-safe: `git diff --check`
  - remote `tertiary`: serialized edit proof for `SimpleResidentialShell.spec.lua`
  - remote `tertiary`: serialized edit proof for `BuildingTerrainFillBatching.spec.lua`
- No Studio ran on this machine. `tertiary` was force-cleaned again after the proofs.

### 2026-04-01: Simple-Shell Beltline Cues And Exact-Span Terrain Rectangles Pushed The Same Hotspot Lower

- The next shared builder tranche stayed in the same `BuildingBuilder.lua` seam and tightened both sides of the same complaint:
  - simple low-rise residential/apartment shells now publish a bounded `ArnisFacadeBeltlineCount` cue and render a cheap opaque `FacadeBeltline` instead of staying at foundation-only readability
  - interior terrain fill no longer stops at row spans; it now merges identical accepted spans across adjacent rows into exact rectangles before calling `_fillTerrainBlock`
- The focused proofs stayed green on `tertiary` after this follow-on change:
  - `PASS SimpleResidentialShell.spec`
  - `PASS BuildingTerrainFillBatching.spec`
- The measured hotspot moved again on the same remote preview surface for `chunkId=-1_0`:
  - `buildingTerrainFillMs=182` on the fresh `SimpleResidentialShell.spec` run
  - `buildingTerrainFillMs=187` on the fresh `BuildingTerrainFillBatching.spec` run
  - `buildingRoofBuildMs` remained about `19ms`
  - `buildingFacadeDetailMs` remained `0`, so this extra readability cue did not spill into the expensive glass-band path
- Current building diagnosis after this follow-on tranche:
  - the shared simple-shell path is now less likely to read as “blank walls” from street level because it carries both a base cue and a bounded mid-wall beltline cue
  - the dominant cost center is still `buildingTerrainFillMs`, but it is down materially from the earlier `220ms` read
  - the next product-side work should keep targeting shared building visual richness and shell terrain-fill cost, not new harness features
- GUI-session screenshot attempt status:
  - a Terminal-launched GUI proof on `tertiary` was attempted to capture `/tmp/arnis-gui-play.png`
  - that run failed before play proof completion because Studio reported the maximum allowed Studio windows were already open
  - the failure mode was operational, not a new Roblox runtime regression, and `tertiary` had to be force-cleaned again by killing the orphaned Studio child windows directly
  - the screenshot lane is therefore still not a green proof surface; the next screenshot attempt must start from a stricter preflight that proves zero existing Studio windows before launching the GUI-session play run
- Verification for this slice:
  - local-safe: `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_preview_telemetry_summary -v`
  - local-safe: `git diff --check`
  - remote `tertiary`: serialized edit proof for `SimpleResidentialShell.spec.lua`
  - remote `tertiary`: serialized edit proof for `BuildingTerrainFillBatching.spec.lua`
  - remote `tertiary`: GUI-session play screenshot attempt reproduced the max-Studio-window blocker and did not produce a trustworthy screenshot artifact

### 2026-04-01: `open_studio()` Now Recognizes Blank-Title Ready Sessions And Clears Recovery Dialogs During Launch

- The next harness change stayed narrowly bounded to startup hygiene and did not add a new proof lane:
  - `open_studio()` now force-cleans pre-existing Studio windows before launch, as already planned
  - for custom place launches, `studio_opened_target_place()` now accepts the live `tertiary` condition where Studio is already `ready_for_harness` but reports `front_window=""` and `window_count=0`
  - the post-launch wait loops now clear `dismiss-dont-save` recovery prompts during launch, not only before launch, so stacked `Lighting Technology Migration` plus `Auto-Recovery` dialogs no longer require a second manual path in the common case
- This was driven directly from the live `tertiary` GUI repro:
  - the GUI-launched run still logged only through `[harness] opening place: ...`
  - direct `studio_ui_control.py get-session-status` from the same run showed Studio had already reached `ready_for_harness` with `front_window=""`, so the old basename-only success predicate was wrong for this lane
  - manually dismissing `Lighting Technology Migration` immediately exposed `Auto-Recovery`, proving the launch loop also needed to clear recovery dialogs after launch, not just at the top of `open_studio()`
- Current truth after the fix:
  - the startup predicates are now more correct and the recovery-dialog handling is stronger
  - the GUI-session screenshot path is still not fully green, because a live GUI-launched harness shell can remain stuck after Studio reaches `ready_for_harness`
  - that remaining blocker is now narrower: it is no longer stale windows, missing ready-session recognition, or missing recovery-dialog clears
  - further harness work should stay in wrap-up mode; the next high-value code change is still product-side shared building detail
- Verification for this slice:
  - local-safe: `python3 -m unittest scripts.tests.test_run_studio_harness scripts.tests.test_convergence_guardrails -v`
  - local-safe: `python3 -m unittest scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_open_studio_accepts_ready_editor_when_custom_place_window_title_is_blank scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_open_studio_preflights_existing_studio_instances_before_launch scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_open_studio_clears_multi_window_launch_failures_before_retry scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_open_studio_post_launch_waits_also_clear_auto_recovery_dialogs -v`
  - local-safe: `bash -n scripts/run_studio_harness.sh`
  - local-safe: `git diff --check`
  - remote `tertiary`: manual live repro confirmed the fix targets the real blank-title and stacked-dialog startup conditions

### 2026-04-02: Simple Shells Now Keep A Bounded Roofline Cornice Cue

- The next shared building-readability tranche landed in the same low-cost seam as the earlier foundation and beltline work:
  - simple low-rise shells now publish `ArnisCorniceCount`
  - both fallback and mesh-backed simple-shell paths retain a bounded roofline cornice cue instead of reserving that read only for the richer non-simple shell path
- This stayed deliberately bounded:
  - no glass bands were re-enabled
  - no full pilaster path was re-enabled
  - no rooftop-detail path was widened
  - the change stayed in the perimeter/readability layer only
- The focused `tertiary` proof is green:
  - `Running tests: SimpleResidentialShell.spec`
  - `PASS SimpleResidentialShell.spec`
  - `TestEZ tests complete. total=1 passed=1 failed=0`
- The same proof slice kept the current outdoor hotspot diagnosis stable rather than regressing it:
  - `chunkId=-1_0`
  - `buildingTerrainFillMs=185`
  - `buildingRoofBuildMs=19`
  - `buildingShellDetailMs=217`
- Current product interpretation:
  - simple shells now have a three-band readability stack at low cost: foundation, beltline, and cornice
  - the next bounded building cue should be vertical, not another horizontal layer; corner accents are the strongest current candidate
  - the next no-trampling outdoor audit extension after structure lineage should target roads
- Verification for this slice:
  - local-safe: `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_preview_telemetry_summary scripts.tests.test_convergence_guardrails -v`
  - local-safe: `git diff --check`
  - remote `tertiary`: focused edit proof for `SimpleResidentialShell.spec.lua`

### 2026-04-02: Truth-Pack Now Records Retained-Original Road Semantics

- The next no-trampling outdoor audit tranche stayed in the producer, not the manifest schema:
  - `arbx_pipeline` now emits retained-original semantic lineage for retained road features
  - the road lineage fields intentionally mirror the existing retained road semantic surface: `lanes`, `surface`, `sidewalk`, `maxspeed`, `lit`, `oneway`, `layer`, `bridge`, and `tunnel`
  - source attribution is explicit and bounded: retained road lineage currently records `overpass` as the source surface for these semantics
- The value of this slice is observability, not cosmetic metric inflation:
  - structure lineage was already explicit, but retained roads still looked like canonical facts with no upstream provenance trail
  - this closes that blind spot for the most immediately valuable non-structure outdoor family without widening the schema or inventing a second audit path
  - the existing Python truth-pack helper now proves the retained-original road lineage is readable from the sqlite artifact without needing Studio
- Current interpretation after the change:
  - roads are now a first-class truth-pack lineage family instead of a retained-semantics-only family
  - merge/conflict-heavy road lineage is still future work; this slice only proves retained-original provenance survives into the truth-pack
  - the next outdoor no-trampling tranche can build on this producer seam instead of starting from zero
- Verification for this slice:
  - local-safe: `cargo test --manifest-path rust/Cargo.toml -p arbx_pipeline overpass_truth_pack_records_retained_original_road_lineage -- --nocapture`
  - local-safe: `python3 -m unittest scripts.tests.test_source_truth_pack.SourceTruthPackHelperTests -v`

### 2026-04-02: Shared Austin Export Defaults Are Higher Fidelity, And Simple Shells Now Add Bounded Corner Accents

- I kept this tranche product-side and explicitly avoided more harness engineering except where it directly affected fidelity work.
- Shared Austin export defaults are now stronger:
  - `export_austin_to_lua.sh` injects `--profile high` when the caller does not already pass explicit fidelity arguments
  - satellite is still supported explicitly, but it is no longer part of the shared default because it is too heavy for the current iteration loop and not aligned with the planetary-streaming direction
  - `export_austin_from_osm.sh` now describes that default honestly as a higher-fidelity default rather than a generic dev fixture profile
  - I did **not** keep the new generated-asset verifier guardrail that rejected universally coarse terrain, because the currently committed Austin sample-data and preview shards are still universally `cellSizeStuds=4`, and flipping that guardrail on before regenerating the shared assets would have created immediate doc/test drift instead of improving truth
- Shared simple-shell readability also moved one bounded step forward:
  - simple low-rise shells now publish `ArnisCornerAccentCount`
  - both fallback and mesh-backed simple-shell paths now add cheap vertical `CornerAccent` geometry at footprint corners
  - this keeps the shared readability stack bounded and cheap: foundation + beltline + cornice + corner accents
  - no glass bands, full pilasters, or broader facade systems were reopened
- Important process note:
  - one dispatched worker violated the standing repo rule and started a local `run_studio_harness.sh` edit proof on this machine
  - I stopped that work, killed/confirmed cleanup of the related local harness/MCP/vsync processes, restored the only repo residue (`RunAllConfig.lua` spec filter), and closed the offending worker
  - local Studio remains forbidden; all future Studio proof stays on `tertiary`
- Current interpretation after the change:
  - the shared export defaults are now pointed at a bounded high-fidelity target, but the committed generated Austin assets have not yet been regenerated from that stronger default
  - the next true proof for this slice is on `tertiary`: regenerate shared Austin assets there, then re-measure terrain/material richness and visually confirm the new simple-shell corner accents
- Verification for this slice:
  - local-safe: `python3 -m unittest scripts.tests.test_austin_fidelity.AustinFidelityScriptTests scripts.tests.test_generated_austin_assets.GeneratedAustinAssetsVerifierTests.test_collect_errors_accepts_split_preview_terrain_fragments scripts.tests.test_play_render_truth.PlayRenderTruthTests.test_simple_shells_keep_bounded_corner_accents_for_vertical_readability -v`

### 2026-04-02: Shared Austin Defaults Are Now Bounded For Planetary Streaming

- I corrected the shared export defaults after the repo briefly drifted toward satellite-backed Austin as the normal path.
- The first pass only removed explicit `--satellite` from the wrapper, but that was not sufficient because `arbx_cli --profile high` still implies satellite internally.
- The committed default behavior is now:
  - `export_austin_to_lua.sh` injects `--terrain-cell-size 2` only
  - satellite imagery remains explicit opt-in, not part of the shared default
  - `build_austin_max_fidelity_place.sh` now builds from bounded `cell=2` fidelity instead of forcing `--satellite`
- This keeps the default iteration path aligned with the current priorities:
  - faster compile/export loops
  - lower data weight
  - better alignment with planetary streaming instead of city-specific heavy imagery
- Verification for this correction:
  - local-safe: `python3 -m unittest scripts.tests.test_austin_fidelity scripts.tests.test_play_render_truth.PlayRenderTruthTests.test_simple_shells_keep_bounded_corner_accents_for_vertical_readability scripts.tests.test_generated_austin_assets.GeneratedAustinAssetsVerifierTests.test_collect_errors_accepts_split_preview_terrain_fragments -v`
  - local-safe: `bash -n scripts/export_austin_from_osm.sh`
  - local-safe: `bash -n scripts/export_austin_to_lua.sh`
  - local-safe: `bash -n scripts/build_austin_max_fidelity_place.sh`
  - local-safe: `git diff --check`

### 2026-04-02: CLI Profiles And Branch Hygiene Were Brought Back Into Line

- The CLI profile surface now matches the shared bounded-default direction instead of fighting it:
  - `arbx_cli --profile high` now means `cell=2, satellite=off`
  - `--satellite` remains explicit opt-in
  - `insane` / `--yolo` remain the imagery-heavy path
- This is a real DX fix, not just a wrapper fix:
  - the CLI help text, runtime profile behavior, and shared Austin wrapper semantics are now aligned
  - the new unit seam is `apply_compile_profile(...)`, which keeps future profile drift easier to catch
- Branch/worktree drift was also burned down:
  - all known merged local worktrees under `.worktrees/` were removed except the current detached clean worktree
  - merged local branches were deleted
  - merged remote branches were deleted from `origin`
  - remote branch state now shows only `main`
- Verification for this correction:
  - local-safe: `cargo test --manifest-path rust/Cargo.toml -p arbx_cli help_text_is_0_4_0_only -- --nocapture`
  - local-safe: `cargo test --manifest-path rust/Cargo.toml -p arbx_cli high_profile_keeps_cell_two_without_enabling_satellite -- --nocapture`
  - local-safe: `python3 -m unittest scripts.tests.test_austin_fidelity -v`

### 2026-04-02: First Real Non-Satellite `tertiary` Austin Refresh Exposed The Next Bottlenecks

- The direct git-backed proof clone on `tertiary` was updated to `origin/main` and the shared Austin export was rerun from the corrected bounded default:
  - `bash scripts/export_austin_to_lua.sh`
  - effective compile command: `target/debug/arbx_cli compile ... --terrain-cell-size 2 ...`
  - no `--satellite`
- The run produced the first honest end-to-end baseline for the non-satellite path:
  - elapsed wall time: `249.65s`
  - max resident set size from `/usr/bin/time -lp`: `4032086016`
  - the run failed with `No space left on device (os error 28)` while writing output
- `tertiary` capacity state at the time of failure:
  - `/System/Volumes/Data` had only `2.1 GiB` free (`99%` used)
  - repo-local footprint was not the primary bloater: `rust/out` was `21M`, `rust/target` was `603M`
- Current interpretation after this proof:
  - removing satellite from the shared path was the correct move and is now proven in the actual remote command line
  - compile/runtime cost is still substantial even without imagery
  - the next performance tranche should target compile-path cost directly, while the next remote proof tranche must first avoid disk-capacity failure on `tertiary`

### 2026-04-02: Exporter Terrain Materials Are Now Lazily Allocated

- The first exporter-side memory optimization for the non-satellite path is now in place:
  - `build_empty_chunk()` no longer eagerly allocates a full per-cell `terrain.materials` vector
  - `paint_terrain_polygon()` now materializes per-cell terrain materials only on first real override
  - the satellite enrichment path now uses the same lazy allocator, so imagery-backed runs still keep their previous behavior when explicitly enabled
- Why this matters:
  - the old path allocated `width * depth` material strings for every touched chunk even when a chunk stayed at the default terrain material
  - on the bounded `cell=2` path that is `16,384` default-material strings per touched chunk
  - this does not solve the whole compile bottleneck, but it removes a large class of unnecessary churn from the normal non-satellite export path
- Verification for this slice:
  - local-safe: `cargo test --manifest-path rust/Cargo.toml -p arbx_roblox_export export_omits_per_cell_terrain_materials_when_no_overrides_are_needed -- --nocapture`
  - local-safe: `cargo test --manifest-path rust/Cargo.toml -p arbx_roblox_export export_paints_terrain_materials_from_landuse_semantics -- --nocapture`

### 2026-04-02: Non-Satellite Export Cut Peak RSS, Then Hit Output-Path Waste

- I reran the direct git-backed proof clone on `tertiary` after the lazy terrain-material change:
  - command: `ssh tertiary 'cd ~/Projects/arnis-roblox-proof && git fetch origin main && git reset --hard origin/main && /usr/bin/time -lp bash scripts/export_austin_to_lua.sh'`
  - the compile path stayed non-satellite: `target/debug/arbx_cli compile ... --terrain-cell-size 2 ...`
- The rerun materially improved memory behavior:
  - manifest JSON written in `175.19s`
  - manifest SQLite written in the same run
  - elapsed wall time before failure: `348.74s`
  - max resident set size from `/usr/bin/time -lp`: `2120515584`
- The failure shifted from compile OOM pressure to output-path waste:
  - `write_source_truth_pack_sqlite()` failed late with `unable to open database file`
  - `tertiary` had only about `116 MiB` free at that point
  - the generated manifest artifacts had already consumed roughly:
    - `austin-manifest.json`: `1.3G`
    - `austin-manifest.sqlite`: `1.1G`
    - `austin-manifest.sqlite-wal`: `1.3G`
- I tightened the export path accordingly:
  - `arbx_pipeline` now uses a small OSM-building spatial index when checking Overture gap-fill overlap, instead of scanning every OSM building candidate
  - generated manifest SQLite now writes with `journal_mode = MEMORY` so the default export path no longer leaves giant `-wal` / `-shm` sidecars behind
  - truth-pack SQLite now creates its parent directory itself instead of assuming the caller already prepared it
- Current interpretation after this slice:
  - the bounded non-satellite path is materially healthier than the earlier baseline
  - the next remote export should no longer burn an extra manifest-sized WAL file just to produce a generated SQLite store
  - the next likely compile bottleneck after disk waste is still Overture/OSM merge cost, which is why the spatial-index tranche landed in the same pass
- Verification for this slice:
  - local-safe: `cargo test --manifest-path rust/Cargo.toml -p arbx_pipeline overture_spatial_index_limits_candidates_to_nearby_osm_buildings -- --nocapture`
  - local-safe: `cargo test --manifest-path rust/Cargo.toml -p arbx_pipeline overture_gap_fill_ -- --nocapture`
  - local-safe: `cargo test --manifest-path rust/Cargo.toml -p arbx_pipeline truth_pack_sqlite_writer_creates_missing_parent_directory -- --nocapture`
  - local-safe: `cargo test --manifest-path rust/Cargo.toml -p arbx_roblox_export manifest_store_ -- --nocapture`

### 2026-04-02: The Default Austin Loop No Longer Requires A Giant JSON Sidecar

- After the `a00480c` rerun, the bounded non-satellite export finally made it past both SQLite outputs on `tertiary`, then failed later while `json_manifest_to_sharded_lua.py` was writing Lua shards:
  - the post-compile artifact footprint was roughly:
    - `rust/out/austin-manifest.json`: `1.3G`
    - `rust/out/austin-manifest.sqlite`: `1.3G`
    - `rust/out/austin.truth-pack.sqlite`: `22M`
    - `roblox/src/ServerStorage/SampleData`: `1.2G`
  - free disk at failure time was only about `116 MiB`
- The important design discovery is that the shared Austin-to-Lua path already shreds from SQLite:
  - `json_manifest_to_sharded_lua.py` is invoked with `--sqlite`
  - `refresh_preview_from_sample_data.py` already prefers `rust/out/austin-manifest.sqlite`
  - `refresh_runtime_harness_from_sample_data.py` already prefers the same SQLite source
  - `run_studio_harness.sh` already prefers `--manifest-sqlite` when it exists
- I changed the default wrapper path accordingly:
  - `export_austin_from_osm.sh` no longer passes `--out out/austin-manifest.json` by default
  - explicit JSON output still works when the caller passes `--out`
  - `export_austin_to_lua.sh` now removes stale `rust/out/austin-manifest.json` before and after the export stage unless the caller explicitly asked for JSON
- Current interpretation after this slice:
  - the default bounded Austin loop is now aligned with the repo's actual downstream consumers instead of dragging along a manifest-sized dead-weight sidecar
  - this should remove another ~`1.3G` from the normal `tertiary` proof path on the next rerun
  - the next likely storage pressure point after this change is the generated Lua shard footprint itself, not the compile artifacts
- Verification for this slice:
  - local-safe: `python3 -m unittest scripts.tests.test_austin_fidelity -v`
  - local-safe: `bash -n scripts/export_austin_from_osm.sh`
  - local-safe: `bash -n scripts/export_austin_to_lua.sh`
  - local-safe: `git diff --check`

### 2026-04-02: Bounded Non-Satellite Austin Export Is End-To-End Green On `tertiary`

- I pushed the JSON-free default wrapper change, cleaned only regenerable proof-clone outputs on `tertiary`, and reran the full shared Austin loop from the git-backed proof clone:
  - proof clone commit: `d5a581f`
  - command: `ssh tertiary 'cd ~/Projects/arnis-roblox-proof && git fetch origin main && git reset --hard origin/main && /usr/bin/time -lp bash scripts/export_austin_to_lua.sh'`
- This run completed successfully end to end on the internal disk:
  - `arbx_cli compile ... --terrain-cell-size 2 --sqlite-out ... --truth-pack-out ... --truth-pack-summary-out ...`
  - compile/store stage timing: `19.86s`
  - Lua sharding: `7400` runtime shard modules written
  - preview refresh: `906` preview shards plus `906` canonical bounded sample-data shards written
  - runtime harness refresh: `143` harness shards written
  - generated-asset verification passed
  - total wall time: `335.72s`
  - max resident set size from `/usr/bin/time -lp`: `2198536192`
- What changed relative to the earlier failing runs:
  - no giant manifest JSON sidecar is written by default
  - manifest SQLite no longer leaves a manifest-sized WAL sidecar behind
  - truth-pack SQLite still lands successfully
  - the full Austin-to-Lua loop now fits within the bounded non-satellite `tertiary` proof lane when regenerable artifacts are cleaned first
- Remaining DX cleanup from this proof:
  - the refresh scripts were still labeling the source as `austin-manifest.json` even when they loaded from SQLite
  - that wording is now fixed in the repo so future runs will report the real source path
- Operator note:
  - `tertiary` also now has an attached external SSD available
  - future proof clones or heavy output roots should move onto operator-local SSD-backed paths instead of relying on the internal disk, but those paths must remain uncommitted local configuration rather than repo truth
- Verification for this slice:
  - remote `tertiary`: full `bash scripts/export_austin_to_lua.sh` completed successfully from the proof clone
  - local-safe: `python3 -m unittest scripts.tests.test_refresh_preview_from_sample_data scripts.tests.test_refresh_runtime_harness_from_sample_data -v`
  - local-safe: `git diff --check`

### 2026-04-02: Simple-Shell Exterior Openings Are Proven On `tertiary`, And The Real Focused-Lane Failure Was `vsync` Path Drift

- I kept the next building-readability tranche bounded and shared:
  - `BuildingBuilder.lua` now adds sparse `SimpleShellDoorCue` and `SimpleShellWindowPane` parts only on the simple low-rise shell path
  - those window panes retain `BaseTransparency` and register through the existing night-window reactive flow
- Before I could prove that tranche, the focused direct-SSH `tertiary` lane failed before Studio launch. The real root cause was not the builder change:
  - the direct command omitted `VSYNC_REPO_DIR`
  - `tertiary` did not have a usable `vsync` at the default adjacent sibling path
  - the only healthy `vsync` source on that machine was the operator-owned stage at `$HOME/.codex-remote-studio/vertigo-sync`
- I reran the focused proof from the SSD-backed proof clone with the explicit operator path:
  - `ssh tertiary 'cd /Volumes/APDataStore/arnis-roblox-proof && VSYNC_REPO_DIR=$HOME/.codex-remote-studio/vertigo-sync bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter SimpleResidentialShell.spec.lua --edit-wait 30 --pattern-wait 120'`
- That produced the first real product-side red/green cycle for this tranche:
  - first red: the spec was still asserting named `CornerAccent` parts in the merged shell-mesh path
  - root cause: merged accumulator detail preserves the authoritative `ArnisCornerAccentCount` attribute, but not per-corner part names
  - fix: remove the stale named-part assertion and keep the stronger attribute-level contract plus the new door/window/reactive assertions
  - second run: `PASS SimpleResidentialShell.spec`
- Additional measured truth from the successful proof run:
  - preview remained the same bounded downtown sample
  - hotspot still centered on chunk `-1_0`
  - slow-chunk totals were roughly unchanged (`slow_chunk_total_ms=238`, `slow_chunk_buildings_ms=236`, `slow_chunk_building_terrain_fill_ms=197`)
  - `slow_chunk_building_facade_detail_ms` stayed `0`, which is expected because this tranche adds bounded simple-shell readability cues, not office-style facade-band work
- Cleanup outcome:
  - the successful harness run exited `0`
  - `quit_studio` still needed TERM/KILL escalation during shutdown
  - after cleanup, `tertiary` had no lingering `run_studio_harness.sh`, `rbx-studio-mcp`, `vsync serve`, or `RobloxStudio` processes
- Repo-truth consequences:
  - the direct SSH operator doc now states that direct remote proof must supply `VSYNC_REPO_DIR` (or `VSYNC_BIN`) explicitly
  - this was operator path drift, not a reason to add another committed machine-specific fallback
- Verification for this slice:
  - remote `tertiary`: focused `SimpleResidentialShell.spec.lua` pass from `/Volumes/APDataStore/arnis-roblox-proof` with explicit `VSYNC_REPO_DIR=$HOME/.codex-remote-studio/vertigo-sync`

### 2026-04-02: Landuse Surface Semantics No Longer Collapse `education` And Richer Urban Materials In The Export Path

- I took the next bounded outdoor seam immediately after the simple-shell tranche: landuse semantic-material truth.
- Root cause from the red/green loop:
  - the pipeline still normalized `amenity=school|university|college` to `school` instead of the richer canonical `education`
  - the exporter still treated `Brick`, `Limestone`, and `Slate` like low-priority unknown terrain materials, so explicit landuse surfaces could lose to the default terrain fill even after their manifest `material` was richer
- Minimal fixes landed in the clean worktree:
  - `arbx_pipeline` now normalizes school-like amenities to `education`
  - `arbx_roblox_export` now maps `pitch`, `golf_course`, `education`, `religious`, `retail`, `hospital`, and `railway` to richer manifest/terrain materials
  - terrain-material priority now lets explicit `Brick`, `Limestone`, and `Slate` overrides beat the default base terrain
  - `LanduseBuilder.lua` now carries `pitch` and `golf_course` in the Roblox fallback table so the downstream fallback vocabulary matches the Rust side
- New local-safe contract coverage:
  - `emit_area_way_normalizes_school_amenity_to_education_landuse`
  - `export_preserves_richer_landuse_material_semantics`
  - `scripts.tests.test_landuse_material_contract`
- Current interpretation after this slice:
  - one of the biggest remaining outdoor-fidelity gaps is now narrowed before Studio even runs
  - large parcels like education/religious/retail/railway landuse shells no longer need to collapse to generic `Ground`/`Concrete` semantics by default
  - the next proof step for this slice is not another harness change; it is a regenerated Austin export on `tertiary` from the SSD-backed proof clone so the new semantics flow into the real bounded sample
- Verification for this slice:
  - local-safe red: `cargo test --manifest-path rust/Cargo.toml -p arbx_pipeline emit_area_way_normalizes_school_amenity_to_education_landuse -- --nocapture`
  - local-safe red: `cargo test --manifest-path rust/Cargo.toml -p arbx_roblox_export export_preserves_richer_landuse_material_semantics -- --nocapture`
  - local-safe green: same two cargo tests after the fixes
  - local-safe green: `python3 -m unittest scripts.tests.test_landuse_material_contract -v`

### 2026-04-02: Runtime-Only Austin Export Proves Python Lua Emission Is The Wrong Layer For Planetary Streaming

- I took the next product-path streaming slice on `main` instead of another Studio loop:
  - bounded derivative selection landed first (`--emit runtime`)
  - bounded canonical runtime fragmentation landed next
  - then I replaced the repeated whole-slice Lua reserialization inside `chunk_fragmentation.py` with a shared linear per-item packer and proved it locally with a dedicated regression test
- The fresh SSD-backed `tertiary` runtime-only Austin measurement is now the decisive datapoint:
  - repo tip: `main@5a66984a`
  - command: `bash scripts/export_austin_to_lua.sh --emit runtime`
  - Rust rebuild: `23.17s`
  - SQLite manifest store write: `20.000396125s`
  - runtime-only wall time: `698.15s`
  - runtime shard modules written: `35881`
  - `/usr/bin/time -lp` maximum resident set size: `2791407616`
- Interpretation:
  - canonical compile/truth-pack is not the long pole
  - Python runtime Lua emission is the long pole
  - runtime file fanout is now itself a first-class product problem, not just a byproduct of the packer
  - this is the wrong layer for a real Roblox planetary-streaming path even after bounded fragmentation work
- Repo-truth consequence:
  - the next streaming tranche should lower runtime shard planning/emission into Rust, where we can use exact serializers, bounded-memory row iteration from SQLite, and real parallelism
  - Python should remain orchestration glue for the bounded sample workflow, not the hot path for tens of thousands of runtime shard files
- Verification for this slice:
  - local-safe green: `python3 -m unittest scripts.tests.test_chunk_fragmentation scripts.tests.test_json_manifest_to_sharded_lua scripts.tests.test_refresh_preview_from_sample_data scripts.tests.test_refresh_runtime_harness_from_sample_data scripts.tests.test_austin_fidelity -v`
  - local-safe green via pre-push gate: `389` tests passed, `1` skipped
  - local-safe green: `bash -n scripts/export_austin_to_lua.sh`
  - local-safe green: `git diff --check`
  - remote `tertiary`: SSD-backed runtime-only `bash scripts/export_austin_to_lua.sh --emit runtime` completed successfully from `/Volumes/APDataStore/arnis-roblox-proof`

### 2026-04-02: Runtime Packaging Needs A Roblox Runtime Budget, Not The Preview VertigoSync Budget

- I continued the product-path runtime packaging work on `main` in two bounded steps:
  - first, I fixed a correctness bug in the Rust runtime emitter so shard modules now actually respect `--max-bytes` instead of only fragmenting chunk payloads under that cap
  - then I measured that stricter runtime path on `tertiary` and confirmed it is the wrong default for real runtime packaging
  - finally, I decoupled runtime shard sizing from the preview/VertigoSync byte budget in `scripts/export_austin_to_lua.sh`, leaving preview bounded but making runtime uncapped by default unless `AUSTIN_RUNTIME_SHARD_MAX_BYTES` is explicitly set
- Fresh `tertiary` measurements on the SSD-backed proof clone now show the full progression:
  - `main@5a66984a` with Python runtime emission: `698.15s`, `35881` runtime shard modules, RSS `2791407616`
  - `main@b45da471` with byte-correct Rust runtime emission still inheriting the preview cap: `576.73s`, `9305` runtime shard modules, RSS `2465284096`
  - `main@102f25e2` with Rust runtime emission and no default runtime byte cap: `481.05s`, `925` runtime shard modules, RSS `2399109120`
- The current no-cap runtime packaging shape on `tertiary` is:
  - runtime shard count: `925`
  - smallest shard module: `845483` bytes
  - `p95` shard module: `1713814` bytes
  - largest shard module: `1987856` bytes
- Interpretation:
  - the preview/VertigoSync source ceiling is a bad runtime default
  - moving runtime emission into Rust was necessary, but not sufficient by itself; the packaging contract matters just as much as the implementation language
  - the new default is the first runtime packaging shape that materially moves the bounded Austin sample toward a planetary-streaming-friendly deployment path
  - the remaining open question is not whether the capped preview budget should remain out of runtime; it should. The next question is what explicit Roblox runtime shard budget or bundle format should replace the current uncapped sample default before wider deployment
- Repo-truth consequence:
  - runtime and preview now have intentionally separate packaging concerns
  - the next streaming tranche should optimize for Roblox runtime deployment directly: either a larger explicit runtime shard ceiling or a deeper runtime bundle format, rather than reusing preview/plugin assumptions
- Verification for this slice:
  - local-safe red: `python3 -m unittest scripts.tests.test_austin_fidelity.AustinFidelityScriptTests.test_export_to_lua_documents_bounded_dev_profile_default -v`
  - local-safe green: same focused test after the shell-contract fix
  - local-safe green: `python3 -m unittest scripts.tests.test_austin_fidelity -v`
  - local-safe green: `cargo test --manifest-path rust/Cargo.toml -p arbx_cli emit_runtime_lua_keeps_each_shard_within_max_bytes -- --nocapture`
  - local-safe green: `cargo test --manifest-path rust/Cargo.toml -p arbx_cli -- --nocapture`
  - local-safe green: `cargo test --manifest-path rust/Cargo.toml -p arbx_roblox_export -- --nocapture`
  - local-safe green: `bash -n scripts/export_austin_to_lua.sh`
  - local-safe green: `git diff --check`
  - remote `tertiary`: SSD-backed runtime-only `bash scripts/export_austin_to_lua.sh --emit runtime` completed successfully from `/Volumes/APDataStore/arnis-roblox-proof` at both the byte-correct capped and uncapped runtime defaults

### 2026-04-02: Runtime Packaging Sweep Picks 16 Chunks Per Shard As The New Conservative Default

- I kept the work on the real product path and measured the runtime packer directly against the already-built Austin SQLite on `tertiary`, avoiding another full compile for every trial.
- Direct SSD-backed `emit-runtime-lua` sweep from `/Volumes/APDataStore/arnis-roblox-proof/rust/out/austin-manifest.sqlite`:
  - `chunks_per_shard=8`: `925` shards, emitter-only `228.18s`, max shard `1987856` bytes
  - `chunks_per_shard=16`: `463` shards, emitter-only `227.94s`, max shard `3685839` bytes
  - `chunks_per_shard=32`: `232` shards, emitter-only `210.31s`, max shard `6634433` bytes
- Decision:
  - `32` is probably too aggressive as the default without a proven Roblox runtime module-size contract
  - `16` is the right conservative next default: it cuts runtime shard fanout almost in half again with effectively unchanged emitter time, while keeping the largest observed shard well below the `32`-chunk extreme
- Repo-truth consequence:
  - `scripts/export_austin_to_lua.sh` now defaults runtime packaging to `AUSTIN_RUNTIME_CHUNKS_PER_SHARD=16`
  - preview packaging remains bounded for Vertigo Sync concerns
  - runtime packaging now follows a separate, deployment-oriented contract
- Verification for this slice:
  - local-safe red: `python3 -m unittest scripts.tests.test_austin_fidelity.AustinFidelityScriptTests.test_export_to_lua_documents_bounded_dev_profile_default -v`
  - local-safe green: same focused test after the shell default change
  - local-safe green: `python3 -m unittest scripts.tests.test_austin_fidelity -v`
  - local-safe green: `bash -n scripts/export_austin_to_lua.sh`
  - local-safe green: `git diff --check`
  - remote `tertiary`: direct `emit-runtime-lua` sweep at `8`, `16`, and `32` chunks per shard from the same Austin SQLite manifest

### 2026-04-02: First Runtime Streaming Engine Budget Contract Landed Locally

- I moved the first runtime-engine slice out of design-only territory and into the shared runtime path on `main`.
- `WorldConfig.lua` now declares explicit `StreamingRings` for `near`, `mid`, and `far`, each with:
  - `MaxRadiusStuds`
  - `EstimatedBudgetBytes`
  - `MaxChunkCount`
- `StreamingService.lua` now resolves those rings through one `resolveStreamingRings(config)` helper and uses them to:
  - classify desired residency by `near`/`mid`/`far`
  - treat `EstimatedBudgetBytes` as the authoritative per-ring residency budget
  - treat `MaxChunkCount` as a secondary ring guardrail
  - surface ring-level resident chunk counts and resident estimated costs
  - surface queued estimated cost and queued work-item count
  - surface explicit prefetch and eviction reasons on the existing loader path
- `scripts/run_studio_harness.sh` now threads those new streaming telemetry fields into the existing play probe payload so the remote proof lane can capture them without inventing a parallel observability path.
- This slice intentionally does **not** add a second scheduler or a legacy toggle. It hardens the current runtime loader into a budget-aware contract that later movement-aware prefetch can build on.
- Current local runtime-engine contract:
  - `near`: up to `1024` studs, `1536 MiB`, `64` chunks
  - `mid`: up to `1536` studs, `1536 MiB`, `96` chunks
  - `far`: up to `2048` studs, `1024 MiB`, `128` chunks
  - `production_server` widens those ring radii and budgets without changing the contract shape
- The measured Austin runtime packaging baseline this sits on is still the current `16`-chunks-per-runtime-shard path:
  - `463` runtime shards
  - emitter-only `227.94s`
  - largest shard `3685839` bytes
- Interpretation:
  - the runtime engine now has explicit residency math instead of only implicit radius behavior
  - packaging and scheduling are starting to separate cleanly
  - the next runtime-engine slice should stay on the product path: movement-aware prefetch, eviction ordering, and ring-value heuristics on top of this contract
- Verification for this slice:
  - local-safe red: `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_play_render_truth scripts.tests.test_run_studio_harness -v`
  - local-safe green: same focused suite after the runtime-engine implementation
  - local-safe green: `git diff --check`
  - remote `tertiary`: still pending for this specific runtime-engine telemetry slice

### 2026-04-02: Movement-Lookahead Scheduler Slice Landed On Top Of The Ring Budget Contract

- I kept the next runtime-engine slice on the single shared streaming path instead of adding a second scheduler.
- `StreamingService.lua` now computes a predicted focal point from observed movement, a bounded lookahead contract, and the existing ring-budget loader cadence.
- `WorldConfig.lua` now exposes the movement-lookahead knobs directly:
  - `StreamingLookaheadSeconds`
  - `StreamingMaxLookaheadStuds`
- The scheduler now:
  - sorts candidate chunks and work items against the predicted focal point instead of only the instantaneous player position
  - emits `movement_lookahead` as a first-class prefetch reason when the prediction path is active
  - publishes the extra runtime telemetry needed to inspect the scheduler instead of guessing:
    - `ArnisStreamingPredictedFocalX`
    - `ArnisStreamingPredictedFocalZ`
    - `ArnisStreamingMovementDeltaStuds`
    - `ArnisStreamingMovementLookaheadStuds`
- I also fixed the two real `tertiary` blockers that were preventing Austin-specific proof from starting cleanly:
  - the harness now resolves `vertigo-sync` outside sibling clones
  - the resolver accepts both source repos and binary-only install roots, which matches the current `tertiary` layout
- `tertiary` proof status for this slice:
  - green: direct SSD-backed clean-place build from `/Volumes/APDataStore/arnis-roblox-proof-clean`
  - measured artifact: `roblox/out/arnis-test-clean-play.rbxlx` built successfully at about `15 MB`
  - still pending: the focused Studio proof that exercises the new `movement_lookahead` runtime behavior end-to-end
- Verification for the landed local slice:
  - local-safe red: `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_play_render_truth -v`
  - local-safe green: `python3 -m unittest scripts.tests.test_run_studio_harness scripts.tests.test_austin_runtime_contract scripts.tests.test_play_render_truth -v`
  - local-safe green: `bash -n scripts/run_studio_harness.sh`
  - local-safe green: `git diff --check`
- Interpretation:
  - the runtime engine now has both budgeted residency math and a bounded predictive prefetch contract
  - Austin clean-place preparation is no longer blocked on SSD clone layout drift
  - the next remote proof should be narrow: prove the scheduler telemetry and then move back to play-fidelity/streaming value work instead of widening the harness again

### 2026-04-02: Focused Tertiary Proof Burned Down Harness Corruption And Binary-Root Drift

- I kept this slice narrow and used it only to unblock the already-landed movement-lookahead runtime proof on `tertiary`.
- Two real harness correctness bugs are now fixed on `main`:
  - `set_runall_config_modes()` no longer depends on exact text matches and now rewrites `RunAllConfig.lua` by field name
  - both `set_runall_config_modes()` and `set_runall_config_filter()` now write atomically, so an interrupted harness run cannot leave `RunAllConfig.lua` truncated to zero bytes
- I also fixed the second `tertiary`-specific launcher bug:
  - `resolve_vsync_binary()` now honors binary-only install roots at `VSYNC_REPO_DIR/target/debug/vsync`
  - this matches the current `tertiary` layout, where `vertigo-sync` is available as a built binary under `~/Projects/vertigo-sync` without relying on a sibling source checkout
- Remote `tertiary` proof consequences:
  - green: the focused harness run now gets past the old `RunAllConfig edit-mode toggle field not found` startup failure
  - green: the proof clone no longer leaves `RunAllConfig.lua` corrupted after failed runs
  - green: the focused harness run now gets through clean-place build, Vertigo Sync server startup, Studio open, and `ARNIS_MCP_READY`
  - remaining blocker: `edit_sync` readiness still times out on the isolated `StreamingPriority.spec.lua` proof path even after one built-in recovery pass
  - after that timeout, the harness now correctly falls back to `RunAllEntry` for the isolated spec, but the current machine/log path still does not emit the expected `Filtering tests to spec:` / `Running tests:` / `TestEZ tests complete` markers before timeout
- This means the active remote blocker has been narrowed substantially:
  - it is no longer clean-place build drift
  - it is no longer sibling-clone vsync resolution
  - it is no longer binary-only vsync root resolution
  - it is no longer `RunAllConfig` truncation/corruption
  - it is now specifically the `edit_sync` readiness contract and/or the isolated-spec completion marker path on `tertiary`
- Verification for this slice:
  - local-safe red then green: `python3 -m unittest scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_harness_defaults_to_clean_preview_without_edit_mode_runall -v`
  - local-safe red then green: `python3 -m unittest scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_vsync_repo_ownership_survives_prebuilt_binary_override -v`
  - local-safe green: `python3 -m unittest scripts.tests.test_run_studio_harness scripts.tests.test_austin_runtime_contract scripts.tests.test_play_render_truth -v`
  - local-safe green: `bash -n scripts/run_studio_harness.sh`
  - local-safe green: `git diff --check`
  - remote `tertiary`: focused `StreamingPriority.spec.lua` harness run reached clean-place build, live Vertigo Sync serve, Studio open, and `ARNIS_MCP_READY`, then exposed the remaining `edit_sync`/RunAll marker blocker above

### 2026-04-02: Isolated StreamingPriority Proof Exposed A Real Whole-Chunk Scheduling Regression

- I kept the next slice on the real product path instead of widening the harness again.
- The focused isolated-spec proof on `tertiary` is now meaningfully better:
  - green: isolated non-preview edit specs can bypass the old `edit_sync` readiness gate when the harness is run with `--no-play`
  - green: the MCP path now runs `StreamingPriority.spec.lua` end-to-end and surfaces real `RunAll` failures instead of timing out before test execution
- That proof exposed a real runtime scheduler regression, not a harness problem:
  - the guardrail section in `StreamingPriority.spec.lua` imported `guardrail_deferred` before `guardrail_anchor`
  - that meant whole-chunk work was being reshuffled after chunk prioritization, so the memory guardrail paused around the wrong first admission
- I fixed that at the scheduling layer:
  - `ChunkPriority.lua` now preserves already-prioritized source order for whole-chunk work items from different chunks
  - `ChunkSubplanPriority.spec.lua` now includes a regression check that whole-chunk work preserves chunk scheduler admission order
- Current remote proof status after the product fix:
  - partial green: the earlier proof now gets through MCP execution and identifies the real scheduler failure correctly
  - remaining blocker: a follow-on `tertiary` rerun hit an intermittent MCP relay/proxy stall (`http://localhost:44755/request` returning repeated `423 Locked` before `ARNIS_MCP_READY`), so I do not yet have a fresh remote-green post-fix `StreamingPriority.spec.lua` run
- Verification for the landed local slice:
  - local-safe red then green: `python3 -m unittest scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_isolated_non_preview_specs_skip_edit_sync_gate -v`
  - local-safe green: `python3 -m unittest scripts.tests.test_run_studio_harness scripts.tests.test_austin_runtime_contract scripts.tests.test_play_render_truth -v`
  - local-safe green: `bash -n scripts/run_studio_harness.sh`
  - local-safe green: `git diff --check`
- Interpretation:
  - the harness unblock has now paid off by finding a real streaming-engine bug
  - the product fix is in with a regression guard
  - the next proof step is to burn down the intermittent MCP `423 Locked` path just enough to re-run the isolated `StreamingPriority.spec.lua` slice and confirm the scheduler is green remotely

### 2026-04-02: Isolated Tertiary Proof No Longer Depends On The MCP Relay, But RunAllEntry Still Fails To Prove Completion

- I kept this slice narrow and only changed the isolated edit-only proof path that was still burning time on `tertiary`.
- The harness is now materially tighter for isolated non-preview edit proofs:
  - it no longer forces proxy-backed MCP env into the isolated readiness/edit-action path
  - it no longer starts the local Studio MCP sidecar for `--no-play --spec-filter <non-preview>`
  - it no longer burns the full MCP readiness wait on that path
  - it now preconfigures `RunAllEntry` before Studio launch for that isolated proof shape instead of trying to toggle it after Studio has already booted
- Remote `tertiary` proof consequences:
  - green: the old repeated `http://localhost:44755/request` `423 Locked` relay contention is gone
  - green: there is no `rbx-studio-mcp --stdio` sidecar process in the isolated proof run anymore
  - green: the harness now reaches the isolated `RunAllEntry` fallback deterministically instead of hanging behind proxy churn
  - remaining blocker: even with `RunAllConfig.lua` pre-armed before launch, the isolated `StreamingPriority.spec.lua` proof still does not emit `Filtering tests to spec:` / `Running tests:` / `TestEZ tests complete` markers on `tertiary`
  - the live Studio log now shows only repeated `CURLINFO_OS_ERRNO: 61 Connection refused` from the MCP plugin polling `localhost:44755`; that noise no longer reflects the harness control path, but the isolated proof is still not self-reporting completion through `RunAllEntry`
- Verification for this slice:
  - local-safe green: `python3 -m unittest scripts.tests.test_run_studio_harness scripts.tests.test_austin_runtime_contract scripts.tests.test_play_render_truth -v`
  - local-safe green: `bash -n scripts/run_studio_harness.sh`
  - local-safe green: `git diff --check`
  - remote `tertiary`: isolated `StreamingPriority.spec.lua` proof now skips sidecar and MCP-ready wait, reaches `RunAllEntry` fallback, and then times out specifically on missing completion markers
- Interpretation:
  - the relay/proxy blocker is no longer the reason the isolated proof is failing
  - the next targeted fix should focus on how isolated edit proofs are triggered and observed in Studio, not on reviving the old localhost relay

### 2026-04-02: Clean Tertiary Proof Turned StreamingPriority Green Again

- I finished the isolated `StreamingPriority.spec.lua` burndown from a clean remote proof clone instead of chasing stale remote dirt.
- The actual fixes that mattered were:
  - `StreamingService.lua`
    - ring byte budgets are now planner targets, not a hard admission gate ahead of the authoritative memory guardrail
    - streaming telemetry no longer assumes `streamingOptions.worldRootName` is always present when publishing loaded chunk counts
  - `RunAll.lua`
    - duplicate concurrent suite entry now skips cleanly when `ArnisRunAllSuiteActive` is already true
  - `RunAll.spec.lua`
    - regression coverage now proves duplicate `RunAll.run()` calls skip instead of re-entering the suite
  - `run_studio_harness.sh`, `studio_ui_control.py`, and their Python tests
    - the isolated `tertiary` play proof lane is now deterministic enough to expose real product failures instead of launcher noise
- The clean remote proof run on `tertiary` is now green:
  - one real `Running tests: StreamingPriority.spec`
  - one explicit `Skipping duplicate RunAll invocation while suite is already active`
  - `PASS StreamingPriority.spec`
  - `TestEZ tests complete. total=1 passed=1 failed=0`
  - harness flow completed cleanly from the isolated play-only proof lane
- Verification for this slice:
  - local-safe green: `python3 -m unittest scripts.tests.test_run_studio_harness scripts.tests.test_studio_ui_control scripts.tests.test_runall_filter -v`
  - local-safe green: `bash -n scripts/run_studio_harness.sh`
  - local-safe green: `git diff --check`
  - remote `tertiary` green: isolated `StreamingPriority.spec.lua` proof from a hard-reset clean clone on pushed `main`
- Interpretation:
  - the focused proof lane is trustworthy again for streaming scheduler work
  - the memory guardrail remains the authoritative admission stop line
  - ring byte budgets now behave like scheduler targets instead of silently starving in-ring work ahead of the real guardrail

### 2026-04-02: Streaming Service Now Publishes Eviction Telemetry And Resets Stale Recovery Timing State

- I kept this slice on the streaming engine instead of widening the harness again.
- The first gap was scheduler observability:
  - the runtime already published queue depth, per-ring desired/resident counts, and last prefetch/eviction reasons
  - but it did not expose how much estimated resident cost or how many chunks were actually shed in a given update
  - I fixed that in `StreamingService.lua` by publishing:
    - `ArnisStreamingEvictedEstimatedCost`
    - `ArnisStreamingEvictedChunkCount`
  - the first post-pause unload path also now preserves the concrete `outside_target_radius` eviction reason instead of collapsing back to a generic not-desired bucket
- The second gap was a real stale-recovery scheduler bug exposed by the new isolated proof:
  - same-session stale chunk recovery correctly cleared resident cost and completed subplan state
  - but it still retained the chunk's observed per-subplan import timings
  - that let the work-item sorter reshuffle sibling subplans on rebuild instead of restarting from the first sibling deterministically
  - I fixed that by clearing observed import-cost history for stale/rebuilt chunks at the same time stale resident cost and subplan state are cleared
- Regression coverage now proves:
  - the runtime contract text includes the new eviction telemetry attributes
  - same-session stale chunk recovery reimports both sibling subplans from the beginning after a destroyed chunk folder
  - the isolated `StreamingPriority.spec.lua` proof stays green on `tertiary`
- Verification for this slice:
  - local-safe green: `python3 -m unittest scripts.tests.test_austin_runtime_contract -v`
  - local-safe green: `git diff --check`
  - remote `tertiary` green:
    - `Running tests: StreamingPriority.spec`
    - `PASS StreamingPriority.spec`
    - `TestEZ tests complete. total=1 passed=1 failed=0`
- Interpretation:
  - per-update eviction pressure is now observable directly from the runtime state machine
  - stale same-session recovery now behaves like a true fresh sibling-subplan restart instead of carrying hidden adaptive timing residue across a destroyed chunk
  - this is a better foundation for bounded residency and forward-prefetch policy work on the Austin sample

### 2026-04-02: Movement Lookahead Now Expands Scheduler Eligibility, Not Just Sort Order

- I kept this slice on the real runtime scheduler instead of widening the harness again.
- The root cause was concrete:
  - movement lookahead already biased chunk sorting toward the predicted focal point
  - but candidate discovery and ring/LOD eligibility still used only the current focal position
  - that meant ahead-of-motion chunks just outside the current target radius could never become eligible, even when the bounded predicted focus placed them squarely inside the next streaming window
- I fixed that in `StreamingService.lua` by:
  - unioning candidate discovery across the current focal point and the bounded predicted focal point
  - using the better of actual distance and predicted-focus distance when deciding chunk eligibility and ring membership
  - leaving resident telemetry tied to the real player position and leaving the memory guardrail as the hard admission stop line
- I added a new regression in `StreamingPriority.spec.lua` that proves:
  - the anchor chunk imports first
  - a chunk slightly outside the current focal radius but inside the lookahead-expanded window becomes eligible on the next movement update
  - the prefetch reason remains `movement_lookahead`
- Verification for this slice:
  - local-safe red then green: isolated `StreamingPriority.spec.lua` on `tertiary`
  - local-safe green: `python3 -m unittest scripts.tests.test_austin_runtime_contract -v`
  - local-safe green: `git diff --check`
  - remote `tertiary` green:
    - `Running tests: StreamingPriority.spec`
    - `PASS StreamingPriority.spec`
    - `TestEZ tests complete. total=1 passed=1 failed=0`
- Interpretation:
  - movement lookahead is now a real residency/prefetch contract instead of just a tie-breaker inside the current radius
  - this is a better planetary-streaming shape because the runtime can start pulling ahead-of-motion chunks before the camera/player reaches the old focal boundary

### 2026-04-02: Ring Pressure Now Breaks Equal-Value Ties By Estimated Memory Cost

- I kept the next slice on runtime scheduler policy rather than widening harness behavior.
- The concrete gap after the lookahead work was:
  - chunk ordering already considered distance band, heading, observed import cost, and `streamingCost`
  - but under ring pressure it still ignored `estimatedMemoryCost`, even though memory is the authoritative residency constraint
  - with equal distance, heading, feature count, and streaming cost, the scheduler could still keep the heavier chunk just because its id sorted earlier
- I fixed that in `ChunkPriority.lua`:
  - chunk-level and subplan-level metrics now carry `estimatedMemoryCost`
  - when higher-value signals tie, lower estimated memory cost now wins before the older `streamingCost` fallback
  - absent explicit `estimatedMemoryCost`, the priority code falls back to the existing `streamingCost` heuristic so old behavior stays stable where no memory hint exists
- I added regression coverage in:
  - `StreamingPriority.spec.lua` proving a one-slot ring prefers the lower estimated-memory chunk under otherwise equal scheduler signals
  - `ChunkSubplanPriority.spec.lua` proving same-layer sibling subplans use estimated memory cost before lexical ids when value signals tie
- Verification for this slice:
  - local-safe green: `python3 -m unittest scripts.tests.test_austin_runtime_contract -v`
  - local-safe green: `git diff --check`
  - remote `tertiary` green:
    - `Running tests: StreamingPriority.spec`
    - `PASS StreamingPriority.spec`
    - `TestEZ tests complete. total=1 passed=1 failed=0`
- Interpretation:
  - the scheduler now behaves more like a real memory-bounded streaming engine instead of a distance-plus-id sorter
  - this should make bounded residency decisions more stable as we push toward larger Austin slices and planetary streaming budgets

### 2026-04-02: Play Bootstrap Now Reuses Shared World State Instead Of A Divergent Lighting Fork

- I pivoted briefly from streaming policy to a higher-signal play-vs-edit parity issue reported from the real Austin sample:
  - play mode looked materially worse than edit mode
  - the bootstrap path was still hand-authoring its own lighting/post-processing stack
  - preview/full-bake already went through `WorldStateApplier`, which also owns canonical `ArnisWorldRootName` publication
- I added a new local contract in `scripts/tests/test_play_render_truth.py` that required:
  - `BootstrapAustin.server.lua` to require `WorldStateApplier`
  - play bootstrap to call `WorldStateApplier.Apply(...)` with `worldRootName = "GeneratedWorld_Austin"`
  - the old direct `Atmosphere` / `BloomEffect` / `SunRaysEffect` / `ColorCorrectionEffect` instantiation to disappear from the bootstrap script
- I then updated `BootstrapAustin.server.lua` so play mode now:
  - applies shared world state through `WorldStateApplier`
  - publishes the canonical world-root contract through the same path as preview/full-bake
  - starts minimap/loading-screen lifecycle through the shared applier instead of the old one-off bootstrap block
- Verification for this slice:
  - local-safe red then green: `python3 -m unittest scripts.tests.test_play_render_truth.PlayRenderTruthTests.test_play_bootstrap_reuses_shared_world_state_application -v`
  - local-safe green: `python3 -m unittest scripts.tests.test_play_render_truth -v`
  - local-safe green: `python3 -m unittest scripts.tests.test_austin_runtime_contract -v`
  - local-safe green: `git diff --check`
- Follow-up:
  - the deeper shell-wall / false-plane parity investigation is preserved separately in `git stash` as `pressure-replacement-and-play-parity-wip-2026-04-02`
  - I did not claim the visual issue fully solved yet; this slice removes one confirmed play-only divergence so the next runtime geometry investigation starts from a cleaner baseline

### 2026-04-02: Isolated Roomed Shell Streaming Parity Is Green In Play Mode

- I stopped widening the generic harness/debug loop and wrote a direct runtime parity proof for the user-reported play issue instead:
  - `PlayStreamingRoomShellParity.spec.lua`
- The spec builds the smallest high-signal reproduction:
  - one roomed `shellMesh` building elevated above flat terrain
  - startup import followed by `StreamingService.Start(...)` and one runtime update
  - assertions that visible shell wall evidence remains present
  - assertions that the room interior voxel remains `Air` with no floating terrain fill
- Remote proof on clean `tertiary` from the isolated play harness is green:
  - `Running tests: PlayStreamingRoomShellParity.spec`
  - `PASS PlayStreamingRoomShellParity.spec`
  - `TestEZ tests complete. total=1 passed=1 failed=0`
- Interpretation:
  - the broad claim that play-mode streaming startup always drops shell walls or reintroduces floating shell terrain fill is now disproven at the isolated runtime contract level
  - the remaining “false planes over terrain / walls missing in play” report is therefore narrower and Austin-sample-specific, not a generic roomed-shell streaming-startup failure

### 2026-04-02: Harness Play Probe Now Respects Failed MCP Readiness Instead Of Burning 100 Seconds

- While trying to pull back a real Austin play screenshot through the harness artifact path, I found a concrete DX regression in the current harness flow:
  - after the harness already logged `Studio MCP helper did not become ready ...`, the play path still entered `run_play_probe_via_mcp()`
  - that wasted about 100 seconds before falling back to the direct play trigger, delaying screenshot/audit collection without improving proof quality
- I added a direct regression in `scripts/tests/test_run_studio_harness.py` that requires `run_play_probe_via_mcp()` to gate on `MCP_READY`.
- I then updated `scripts/run_studio_harness.sh` so the authoritative MCP play probe now returns immediately when the helper is not actually ready, instead of burning the fallback timeout.
- Verification for this slice:
  - local-safe green: `python3 -m unittest scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_play_probe_requires_ready_mcp_helper_before_authoritative_probe -v`
  - local-safe green: `bash -n scripts/run_studio_harness.sh`
  - local-safe green: `git diff --check`
- Follow-up:
  - rerun the Austin play proof through the harness screenshot path after this lands on `main`
  - use the returned `...-play.png` and scene-audit artifacts to pin the real false-surface / missing-wall Austin parity bug

### 2026-04-03: Remote Wrapper Now Seeds Scene Index Inputs For Staged Proof Runs

- I stayed on harness DX rather than widening runtime behavior because the current remote blocker had narrowed to staged-proof ergonomics:
  - the remote play flow on `tertiary` was already reaching `PlaySoloSuccess` and Austin `gameplay_ready`
  - but staged clones under `~/.codex-remote-studio/arnis-roblox` had no ignored `rust/out` artifacts, so `run_scene_fidelity_audits()` could only log `scene fidelity audit unavailable`
  - the wrapper also only synced back logs, screenshot attempts, and preview/plugin artifacts, not scene-audit outputs
- I tightened that in two places:
  - `scripts/run_studio_harness.sh` now accepts a precomputed `rust/out/austin-manifest.scene-index.json` as sufficient audit input and only requires raw manifest/sqlite outputs when it actually needs to regenerate the summary
  - `scripts/run_studio_harness_remote.sh` now seeds that summary into the staged clone from either:
    - the local workspace, when `rust/out/austin-manifest.scene-index.json` exists locally
    - or the configured remote base clone, when the staged sync is clean but the operator-owned remote proof clone already has the summary
  - the remote wrapper now also syncs `/tmp/arnis-scene-fidelity-*.json`, `/tmp/arnis-scene-fidelity-*.html`, and `/tmp/arnis-scene-parity.*` back into the local artifact directory when those artifacts exist
- Regression coverage added:
  - `scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_scene_fidelity_audits_can_run_from_seeded_manifest_summary_without_raw_outputs`
  - `scripts.tests.test_run_studio_harness_remote.RunStudioHarnessRemoteTests.test_seeds_manifest_summary_and_fetches_scene_audit_artifacts`
- Verification for this slice:
  - local-safe red then green:
    - `python3 -m unittest scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_scene_fidelity_audits_can_run_from_seeded_manifest_summary_without_raw_outputs -v`
    - `python3 -m unittest scripts.tests.test_run_studio_harness_remote.RunStudioHarnessRemoteTests.test_seeds_manifest_summary_and_fetches_scene_audit_artifacts -v`
  - local-safe green:
    - `python3 -m unittest scripts.tests.test_run_studio_harness scripts.tests.test_run_studio_harness_remote -v`
    - `bash -n scripts/run_studio_harness.sh scripts/run_studio_harness_remote.sh`
    - `git diff --check`
  - remote `tertiary` proof with no local `rust/out` summary:
    - invoked wrapper with `ARNIS_REMOTE_STUDIO_BASE_ARNIS=/Volumes/APDataStore/arnis-roblox-proof` so the staged clone could borrow the existing remote scene index
    - observed `[harness] using precomputed manifest scene index: /Users/adpena/.codex-remote-studio/arnis-roblox/rust/out/austin-manifest.scene-index.json`
    - observed Austin play proof still reaching `gameplay_ready` with valid client bootstrap/world/minimap/local-experience verdicts
- Remaining gap after this slice:
  - screenshot capture on `tertiary` still fails at the machine/display layer with `could not create image from display`
  - this specific play-focused proof did not leave `/tmp/arnis-scene-fidelity-*.json` artifacts behind, so the next audit gap is no longer missing staged manifest inputs; it is whether the selected play lane is emitting `ARNIS_SCENE_PLAY` markers/artifacts at all

### 2026-04-03: Proof-First Remote Sync Now Clears Stale Volatile Artifacts

- I stayed on remote-wrapper DX because the next confusing proof symptom was no longer inside Austin runtime behavior:
  - a fresh proof-first `tertiary` run synced `arnis-scene-fidelity-play.{json,html}` back locally even though the current run had only reported the MCP false-positive path and no validated live play scene marker
  - inspecting the synced files showed they predated the current run by hours, so the wrapper was telling the truth about the latest log but could still carry forward stale `/tmp` proof outputs
- I fixed that in `scripts/run_studio_harness_remote.sh` by:
  - defining the volatile proof-artifact set once
  - clearing that set inside the chosen local artifact directory before each run
  - clearing the same set on `tertiary` before launching the remote harness so proof-first sync starts from an empty remote `/tmp` surface
  - continuing to mirror remote output, sync early after authoritative proof, and bound the late quit tail after `main harness flow complete; exiting`
- Regression coverage added:
  - `scripts.tests.test_run_studio_harness_remote.RunStudioHarnessRemoteTests.test_clears_volatile_remote_tmp_artifacts_before_each_run`
- Verification for this slice:
  - local-safe red then green:
    - `python3 -m unittest scripts.tests.test_run_studio_harness_remote.RunStudioHarnessRemoteTests.test_clears_volatile_remote_tmp_artifacts_before_each_run -v`
  - local-safe green:
    - `python3 -m unittest scripts.tests.test_run_studio_harness_remote -v`
    - `bash -n scripts/run_studio_harness.sh scripts/run_studio_harness_remote.sh`
    - `git diff --check`
  - remote `tertiary` proof:
    - `ARNIS_REMOTE_STUDIO_ARTIFACT_DIR=/tmp/arnis-remote-studio-cleanproof ARNIS_REMOTE_STUDIO_BASE_ARNIS=/Volumes/APDataStore/arnis-roblox-proof bash scripts/run_studio_harness_remote.sh --remote-host tertiary -- --takeover --hard-restart --skip-edit-tests --play-wait 30 --pattern-wait 150 --screenshot /tmp/arnis-studio-harness.png`
    - remote stale `/tmp/arnis-scene-fidelity-play.*` and `/tmp/arnis-scene-parity.*` files were gone before the harness body started
    - the run still reached `play bootstrap trace verdict (authoritative client bootstrap marker): valid`
    - the run still hit the honest MCP failure `RuntimeError('run_code resolved against edit context instead of the live play session')`
    - screenshot capture still failed with `could not create image from display`
    - the wrapper still bounded the late quit tail after completion instead of hanging on cleanup
    - the fresh local artifact directory contained only the current Studio log, with no carried-over play scene-fidelity or parity artifacts
- Interpretation:
  - proof-first sync is now materially trustworthy for negative evidence as well as positive artifacts
  - the next real blocker is back where it belongs: a true live-play scene-audit lane on `tertiary`, not stale wrapper carryover

### 2026-04-03: Live Play Scene Audit Now Comes From The Runtime, Not Remote MCP

- I stopped pushing on the remote MCP boundary because the failure was stable and specific:
  - Austin still reached authoritative client `gameplay_ready` on `tertiary`
  - but the follow-on `run_code` proof step still resolved against edit context and kept emitting `generatedExists=false`
  - that meant the harness had a real play session and a false MCP session at the same time, which is the wrong place to anchor production proof
- I changed the contract instead of chasing that transport:
  - added `roblox/src/ServerScriptService/ImportService/SceneMarkerEmitter.lua` so Roblox-side runtime code can emit the full `ARNIS_SCENE_*` marker family without going through the harness-embedded Luau helper
  - updated `BootstrapAustin.server.lua` to emit `ARNIS_SCENE_PLAY` from the live Austin runtime after `gameplay_ready` using `SceneAudit.summarizeWorld(...)` plus canonical play metadata
  - updated `scripts/run_studio_harness.sh` so play scene-fidelity artifacts now build from:
    - the runtime-emitted `ARNIS_SCENE_PLAY` marker
    - the authoritative client bootstrap trace validation
  - the harness no longer requires `ARNIS_MCP_PLAY_SCENE_VALIDATED` before it can write `arnis-scene-fidelity-play.*`
- Regression coverage added:
  - `scripts.tests.test_austin_runtime_contract.AustinRuntimeContractTests.test_bootstrap_emits_authoritative_play_scene_marker_from_live_runtime`
  - updated `scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_scene_fidelity_audits_emit_edit_play_parity_when_both_reports_exist`
- Verification for this slice:
  - local-safe red then green:
    - `python3 -m unittest scripts.tests.test_austin_runtime_contract.AustinRuntimeContractTests.test_bootstrap_emits_authoritative_play_scene_marker_from_live_runtime -v`
    - `python3 -m unittest scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_scene_fidelity_audits_emit_edit_play_parity_when_both_reports_exist -v`
  - local-safe green:
    - `python3 -m unittest scripts.tests.test_run_studio_harness scripts.tests.test_austin_runtime_contract -v`
    - `bash -n scripts/run_studio_harness.sh`
    - `git diff --check`
  - remote `tertiary` proof:
    - `ARNIS_REMOTE_STUDIO_ARTIFACT_DIR=/tmp/arnis-remote-studio-runtimeplay ARNIS_REMOTE_STUDIO_BASE_ARNIS=/Volumes/APDataStore/arnis-roblox-proof bash scripts/run_studio_harness_remote.sh --remote-host tertiary -- --takeover --hard-restart --skip-edit-tests --play-wait 30 --pattern-wait 150 --screenshot /tmp/arnis-studio-harness.png`
    - the run still hit the known MCP failure: `RuntimeError('run_code resolved against edit context instead of the live play session')`
    - the same run emitted runtime `ARNIS_SCENE_PLAY` markers from `BootstrapAustin`
    - the harness logged `play bootstrap trace verdict (authoritative client bootstrap marker): valid`
    - the harness logged `writing scene fidelity play artifact`
    - the synced local artifact directory contained:
      - `/tmp/arnis-remote-studio-runtimeplay/0.715.0.7151115_20260403T145828Z_Studio_2b93a_last.log`
      - `/tmp/arnis-remote-studio-runtimeplay/arnis-scene-fidelity-play.json`
      - `/tmp/arnis-remote-studio-runtimeplay/arnis-scene-fidelity-play.html`
- Remaining gaps after this slice:
  - screenshot capture on `tertiary` still fails with `could not create image from display`
  - the remote quit path still needs bounded wrapper cleanup instead of a clean Studio-side shutdown
  - MCP may still become a useful SDK/agentic surface later, but it is no longer on the critical path for trustworthy live-play proof

### 2026-04-03: Remote Screenshot Failures Now Sync Back As Explicit Capture Diagnostics

- I pushed on the last confusing part of the `tertiary` proof loop: screenshot capture was still failing, but the old lane only gave a one-line harness log and no structured evidence about what Studio session the failure happened in.
- I changed the screenshot contract instead of inventing another wrapper path:
  - added `capture-screenshot --target ...` to `scripts/studio_ui_control.py`
  - the helper now captures the current Studio UI/session snapshot, tries window-targeted capture first when a front-window target can be resolved, falls back to display capture, and always writes a sibling `*.capture.json` file
  - updated `scripts/run_studio_harness.sh` to use that helper instead of raw `screencapture`
  - updated `scripts/run_studio_harness_remote.sh` so proof-first sync also brings back `arnis-studio-harness-*.capture.json`
  - tightened the sidecar format after remote proof to include `window_lookup_error` text, not just a numeric `window_lookup_code`
- Regression coverage added:
  - `scripts.tests.test_studio_ui_control.StudioUiControlTests.test_capture_screenshot_prefers_window_capture_then_falls_back_to_display`
  - `scripts.tests.test_studio_ui_control.StudioUiControlTests.test_capture_screenshot_records_failure_metadata_when_display_capture_fails`
  - updated `scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_screenshot_capture_uses_shared_ui_control_helper_and_is_best_effort_only`
  - updated `scripts.tests.test_run_studio_harness_remote.RunStudioHarnessRemoteTests.test_seeds_manifest_summary_and_fetches_scene_audit_artifacts`
- Verification for this slice:
  - local-safe red then green:
    - `python3 -m unittest scripts.tests.test_studio_ui_control.StudioUiControlTests.test_capture_screenshot_prefers_window_capture_then_falls_back_to_display -v`
    - `python3 -m unittest scripts.tests.test_studio_ui_control.StudioUiControlTests.test_capture_screenshot_records_failure_metadata_when_display_capture_fails -v`
  - local-safe green:
    - `python3 -m unittest scripts.tests.test_studio_ui_control scripts.tests.test_run_studio_harness scripts.tests.test_run_studio_harness_remote -v`
    - `bash -n scripts/run_studio_harness.sh scripts/run_studio_harness_remote.sh`
    - `git diff --check`
  - remote `tertiary` proof:
    - `ARNIS_REMOTE_STUDIO_ARTIFACT_DIR=/tmp/arnis-remote-studio-screenshot ARNIS_REMOTE_STUDIO_BASE_ARNIS=/Volumes/APDataStore/arnis-roblox-proof bash scripts/run_studio_harness_remote.sh --remote-host tertiary -- --takeover --hard-restart --skip-edit-tests --play-wait 30 --pattern-wait 150 --screenshot /tmp/arnis-studio-harness.png`
    - the run again reached authoritative live proof:
      - runtime `ARNIS_SCENE_PLAY` emitted
      - `ARNIS_CLIENT_BOOTSTRAP ... bootstrapState=gameplay_ready`
      - harness logged `play bootstrap trace verdict (authoritative client bootstrap marker): valid`
      - harness logged `writing scene fidelity play artifact`
    - the synced local artifact directory now contained:
      - `/tmp/arnis-remote-studio-screenshot/0.715.0.7151115_20260403T151340Z_Studio_1577b_last.log`
      - `/tmp/arnis-remote-studio-screenshot/arnis-scene-fidelity-play.json`
      - `/tmp/arnis-remote-studio-screenshot/arnis-scene-fidelity-play.html`
      - `/tmp/arnis-remote-studio-screenshot/arnis-studio-harness-play.capture.json`
    - that sidecar proved the failure is now explicit and trustworthy:
      - `capture_method="failed"`
      - display attempt stderr was `could not create image from display`
      - session status at failure was `ready_play` with one live Studio window
      - window lookup also failed, which is now recorded in structured metadata instead of disappearing behind a generic screenshot failure
- Interpretation:
  - the remote display lane is still blocked at the machine/window-server level, but it is no longer a black box
  - productive proof work can move on without screenshot babysitting because failed capture attempts now sync back with enough context to distinguish infrastructure failure from Austin/runtime failure

## 2026-04-03 play-mode terrain seam + startup settlement hardening

- The newest product-facing symptom report narrowed the likely play-mode regressions:
  - terrain in play looked chunk-bounded, with false vertical planes around peaks
  - some walls/roofs appeared only after walking through or past them
  - edit mode still looked materially better than play mode
- I treated this as two separate runtime truths instead of one vague “renderer is broken” complaint:
  - terrain seam continuity was broken by chunk-local edge sampling in `TerrainBuilder`
  - bootstrap was still allowed to declare `gameplay_ready` after only one streaming update, which is too weak for near-field building/roof residency in play
- Implementation landed:
  - `TerrainBuilder` now has neighbor-aware edge sampling through `resolveNeighborHeightSample(...)`
  - the same seam context now feeds both interpolated terrain heights and slope-derived material classification, so chunk-border voxels stop clamping back into the local chunk at shared edges
  - `ImportService.ImportManifest(...)` now builds `buildTerrainNeighborContextByChunkId(...)` and threads that context into per-chunk terrain planning
  - `StreamingService` now derives the same terrain neighbor context for streamed chunk admissions so startup import and play-time streaming stay on one terrain seam contract
  - `BootstrapAustin.server.lua` now waits on near-ring startup streaming settlement before releasing the player and declaring `gameplay_ready`; it warns loudly on timeout instead of silently treating one update as enough
- Coverage added:
  - `scripts.tests.test_play_render_truth.PlayRenderTruthTests.test_terrain_builder_supports_neighbor_aware_chunk_edge_sampling`
  - `scripts.tests.test_austin_runtime_contract.AustinRuntimeContractTests.test_bootstrap_waits_for_near_ring_streaming_settlement_before_gameplay_ready`
  - `roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua` now includes a seam-focused neighbor sampling assertion
- Docs updated:
  - `AGENTS.md` and `CLAUDE.md` now include the default harness/screenshot invocation shapes and the `*.capture.json` sidecar guidance
- Local-safe verification for this slice:
  - `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `git diff --check`
- Important limitation:
  - I did not run Studio locally
  - the next proof step still belongs on `tertiary`, where we should verify whether the terrain seam fix removes the chunk-wall artifacts and whether the bootstrap wait materially reduces late wall/roof visibility in play

### 2026-04-03 remote `tertiary` check after terrain seam + startup settlement patch

- Remote proof command:
  - `ARNIS_REMOTE_STUDIO_ARTIFACT_DIR=/tmp/arnis-remote-studio-terrainfix bash scripts/run_studio_harness_remote.sh --remote-host tertiary -- --takeover --hard-restart --skip-edit-tests --play-wait 30 --pattern-wait 150 --screenshot /tmp/arnis-studio-harness.png`
- The wrapper’s local profile alias was not configured on this machine, so I used explicit `--remote-host tertiary`.
- The run reached authoritative play proof in the remote Studio log:
  - `ARNIS_SCENE_PLAY` emitted
  - `ARNIS_CLIENT_BOOTSTRAP ... bootstrapState=gameplay_ready`
  - `ARNIS_CLIENT_WORLD_COMPACT` emitted at multiple readiness phases
- The most useful product-facing signal from this run:
  - at `streaming_ready`, the client still reported `nearbyBuildingModels=0`, `nearbyWallParts=0`, `nearbyRoofParts=0`
  - by `gameplay_ready`, the client reported `nearbyBuildingModels=7`, `nearbyWallParts=4`, `collidableWallPartsNearby=4`, `nearbyRoofParts=14`, `overheadRoofParts=4`
  - this is consistent with the new bootstrap wait improving what is actually present before gameplay is declared ready
- Additional extracted truth from the remote log:
  - latest `ARNIS_SCENE_PLAY` contained `buildingVisibleWallGapDetails=[]`
  - latest `ARNIS_CLIENT_LOCAL_EXPERIENCE` still had `playerLocalTelemetryEnabled=false`, so this run did not yet provide terrain roughness diagnostics for the false-plane report
- Screenshot status remains unchanged infrastructure-wise:
  - remote `/tmp/arnis-studio-harness-play.capture.json` existed
  - it still reported `capture_method="failed"`
  - `window_lookup_error="execution error: Error: TypeError: {} is not iterable (-2700)"`
- Remaining gap from this proof:
  - the harness tail did not sync a final local artifact bundle before I cut the lingering remote session
  - a direct remote check showed `/tmp/arnis-scene-fidelity-play.json` had not been written yet for this run, so structured play-audit artifact generation still trails the proof marker path on `tertiary`

## 2026-04-03 play-mode fidelity tranche: terrain edge truth, startup structural settlement, and closure-only roof audits

- This tranche focused on converting three vague play-mode complaints into concrete runtime contracts:
  - terrain seams that still looked chunk-bounded in play
  - startup readiness that could still advance before nearby walls/roofs were really present
  - shaped-roof buildings that could degrade into closure-only roof evidence without showing up as a specific audit failure
- Implementation landed:
  - `TerrainBuilder.lua`
    - sub-4-stud terrain edge subsamples now keep raw neighbor-aware cell coordinates for height interpolation instead of clamping back into the local chunk before interpolation
    - this tightens the seam fix so border voxels actually consult adjacent chunk heights during high-resolution sampling
  - `WorldProbeTerrain.lua` and `WorldProbe.client.lua`
    - local terrain summaries now include `missingEdgeSampleCount`, `edgeTerrainYRangeStuds`, and `centerEdgeMaxDeltaStuds`
    - the player-local terrain cross sample now marks the explicit edge indices so seam cliffs and missing edge hits become measurable instead of anecdotal
  - `StreamingService.lua` and `BootstrapAustin.server.lua`
    - startup readiness now uses `StreamingService.GetStartupResidencySnapshot(...)`
    - that snapshot combines near-ring residency telemetry with a bounded nearby building/wall/roof scan around spawn
    - `BootstrapAustin` now waits on that combined structural envelope before moving past `streaming_ready`
  - `SceneAudit.lua`
    - closure-only shaped-roof cases now increment `buildingModelsWithClosureOnlyRoofGap`
    - those cases also emit `buildingClosureOnlyRoofGapDetails`, so they no longer disappear into the generic no-roof bucket without explanation
- Coverage added:
  - `scripts/tests/test_terrain_chunk_edge_truth.py`
  - `roblox/src/ServerScriptService/Tests/StartupStructuralEnvelopeSettlement.spec.lua`
  - `roblox/src/ServerScriptService/Tests/ClosureOnlyRoofGapAudit.spec.lua`
  - `roblox/src/ServerScriptService/Tests/WorldProbeTerrain.spec.lua`
  - `scripts/tests/test_play_render_truth.py`
  - `scripts/tests/test_austin_runtime_contract.py`
- Local-safe verification for the tranche:
  - `python3 -m unittest scripts.tests.test_terrain_chunk_edge_truth scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ReplicatedStorage/Shared/WorldProbeTerrain.lua roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/ImportService/SceneAudit.lua roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/BootstrapAustin.server.lua roblox/src/ServerScriptService/Tests/WorldProbeTerrain.spec.lua roblox/src/ServerScriptService/Tests/ClosureOnlyRoofGapAudit.spec.lua roblox/src/ServerScriptService/Tests/StartupStructuralEnvelopeSettlement.spec.lua`
  - `git diff --check`
- Remote `tertiary` follow-up from this exact tranche did not yet yield a new authoritative comparison run:
  - command attempted:
    - `ARNIS_REMOTE_STUDIO_ARTIFACT_DIR=/tmp/arnis-remote-studio-playfidelity ARNIS_TELEMETRY_FAMILIES=terrain,structures,player_local bash scripts/run_studio_harness_remote.sh --remote-host tertiary -- --takeover --hard-restart --skip-edit-tests --play-wait 35 --pattern-wait 180`
  - the wrapper rebuilt and opened the clean play place, but the run then sat in a long remote tail/open-place state without syncing proof artifacts or producing a new log with `ARNIS_CLIENT_*` / `ARNIS_SCENE_PLAY` markers
  - the latest new remote Studio log for that attempt (`0.715.0.7151115_20260403T161430Z_Studio_8be9d_last.log`) only showed repeated localhost connection-refused chatter and no new Austin bootstrap/play markers
  - I terminated the lingering local wrapper/SSH tail after confirming there was no new proof signal yet
- Updated understanding after this slice:
  - the code-side play fidelity contracts are materially better and locally verified
  - the next remote proof run should reuse this tranche and specifically confirm:
    - whether `player_local` terrain edge metrics now show nontrivial seam deltas around the reported false planes
    - whether `streaming_ready` is now structurally closer to the old `gameplay_ready` envelope
    - whether closure-only roof cases appear explicitly in play audit output instead of hiding inside generic roofless counts

## 2026-04-03 17:32 CDT

- Closed two product-facing play-fidelity gaps locally:
  - `BuildingBuilder.lua`
    - irregular shaped-roof fallbacks (`gabled`, `gambrel`, `hipped`, `pyramidal`, `skillion`, `mansard`) now emit visible flat roof geometry instead of invisible closure-only decks
    - updated `GabledIrregularFootprintTruth.spec.lua` so irregular gabled shells now require visible roof evidence and reject closure-only fallback
  - `TerrainBuilder.lua`
    - supersampled terrain columns now keep `averageHeight` for telemetry but also compute `heightRange` and `surfaceHeight`
    - steep mixed-height 4x4 write voxels now render to the local peak (`surfaceHeight`) instead of collapsing to an averaged fake plane, which is the current code-side mitigation for the reported play-mode terrain peak boxes/false planes
- Verification:
  - `python3 -m unittest scripts.tests.test_terrain_chunk_edge_truth scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ReplicatedStorage/Shared/WorldProbeTerrain.lua roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua roblox/src/ServerScriptService/ImportService/SceneAudit.lua roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/BootstrapAustin.server.lua roblox/src/ServerScriptService/Tests/GabledIrregularFootprintTruth.spec.lua roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua roblox/src/ServerScriptService/Tests/WorldProbeTerrain.spec.lua roblox/src/ServerScriptService/Tests/ClosureOnlyRoofGapAudit.spec.lua roblox/src/ServerScriptService/Tests/StartupStructuralEnvelopeSettlement.spec.lua`
  - `git diff --check`
- Next remote proof on `tertiary` should answer two concrete questions:
  - whether the terrain false planes/peak boxes are materially reduced with `surfaceHeight` bias on steep mixed voxels
  - whether the newly visible irregular roof fallback materially improves play-mode roof coverage near spawn and during movement

## 2026-04-03 18:04 CDT

- Another local-safe play-fidelity tranche is in:
  - `StreamingService.lua`
    - startup structural telemetry now walks only registered loaded chunks via `ChunkLoader.ListLoadedChunks(...)` / `ChunkLoader.GetChunkEntry(...)`
    - this closes the gap where nearby shell geometry inside the world root but outside registered residency could make startup look ready too early
  - `TerrainBuilder.lua`
    - steep mixed-height terrain voxels now compute `surfaceFillDepth` in addition to `surfaceHeight`
    - the renderer keeps the local peak for steep voxels but shrinks the filled depth to the elevated coverage ratio, which is the current mitigation for the reported terrain peak boxes / false planes
- Verification:
  - `python3 -m unittest scripts.tests.test_terrain_chunk_edge_truth scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/BootstrapAustin.server.lua roblox/src/ServerScriptService/Tests/GabledIrregularFootprintTruth.spec.lua roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua roblox/src/ServerScriptService/Tests/StartupStructuralEnvelopeSettlement.spec.lua roblox/src/ReplicatedStorage/Shared/WorldProbeTerrain.lua roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua roblox/src/ServerScriptService/ImportService/SceneAudit.lua roblox/src/ServerScriptService/Tests/WorldProbeTerrain.spec.lua roblox/src/ServerScriptService/Tests/ClosureOnlyRoofGapAudit.spec.lua`
  - `git diff --check`
- Process note:
  - a worker attempted a local Studio harness command despite the standing machine constraint
  - all workers were shut down, local `rbx-studio-mcp` processes were killed, and this branch continued local-safe only

## 2026-04-03 18:19 CDT

- Added a building-shell play-fidelity mitigation in `BuildingBuilder.lua`:
  - simple low-rise opaque buildings that still qualify for `preferSimpleShellDetail` now keep explicit shell wall parts even under `shellMesh` mode
  - only the simple-shell subset changes; larger opaque buildings still use merged wall meshes
  - intent: reduce renderer/streaming brittleness for the exact “simple geometry is janky or late in play” slice without throwing away the broader mesh path
- Verification:
  - `python3 -m unittest scripts.tests.test_terrain_chunk_edge_truth scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/BootstrapAustin.server.lua roblox/src/ServerScriptService/Tests/GabledIrregularFootprintTruth.spec.lua roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua roblox/src/ServerScriptService/Tests/StartupStructuralEnvelopeSettlement.spec.lua roblox/src/ReplicatedStorage/Shared/WorldProbeTerrain.lua roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua roblox/src/ServerScriptService/ImportService/SceneAudit.lua roblox/src/ServerScriptService/Tests/WorldProbeTerrain.spec.lua roblox/src/ServerScriptService/Tests/ClosureOnlyRoofGapAudit.spec.lua`
  - `git diff --check`

## 2026-04-03 12:23 CDT

- Integrated a wider local-safe play-fidelity tranche on `main`:
  - `StreamingService.lua`
    - startup readiness now ignores structure evidence unless chunk ownership is explicit on the registered chunk folder, buildings folder, and building model
    - LOD visibility now resolves per-group anchors instead of relying only on chunk-center distance, which should reduce play-only detail disappearance near chunk edges
  - `BuildingBuilder.lua` / `SceneAudit.lua`
    - explicit simple-shell wall parts keep `ArnisShellWallEvidence` so play/runtime audits can distinguish true wall evidence from closure-only roof support without broadening merged-wall truth
  - `TerrainBuilder.lua` / `WorldProbeTerrain.lua`
    - terrain peak bias now scales with peak coverage instead of snapping every steep mixed voxel to the max
    - compact terrain convergence metrics now expose coverage and incomplete-edge status without adding log spam
  - `SceneAudit.lua`
    - compact building convergence metrics now summarize wall/roof coverage and expose whether the scene is missing walls, roofs, or both
- Verification:
  - `python3 -m unittest scripts.tests.test_terrain_chunk_edge_truth scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ReplicatedStorage/Shared/WorldProbeTerrain.lua roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/ImportService/SceneAudit.lua roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/ImportService/init.lua roblox/src/ServerScriptService/Tests/HippedRoofTruth.spec.lua roblox/src/ServerScriptService/Tests/PlayStreamingRoomShellParity.spec.lua roblox/src/ServerScriptService/Tests/SceneAudit.spec.lua roblox/src/ServerScriptService/Tests/StartupStructuralEnvelopeSettlement.spec.lua roblox/src/ServerScriptService/Tests/TerrainAlignment.spec.lua roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua roblox/src/ServerScriptService/Tests/WorldProbeTerrain.spec.lua`
  - `git diff --check`
- Next `tertiary` proof should answer:
  - whether per-group LOD anchors materially reduce play-only facade/detail disappearance near chunk edges
  - whether peak-coverage terrain shaping reduces the false peak planes without introducing underfilled hilltops

## 2026-04-03 12:26 CDT

- Tightened scene-truth accounting after the LOD-anchor slice:
  - `SceneAudit.lua` now tracks visible building detail/facade counts separately from total detail/facade counts, so hidden runtime groups no longer look equivalent to visible play-mode detail
  - updated `SceneAudit.spec.lua`, `test_play_render_truth.py`, and `test_austin_runtime_contract.py` to lock the new visible-vs-total contract
- Verification:
  - `python3 -m unittest scripts.tests.test_terrain_chunk_edge_truth scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/SceneAudit.lua roblox/src/ServerScriptService/Tests/SceneAudit.spec.lua`
  - `git diff --check`

## 2026-04-03 12:36 CDT

- Integrated another local-safe play-fidelity tranche on `main`:
  - `StreamingService.lua`
    - startup readiness now selects a coherent nearby structural envelope by source building instead of trusting pooled wall/roof counts across unrelated models
    - this keeps spawn readiness false when nearby wall-only and roof-only buildings happen to coexist but no single nearby building is actually complete enough for play
  - `BuildingBuilder.lua`
    - shellMesh explicit wall fallback is now widened one step beyond the narrow simple-shell path, but it stays shape/size bounded instead of usage-wide
  - `TerrainBuilder.lua` / `WorldProbeTerrain.lua`
    - mixed-voxel surface bias now uses both peak coverage and average-vs-min spread, and sparse-slot terrain telemetry now counts numeric sample slots truthfully even when the sample table has holes
- Verification:
  - `python3 -m unittest scripts.tests.test_terrain_chunk_edge_truth scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ReplicatedStorage/Shared/WorldProbeTerrain.lua roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/Tests/PlayStreamingRoomShellParity.spec.lua roblox/src/ServerScriptService/Tests/StartupStructuralEnvelopeSettlement.spec.lua roblox/src/ServerScriptService/Tests/WorldProbeTerrain.spec.lua`
  - `git diff --check`
- Next `tertiary` proof should answer:
  - whether coherent spawn-envelope readiness reduces the “looks empty until I walk through it” play-start behavior
  - whether the bounded shell-wall fallback materially improves wall presence for medium-complexity shellMesh buildings
  - whether the revised mixed-voxel shaping reduces both flat false planes and underfilled hilltops

## 2026-04-03 13:05 CDT

- Extended shellMesh play-visible wall fallback to bounded courtyard buildings:
  - `BuildingBuilder.lua`
    - one-hole/courtyard shellMesh buildings can now stay on the explicit shell-wall path when they remain low-rise and bounded, instead of dropping straight to the merged wall path
  - `ShellMeshCourtyardTruth.spec.lua`
    - added direct Luau coverage for a shellMesh courtyard building so wall evidence, roof presence, and an open interior void are all locked together
  - `test_play_render_truth.py` / `test_austin_runtime_contract.py`
    - updated source-contract coverage for the new bounded-hole fallback rules
- Verification:
  - `python3 -m unittest scripts.tests.test_terrain_chunk_edge_truth scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/Tests/ShellMeshCourtyardTruth.spec.lua`
  - `git diff --check`

## 2026-04-03 14:02 CDT

- Landed a broader local-safe play-fidelity tranche on `main`:
  - `BootstrapAustin.server.lua`
    - startup now requires a short consecutive-ready window before trusting streaming settlement, so `gameplay_ready` is less vulnerable to one transient good poll
  - `StreamingService.lua`
    - LOD visibility now uses camera distance plus the last runtime focal/avatar point, which should reduce near-player detail disappearing when the play camera is offset
  - `BuildingBuilder.lua`
    - bounded shaped-roof shellMesh buildings that still use merged walls now keep low-cost readable cues via facade beltlines and corner accents instead of depending on merged shell mass alone
  - `TerrainBuilder.lua`
    - steep mixed-height voxel columns now apply a secondary shallow ridge fill cap, which should reduce the false peak-box / vertical-plane terrain artifact
- Added focused local-safe coverage in:
  - `BuildingShellMeshReadableCues.spec.lua`
  - `LODCameraAvatarOffset.spec.lua`
  - `test_bootstrap_streaming_stability_truth.py`
  - `test_building_shell_mesh_readability_contract.py`
  - `test_streaming_lod_avatar_offset_contract.py`
  - `test_terrain_steep_mixed_fill_depth_truth.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_terrain_chunk_edge_truth scripts.tests.test_terrain_steep_mixed_fill_depth_truth scripts.tests.test_building_shell_mesh_readability_contract scripts.tests.test_streaming_lod_avatar_offset_contract scripts.tests.test_bootstrap_streaming_stability_truth scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/BootstrapAustin.server.lua roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/Tests/BuildingShellMeshReadableCues.spec.lua roblox/src/ServerScriptService/Tests/LODCameraAvatarOffset.spec.lua roblox/src/ServerScriptService/Tests/ShellMeshCourtyardTruth.spec.lua roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua`
  - `git diff --check`

## 2026-04-03 15:01 CDT

- Landed another product-facing play-convergence tranche on `main`:
  - `ChunkPriority.lua` / `StreamingService.lua`
    - movement-lookahead scheduling now keeps a secondary actual-player distance in chunk/work-item priority, so forward prediction no longer starves nearer real-player chunks quite as easily
    - LOD visibility now uses cached group footprint bounds instead of center-anchor distance, so large edge-spanning detail/interior groups stay visible when the player is near their actual footprint
  - `BuildingBuilder.lua`
    - merged shellMesh readability cues now include bounded roofline and perimeter silhouette parts in addition to facade beltlines/corner accents, improving roof and wall legibility without switching those buildings to explicit wall loops
  - `TerrainBuilder.lua`
    - steep mixed-height terrain columns now taper top/bottom occupancy with an edge occupancy scale, reducing slabby vertical cliff planes on top of the earlier shallow ridge fill cap
- Added focused local-safe coverage in:
  - `LODGroupFootprintVisibility.spec.lua`
  - `test_streaming_dual_focus_priority_contract.py`
  - `test_streaming_lod_footprint_contract.py`
  - `test_terrain_column_occupancy_shaping_truth.py`
  - updated `BuildingShellMeshReadableCues.spec.lua`, `StreamingPriority.spec.lua`, `TerrainOutdoorFidelity.spec.lua`, `test_building_shell_mesh_readability_contract.py`, and `test_austin_runtime_contract.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_streaming_dual_focus_priority_contract scripts.tests.test_streaming_lod_footprint_contract scripts.tests.test_terrain_column_occupancy_shaping_truth scripts.tests.test_terrain_steep_mixed_fill_depth_truth scripts.tests.test_building_shell_mesh_readability_contract scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/ChunkPriority.lua roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/Tests/BuildingShellMeshReadableCues.spec.lua roblox/src/ServerScriptService/Tests/LODGroupFootprintVisibility.spec.lua roblox/src/ServerScriptService/Tests/StreamingPriority.spec.lua roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua`
  - `git diff --check`

## 2026-04-03 15:42 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `BuildingBuilder.lua`
    - merged shellMesh readability cues now add bounded street-level wall-presence strips on the merged-shell cue path, so wall mass stays legible in play without switching those buildings to full explicit wall loops
  - `TerrainBuilder.lua`
    - sparse steep peaks now apply bounded surface-height coverage damping before the earlier fill-depth/occupancy shaping, reducing exaggerated cliff lift on lightly represented peaks
  - `StreamingService.lua`
    - `updateLOD()` now prefers a live player root focus when available and only falls back to the last scheduler focal point, reducing play-mode LOD lag while moving between scheduler updates
- Added focused local-safe coverage in:
  - `BuildingShellMeshWallPresenceCues.spec.lua`
  - `test_building_shell_mesh_wall_presence_contract.py`
  - `test_streaming_lod_live_root_focus_contract.py`
  - `test_terrain_sparse_peak_surface_damping_truth.py`
  - updated `LODCameraAvatarOffset.spec.lua`, `TerrainOutdoorFidelity.spec.lua`, and `test_streaming_lod_avatar_offset_contract.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_building_shell_mesh_wall_presence_contract scripts.tests.test_building_shell_mesh_readability_contract scripts.tests.test_terrain_sparse_peak_surface_damping_truth scripts.tests.test_terrain_column_occupancy_shaping_truth scripts.tests.test_streaming_lod_live_root_focus_contract scripts.tests.test_streaming_lod_avatar_offset_contract scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/Tests/BuildingShellMeshWallPresenceCues.spec.lua roblox/src/ServerScriptService/Tests/LODCameraAvatarOffset.spec.lua roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua`
  - `git diff --check`

## 2026-04-03 14:02 CDT

- Landed a broader local-safe play-fidelity/runtime-convergence tranche on `main`:
  - `StreamingService.lua`
    - startup structural readiness now accepts merged-shell readable facade cues when explicit collidable wall parts are absent, so near-spawn high-fidelity shellMesh buildings can satisfy readiness truth without waiting for the wrong wall contract
    - high-detail chunks with pending `buildings` subplans now bypass the throttled subplan rollout path and queue a whole-chunk import instead, reducing late wall/roof/facade appearance around the player
  - `SceneAudit.lua`
    - merged-shell readable cues now count as visible wall evidence, keeping play-mode wall-gap audits aligned with the intended merged-shell readability path
  - `TerrainBuilder.lua`
    - prepared terrain plans now derive a deterministic neighbor signature from supplied neighbor context when callers omit an explicit signature, so seam-aware plans invalidate correctly instead of reusing stale seam-blind cache entries
- Added focused local-safe coverage in:
  - `StartupMergedShellReadableEnvelope.spec.lua`
  - `HighDetailWholeChunkAdmission.spec.lua`
  - updated `TerrainPlanReuse.spec.lua`, `test_play_render_truth.py`, and `test_austin_runtime_contract.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/ImportService/SceneAudit.lua roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/Tests/StartupMergedShellReadableEnvelope.spec.lua roblox/src/ServerScriptService/Tests/HighDetailWholeChunkAdmission.spec.lua roblox/src/ServerScriptService/Tests/TerrainPlanReuse.spec.lua`
  - `git diff --check`

## 2026-04-03 14:10 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `ChunkPriority.lua` / `StreamingService.lua`
    - high-detail whole-chunk building admissions now carry an explicit priority bit through work-item sorting, so near-player full building imports outrank competing subplan work during the startup/play window instead of getting starved by cheaper sibling subplans
  - `TerrainBuilder.lua`
    - implicit follow-up `PrepareChunk(chunk)` calls now reuse the best cached prepared plan when no new options are supplied, so a seam-aware terrain plan does not silently downgrade back to seam-blind sampling on later build passes
- Added focused local-safe coverage in:
  - `HighDetailWholeChunkPriority.spec.lua`
  - updated `TerrainPlanReuse.spec.lua`, `test_play_render_truth.py`, and `test_austin_runtime_contract.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/ImportService/ChunkPriority.lua roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/Tests/HighDetailWholeChunkPriority.spec.lua roblox/src/ServerScriptService/Tests/TerrainPlanReuse.spec.lua`
  - `git diff --check`

## 2026-04-03 14:16 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `StreamingService.lua`
    - the immediate post-streaming LOD refresh now uses the current camera position as a secondary focus instead of waiting for the slower 2-second camera-aware LOD cadence, so chunks admitted while the camera is near but the avatar has moved no longer stay visually stale until the next delayed pass
- Added focused local-safe coverage in:
  - updated `LODCameraAvatarOffset.spec.lua`
  - updated `test_austin_runtime_contract.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/ImportService/ChunkPriority.lua roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/Tests/LODCameraAvatarOffset.spec.lua roblox/src/ServerScriptService/Tests/HighDetailWholeChunkPriority.spec.lua roblox/src/ServerScriptService/Tests/TerrainPlanReuse.spec.lua`
  - `git diff --check`

## 2026-04-03 15:07 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `TerrainBuilder.lua` / `ImportService/init.lua` / `StreamingService.lua`
    - terrain neighbor signatures now include neighbor terrain identity, not just chunk id, so seam-aware prepared plans invalidate when a neighbor chunk is reloaded or revised under the same stable id instead of reusing stale border samples
    - streaming LOD visibility now uses split focus policy: exterior detail remains camera-aware with avatar/root fallback, while interiors stay gated to avatar/root focus only
    - `StreamingService.Start(...)` now applies one immediate LOD sync after seeding loaded chunk LOD state, so already-loaded startup chunks do not wait for the delayed heartbeat before their detail visibility is corrected
- Added focused local-safe coverage in:
  - updated `TerrainPlanReuse.spec.lua`
  - updated `test_play_render_truth.py`, `test_austin_runtime_contract.py`, `test_streaming_lod_avatar_offset_contract.py`, and `test_streaming_lod_footprint_contract.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract scripts.tests.test_streaming_lod_avatar_offset_contract scripts.tests.test_streaming_lod_live_root_focus_contract scripts.tests.test_streaming_lod_footprint_contract scripts.tests.test_streaming_residency_footprint_contract scripts.tests.test_terrain_chunk_edge_truth -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/ImportService/init.lua roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/Tests/TerrainPlanReuse.spec.lua`
  - `git diff --check`

## 2026-04-03 15:28 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `StreamingService.lua`
    - `StreamingService.Start(...)` now primes one immediate `StreamingService.Update()` before the heartbeat scheduler takes over, so startup residency can begin loading near-player chunks immediately instead of waiting for the first movement-driven cadence
- Added focused local-safe coverage in:
  - new `test_streaming_startup_prime_contract.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_streaming_startup_prime_contract scripts.tests.test_streaming_lod_avatar_offset_contract scripts.tests.test_streaming_lod_live_root_focus_contract scripts.tests.test_streaming_lod_footprint_contract scripts.tests.test_streaming_residency_footprint_contract scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/StreamingService.lua`
  - `git diff --check`

## 2026-04-03 15:41 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `BuildingBuilder.lua`
    - merged `shellMesh` readable-cue buildings now emit bounded `MergedShellWindowPaneCue` parts with an explicit count attribute, so tall merged shells keep some facade/window legibility in play mode instead of reading like plain wall mass plus a door
  - `TerrainBuilder.lua`
    - very sparse cliff columns now square the sparse-coverage damping before occupancy clamping, dropping the minimum occupancy floor for those extreme cases so false vertical slab faces taper harder
- Added focused local-safe coverage in:
  - updated `BuildingShellMeshReadableCues.spec.lua`
  - updated `TerrainOutdoorFidelity.spec.lua`
  - updated `test_building_shell_mesh_readability_contract.py`
  - updated `test_terrain_sparse_cliff_occupancy_shaping_truth.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_building_shell_mesh_readability_contract scripts.tests.test_building_shell_mesh_wall_presence_contract scripts.tests.test_terrain_sparse_cliff_occupancy_shaping_truth scripts.tests.test_terrain_chunk_edge_truth scripts.tests.test_terrain_steep_mixed_fill_depth_truth scripts.tests.test_terrain_column_occupancy_shaping_truth -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/Tests/BuildingShellMeshReadableCues.spec.lua roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua`
  - `git diff --check`

## 2026-04-03 15:09 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `StreamingService.lua` / `ChunkPriority.lua`
    - high-detail structural work now carries an explicit scheduler priority bit, so equally-near building work can outrank same-band non-structural subplans instead of waiting behind roads or other cheaper layers during startup and movement-time admissions
    - whole-chunk high-detail building imports, high-detail building subplans, and high-detail whole-chunk fallbacks with building content all carry the same structure-priority hint
  - `SceneAudit.lua`
    - merged-shell window pane cues now count as readable facade truth in scene audits instead of disappearing from visible-facade accounting
  - `StreamingService.lua`
    - merged-shell window pane cues now count toward startup readable-facade envelope truth, so startup readiness better reflects what the player can actually see on merged-shell buildings
- Added focused local-safe coverage in:
  - updated `ChunkSubplanPriority.spec.lua`
  - new `test_streaming_structure_priority_contract.py`
  - updated `test_play_render_truth.py`
  - updated `test_austin_runtime_contract.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract scripts.tests.test_building_shell_mesh_readability_contract scripts.tests.test_streaming_structure_priority_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/ImportService/ChunkPriority.lua roblox/src/ServerScriptService/ImportService/SceneAudit.lua roblox/src/ServerScriptService/Tests/ChunkSubplanPriority.spec.lua`
  - `git diff --check`

## 2026-04-03 15:23 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `TerrainBuilder.lua`
    - sparse steep peak columns now apply an extra `isolatedPeakSupportDamping` term on top of the existing coverage damping, so single-sample or weakly-supported peaks stay closer to the surrounding terrain surface instead of lifting into false ridge planes in play mode
  - `TerrainOutdoorFidelity.spec.lua`
    - tightened the sparse-peak expectation so the local test fixture now locks a lower allowed lifted surface for the worst isolated-peak case
- Added focused local-safe coverage in:
  - updated `test_terrain_sparse_peak_surface_damping_truth.py`
  - updated `TerrainOutdoorFidelity.spec.lua`
- Verification:
  - `python3 -m unittest scripts.tests.test_terrain_sparse_peak_surface_damping_truth scripts.tests.test_terrain_sparse_cliff_occupancy_shaping_truth scripts.tests.test_terrain_chunk_edge_truth scripts.tests.test_terrain_steep_mixed_fill_depth_truth scripts.tests.test_terrain_column_occupancy_shaping_truth -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua`
  - `git diff --check`

## 2026-04-03 15:25 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `SceneAudit.lua`
    - merged-shell `MergedShellRooflineCue` and `MergedShellPerimeterCue` parts now count as merged roof evidence when visible, so intentionally roof-readable shellMesh buildings stop falling into the generic no-roof bucket during play-fidelity audits
    - closure-only roof gaps remain gated on `evidenceKind == "none"`, so closure decks still do not masquerade as visible roof truth
- Added focused local-safe coverage in:
  - updated `test_play_render_truth.py`
  - updated `test_austin_runtime_contract.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/SceneAudit.lua`
  - `git diff --check`

## 2026-04-03 15:33 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `StreamingService.lua`
    - startup structure readiness now treats visible `MergedShellRooflineCue` and `MergedShellPerimeterCue` parts as nearby and overhead roof evidence, so merged-shell buildings with intentionally cheap roof readability can satisfy the same roof envelope used for `gameplay_ready`
    - startup roof truth is now expressed through an explicit `hasRoofEnvelope` gate instead of repeating the raw roof counters inline
- Added focused local-safe coverage in:
  - updated `test_play_render_truth.py`
  - updated `test_austin_runtime_contract.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/StreamingService.lua`
  - `git diff --check`

## 2026-04-03 15:49 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `WorldProbe.client.lua`
    - player-local roof-cover telemetry now treats visible `MergedShellRooflineCue` and `MergedShellPerimeterCue` parts as roof cover instead of filtering them out as generic detail, so live play telemetry matches the startup roof envelope and scene-audit roof evidence rules
    - merged-shell roof cues are excluded from the generic non-roof shell-wall path, avoiding double-classification in the local world probe
- Added focused local-safe coverage in:
  - updated `test_play_render_truth.py`
  - updated `test_austin_runtime_contract.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua`
  - `git diff --check`

## 2026-04-03 16:07 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `TerrainBuilder.lua`
    - when the true diagonal neighbor chunk is missing, diagonal seam samples now blend the two adjacent edge-neighbor heights instead of snapping to a single edge sample, reducing corner seam discontinuities and boxy cliff corners at chunk boundaries
- Added focused local-safe coverage in:
  - updated `test_terrain_chunk_edge_truth.py`
  - updated `TerrainOutdoorFidelity.spec.lua`
- Verification:
  - `python3 -m unittest scripts.tests.test_terrain_chunk_edge_truth scripts.tests.test_terrain_sparse_peak_surface_damping_truth scripts.tests.test_terrain_sparse_cliff_occupancy_shaping_truth scripts.tests.test_terrain_steep_mixed_fill_depth_truth scripts.tests.test_terrain_column_occupancy_shaping_truth -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua`
  - `git diff --check`

## 2026-04-03 17:23 CDT

- Performed local session cleanup before continuing:
  - found the shell open-file limit pinned at `256`
  - found about `23k` open files across the user session and many zombie `rbx-studio-mcp` children hanging off stale `codex` parents
  - reaped the oldest stale `codex` parents with leaked Studio-MCP children, dropping user-session open files materially and restoring stable local exec capacity without touching the current session
- Landed another local-safe play-fidelity tranche on `main`:
  - `BuildingBuilder.lua`
    - merged-shell readable roofline/perimeter cues are no longer artificially disabled for bounded flat-roof shellMesh buildings, so flat shells can keep cheap rooftop silhouette readability instead of reading like plain roof slabs
  - `FlatShellMeshRoofTruth.spec.lua`
    - flat shellMesh roof truth now requires bounded merged-shell roofline and perimeter cues alongside direct roof geometry
- Added focused local-safe coverage in:
  - updated `test_building_shell_mesh_readability_contract.py`
  - updated `FlatShellMeshRoofTruth.spec.lua`
- Verification:
  - `python3 -m unittest scripts.tests.test_building_shell_mesh_readability_contract scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/Tests/FlatShellMeshRoofTruth.spec.lua`
  - `git diff --check`

## 2026-04-03 17:31 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `BuildingBuilder.lua`
    - the bounded `preferPlayVisibleShellWalls` shellMesh path now adds cheap roofline and facade beltline readability cues instead of leaving the detail folder mostly blank, so low-cost explicit-wall shells read closer to edit mode without abandoning the explicit wall fallback
  - `WorldProbe.client.lua`
    - player-local enclosure telemetry now reports `nearbyReadableFacadeCueParts`, so merged-shell readability cues show up in live local enclosure truth instead of only in startup and scene-audit paths
- Added focused local-safe coverage in:
  - updated `test_building_shell_mesh_readability_contract.py`
  - updated `test_play_render_truth.py`
  - updated `test_austin_runtime_contract.py`
  - reused `FlatShellMeshRoofTruth.spec.lua` as the flat-roof proof point
- Verification:
  - `python3 -m unittest scripts.tests.test_building_shell_mesh_readability_contract scripts.tests.test_play_render_truth scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/Tests/FlatShellMeshRoofTruth.spec.lua roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua`
  - `git diff --check`

## 2026-04-03 17:52 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `TerrainBuilder.lua`
    - added an extra `sparsePeakPlaneDamping` term on top of the existing sparse-peak support damping so the sparsest steep peaks stay flatter and stop inflating into broad false top planes
  - `TerrainOutdoorFidelity.spec.lua`
    - tightened the sparse-peak runtime expectation again so the unit fixture now locks an even lower surface height ceiling for the worst peak-plane case
- Repo hygiene:
  - added `.omx/` to `.gitignore` so Codex-side local tooling state no longer pollutes the worktree
- Added focused local-safe coverage in:
  - updated `test_terrain_sparse_peak_surface_damping_truth.py`
  - updated `test_play_render_truth.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_terrain_sparse_peak_surface_damping_truth scripts.tests.test_play_render_truth -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua`
  - `git diff --check`

## 2026-04-03 17:56 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `StreamingService.lua`
    - added a movement-triggered LOD refresh hook in the heartbeat path so meaningful live player-root movement can force the next detail visibility pass immediately instead of waiting for the full 2-second LOD cadence
    - this specifically targets the movement-time “walk into it and then it appears” failure mode for already-loaded chunks
- Added focused local-safe coverage in:
  - new `test_streaming_movement_lod_refresh_contract.py`
  - updated `test_austin_runtime_contract.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_streaming_movement_lod_refresh_contract scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/StreamingService.lua`
  - `git diff --check`

## 2026-04-03 18:01 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `StreamingService.lua`
    - the scheduler now falls back to live avatar/root motion when there is not enough prior focal history yet, so the very first meaningful player movement can still get a forward-biased lookahead instead of waiting for a second movement sample
    - the existing movement-triggered LOD refresh path now pairs with that earlier motion fallback, reducing startup-to-first-motion visible lag for already-loaded chunks
  - `WorldProbe.client.lua`
    - player-local world-probe sampling now switches to a faster cadence and tighter resample distance while the avatar is actually moving, so live telemetry stays closer to what the player sees in motion instead of lagging behind the traversal state
- Added focused local-safe coverage in:
  - updated `test_streaming_dual_focus_priority_contract.py`
  - updated `test_austin_runtime_contract.py`
  - updated `test_play_render_truth.py`
  - reused the runtime `StreamingPriority.spec.lua` lane as the play-focused motion/startup tie-break proof
- Verification:
  - `python3 -m unittest scripts.tests.test_streaming_movement_lod_refresh_contract scripts.tests.test_streaming_dual_focus_priority_contract scripts.tests.test_austin_runtime_contract scripts.tests.test_play_render_truth -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua roblox/src/ServerScriptService/Tests/StreamingPriority.spec.lua`
  - `git diff --check`

## 2026-04-03 18:06 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `BuildingBuilder.lua`
    - the bounded `preferPlayVisibleShellWalls` shellMesh path now carries the full cheap player-facing readability kit: facade beltlines, roofline cues, corner accents, and a bounded street-facing door cue
    - this keeps the medium-cost explicit-wall lane materially closer to edit-mode legibility without promoting those buildings into the heavier merged readability path
  - `PlayVisibleShellReadableCues.spec.lua`
    - expanded the dedicated explicit-wall readability spec so this lane now stays pinned for roofline, beltline, corner-accent, and door-cue coverage together
- Added focused local-safe coverage in:
  - updated `test_building_shell_mesh_readability_contract.py`
  - updated `test_play_render_truth.py`
  - updated `PlayVisibleShellReadableCues.spec.lua`
- Verification:
  - `python3 -m unittest scripts.tests.test_building_shell_mesh_readability_contract scripts.tests.test_play_render_truth -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/Tests/PlayVisibleShellReadableCues.spec.lua`
  - `git diff --check`

## 2026-04-03 18:05 CDT

- Landed another local-safe play-fidelity tranche on `main`:
  - `BuildingBuilder.lua`
    - the bounded `preferPlayVisibleShellWalls` shellMesh path now keeps corner accents in addition to facade beltlines and roofline cues, so medium-cost explicit-wall shells read less flat and more like their edit-mode silhouettes without escalating to the full merged-cue path
  - `PlayVisibleShellReadableCues.spec.lua`
    - added a dedicated shellMesh truth spec for the explicit-wall readability path so this medium-cost play-visible lane stays pinned separately from simple-shell and merged-shell lanes
- Added focused local-safe coverage in:
  - updated `test_building_shell_mesh_readability_contract.py`
  - updated `test_play_render_truth.py`
  - new `PlayVisibleShellReadableCues.spec.lua`
- Verification:
  - `python3 -m unittest scripts.tests.test_building_shell_mesh_readability_contract scripts.tests.test_play_render_truth -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/Tests/PlayVisibleShellReadableCues.spec.lua`
  - `git diff --check`

## 2026-04-03 18:11 CDT

- Landed another local-safe bundled play-fidelity tranche on `main`:
  - `BuildingBuilder.lua`
    - the bounded `preferPlayVisibleShellWalls` shellMesh path now adds bounded window pane cues on top of beltlines, roofline cues, corner accents, and the street-facing door cue, so medium-cost explicit-wall shells stop reading like mostly blank slabs at eye level
  - `StreamingService.lua`
    - newly imported chunks now get their LOD visibility applied immediately after import succeeds instead of waiting until the end-of-update reconciliation pass, reducing intra-update visible lag for chunks that are already resident but freshly materialized
  - `PlayVisibleShellReadableCues.spec.lua`
    - expanded again so the dedicated explicit-wall shell readability lane now stays pinned for roofline, beltline, corner-accent, door-cue, and window-cue coverage together
- Added focused local-safe coverage in:
  - new `test_streaming_import_lod_refresh_contract.py`
  - updated `test_building_shell_mesh_readability_contract.py`
  - updated `test_play_render_truth.py`
  - updated `test_austin_runtime_contract.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_building_shell_mesh_readability_contract scripts.tests.test_play_render_truth scripts.tests.test_streaming_import_lod_refresh_contract scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/Tests/PlayVisibleShellReadableCues.spec.lua`
  - `git diff --check`

## 2026-04-03 18:13 CDT

- Landed another local-safe bundled play-fidelity tranche on `main`:
  - `BuildingBuilder.lua`
    - the bounded `preferPlayVisibleShellWalls` shellMesh path now carries the full cheap facade kit as well: street-facade cues and window-pane cues join beltlines, roofline cues, corner accents, and the street-facing door cue, so medium-cost explicit-wall shells stop reading like plain walls at player height
  - `StreamingService.lua`
    - newly imported chunks now get their LOD visibility applied immediately after import succeeds, reducing the last bit of intra-update “it loaded but still looks stale” lag for fresh resident chunks
  - `PlayVisibleShellReadableCues.spec.lua`
    - expanded again so the dedicated explicit-wall readability lane now stays pinned for street-facade and window-pane coverage too
- Added focused local-safe coverage in:
  - updated `test_building_shell_mesh_readability_contract.py`
  - updated `test_play_render_truth.py`
  - new `test_streaming_import_lod_refresh_contract.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_building_shell_mesh_readability_contract scripts.tests.test_play_render_truth scripts.tests.test_streaming_import_lod_refresh_contract scripts.tests.test_austin_runtime_contract -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua roblox/src/ServerScriptService/ImportService/StreamingService.lua roblox/src/ServerScriptService/Tests/PlayVisibleShellReadableCues.spec.lua`
  - `git diff --check`

## 2026-04-03 18:17 CDT

- Landed another local-safe terrain-fidelity tranche on `main`:
  - `TerrainBuilder.lua`
    - sparse steep peaks now apply an extra `sparsePeakEdgeOccupancyDamping` term before voxel top/bottom occupancy shaping, so the thinnest peak cases taper their caps more aggressively instead of keeping a thicker false top plane
  - `TerrainOutdoorFidelity.spec.lua`
    - tightened the sparse-peak runtime expectation to require a lower edge-occupancy scale on the pathological single-sample peak fixture
- Added focused local-safe coverage in:
  - updated `test_terrain_column_occupancy_shaping_truth.py`
  - updated `test_play_render_truth.py`
- Verification:
  - `python3 -m unittest scripts.tests.test_terrain_column_occupancy_shaping_truth scripts.tests.test_terrain_sparse_peak_surface_damping_truth scripts.tests.test_play_render_truth -v`
  - `stylua --check roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua`
  - `git diff --check`

## 2026-04-03 18:30 CDT

- Landed the first real planetary backbone implementation slice on `main`:
  - new Rust crate `arbx_planetary_store`
    - initializes a canonical SQLite planetary store
    - ingests existing manifest SQLite stores as named scenes
    - summarizes total scene/chunk/feature counts
    - reads scene-local chunk subsets by stud-space bounding box, which is the first concrete retrieval primitive needed for realtime planetary streaming
  - `arbx_cli`
    - added `planetary-store init`
    - added `planetary-store ingest-manifest`
    - added `planetary-store summary`
    - added `planetary-store subset --bbox-studs ...`
  - `arbx_roblox_export::manifest_store`
    - promoted stored manifest structs to `Serialize` so the planetary subset path can emit machine-readable JSON cleanly
- Verification:
  - `cargo test -p arbx_planetary_store -p arbx_cli planetary_store --quiet`
  - `cargo test -p arbx_planetary_store --quiet`
  - `git diff --check rust/Cargo.toml rust/crates/arbx_planetary_store/Cargo.toml rust/crates/arbx_planetary_store/src/lib.rs rust/crates/arbx_cli/Cargo.toml rust/crates/arbx_cli/src/main.rs rust/crates/arbx_roblox_export/src/manifest_store.rs`

## 2026-04-03 18:34 CDT

- Extended the planetary backbone on `main`:
  - `arbx_planetary_store`
    - added scene discovery through `list_scenes`
    - added per-scene metadata reads through `read_scene_catalog_entry`
    - kept bbox-based chunk subset reads as the scene-local retrieval primitive
  - `arbx_cli`
    - added `planetary-store list-scenes`
    - added `planetary-store scene --scene ...`
- Verification:
  - `cargo test -p arbx_planetary_store -p arbx_cli planetary_store --quiet`
  - `cargo test -p arbx_planetary_store --quiet`
  - `git diff --check rust/crates/arbx_planetary_store/src/lib.rs rust/crates/arbx_cli/src/main.rs`

## 2026-04-03 18:40 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - added lightweight `PlanetaryChunkSummary` reads by stud-space bbox with optional result limits
    - this gives the canonical store a metadata-only chunk query path suitable for realtime streaming orchestration before fetching full chunk payloads
  - `arbx_cli`
    - added `planetary-store subset-summary --bbox-studs ... [--limit N]`
- Verification:
  - `cargo test -p arbx_planetary_store -p arbx_cli planetary_store --quiet`
  - `cargo test -p arbx_planetary_store --quiet`
  - `git diff --check rust/crates/arbx_planetary_store/src/lib.rs rust/crates/arbx_cli/src/main.rs`

## 2026-04-03 18:44 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - added geographic scene discovery by bbox intersection
    - added geographic scene discovery by point containment
    - the canonical store can now answer both “what scenes exist here?” and “what chunk summaries cover this local bbox?” which are the two basic orchestration queries for planetary streaming
  - `arbx_cli`
    - added `planetary-store find-scenes --bbox ...`
    - added `planetary-store find-scenes --point ...`
- Verification:
  - `cargo test -p arbx_planetary_store -p arbx_cli planetary_store --quiet`
  - `cargo test -p arbx_planetary_store --quiet`
  - `git diff --check rust/crates/arbx_planetary_store/src/lib.rs rust/crates/arbx_cli/src/main.rs`

## 2026-04-03 18:55 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - added direct JSON manifest ingestion into the canonical planetary store, so the backbone can sit in front of both existing JSON-first and SQLite-first workflows
    - kept scene discovery, bbox subset, lightweight chunk summary, and geo scene lookup working across both ingest paths
  - `arbx_cli`
    - added `planetary-store ingest-json --manifest-json ...`
- Verification:
  - `cargo test -p arbx_planetary_store -p arbx_cli planetary_store --quiet`
  - `cargo test -p arbx_planetary_store --quiet`
  - `git diff --check rust/crates/arbx_planetary_store/src/lib.rs rust/crates/arbx_cli/src/main.rs`

## 2026-04-03 19:06 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - added chunk payload fetch by scene and chunk id list
    - the canonical store now has the complete basic loop for orchestration: discover scenes, query chunk summaries, and then fetch the selected chunk payloads
  - `arbx_cli`
    - added `planetary-store fetch-chunks --scene ... --chunk-ids ...`
- Verification:
  - `cargo test -p arbx_planetary_store -p arbx_cli planetary_store --quiet`
  - `cargo test -p arbx_planetary_store --quiet`
  - `git diff --check rust/crates/arbx_planetary_store/src/lib.rs rust/crates/arbx_cli/src/main.rs`

## 2026-04-03 19:10 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - added manifest-subset materialization by bbox and by chunk-id list, so the canonical store can reconstruct downstream-consumable manifest subsets instead of only serving metadata and payload fragments
  - `arbx_cli`
    - added `planetary-store emit-manifest-subset --scene ... --bbox-studs ...`
    - added `planetary-store emit-manifest-subset --scene ... --chunk-ids ...`
    - supports `--out` so the subset can be written directly to disk for downstream runtime/export/audit flows
- Verification:
  - `cargo test -p arbx_planetary_store -p arbx_cli planetary_store --quiet`
  - `cargo test -p arbx_planetary_store --quiet`
  - `git diff --check rust/crates/arbx_planetary_store/src/lib.rs rust/crates/arbx_cli/src/main.rs`

## 2026-04-03 19:15 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_roblox_export`
    - added direct runtime shard emission from a stored manifest subset, not just from a standalone manifest SQLite file
  - `arbx_cli`
    - added `planetary-store emit-runtime-lua --scene ...`
    - supports both `--bbox-studs ...` and `--chunk-ids ...` subset selection against the canonical store, then emits Roblox runtime Lua shards directly from that selected subset
  - `arbx_planetary_store`
    - the canonical store now has the full minimal delivery loop: discover scenes, select chunk summaries, fetch payloads, materialize manifest subsets, and emit Roblox runtime shards from those subsets
- Verification:
  - `cargo test -p arbx_planetary_store -p arbx_cli planetary_store --quiet`
  - `cargo test -p arbx_roblox_export --quiet`
  - `git diff --check rust/crates/arbx_planetary_store/src/lib.rs rust/crates/arbx_cli/src/main.rs rust/crates/arbx_roblox_export/src/lua_runtime_shards.rs rust/crates/arbx_roblox_export/src/lib.rs`

## 2026-04-03 19:21 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - added payload-class metadata per chunk: terrain presence plus road, rail, building, water, prop, landuse, and barrier counts
    - added filtered lightweight chunk-summary queries so orchestration can ask for windows that specifically require terrain-bearing or building-bearing chunks
  - `arbx_cli`
    - `planetary-store subset-summary` now supports `--require-buildings` and `--require-terrain`
- Verification:
  - `cargo test -p arbx_planetary_store -p arbx_cli planetary_store --quiet`
  - `cargo test -p arbx_planetary_store --quiet`
  - `git diff --check rust/crates/arbx_planetary_store/src/lib.rs rust/crates/arbx_cli/src/main.rs`

## 2026-04-03 19:30 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - added point-centered local stud-space delivery windows sorted by chunk-center distance
    - this gives the canonical store a true “around the player/focus point” selection primitive instead of only rectangular bbox selection
    - added extra query-path indexes for geographic scene lookup and payload-aware chunk selection to keep the store fast without compromising fidelity
  - `arbx_cli`
    - `planetary-store subset-summary` now supports `--around-studs X,Z --radius-studs R`
    - `planetary-store emit-manifest-subset` now supports around-point selection
    - `planetary-store emit-runtime-lua` now supports around-point selection
- Verification:
  - `cargo test -p arbx_planetary_store -p arbx_cli planetary_store --quiet`
  - `cargo test -p arbx_planetary_store --quiet`
  - `git diff --check rust/crates/arbx_planetary_store/src/lib.rs rust/crates/arbx_cli/src/main.rs`

## 2026-04-03 19:35 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - added geo-to-local focus bridging: the canonical store can now turn a geographic focus point into point-centered local delivery windows and manifest subsets, instead of forcing callers to compute Mercator/local stud conversions themselves
  - this keeps the store as the orchestration boundary for both geo scene discovery and local delivery selection
- Verification:
  - `cargo test -p arbx_planetary_store -p arbx_cli planetary_store --quiet`
  - `cargo test -p arbx_planetary_store --quiet`
  - `git diff --check rust/crates/arbx_planetary_store/src/lib.rs rust/crates/arbx_cli/src/main.rs`

## 2026-04-03 19:47 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - added `PlanetaryDeliveryWindow`, which packages the chosen scene, geo focus, local focus, radius, and selected chunk summaries as one orchestration result
    - the canonical store now supports a one-shot geo-point delivery-window query instead of forcing callers to manually compose scene lookup plus point-centered chunk selection
  - `arbx_cli`
    - added `planetary-store delivery-window --point LAT,LON --radius-studs R`
- Verification:
  - `cargo test -p arbx_planetary_store -p arbx_cli planetary_store --quiet`
  - `cargo test -p arbx_planetary_store --quiet`
  - `git diff --check rust/crates/arbx_planetary_store/src/lib.rs rust/crates/arbx_cli/src/main.rs`

## 2026-04-03 19:57 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - added slippy-tile geographic coverage lookup
    - added tile-to-local subset and chunk-summary conversion for the canonical store
    - the store now supports rectangular bbox, local point, geo point, and geographic tile selection against the same canonical scene data
  - `arbx_cli`
    - `planetary-store find-scenes` now supports `--tile Z,X,Y`
    - added `planetary-store tile-scenes --tile Z,X,Y`
    - `planetary-store subset-summary`, `emit-manifest-subset`, and `emit-runtime-lua` now support tile-based selection
- Verification:
  - `cargo test -p arbx_planetary_store -p arbx_cli planetary_store --quiet`
  - `cargo test -p arbx_planetary_store --quiet`
  - `git diff --check rust/crates/arbx_planetary_store/src/lib.rs rust/crates/arbx_cli/src/main.rs`

## 2026-04-03 20:01 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_cli`
    - `planetary-store subset-summary` now supports direct geographic point selection
    - `planetary-store emit-manifest-subset` now supports direct geographic point selection
    - `planetary-store emit-runtime-lua` now supports direct geographic point selection
  - this means callers can now go straight from real lat/lon to summary, manifest subset, or Roblox runtime shard output without first translating into local stud-space arguments
- Verification:
  - `cargo test -p arbx_planetary_store -p arbx_cli planetary_store --quiet`
  - `cargo test -p arbx_planetary_store --quiet`
  - `git diff --check rust/crates/arbx_planetary_store/src/lib.rs rust/crates/arbx_cli/src/main.rs`

## 2026-04-03 20:11 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - completed the provenance side of the canonical store by adding an attach/update path for `SourceTruthPackSummary`
    - scene catalog and scene metadata now have a place to carry source-truth summary counts alongside delivery metadata
  - `arbx_cli`
    - added `planetary-store attach-truth-pack-summary --scene ... --truth-pack-summary ...`
  - `arbx_pipeline`
    - `SourceTruthPackSummary` is now deserializable as well as serializable, so truth-pack summary JSON can be round-tripped into the planetary backbone cleanly
- Verification:
  - `cargo test -p arbx_planetary_store -p arbx_cli planetary_store --quiet`
  - `cargo test -p arbx_planetary_store --quiet`
  - `git diff --check rust/crates/arbx_pipeline/src/truth_pack.rs rust/crates/arbx_planetary_store/Cargo.toml rust/crates/arbx_planetary_store/src/lib.rs rust/crates/arbx_cli/src/main.rs`

## 2026-04-03 20:32 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - scene selection now ranks overlapping geo candidates by attached truth-pack richness first, then by tighter geographic coverage
    - geo-bbox scene queries now surface truth-pack metadata consistently instead of dropping provenance on one path
    - added best-scene helpers for geo points and slippy tiles so scene choice can stay inside the canonical store
    - added tile-driven `PlanetaryDeliveryWindow` construction, so geographic tiles can produce the same orchestration artifact as geo-point delivery
  - `arbx_cli`
    - `emit-manifest-subset` and `emit-runtime-lua` no longer require `--scene` when `--point` or `--tile` already identifies the target scene
    - `delivery-window` now supports `--tile Z,X,Y` in addition to `--point LAT,LON`
- Verification:
  - `cargo fmt --all`
  - `cargo test -p arbx_planetary_store planetary_store_ -- --nocapture`
  - `cargo test -p arbx_cli planetary_store_ -- --nocapture`
  - `git diff --check`

## 2026-04-03 20:41 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - `PlanetaryDeliveryWindow` now carries aggregate chunk count, feature count, streaming cost, and estimated memory cost so higher-level schedulers can act on costed selections directly
    - both geo-point and tile delivery windows now emit the same cost summary shape
- Verification:
  - `cargo fmt --all`
  - `cargo test -p arbx_planetary_store planetary_store_ -- --nocapture`
  - `cargo test -p arbx_cli planetary_store_delivery_window_ -- --nocapture`
  - `git diff --check`

## 2026-04-03 21:02 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - chunk-summary selection now supports cumulative `max_streaming_cost` and `max_estimated_memory_cost` ceilings in stable order, not just raw count limits
    - geo-point and tile delivery windows now inherit those ceilings and expose the trimmed budgeted chunk set
  - `arbx_cli`
    - `subset-summary`, `emit-manifest-subset`, `emit-runtime-lua`, and `delivery-window` now accept `--max-streaming-cost` and `--max-estimated-memory-cost`
    - bbox, around-point, geo-point, and tile flows now all use the same budget-aware selection contract
- Verification:
  - `cargo fmt --all`
  - `cargo test -p arbx_planetary_store planetary_store_ -- --nocapture`
  - `cargo test -p arbx_cli planetary_store_ -- --nocapture`
  - `git diff --check`

## 2026-04-03 21:15 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - added scene-local delivery-window construction for local focus points and scene-local bbox selection, not just geo points and tiles
    - delivery windows are now the shared orchestration artifact across bbox, around-point, geo-point, and tile selection modes
  - `arbx_cli`
    - `delivery-window` now supports `--scene --around-studs X,Z --radius-studs R` and `--scene --bbox-studs MIN_X,MIN_Z,MAX_X,MAX_Z`
    - `subset-summary` now resolves the scene automatically for `--point` and `--tile` instead of requiring redundant `--scene` input
- Verification:
  - `cargo fmt --all`
  - `cargo test -p arbx_planetary_store planetary_store_ -- --nocapture`
  - `cargo test -p arbx_cli planetary_store_ -- --nocapture`
  - `git diff --check`

## 2026-04-03 21:26 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - delivery windows now support scene-local local-point and scene-local bbox selection in addition to geo-point and tile selection
    - local and geographic scheduling now share the same costed delivery-window artifact instead of splitting into parallel summary-only paths
  - `arbx_cli`
    - `delivery-window` now supports `--scene --around-studs X,Z --radius-studs R` and `--scene --bbox-studs MIN_X,MIN_Z,MAX_X,MAX_Z`
    - `subset-summary` now resolves the scene automatically for point/tile selection, matching the other high-level planetary commands
- Verification:
  - `cargo fmt --all`
  - `cargo test -p arbx_planetary_store planetary_store_ -- --nocapture`
  - `cargo test -p arbx_cli planetary_store_ -- --nocapture`
  - `git diff --check`

## 2026-04-03 21:40 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - added `PlanetaryDeliveryPlan`, a reusable orchestration artifact carrying scene choice, selection mode, chunk ids, and aggregate cost totals
    - added plan builders for geo-point, local-point, scene-bbox, and tile selection
  - `arbx_cli`
    - added `planetary-store delivery-plan` with the same selector surface as `delivery-window` plus optional `--out`
    - `fetch-chunks`, `emit-manifest-subset`, and `emit-runtime-lua` now accept `--plan PATH` to consume a previously saved delivery plan directly
- Verification:
  - `cargo fmt --all`
  - `cargo test -p arbx_planetary_store planetary_store_ -- --nocapture`
  - `cargo test -p arbx_cli planetary_store_ -- --nocapture`
  - `git diff --check`

## 2026-04-03 21:54 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - added plan hydration support so saved delivery plans can be turned back into chunk summaries and delivery-window artifacts
    - local selection plans now record only the focus dimensions they actually know instead of fake geo coordinates
  - `arbx_cli`
    - `subset-summary` and `delivery-window` now accept `--plan PATH` in addition to selector arguments
    - the delivery-plan artifact now round-trips across selection, inspection, chunk fetch, manifest subset emission, runtime shard emission, and window rehydration
- Verification:
  - `cargo fmt --all`
  - `cargo test -p arbx_planetary_store planetary_store_ -- --nocapture`
  - `cargo test -p arbx_cli planetary_store_ -- --nocapture`
  - `git diff --check`

## 2026-04-03 22:08 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - delivery plans now carry the originating planetary store path so saved plans are self-sufficient for downstream store-backed steps
    - added delivery-plan hydration back into chunk-summary and delivery-window artifacts
    - local plans no longer fake geo coordinates; only real geo-selected plans carry geo focus
  - `arbx_cli`
    - `fetch-chunks`, `subset-summary`, `delivery-window`, `emit-manifest-subset`, and `emit-runtime-lua` can now consume `--plan PATH` without separately restating `--store`
    - the saved delivery-plan artifact now round-trips end-to-end without duplicated selector or store arguments
- Verification:
  - `cargo fmt --all`
  - `cargo test -p arbx_planetary_store planetary_store_ -- --nocapture`
  - `cargo test -p arbx_cli planetary_store_ -- --nocapture`
  - `git diff --check`

## 2026-04-03 22:23 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - delivery plans now include the originating planetary store path so downstream plan execution does not require duplicated store wiring
    - saved plans can now hydrate back into delivery-window and chunk-summary views, completing the inspection/execution loop
  - `arbx_cli`
    - plan-driven `fetch-chunks`, `subset-summary`, `delivery-window`, `emit-manifest-subset`, and `emit-runtime-lua` can now infer the store directly from the saved plan
    - plan-driven flows now work with fewer duplicated arguments, making the saved plan a more portable execution artifact
- Verification:
  - `cargo fmt --all`
  - `cargo test -p arbx_planetary_store planetary_store_ -- --nocapture`
  - `cargo test -p arbx_cli planetary_store_ -- --nocapture`
  - `git diff --check`

## 2026-04-03 22:39 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_cli`
    - added `planetary-store delivery-bundle`, a single execution lane that can build or consume a delivery plan, optionally persist that plan, write a manifest subset, and emit runtime Lua shards from the exact same chosen chunk set
    - the bundle command returns one structured result summarizing the plan plus any emitted manifest/runtime outputs
  - the planetary orchestration surface now has a full selector -> plan -> hydrate/inspect -> fetch/export -> bundled execute loop
- Verification:
  - `cargo fmt --all`
  - `cargo test -p arbx_planetary_store planetary_store_ -- --nocapture`
  - `cargo test -p arbx_cli planetary_store_ -- --nocapture`
  - `git diff --check`

## 2026-04-03 22:55 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_cli`
    - `delivery-bundle` now supports durable result persistence via `--out`, plus optional hydrated `--summary-out` and `--window-out` artifacts alongside `--plan-out`, `--manifest-out`, and runtime shard output
    - the bundle result is now a stable structured execution artifact rather than stdout-only glue
  - the planetary execution lane can now produce persisted plan, summary, window, manifest, runtime, and bundle-result artifacts from one selected chunk set
- Verification:
  - `cargo fmt --all`
  - `cargo test -p arbx_planetary_store planetary_store_ -- --nocapture`
  - `cargo test -p arbx_cli planetary_store_ -- --nocapture`
  - `git diff --check`

## 2026-04-03 23:08 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_cli`
    - `delivery-bundle` now supports persisted bundle-result output via `--out`, plus optional hydrated `--summary-out` and `--window-out` artifacts alongside plan, manifest, and runtime outputs
    - the bundle result is now a durable execution record instead of a transient stdout-only summary
  - one selector and one chosen chunk set can now yield persisted plan, summary, window, manifest, runtime, and bundle-result artifacts in a single command
- Verification:
  - `cargo fmt --all`
  - `cargo test -p arbx_planetary_store planetary_store_ -- --nocapture`
  - `cargo test -p arbx_cli planetary_store_ -- --nocapture`
  - `git diff --check`

## 2026-04-03 23:28 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_planetary_store`
    - added merged delivery plans so multiple sampled plans can collapse into one deduped, cost-recomputed chunk set for route/prefetch style orchestration
  - `arbx_cli`
    - added `planetary-store merge-delivery-plans`
    - `delivery-bundle` now accepts multiple `--plan` inputs and executes against the merged chunk set
  - this gives the planetary lane a first real multi-sample / route-prefetch primitive on top of the existing single-window selection flow
- Verification:
  - `cargo fmt --all`
  - `cargo test -p arbx_planetary_store planetary_store_ -- --nocapture`
  - `cargo test -p arbx_cli planetary_store_ -- --nocapture`
  - `git diff --check`

## 2026-04-04 00:02 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_cli`
    - added `planetary-store route-plan`, which samples repeated geo points and emits one merged delivery plan
    - `delivery-bundle` now accepts route-style multi-plan inputs through merged delivery plans, giving the repo a native multi-point prefetch path instead of shell-stitching multiple windows
  - `arbx_planetary_store`
    - merged delivery plans now serve as the canonical deduped, cost-recomputed artifact for route/prefetch assembly
- Verification:
  - `cargo fmt --all`
  - `cargo test -p arbx_planetary_store planetary_store_ -- --nocapture`
  - `cargo test -p arbx_cli planetary_store_ -- --nocapture`
  - `git diff --check`

## 2026-04-04 00:21 CDT

- Extended the planetary backbone again on `main`:
  - `arbx_cli`
    - `delivery-bundle` now accepts repeated `--point LAT,LON` selectors directly and internally collapses them into the merged route-prefetch plan
    - multi-point lookahead no longer requires an explicit `route-plan` pre-step when the goal is immediate execution
  - the route lane now supports both explicit staged planning (`route-plan`) and direct one-shot execution (`delivery-bundle --point ... --point ...`) against the same merged plan semantics
- Verification:
  - `cargo fmt --all`
  - `cargo test -p arbx_planetary_store planetary_store_ -- --nocapture`
  - `cargo test -p arbx_cli planetary_store_ -- --nocapture`
  - `git diff --check`
