# Outdoor Fidelity And Source-Truth Implementation Plan

Status: Active

This is the active implementation surface for the March 30 outdoor fidelity and source-truth tranche.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Strengthen outdoor fidelity, outdoor hotspot observability, and upstream source-truth preservation from the current clean baseline without adding a second world-definition path or silent signal loss.

**Architecture:** Add a bounded pre-canonical outdoor truth-pack, then join it to the existing manifest, scene-fidelity, and scene-parity audits. Use those truth surfaces to drive terrain/outdoor fidelity fixes and prove the results only on `tertiary`, while keeping docs continuously synchronized with the active tranche.

**Tech Stack:** Python audit/report tooling, Rust pipeline/export crates, Luau runtime/import builders, shell harness scripts, remote `tertiary` Studio proof

Status note: On 2026-04-01, `codex/breaking-compatibility-purge` and `codex/outdoor-fidelity-source-truth` were merged by hand into `codex/manual-main-integration` from a clean worktree so the active plan/docs stack could advance without touching the dirty root checkout. The merged branch keeps the `0.4.0` hard break and the current truth-pack/outdoor observability work together; use the rolling status file for the exact verification record.

Repo governance note: this plan is part of the only active superpowers truth stack. Every other superpowers spec/plan/status file must be marked `Historical` or `Completed`, and guardrail tests enforce that contract.

---

## File Map

### Docs And Status

- Create: `docs/superpowers/archive-index.md`
- Modify: `docs/superpowers/specs/2026-03-30-outdoor-fidelity-and-source-truth-design.md`
- Modify: `docs/superpowers/plans/2026-03-30-outdoor-fidelity-and-source-truth.md`
- Modify: `docs/superpowers/status/2026-03-28-canonical-baseline-status.md`
- Modify: `docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md`
- Modify: `docs/remote-studio-development.md`

### Truth-Pack Extraction And Audit

- Create: `scripts/source_truth_pack.py`
- Create: `scripts/source_truth_pack_audit.py`
- Create: `scripts/tests/test_source_truth_pack.py`
- Create: `scripts/tests/test_source_truth_pack_audit.py`
- Modify: `scripts/export_austin_from_osm.sh`
- Modify: `scripts/manifest_quality_audit.py`
- Modify: `scripts/tests/test_manifest_quality_audit.py`
- Modify: `scripts/tests/test_austin_fidelity.py`
- Modify: `rust/crates/arbx_cli/src/main.rs`
- Modify: `rust/crates/arbx_pipeline/`

### Scene Fidelity / Telemetry / Harness

- Modify: `scripts/scene_fidelity_audit.py`
- Modify: `scripts/scene_parity_audit.py`
- Modify: `scripts/run_studio_harness.sh`
- Modify: `scripts/tests/test_scene_fidelity_audit.py`
- Modify: `scripts/tests/test_scene_parity_audit.py`
- Modify: `scripts/tests/test_run_studio_harness.py`
- Modify: `roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua`
- Modify: `roblox/src/ReplicatedStorage/Shared/WorldProbeTerrain.lua`
- Create: `roblox/src/ReplicatedStorage/Shared/WorldProbeTelemetryFlags.lua`
- Create: `roblox/src/ServerScriptService/Tests/WorldProbeTelemetryFlags.spec.lua`

### Outdoor Fidelity / Hotspots

- Modify: `roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua`
- Modify: `roblox/src/ServerScriptService/StudioPreview/AustinPreviewTelemetry.lua`
- Modify: `scripts/preview_telemetry_summary.py`
- Modify: `scripts/tests/test_preview_telemetry_summary.py`
- Modify: `scripts/tests/test_play_render_truth.py`
- Create: `roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua`

### Runtime Streaming Engine

- Modify: `roblox/src/ReplicatedStorage/Shared/WorldConfig.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/StreamingService.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/ChunkPriority.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/ManifestLoader.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/RunAustin.lua`
- Modify: `roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua`
- Modify: `scripts/tests/test_austin_runtime_contract.py`
- Modify: `scripts/tests/test_play_render_truth.py`
- Modify: `scripts/tests/test_run_studio_harness.py`
- Modify: `docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md`

## Success Thresholds

- Truth-pack output is bounded and query-oriented: a compile run writes `rust/out/<scene>.truth-pack.sqlite` plus a compact audit summary JSON, not a monolithic full-scene JSON blob.
- Truth-pack audit explicitly reports outdoor source overlap, collapse, and dropped-semantics findings for terrain/landuse, roads, water, vegetation, and outdoor structure-shell semantics.
- Scene-fidelity reports can show both truth-pack-backed outdoor provenance findings and runtime player-local outdoor experience metrics without requiring raw JSON inspection.
- Remote `tertiary` proof produces at least one focused edit slice and one focused play slice for the tranche and records the result in the active status doc the same day.
- Preview/edit hotspot reporting exposes the worst outdoor-heavy chunk cost with structured phase timings and artifact counts in the main audit surfaces.
- The runtime streaming engine exposes explicit near/mid/far residency rings with authoritative `estimatedMemoryCost` budgets and secondary chunk-count caps.
- Runtime telemetry reports resident chunk counts, resident estimated memory, inflight estimated memory, queued work, and eviction/prefetch reasons by ring.
- The Austin sample remains correct on the same shared runtime path while the scheduler becomes budget-enforcing instead of distance-only.

## Task 1: Roll The Active Docs Stack Forward

**Files:**
- Create: `docs/superpowers/archive-index.md`
- Modify: `docs/superpowers/specs/2026-03-30-outdoor-fidelity-and-source-truth-design.md`
- Modify: `docs/superpowers/plans/2026-03-30-outdoor-fidelity-and-source-truth.md`
- Modify: `docs/superpowers/status/2026-03-28-canonical-baseline-status.md`
- Modify: `docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md`
- Modify: `docs/remote-studio-development.md`

