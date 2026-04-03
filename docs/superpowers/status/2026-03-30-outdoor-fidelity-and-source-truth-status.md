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
