# Outdoor Fidelity And Source-Truth Status

Date: 2026-03-30
Status: Active

## Purpose

This is the rolling status and handoff document for the active outdoor fidelity and source-truth tranche.

The active design spec is:

- `docs/superpowers/specs/2026-03-30-outdoor-fidelity-and-source-truth-design.md`

The active implementation plan is:

- `docs/superpowers/plans/2026-03-30-outdoor-fidelity-and-source-truth.md`

## Current Snapshot

- Inherited from the March 28 baseline and archived play-fidelity tranche: edit/play parity, canonical runtime ownership, and player-local observability are already proven.
- The compatibility purge is in place: the repo treats `0.4.0` as the supported manifest schema and does not keep older manifest compatibility active.
- The active tranche is now the March 30 outdoor fidelity and source-truth stack.
- The earlier March 28 play-fidelity tranche is historical context only.

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

### Remote `tertiary`

No verification recorded yet.

## Residual Gaps

- Outdoor fidelity still needs dedicated work on terrain detail, shell nuance, and player-visible exterior realism.
- Outdoor hotspots still need tighter measurement so preview/edit cost can be traced at chunk scope instead of only at the whole-run level.
- Source-truth preservation still needs explicit proof across upstream source union, canonical collapse, and downstream audits.

## Status Notes

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