- [x] **Step 1: Wrote the new outdoor tranche status document**

Create `docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md` with:
- date/status header
- purpose and active-spec/plan links
- current snapshot inherited from the March 28 baseline and compatibility purge
- empty verification snapshot sections for `Local Static` and `Remote tertiary`
- residual gaps focused on outdoor fidelity, outdoor hotspots, and source-truth preservation

- [x] **Step 2: Consolidated the historical docs surface**

Add `docs/superpowers/archive-index.md` as the only historical navigation surface and fold the deleted March 28 play-fidelity and other superseded tranche references into the retained baseline/active docs.

- [x] **Step 3: Marked the approved design doc active**

Update `docs/superpowers/specs/2026-03-30-outdoor-fidelity-and-source-truth-design.md` so it clearly reads as the active design surface for the tranche rather than a proposed draft.

- [x] **Step 4: Marked the implementation plan active**

Update `docs/superpowers/plans/2026-03-30-outdoor-fidelity-and-source-truth.md` so it clearly reads as the active implementation surface for the tranche.

- [x] **Step 5: Updated the operator doc**

Update `docs/remote-studio-development.md` so it points operators at the new rolling status file and keeps `tertiary` as the current proof lane without duplicating volatile proof claims.

- [x] **Step 6: Verified the active-marker guardrail for the doc stack**

Run:

```bash
rg -n "^Status: Active$" docs/superpowers/specs docs/superpowers/plans docs/superpowers/status
```

Expected:
- the new outdoor status/plan/spec stack is the only set of docs with active markers
- the March 28 play-fidelity docs are clearly historical/completed context

- [x] **Step 7: Committed**

```bash
git add docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md \
  docs/superpowers/specs/2026-03-30-outdoor-fidelity-and-source-truth-design.md \
  docs/superpowers/plans/2026-03-30-outdoor-fidelity-and-source-truth.md \
  docs/superpowers/archive-index.md \
  docs/superpowers/status/2026-03-28-canonical-baseline-status.md \
  docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md \
  docs/remote-studio-development.md
git commit -m "docs: roll active outdoor fidelity stack forward"
```

## Task 2: Add The Bounded Outdoor Truth-Pack

Status note: On 2026-04-01, Task 2 was narrowed to the smallest honest slice the current pipeline can support. The first implementation only needs to support `arbx_cli compile` for `OverpassAdapter` / `LiveOverpassAdapter`, with real cross-source truth focused on Overpass-derived retained features plus Overture building candidates and Overture-to-OSM collapse rows. `FileSourceAdapter` remains out of scope until the pipeline preserves raw upstream lineage there.

Status note: On 2026-04-01, the first Task 2 implementation landed and passed local-safe verification, but spec review found two corrections that are now part of the active contract:
- Overture collapse rows must only target overlapping OSM buildings, not previously retained Overture buildings.
- The active file map must keep truth-pack ownership out of `arbx_roblox_export`.

Verification after the review correction:
- `cargo test --manifest-path rust/Cargo.toml -p arbx_pipeline overture_gap_fill_does_not_collapse_against_previously_retained_overture`
- `python3 -m unittest scripts.tests.test_source_truth_pack -v`
- `git diff --check`

**Files:**
- Create: `scripts/source_truth_pack.py`
- Create: `scripts/tests/test_source_truth_pack.py`
- Modify: `scripts/export_austin_from_osm.sh`
- Modify: `scripts/tests/test_austin_fidelity.py`
- Modify: `rust/crates/arbx_cli/src/main.rs`
- Modify: `rust/crates/arbx_pipeline/`

- [x] **Step 1: Refresh the canonical Austin compile artifacts on a clean checkout**

Run:

```bash
bash scripts/export_austin_from_osm.sh --profile high
```

Expected:
- `rust/out/austin-manifest.json` exists
- `rust/out/austin-manifest.sqlite` exists

- [x] **Step 2: Write failing truth-pack contract tests**

Create `scripts/tests/test_source_truth_pack.py` with cases that require:
- per-feature provenance across Overpass-derived retained features and Overture building candidates
- recorded Overture-to-OSM overlap/collapse rows for outdoor building features
- retained vs dropped semantic fields for fields the current adapter code already maps truthfully
- bounded output contract pointing to `arbx_cli compile` writing:
  - `rust/out/<scene>.truth-pack.sqlite`
  - `rust/out/<scene>.truth-pack.summary.json`

Also extend `scripts/tests/test_austin_fidelity.py` to require that the Austin export wrapper drives the new compile truth-pack outputs.

- [x] **Step 3: Run the focused tests to verify they fail**

Run:

```bash
python3 -m unittest scripts.tests.test_source_truth_pack scripts.tests.test_austin_fidelity -v
```

Expected:
- FAIL because the truth-pack extractor/output path does not exist yet

- [x] **Step 4: Implement compile-path truth-pack emission**

Update the compile path so `arbx_cli compile` emits the truth-pack directly from the pre-canonical Overpass/live adapter path. Touch:
- `rust/crates/arbx_cli/src/main.rs`
- the relevant `rust/crates/arbx_pipeline/` plumbing
- `scripts/export_austin_from_osm.sh`

The SQLite schema should minimally include:
- `features`
- `sources`
- `feature_sources`
- `retained_semantics`
- `semantic_lineage`
- `collapses`
- `dropped_semantics`

Keep it bounded and query-oriented. Do not add a giant scene-wide JSON dump.

Support this first slice only for:
- `OverpassAdapter`
- `LiveOverpassAdapter`

Do not invent fake lineage for:
- `FileSourceAdapter`
- synthetic adapters
- generic post-canonical export paths

Add `scripts/source_truth_pack.py` only as a bounded inspection/query helper over the emitted truth-pack outputs, not as a second source-of-truth generation path, and keep ownership out of `arbx_roblox_export`.

- [x] **Step 5: Re-run the focused truth-pack tests**

Run:

