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