```bash
python3 -m unittest scripts.tests.test_source_truth_pack scripts.tests.test_austin_fidelity -v
```

Expected:
- PASS

- [x] **Step 6: Regenerate fresh local truth-pack artifacts for downstream tasks**

Run:

```bash
bash scripts/export_austin_from_osm.sh --profile high
```

Expected:
- `rust/out/austin.truth-pack.sqlite` exists
- `rust/out/austin.truth-pack.summary.json` exists

- [x] **Step 7: Run Rust verification for the touched crates**

Run:

```bash
cargo test --manifest-path rust/Cargo.toml --workspace
```

Expected:
- PASS, including any new source-truth preservation coverage

- [x] **Step 8: Commit**

```bash
git add scripts/source_truth_pack.py \
  scripts/tests/test_source_truth_pack.py \
  scripts/export_austin_from_osm.sh \
  scripts/tests/test_austin_fidelity.py \
  rust/crates/arbx_cli/src/main.rs \
  rust/crates/arbx_pipeline
git commit -m "feat: add bounded outdoor truth-pack"
```

## Task 3: Surface Truth-Pack Findings In The Audits

Status note: On 2026-04-01, Task 3 was split into smaller reviewable slices. Task 3a is the manifest-quality tranche only:
- add `scripts/source_truth_pack_audit.py` as the bounded SQLite + compact-summary reader/auditor
- thread truth-pack findings into `scripts/manifest_quality_audit.py` via the existing summary/findings path
- keep carry-through compact and capped in JSON/HTML
- defer `scene_fidelity_audit.py` and `scene_parity_audit.py` changes to later Task 3 slices

Status note: On 2026-04-01, Task 3a spec review tightened the auditor contract further:
- headline retained/dropped/overlap counts must be scoped to the outdoor families only
- the truth-pack auditor must use bounded aggregate/sample queries instead of materializing full SQLite tables

Status note: On 2026-04-01, Task 3b landed as the scene-audit truth-pack carry-through slice:
- `scene_fidelity_audit.py` now accepts an optional `--truth-pack` seam and reuses `source_truth_pack_audit.py` as the bounded reader
- the carried-through `summary.truthPack` payload is compact and capped: family counts, coverage, capped samples, and compact finding rows only
- `scene_parity_audit.py` now compares that compact truth-pack surface directly
- bounded-preview subset allowances still apply to scene geometry metrics only; truth-pack mismatches remain real parity mismatches
- preview/edit hotspot telemetry remains deferred to a later slice

**Task 3a Files:**
- Create: `scripts/source_truth_pack_audit.py`
- Create: `scripts/tests/test_source_truth_pack_audit.py`
- Modify: `scripts/manifest_quality_audit.py`
- Modify: `scripts/tests/test_manifest_quality_audit.py`

**Later Task 3 Files:**
- Modify: `scripts/scene_fidelity_audit.py`
- Modify: `scripts/scene_parity_audit.py`
- Modify: `scripts/tests/test_scene_fidelity_audit.py`
- Modify: `scripts/tests/test_scene_parity_audit.py`

- [x] **Task 3a Step 1: Write failing truth-pack audit tests**

Create `scripts/tests/test_source_truth_pack_audit.py` and extend:
- `scripts/tests/test_manifest_quality_audit.py`

Cover:
- outdoor overlap-loss findings
- dropped-semantic findings
- retained-semantic findings and counts by outdoor family
- per-family outdoor source coverage for `terrain`, `landuse`, `roads`, `water`, `vegetation`, `structures`

- [x] **Task 3a Step 2: Run the focused audit tests to verify they fail**

Run:

```bash
python3 -m unittest \
  scripts.tests.test_source_truth_pack_audit \
  scripts.tests.test_manifest_quality_audit -v
```

Expected:
- FAIL because the new truth-pack findings and report surfaces do not exist yet

- [x] **Task 3a Step 3: Implement the truth-pack audit and manifest-quality carry-through**

Add `scripts/source_truth_pack_audit.py` and update `scripts/manifest_quality_audit.py` so they can:
- read the SQLite truth-pack and compact summary JSON
- emit explicit outdoor-source findings
- surface truth-pack-backed provenance and collapse summaries in the manifest-quality JSON/HTML report
- keep token usage bounded by default while allowing deeper report slices when requested

Defer scene-fidelity, scene-parity, and hotspot carry-through to later Task 3 slices.

- [x] **Task 3a Step 4: Re-run the focused audit tests**

Run:

```bash
python3 -m unittest \
  scripts.tests.test_source_truth_pack_audit \
  scripts.tests.test_manifest_quality_audit -v
```

Expected:
- PASS

- [x] **Task 3a Step 5: Commit**

```bash
git add scripts/source_truth_pack_audit.py \
  scripts/tests/test_source_truth_pack_audit.py \
  scripts/manifest_quality_audit.py \
  scripts/tests/test_manifest_quality_audit.py
git commit -m "feat: audit outdoor source truth"
```

### Task 3b: Carry Compact Truth-Pack Summary Into Scene Audits

Status: Complete (local-safe). See the 2026-04-01 Task 3b status note in the rolling status file for the verification record.

**Task 3b Files:**
- Modify: `scripts/scene_fidelity_audit.py`
- Modify: `scripts/scene_parity_audit.py`
- Modify: `scripts/tests/test_scene_fidelity_audit.py`
- Modify: `scripts/tests/test_scene_parity_audit.py`

Task 3b coverage:
- carry the bounded truth-pack summary into `scene_fidelity_audit.py` JSON/HTML surfaces
- compare that compact truth-pack surface in `scene_parity_audit.py`
- keep bounded-preview subset allowances limited to scene geometry metrics
- treat truth-pack source-truth mismatches as real mismatches
- defer hotspot telemetry carry-through to a later slice

## Task 4: Add Selective Outdoor Telemetry Flags

Status note: On 2026-04-01, Task 4a1 landed the harness/operator contract slice only. `ARNIS_TELEMETRY_FAMILIES` is now an explicit `scripts/run_studio_harness.sh` contract and the preview summary can surface a requested family subset compactly, but Luau/runtime gating remained deferred to the later Task 4 implementation steps.

Status note: On 2026-04-01, Task 4a2 added the runtime marker-shaping seam. The shared family vocabulary now flows through `WorldProbeTelemetryFlags.lua`, the harness mirrors the requested family list into `Workspace`, and `WorldProbe.client.lua` annotates emitted markers with the canonical `telemetryFamilies` subset when one is requested. `player_local` now emits a deterministic tombstone payload when disabled so stale local-experience lines do not remain authoritative. Only the real heavy/local slices remain gated where data exists.

**Files:**
- Create: `roblox/src/ReplicatedStorage/Shared/WorldProbeTelemetryFlags.lua`
- Create: `roblox/src/ServerScriptService/Tests/WorldProbeTelemetryFlags.spec.lua`
- Modify: `roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua`
- Modify: `scripts/run_studio_harness.sh`
- Modify: `scripts/tests/test_run_studio_harness.py`
- Modify: `scripts/tests/test_austin_runtime_contract.py`

- [x] **Step 1: Write the failing telemetry-flag tests**

Add:
- `roblox/src/ServerScriptService/Tests/WorldProbeTelemetryFlags.spec.lua`
- Python contract coverage in `scripts/tests/test_austin_runtime_contract.py`
- harness argument coverage in `scripts/tests/test_run_studio_harness.py`

Require:
- default compact telemetry behavior
- opt-in signal families for `terrain`, `roads`, `water`, `vegetation`, `structures`, `hotspots`, `player_local`
- stable marker output when only a subset of families is enabled

- [x] **Step 2: Run the local static tests to verify they fail**

Run:

```bash
python3 -m unittest \
  scripts.tests.test_austin_runtime_contract \
  scripts.tests.test_run_studio_harness \
  scripts.tests.test_preview_telemetry_summary -v
```

Expected:
- FAIL because the flag contract and harness wiring do not exist yet

- [x] **Step 3: Implement the minimal flag wiring**

Add `WorldProbeTelemetryFlags.lua` and update:
- `WorldProbe.client.lua`
- `scripts/run_studio_harness.sh`

So deep outdoor telemetry is explicit and selectively enabled instead of always-on. Use one harness env contract for proof and debugging:

```bash
ARNIS_TELEMETRY_FAMILIES=terrain,roads,water,vegetation,structures,hotspots,player_local
```

Preview-summary and Austin preview telemetry changes were not required for this slice; the shared family contract now flows through the harness into Workspace and the client probe reads it directly.

- [x] **Step 4: Re-run the local static tests**

Run:

```bash
python3 -m unittest \
  scripts.tests.test_austin_runtime_contract \
  scripts.tests.test_run_studio_harness \
  scripts.tests.test_preview_telemetry_summary -v
```

Expected:
- PASS

- [x] **Step 5: Verify the focused Luau spec on `tertiary`**

Run on `tertiary`:

```bash
bash scripts/run_studio_harness_remote.sh --remote-profile tertiary -- --help >/tmp/arnis-tertiary-stage-sync.txt
ssh tertiary 'cd ~/.codex-remote-studio/arnis-roblox && ARNIS_TELEMETRY_FAMILIES=terrain,roads,water,vegetation,structures,hotspots,player_local bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter WorldProbeTelemetryFlags.spec.lua --edit-wait 30 --pattern-wait 120'
```

Expected:
- the current worktree snapshot is synced to `~/.codex-remote-studio/arnis-roblox`
- `PASS WorldProbeTelemetryFlags.spec`
- `ARNIS_MCP_EDIT_ACTION total=1 passed=1 failed=0`

- [x] **Step 6: Capture preview hotspot telemetry on `tertiary`**

Run:

```bash
bash scripts/run_studio_harness_remote.sh --remote-profile tertiary -- --help >/tmp/arnis-tertiary-stage-sync.txt
ssh tertiary 'cd ~/.codex-remote-studio/arnis-roblox && ARNIS_TELEMETRY_FAMILIES=terrain,roads,water,vegetation,structures,hotspots,player_local ARNIS_PREVIEW_TELEMETRY_DIR=/tmp bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-wait 30 --pattern-wait 120'
```

Expected:
- `/tmp/arnis-preview-plugin-state.json` exists on `tertiary`
- `/tmp/arnis-preview-telemetry-summary.txt` exists on `tertiary`

- [x] **Step 7: Sync the preview hotspot artifacts back to the local machine**

Run:

```bash
scp tertiary:/tmp/arnis-preview-plugin-state.json /tmp/arnis-preview-plugin-state.json
scp tertiary:/tmp/arnis-preview-telemetry-summary.txt /tmp/arnis-preview-telemetry-summary.txt
```

Expected:
- local `/tmp/arnis-preview-plugin-state.json` exists for offline audit use
- local `/tmp/arnis-preview-telemetry-summary.txt` exists for hotspot target selection without local Studio runs

- [x] **Step 8: Append the proof result to the active status doc**

Add a dated status note to:
- `docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md`

- [ ] **Step 9: Commit any remaining remote-proof/doc follow-up**

```bash
git add roblox/src/ReplicatedStorage/Shared/WorldProbeTelemetryFlags.lua \
  roblox/src/ServerScriptService/Tests/WorldProbeTelemetryFlags.spec.lua \
  roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua \
  roblox/src/ReplicatedStorage/Shared/WorldProbeTerrain.lua \
  roblox/src/ServerScriptService/StudioPreview/AustinPreviewTelemetry.lua \
  scripts/run_studio_harness.sh \
  scripts/preview_telemetry_summary.py \
  scripts/tests/test_run_studio_harness.py \
  scripts/tests/test_austin_runtime_contract.py \
  scripts/tests/test_preview_telemetry_summary.py \
  docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md
git commit -m "feat: add selective outdoor telemetry flags"
```

## Task 5: Align Canonical Outdoor Contract When Required

Status note: On 2026-04-01, the fresh local truth-pack audit plus the synced `tertiary` preview summary did not justify a canonical manifest/schema expansion for the first outdoor tranche. Current pressure remains in truth-pack/audit surfaces, not missing canonical manifest fields, so Task 5 stays on the `no schema change required` path unless later proof shows otherwise.

**Files:**
- Modify when required: `specs/chunk-manifest.schema.json`
- Modify when required: `docs/chunk_schema.md`
- Modify when required: `roblox/src/ReplicatedStorage/Shared/ChunkSchema.lua`
- Modify when required: `scripts/json_manifest_to_sharded_lua.py`
- Modify when required: `scripts/tests/test_json_manifest_to_sharded_lua.py`
- Modify when required: `scripts/tests/test_manifest_quality_audit.py`
- Modify when required: `docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md`

- [x] **Step 1: Decide whether the truth-pack findings require canonical outdoor manifest changes**

Review the outputs from Tasks 2 and 3. If the new audit findings reveal that outdoor fidelity improvements require new canonical outdoor semantics or metadata, record the required contract additions before changing runtime builders. If not, append a status note saying no manifest/schema expansion was required for the first outdoor tranche.

- [ ] **Step 2: Write the failing contract tests when schema changes are required**

If new semantics are required, add focused failing coverage in:
- `scripts/tests/test_json_manifest_to_sharded_lua.py`
- `scripts/tests/test_manifest_quality_audit.py`

and update `specs/chunk-manifest.schema.json` expectations first.

- [ ] **Step 3: Run the focused contract tests**

Run only if Step 1 found a schema gap:

```bash
python3 -m unittest \
  scripts.tests.test_json_manifest_to_sharded_lua \
  scripts.tests.test_manifest_quality_audit -v
```

Expected:
- FAIL before the schema/loader changes exist

- [ ] **Step 4: Implement the schema-first contract update**

If required, update:
- `specs/chunk-manifest.schema.json`
- `docs/chunk_schema.md`
- `roblox/src/ReplicatedStorage/Shared/ChunkSchema.lua`
- `scripts/json_manifest_to_sharded_lua.py`

Keep the change minimal and specific to the outdoor semantics the truth-pack proved were missing.

- [ ] **Step 5: Re-run the focused contract tests**

Run only if Step 1 found a schema gap:

```bash
python3 -m unittest \
  scripts.tests.test_json_manifest_to_sharded_lua \
  scripts.tests.test_manifest_quality_audit -v
```

Expected:
- PASS

- [x] **Step 6: Append the contract decision to the active status doc**

Record whether:
- no canonical manifest change was required, or
- a specific minimal outdoor contract expansion landed

- [ ] **Step 7: Commit**

```bash
git add specs/chunk-manifest.schema.json \
  docs/chunk_schema.md \
  roblox/src/ReplicatedStorage/Shared/ChunkSchema.lua \
  scripts/json_manifest_to_sharded_lua.py \
  scripts/tests/test_json_manifest_to_sharded_lua.py \
  scripts/tests/test_manifest_quality_audit.py \
  docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md
git commit -m "feat: align outdoor manifest contract"
```

Skip this commit if Step 1 concluded no canonical contract change was needed and only the status note changed; fold that doc note into the next commit instead.

## Task 6: Use The New Audits To Drive Outdoor Fidelity And Hotspot Fixes

**Files:**
- Modify: `roblox/src/ReplicatedStorage/Shared/WorldProbeTerrain.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/init.lua`
- Modify: `roblox/src/ServerScriptService/StudioPreview/AustinPreviewBuilder.lua`
- Modify: `roblox/src/ServerScriptService/Tests/WorldProbeTerrain.spec.lua`
- Modify: `scripts/preview_telemetry_summary.py`
- Modify: `scripts/scene_fidelity_audit.py`
- Modify: `scripts/scene_parity_audit.py`
- Modify: `scripts/run_studio_harness.sh`
- Modify: `scripts/tests/test_run_studio_harness.py`
- Modify: `scripts/tests/test_preview_telemetry_summary.py`
- Modify: `scripts/tests/test_play_render_truth.py`
- Modify: `roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua`
- Create: `roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua`
- Modify: `docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md`

Status note: On 2026-04-01, the first bounded Task 6 slice targets explicit hotspot availability for missing/errored preview snapshots and truthful terrain-material richness threading from `TerrainBuilder` through `ImportService` into `AustinPreviewBuilder` and the compact preview summary. `BuildingBuilder.lua` and `AustinPreviewTelemetry.lua` were not required for this slice.

Status note: On 2026-04-01, the second bounded Task 6 slice targets terrain supersampling inside the fixed 4-stud Roblox terrain write path plus stronger hotspot classification/context in the preview summary. The current shared-path implementation is local-safe and verified; only the `tertiary` proof remains open for this slice.

Status note: On 2026-04-01, the current Task 6 tranche was tightened around two measured targets from the fresh truth-pack plus synced `tertiary` preview telemetry:
- terrain/material/detail: player-local terrain telemetry now needs material richness, not just roughness, so edit/play audits can quantify “default grass / textureless” complaints near the player
- hotspot: the preview slow chunk at `-1_0` remains building-dominant (`153/155 ms`), so the compact hotspot surface now needs building-mesh breakdown plus chunk-shape context instead of only coarse `buildingsMs`

Status note: On 2026-04-01, the direct `tertiary` proof path replaced the staged-clone proof path for this slice. The staged remote clone is still not trustworthy for `arnis-roblox/scripts/` completeness, so focused Studio proof now runs against the git-backed `~/Projects/arnis-roblox-main` clone on `tertiary` with `VSYNC_REPO_DIR=$HOME/.codex-remote-studio/vertigo-sync`.

Status note: On 2026-04-01, harness work was explicitly put into wrap-up mode for this tranche. New work should improve outdoor fidelity, hotspot attribution, and truth-pack usefulness; harness changes are limited to stabilization or proof-lane hygiene that unblock `tertiary`, not new harness feature surfaces.

Status note: On 2026-04-01, the current continuation slice expanded the slow-chunk building breakdown and compact truth-pack headline instead of opening a new harness subproject:
- `scripts/source_truth_pack_audit.py` now emits `summary.headline` for the largest outdoor coverage gap, dropped-semantics family, and overlap-loss family
- `BuildingBuilder.lua`, `ImportService/init.lua`, and `AustinPreviewBuilder.lua` now preserve `buildingShellDetailMs` and `buildingInteriorMs`
- `scripts/preview_telemetry_summary.py` now exposes derived building hotspot ratios and residual cost so the current `-1_0` chunk can be reasoned about without a raw log
- proof continues to run only against the persistent `tertiary` repo, not the staged clone

Status note: On 2026-04-01, the current continuation slice now advances the shared outdoor baseline instead of the harness:
- `scripts/export_austin_from_osm.sh` now defaults the shared Austin export path to `high`, and `scripts/export_austin_to_lua.sh` now documents that higher shared default explicitly
- `rust/crates/arbx_pipeline/` now preserves field-level structure semantics from collapsed Overture features into retained OSM buildings and emits `semantic_lineage` rows for merged, identical, and conflict-lost values
- `scripts/source_truth_pack.py` and `scripts/source_truth_pack_audit.py` now surface merged/conflict lineage compactly, so the current structure-heavy truth-pack pressure is no longer just “dropped semantics” with no winner/loser detail
- `BuildingBuilder.lua`, `ImportService/init.lua`, `AustinPreviewBuilder.lua`, and `scripts/preview_telemetry_summary.py` now split `buildingShellDetailMs` into roof/facade/perimeter/terrain-fill/rooftop/name-label subphases and expose the dominant shell-detail phase for the current hotspot

- [x] **Step 1: Use the truth-pack and current audits to name the first outdoor fixes**

Generate a focused report locally from existing seeds/artifacts and record the first two measured targets in the status doc. Use the concrete report command below so target selection stays reproducible:

```bash
python3 scripts/source_truth_pack_audit.py \
  --truth-pack rust/out/austin.truth-pack.sqlite \
  --summary-json rust/out/austin.truth-pack.summary.json \
  --report-json /tmp/arnis-outdoor-truth-pack-report.json
cat /tmp/arnis-preview-telemetry-summary.txt
```

Then choose at minimum:
- one terrain/material/detail target
- one outdoor-heavy hotspot target from `/tmp/arnis-preview-telemetry-summary.txt`, which was synced locally from `tertiary` in Task 4

- [x] **Step 2: Write failing tests for those targets**

Add or extend:
- `scripts/tests/test_play_render_truth.py`
- `scripts/tests/test_preview_telemetry_summary.py`
- `roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua`

Cover:
- the selected terrain/material/detail contract
- the selected hotspot metric/output contract

- [x] **Step 3: Run the focused local tests to verify they fail**

Run:

```bash
python3 -m unittest \
  scripts.tests.test_play_render_truth \
  scripts.tests.test_preview_telemetry_summary -v
```

Expected:
- FAIL on the newly added outdoor-fidelity/hotspot assertions

- [x] **Step 4: Implement the minimal shared fixes**

Update:
- `TerrainBuilder.lua`
- `ImportService/init.lua`
- `AustinPreviewBuilder.lua`
- `BuildingBuilder.lua` only if an outdoor shell artifact is one of the chosen targets
- `AustinPreviewTelemetry.lua`
- `preview_telemetry_summary.py`

Keep fixes shared across edit/play truth. Do not add a play-only patch.

- [x] **Step 5: Re-run the focused local tests**

Run:

```bash
python3 -m unittest \
  scripts.tests.test_play_render_truth \
  scripts.tests.test_preview_telemetry_summary -v
```

Expected:
- PASS

- [x] **Step 6: Prove the selected outdoor fixes on `tertiary`**

Run the narrowest `tertiary` slices that prove the change:

```bash
ssh tertiary 'cd ~/Projects/arnis-roblox-main && ARNIS_TELEMETRY_FAMILIES=terrain,roads,water,vegetation,structures,hotspots,player_local VSYNC_REPO_DIR=$HOME/.codex-remote-studio/vertigo-sync bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter WorldProbeTerrain.spec.lua --edit-wait 30 --pattern-wait 120'
ssh tertiary 'cd ~/Projects/arnis-roblox-main && ARNIS_TELEMETRY_FAMILIES=terrain,roads,water,vegetation,structures,hotspots,player_local VSYNC_REPO_DIR=$HOME/.codex-remote-studio/vertigo-sync bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter TerrainOutdoorFidelity.spec.lua --edit-wait 30 --pattern-wait 120'
scp tertiary:/tmp/arnis-preview-plugin-state.json /tmp/arnis-preview-plugin-state.json
scp tertiary:/tmp/arnis-preview-telemetry-summary.txt /tmp/arnis-preview-telemetry-summary.txt
ssh tertiary 'cd ~/Projects/arnis-roblox-main && ARNIS_TELEMETRY_FAMILIES=terrain,roads,water,vegetation,structures,hotspots,player_local ARNIS_SCENE_AUDIT_DIR=/tmp/arnis-outdoor-audit-play VSYNC_REPO_DIR=$HOME/.codex-remote-studio/vertigo-sync bash scripts/run_studio_harness.sh --takeover --hard-restart --skip-edit-tests --play-wait 30 --pattern-wait 120'
```

Expected:
- `WorldProbeTerrain.spec.lua` passes on `tertiary`
- `TerrainOutdoorFidelity.spec.lua` passes on `tertiary`
- the synced preview summary keeps the richer hotspot fields (`buildingFeatureCount`, `terrainCellCount`, `terrainSubsampleCount`, `dominantCostCenter`, `terrainSignalStatus`)
- the play proof reaches `gameplay_ready`
- raw-log markers show `telemetryFamilies` propagated into play and `ARNIS_CLIENT_LOCAL_EXPERIENCE` carries `localTerrain`

- [x] **Step 7: Append the measured result to the active status doc**

Record:
- what was targeted
- what changed
- the exact `tertiary` proof lane used
- any remaining outdoor hotspot or fidelity gaps revealed by the run

- [ ] **Step 8: Commit**

```bash
git add roblox/src/ServerScriptService/ImportService/Builders/TerrainBuilder.lua \
  roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua \
  roblox/src/ServerScriptService/StudioPreview/AustinPreviewTelemetry.lua \
  scripts/preview_telemetry_summary.py \
  scripts/tests/test_preview_telemetry_summary.py \
  scripts/tests/test_play_render_truth.py \
  roblox/src/ServerScriptService/Tests/TerrainOutdoorFidelity.spec.lua \
  docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md
git commit -m "feat: improve outdoor fidelity and hotspot proof"
```

## Task 7: Run The Full Local-Safe Verification Lane

**Files:**
- Modify only if required by failures from earlier tasks

- [ ] **Step 1: Run the combined Python verification lane**

Run:

```bash
python3 -m unittest \
  scripts.tests.test_source_truth_pack \
  scripts.tests.test_source_truth_pack_audit \
  scripts.tests.test_austin_fidelity \
  scripts.tests.test_manifest_quality_audit \
  scripts.tests.test_scene_fidelity_audit \
  scripts.tests.test_scene_parity_audit \
  scripts.tests.test_austin_runtime_contract \
  scripts.tests.test_run_studio_harness \
  scripts.tests.test_json_manifest_to_sharded_lua \
  scripts.tests.test_play_render_truth \
  scripts.tests.test_preview_telemetry_summary -v
```

Expected:
- PASS

- [ ] **Step 2: Run shell/static verification**

Run:

```bash
bash -n scripts/run_studio_harness.sh
git diff --check
```

Expected:
- PASS

- [ ] **Step 3: Run workspace Rust verification**

Run:

```bash
cargo test --manifest-path rust/Cargo.toml --workspace
```

Expected:
- PASS

- [ ] **Step 4: Commit any last verification-driven fixes**

```bash
git add -A
git commit -m "test: close outdoor fidelity tranche verification gaps"
```

Only do this if verification required a real code or doc fix.

## Task 8: Final `tertiary` Proof And Docs Closeout

**Files:**
- Modify: `docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md`
- Modify: `docs/remote-studio-development.md`

- [ ] **Step 1: Run the final remote proof slices on `tertiary`**

Current remote-lane note as of 2026-04-01:
- the narrowed edit/preview body is green again on `tertiary` with fresh preview telemetry, `ARNIS_MCP_READY`, `ARNIS_MCP_EDIT_ACTION`, and a passing Vertigo Sync plugin smoke check
- the repo-side live-log plugin-smoke stall is fixed, raw `osascript` children are now internally timed out, and the watchdog shutdown path is bounded
- the remaining blocker is now narrower and partly transport-shaped: keep this step in serial debug mode until one direct `tertiary` run records a stable `cleanup finished` / `HARNESS_EXIT:0` transcript without an SSH `255` drop
- continue force-cleaning `tertiary` after each attempt until that transcript is captured

Run the smallest set that proves the tranche end state:

```bash
ssh tertiary 'cd ~/Projects/arnis-roblox-main && python3 -m unittest scripts.tests.test_source_truth_pack_audit scripts.tests.test_scene_fidelity_audit scripts.tests.test_scene_parity_audit scripts.tests.test_austin_runtime_contract -v'
ssh tertiary 'cd ~/Projects/arnis-roblox-main && ARNIS_TELEMETRY_FAMILIES=terrain,roads,water,vegetation,structures,hotspots,player_local ARNIS_SCENE_AUDIT_DIR=/tmp/arnis-outdoor-audit-edit VSYNC_REPO_DIR=$HOME/.codex-remote-studio/vertigo-sync bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter TerrainOutdoorFidelity.spec.lua --edit-wait 30 --pattern-wait 120'
ssh tertiary 'cd ~/Projects/arnis-roblox-main && ARNIS_TELEMETRY_FAMILIES=terrain,roads,water,vegetation,structures,hotspots,player_local ARNIS_PREVIEW_TELEMETRY_DIR=/tmp ARNIS_SCENE_AUDIT_DIR=/tmp/arnis-outdoor-audit-play VSYNC_REPO_DIR=$HOME/.codex-remote-studio/vertigo-sync bash scripts/run_studio_harness.sh --takeover --hard-restart --skip-edit-tests --play-wait 30 --pattern-wait 120'
```

Expected:
- the current verified repo truth is present in `~/Projects/arnis-roblox-main`
- remote static tests pass
- focused edit proof passes
- play proof reaches `gameplay_ready`
- raw proof markers and audit inputs are present for rebuild

- [ ] **Step 2: Rebuild the remote audit artifacts from the proof outputs**

Run on `tertiary` after the proof slice completes:

```bash
ssh tertiary 'cd ~/Projects/arnis-roblox-main && bash scripts/export_austin_from_osm.sh --profile high'
ssh tertiary 'cd ~/Projects/arnis-roblox-main && python3 scripts/source_truth_pack_audit.py --truth-pack rust/out/austin.truth-pack.sqlite --summary-json rust/out/austin.truth-pack.summary.json --report-json /tmp/arnis-outdoor-truth-pack-report.json'
ssh tertiary 'cd ~/Projects/arnis-roblox-main && test -f /tmp/arnis-outdoor-audit-edit/arnis-scene-fidelity-edit.json'
ssh tertiary 'cd ~/Projects/arnis-roblox-main && test -f /tmp/arnis-outdoor-audit-play/arnis-scene-fidelity-play.json'
ssh tertiary 'cd ~/Projects/arnis-roblox-main && python3 scripts/scene_parity_audit.py --edit-report /tmp/arnis-outdoor-audit-edit/arnis-scene-fidelity-edit.json --play-report /tmp/arnis-outdoor-audit-play/arnis-scene-fidelity-play.json --json-out /tmp/arnis-outdoor-audit-play/arnis-scene-parity.json --html-out /tmp/arnis-outdoor-audit-play/arnis-scene-parity.html'
```

If the staged clone lacks ignored compiled outputs, use the committed seed-manifest regeneration path already supported by the harness/report tooling instead of inventing a second artifact flow. If either edit or play artifact is missing, rerun only that proof slice with its dedicated `ARNIS_SCENE_AUDIT_DIR` instead of trying to rediscover the correct Roblox log later.

Expected:
- the truth-pack report is regenerated from the remote proof inputs
- the edit and play scene-fidelity artifacts exist in distinct harness-produced audit dirs
- the scene-parity artifact is regenerated from those distinct edit/play reports
- the resulting audit surfaces demonstrate the selected outdoor fidelity and source-truth gains

- [ ] **Step 3: Update the active status doc with final measured truth**

Add:
- verification snapshot entries
- a final dated status note summarizing the tranche outcome
- any residual gaps that should become the next tranche

- [ ] **Step 4: Tighten the operator doc if the proof lane changed**

Update `docs/remote-studio-development.md` only if the tranche changed how `tertiary` proof should be run or interpreted.

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md \
  docs/remote-studio-development.md
git commit -m "docs: close outdoor fidelity tranche status"
```

## Task 9: Add The First Alpha Runtime Streaming Engine Contract

**Files:**
- Modify: `roblox/src/ReplicatedStorage/Shared/WorldConfig.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/StreamingService.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/ChunkPriority.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/RunAustin.lua`
- Modify: `roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua`
- Modify: `scripts/tests/test_austin_runtime_contract.py`
- Modify: `scripts/tests/test_play_render_truth.py`
- Modify: `scripts/tests/test_run_studio_harness.py`
- Modify: `docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md`

- [ ] **Step 1: Write the failing runtime-engine contract tests**

Extend:
- `scripts/tests/test_austin_runtime_contract.py`
- `scripts/tests/test_play_render_truth.py`
- `scripts/tests/test_run_studio_harness.py`

Require the first alpha contract to expose:
- explicit `near`, `mid`, and `far` runtime rings in `WorldConfig`
- per-ring `estimatedMemoryCost` budgets
- per-ring chunk-count caps
- workspace telemetry for resident estimated memory by ring
- workspace telemetry for resident chunk count by ring
- workspace telemetry for queued and inflight estimated memory
- explicit prefetch/eviction reason surfaces instead of implicit distance-only behavior

- [ ] **Step 2: Run the focused local tests to verify they fail**

Run:

```bash
python3 -m unittest \
  scripts.tests.test_austin_runtime_contract \
  scripts.tests.test_play_render_truth \
  scripts.tests.test_run_studio_harness -v
```

Expected:
- FAIL on the new runtime-engine assertions

- [ ] **Step 3: Implement the first shared runtime-engine slice**

Update:
- `WorldConfig.lua`
- `StreamingService.lua`
- `ChunkPriority.lua` only if needed for prefetch ordering signals
- `RunAustin.lua` only if startup telemetry publication needs a narrow seam
- `WorldProbe.client.lua` only if the runtime telemetry surface needs client-visible carry-through

The first slice should:
- replace implicit two-radius behavior with explicit `near`/`mid`/`far` ring classification
- treat `estimatedMemoryCost` as the authoritative residency budget
- treat chunk-count caps as secondary guardrails
- publish ring-level workspace telemetry every update
- keep the current canonical manifest + loader path intact

Do not add a legacy fallback mode.

- [ ] **Step 4: Re-run the focused local tests**

Run:

```bash
python3 -m unittest \
  scripts.tests.test_austin_runtime_contract \
  scripts.tests.test_play_render_truth \
  scripts.tests.test_run_studio_harness -v
```

Expected:
- PASS

- [ ] **Step 5: Prove the runtime-engine contract on `tertiary`**

Run the narrowest remote slice that proves the new runtime telemetry:

```bash
ssh tertiary 'cd /Volumes/APDataStore/arnis-roblox-proof && ARNIS_TELEMETRY_FAMILIES=terrain,roads,water,vegetation,structures,hotspots,player_local ARNIS_SCENE_AUDIT_DIR=/tmp/arnis-runtime-engine-play bash scripts/run_studio_harness.sh --takeover --hard-restart --skip-edit-tests --play-wait 30 --pattern-wait 120'
```

Expected:
- play reaches `gameplay_ready`
- workspace/runtime markers now include ring-budget telemetry and queue/inflight surfaces
- no change to canonical world truth or parity semantics

- [ ] **Step 6: Append the measured result to the active status doc**

Record:
- the chosen ring contract
- the measured Austin runtime packaging baseline it sits on
- what telemetry now exists
- what the next runtime-engine slice should do (likely movement-aware prefetch/eviction policy)

- [ ] **Step 7: Commit**

```bash
git add roblox/src/ReplicatedStorage/Shared/WorldConfig.lua \
  roblox/src/ServerScriptService/ImportService/StreamingService.lua \
  roblox/src/ServerScriptService/ImportService/ChunkPriority.lua \
  roblox/src/ServerScriptService/ImportService/RunAustin.lua \
  roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua \
  scripts/tests/test_austin_runtime_contract.py \
  scripts/tests/test_play_render_truth.py \
  scripts/tests/test_run_studio_harness.py \
  docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md
git commit -m "feat: add runtime streaming engine budgets"
```
