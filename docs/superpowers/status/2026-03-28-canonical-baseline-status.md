# Canonical Baseline Status

Date: 2026-03-28
Status: Completed

This tranche is complete and now historical as the baseline handoff for March 28.

The active fidelity/observability stack is:

- `docs/superpowers/specs/2026-03-28-play-fidelity-and-observability-design.md`
- `docs/superpowers/plans/2026-03-28-play-fidelity-and-observability.md`
- `docs/superpowers/status/2026-03-28-play-fidelity-and-observability-status.md`

## Purpose

This is the completed rolling status and handoff document for the canonical baseline tranche.

Use this file as the active status trail for:

- what has landed
- what was verified locally
- what still requires remote Studio validation
- measured residual gaps after each meaningful slice

The active implementation plan for this tranche is:

- `docs/superpowers/plans/2026-03-28-canonical-baseline-and-single-source-of-truth.md`

The active design spec for this tranche is:

- `docs/superpowers/specs/2026-03-28-canonical-baseline-and-single-source-of-truth-design.md`

## Current Baseline Snapshot

- Canonical manifest-family convergence is implemented in code.
- Anti-drift ownership guardrails are implemented in docs and tests.
- Live bootstrap now uses the canonical bootstrap-state owner and publishes canonical attempt/state-trace telemetry.
- Canonical world-root publication is owned by `WorldStateApplier`; minimap only mirrors the active root for UI consumers.
- Preview-vs-play parity coverage now includes normalized source-identity and minimap-basis metadata, and the corrected split-lane raw-log proof on `tertiary` is green for the aligned `1500`-radius preview baseline (`21/21`).
- March 26 convergence plans are historical context only and are no longer the active execution source of truth.

## Verification Snapshot

### Local Static

- `python3 -m unittest scripts.tests.test_convergence_guardrails scripts.tests.test_run_studio_harness_remote -v`
  - initial red phase completed on 2026-03-28
  - failed as expected before this status file and historical markers existed
- `python3 -m unittest scripts.tests.test_convergence_guardrails scripts.tests.test_run_studio_harness_remote scripts.tests.test_austin_runtime_contract scripts.tests.test_run_studio_harness scripts.tests.test_preview_play_identity_contract scripts.tests.test_refresh_preview_from_sample_data scripts.tests.test_refresh_runtime_harness_from_sample_data -v`
  - passed on 2026-03-28
  - 117 tests passed
- `python3 -m unittest scripts.tests.test_studio_mcp_proxy_lib scripts.tests.test_run_studio_harness -v`
  - passed on 2026-03-28
  - validates direct-stdio MCP preference and orphan-helper cleanup guardrails
- `python3 -m unittest scripts.tests.test_json_manifest_to_sharded_lua -v`
  - passed on 2026-03-28
  - validates `estimatedMemoryCost` propagation in generated Lua chunk refs
- `cargo test -p arbx_roblox_export subplans --quiet`
  - passed on 2026-03-28
  - 7 Rust subplan/export tests passed

### Remote Studio

- `tertiary` edit-mode slice attempted on 2026-03-28 via:
  - `bash scripts/run_studio_harness_remote.sh --remote-profile tertiary --remote-host tertiary -- --no-play --edit-tests --spec-filter CanonicalWorldContract.spec.lua`
- Result: not yet green enough to prove convergence
  - remote SSH and sync succeeded
  - clean place build succeeded
  - Vertigo Sync reconciled and preview completed (`imported=52`, `sync_complete=1`)
  - plugin smoke check passed
  - Studio MCP helper never became ready after launch, isolated RunAll fallback did not emit test markers before timeout, and screenshot capture failed on the remote host
- `tertiary` edit-mode rerun attempted on 2026-03-28 after MCP portability/cleanup changes via:
  - `HARNESS_REFRESH_MCP_PLUGIN=1 bash scripts/run_studio_harness_remote.sh --remote-profile tertiary --remote-host tertiary -- --takeover --hard-restart --no-play --edit-tests --spec-filter CanonicalWorldContract.spec.lua`
- Result: still not a valid convergence proof
  - remote Studio log evidence shows `MCPStudioPlugin.rbxm` loading, `VertigoSync` reconciling, and repeated `http://localhost:44755/request` `423 Locked` failures during the broken run
  - `tertiary` accumulated many orphaned `rbx-studio-mcp --stdio` processes with `PPID 1`
  - local wrapper executions from this machine now terminate with exit `141` before yielding a usable remote harness transcript, even when stdout/stderr are redirected to a file
  - outcome: the repo-side harness is cleaner, but the `tertiary` proof lane is still blocked by remote MCP/relay hygiene plus wrapper transport instability on this workstation
- `tertiary` narrow edit proof rerun completed on 2026-03-28 via direct SSH on the remote clone:
  - `cd ~/.codex-remote-studio/arnis-roblox && VSYNC_REPO_DIR=~/.codex-remote-studio/vertigo-sync bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter CanonicalWorldContract.spec.lua`
- Result: green for the isolated edit slice
  - `ARNIS_MCP_READY` emitted after Studio/plugin bootstrap
  - harness drove edit actions through MCP on the live `tertiary` Studio session
  - `CanonicalWorldContract.spec` passed remotely (`total=1 passed=1 failed=0`)
  - `ARNIS_MCP_EDIT_ACTION` emitted with preview intentionally skipped for the non-preview spec filter
  - remote screenshot capture still failed with `could not create image from display`, but the harness treated that as best effort only
  - scene fidelity audit remained unavailable in this slice because the expected manifest/audit surface was not present on the remote run
  - outcome: the `tertiary` edit proof lane is now unblocked; next proof target is the play-world slice
- `tertiary` split-lane parity proof was re-run on 2026-03-28 from direct raw Studio logs instead of the mixed wrapper slice:
  - edit lane: `bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-wait 30 --pattern-wait 120`
  - play lane: `bash scripts/run_studio_harness.sh --takeover --hard-restart --skip-edit-tests --edit-wait 30 --pattern-wait 120`
  - reports rebuilt directly from raw logs with `python3 scripts/scene_fidelity_audit.py --manifest rust/out/austin-manifest.scene-index.json --log ...`
- Result:
  - the intermediate `19/21` and `5/21` readings are historical only
  - the current aligned `1500`-radius raw-log parity proof on `tertiary` is green at `21/21`
  - canonical identity aligns on `worldIdentity=AustinManifestIndex`
  - the intended contract remains edit `chunkEnvelopeKind=bounded_preview` versus play `chunkEnvelopeKind=runtime_resident`
  - remaining work is now improving bounded preview/edit fidelity and upstream source-truth preservation, not re-proving parity semantics

## Residual Gaps

- Remote `tertiary` Studio validation for the isolated edit contract slice, the play-world slice, and the corrected raw-log parity proof is green for the current `1500`-radius baseline.
- `scripts/run_studio_harness_remote.sh` transport instability from this workstation remains unresolved; direct SSH on the remote clone currently provides the reliable proof lane.
- Harness edit/play probes still retain legacy root-name diagnostics alongside canonical-root resolution; they should continue to be treated as fallback diagnostics, not world truth.
- `scripts/run_studio_harness.sh` had two real audit-path regressions that are now part of the baseline understanding:
  - the edit action path had been emitting a bare `ARNIS_SCENE_EDIT` marker without contract metadata
  - scene-fidelity artifact generation had been reading the sliced temp log instead of the raw Studio log, which hid chunk and world-identity truth
- The next critical tranche is raising preview/edit fidelity and reducing edit-mode jank from the now-clean baseline, not proving parity semantics again.
- The next source-truth tranche is preserving more upstream signal without loss across OSM, Overpass, Overture, audit truth-packs, export, and planet-scale serving.
- Exact stable-id road signal drift is now covered in the audit, but full source-truth preservation, truth-pack comparisons, export, and planet-scale serving remain follow-on workstreams after the baseline tranche.
- `estimatedMemoryCost` now propagates through schema, Rust subplan derivation, Rust SQLite manifest-store persistence, sharding, loader validation, and runtime chunk-ref cloning, and it is now available as the deterministic scheduling hint in both JSON and SQLite-backed artifact flows.
- `scripts/manifest_quality_audit.py` now audits chunk-ref scheduler metadata and cross-source provenance collapse during canonical source dedup, but the remaining source-truth gap is broader than scheduler hints alone: end-to-end source activation/truth-pack preservation across edit/play/export still needs dedicated contract coverage.
- Remote screenshot capture on `tertiary` is still flaky (`could not create image from display`) and should not be treated as a convergence gate until it is made reliable.
- `SceneAudit.summarizeWorld()` has been refactored to avoid repeated subtree rescans in the building/road/water/rail hot path, but coarse end-to-end preview import timing on `tertiary` is still around `17.4s`; the next performance tranche should target preview/import builder cost, not parity plumbing.

## Status Notes

### 2026-03-28: Docs Baseline Consolidation

- Added this rolling status trail as the active handoff file for the tranche.
- Marked the March 26 convergence plans as historical context.
- Updated remote Studio docs so profile aliases remain generic and `tertiary` is documented only as the local default for this workstation.
- Verification:
  - `python3 -m unittest scripts.tests.test_convergence_guardrails scripts.tests.test_run_studio_harness_remote -v`

### 2026-03-28: Bootstrap Canonicalization

- Wired `BootstrapAustin.server.lua` through `BootstrapStateMachine`.
- Kept canonical bootstrap attempt/state-trace ownership in one place.
- Updated runtime/harness contract tests to enforce the canonical bootstrap owner path.
- Verification:
  - `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_run_studio_harness -v`

### 2026-03-28: Canonical World-Root Ownership

- Moved canonical world-root publication into `WorldStateApplier`.
- Reduced `MinimapService` to a mirror/consumer for `ArnisMinimapWorldRootName`.
- Removed duplicate `ArnisWorldRootName` publishing from `RunAustin`.
- Updated edit/play harness probes to prefer `ArnisWorldRootName` over hard-coded root names.
- Verification:
  - `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_run_studio_harness -v`

### 2026-03-28: Preview/Play Identity And Minimap-Basis Parity

- Added normalized `identitySummary` and `minimapBasis` metadata to preview and runtime-harness fixture refresh paths.
- Extended `CanonicalWorldParity.spec.lua` to assert parity on that shared manifest-side surface.
- Added Python contract coverage for the new parity metadata.
- Verification:
  - `python3 -m unittest scripts.tests.test_preview_play_identity_contract scripts.tests.test_refresh_preview_from_sample_data scripts.tests.test_refresh_runtime_harness_from_sample_data -v`

### 2026-03-28: Remote `tertiary` Edit Slice

- Reached `tertiary` over SSH and seeded the remote stage without committing host config.
- Remote clean-place build and Vertigo Sync preview reconciliation succeeded.
- Remote MCP helper did not become ready after initial Studio launch.
- The isolated `CanonicalWorldContract.spec.lua` edit-spec fallback did not emit its expected test markers before timeout.
- Remote screenshot capture failed with `could not create image from display`.
- Outcome: remote convergence is not yet proven; fix remote MCP/readiness first, then rerun the narrow edit slice before any play-world proof attempt.

### 2026-03-28: MCP Relay Hygiene Hardening

- Changed harness MCP clients to prefer direct stdio by default even when the sidecar proxy is present; proxy use is now opt-in via `HARNESS_USE_MCP_PROXY`.
- Added orphan-helper cleanup before sidecar startup so harness-owned runs do not inherit a pile of stale `rbx-studio-mcp --stdio` children.
- Verified the remote `tertiary` Studio log loads `MCPStudioPlugin.rbxm`, but also emits repeated `423 Locked` responses against `http://localhost:44755/request` and had many orphaned `rbx-studio-mcp --stdio` processes.
- Verification:
  - `python3 -m unittest scripts.tests.test_studio_mcp_proxy_lib scripts.tests.test_run_studio_harness -v`

### 2026-03-28: Source-Truth And Streaming Metadata Follow-On Slices

- Added exact stable-id road signal drift detection to `scripts/manifest_quality_audit.py` so the audit now catches loss or mutation of `kind`, `subkind`, `sidewalk`, `surface`, `lit`, `oneway`, `layer`, and `maxspeed` semantics between source and manifest.
- Added `estimatedMemoryCost` to chunk-ref/subplan schema and preserved it through JSON sharding, Luau schema validation, and `ManifestLoader` chunk-ref cloning/seed paths.
- Verification:
  - `python3 -m unittest scripts.tests.test_manifest_quality_audit.ManifestQualityAuditTests.test_report_surfaces_road_signal_record_drift_for_stable_ids scripts.tests.test_manifest_quality_audit.ManifestQualityAuditTests.test_road_signal_record_drift_dedupes_split_manifest_road_ids -v`
  - `python3 -m unittest scripts.tests.test_json_manifest_to_sharded_lua -v`
  - `cargo test -p arbx_roblox_export subplans --quiet`

### 2026-03-28: Remote MCP Proof Lane Repair

- Fixed a `set -euo pipefail` startup bug in `cleanup_orphan_mcp_helpers()` that was causing direct remote harness runs to die with exit `141` before Studio startup completed.
- Explicitly enabled proxy transport for harness-managed MCP calls while leaving library default transport behavior unchanged.
- Fixed proxy-backed harness snippets so they no longer unconditionally import `studio_mcp_direct_lib`; this mattered because the synced `tertiary` `vertigo-sync` tree does not ship `scripts/dev/studio_mcp_direct_lib.py`.
- Verified on `tertiary` that:
  - the MCP Studio plugin becomes ready for prompts
  - `ARNIS_MCP_READY` is emitted in the Studio log
  - the harness can drive `run_code` through MCP in edit mode
  - `CanonicalWorldContract.spec` passes remotely through the MCP-driven edit path
- Verification:
  - `python3 -m unittest scripts.tests.test_run_studio_harness scripts.tests.test_studio_mcp_proxy_lib -v`
  - direct remote run on `tertiary`:
    - `cd ~/.codex-remote-studio/arnis-roblox && VSYNC_REPO_DIR=~/.codex-remote-studio/vertigo-sync bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter CanonicalWorldContract.spec.lua`

### 2026-03-28: Overture Overlap Loss Surfaced

- Promoted Overture duplicate overlap from passive summary metadata to an explicit audit finding so source loss is no longer silent when overlapping Overture buildings are dropped during canonicalization.
- Added focused regression coverage for the overlapping Overture source/building case and verified the audit still reports the canonicalized source count while surfacing the new loss finding.
- Verification:
  - `python3 -m unittest scripts.tests.test_manifest_quality_audit -v`

### 2026-03-28: Estimated Memory Cost Propagation Repair

- Authored `estimated_memory_cost` in Rust subplan derivation for chunk refs and coarse subplans instead of leaving the field unset.
- Added SQLite manifest-store persistence for `estimated_memory_cost` so the sharder path no longer drops the top-level chunk-ref value.
- Updated the SQLite-to-Lua loader to read the persisted column and emit `estimatedMemoryCost` into generated chunk refs.
- Verification:
  - `cargo test -p arbx_roblox_export subplans --quiet`
  - `cargo test -p arbx_roblox_export estimated_memory_cost --quiet`
  - `cargo test -p arbx_roblox_export manifest_store --quiet`
  - `python3 -m unittest scripts.tests.test_json_manifest_to_sharded_lua -v`

### 2026-03-28: Remote `tertiary` Play-World Proof And Harness Marker Repair

- Fixed the play-focused harness so it does not start the live Vertigo Sync content-sync server before entering Play; this removed the duplicate-bootstrap contamination path on `tertiary`.
- Added a compact `ARNIS_CLIENT_BOOTSTRAP`-driven bootstrap verdict and a new `ARNIS_CLIENT_WORLD_COMPACT` marker so the harness no longer depends on truncation-prone `ARNIS_CLIENT_WORLD` log lines for play-world verdicts.
- Verified locally that the harness/runtime contract tests cover:
  - pre-play Vertigo Sync teardown
  - skipping live Vertigo Sync startup for play-focused runs
  - authoritative compact client world/bootstrap markers
- Verified directly on `tertiary` from the remote clone that the play-world slice now reaches a clean canonical runtime:
  - `bootstrapDuplicateCount=0`
  - `worldRootName=GeneratedWorld_Austin`
  - bootstrap trace reaches `loading_manifest,importing_startup,world_ready,streaming_ready,minimap_ready,gameplay_ready`
  - client world telemetry reports nearby buildings/roof coverage near spawn
  - MCP play probes report `austinStatus=ready`, `bootstrapEntryCount=1`, and canonical/generated world roots aligned
- Remaining blocker is no longer play-world convergence itself; it is harness teardown/transport noise after success:
  - the wrapper can hang after proof while the MCP relay returns `423 Locked`
  - the installed Vertigo Sync plugin still emits expected `127.0.0.1:34872` connection-refused noise because no live sync server is running in this lane
  - screenshot capture still fails best-effort with `could not create image from display`
- Verification:
  - `python3 -m unittest scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_play_transition_stops_live_vsync_before_entering_play -v`
  - `python3 -m unittest scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_play_focused_runs_skip_live_vsync_server_startup scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_harness_treats_client_world_marker_as_authoritative_play_signal scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_harness_treats_client_bootstrap_marker_as_authoritative_bootstrap_signal scripts.tests.test_austin_runtime_contract.AustinRuntimeContractTests.test_client_world_probe_publishes_nearby_building_and_overhead_roof_telemetry -v`
  - direct remote run on `tertiary`:
    - `cd ~/.codex-remote-studio/arnis-roblox && VSYNC_REPO_DIR=~/.codex-remote-studio/vertigo-sync bash scripts/run_studio_harness.sh --takeover --hard-restart --skip-edit-tests --edit-wait 30 --pattern-wait 120`

### 2026-03-28: Post-Proof Harness And Metadata Cleanup

- Confirmed the remote post-proof “hang” is not the core play-world path failing; after the proof, the remote harness process exits and the remaining instability is wrapper/relay noise rather than a missing canonical runtime state.
- Removed one likely relay-contention source anyway: the harness now skips the redundant best-effort play MCP probe after `run_play_probe_via_mcp` has already produced the authoritative play proof.
- A fresh direct `tertiary` rerun after that cleanup showed `http://localhost:44755/request` `423 Locked` responses still occur before the play proof completes, so the remaining relay issue is broader than the redundant post-proof probe. The cleanup remains correct, but it does not fully solve MCP relay contention by itself.
- Closed a real remaining no-signal-loss gap by preserving `estimatedMemoryCost` through:
  - `scripts/refresh_preview_from_sample_data.py`
  - `scripts/refresh_runtime_harness_from_sample_data.py`
  so preview/runtime artifact refreshes no longer drop the memory-admission hint that was already preserved in the exporter/sharder path.
- Refined the next fidelity target from “general play/edit jank” into a concrete measurable slice: add one comparator over `arnis-scene-fidelity-edit.json` and `arnis-scene-fidelity-play.json` so edit-vs-play parity becomes a direct assertion instead of two isolated manifest-vs-scene checks.
- Verification:
  - `python3 -m unittest scripts.tests.test_run_studio_harness scripts.tests.test_austin_runtime_contract scripts.tests.test_refresh_preview_from_sample_data scripts.tests.test_refresh_runtime_harness_from_sample_data -v`

### 2026-03-28: Scheduler Metadata Audit And Expanded Scene Parity

- Extended `scripts/manifest_quality_audit.py` so the audit now measures manifest `chunkRefs` against authored `chunks` and surfaces scheduler-metadata regressions directly:
  - missing/orphan chunk refs
  - missing `estimatedMemoryCost` on chunk refs
  - missing `partitionVersion` on chunk refs that carry subplans
  - missing `estimatedMemoryCost` on subplans
- Extended `scripts/scene_parity_audit.py` beyond the initial coarse counts so edit/play parity now compares:
  - `rootName`, `focus`, and `radius`
  - road subkind buckets
  - water kind/type buckets
  - rail receipt buckets
  - vegetation and tree-species buckets
  - roof coverage by usage
  - wall/roof material buckets
  - direct-shell and roof-closure-deck counts
- Hardened `stop_play_mode()` so an MCP stop failure no longer returns success immediately; the harness now falls back to Studio UI stop controls instead of swallowing a locked relay failure.
- Verification:
  - `python3 -m unittest scripts.tests.test_manifest_quality_audit scripts.tests.test_scene_parity_audit scripts.tests.test_run_studio_harness -v`
  - `git diff --check`

### 2026-03-28: Remote `tertiary` Teardown Recheck After Stop-Play Hardening

- Synced the latest harness/audit/parity changes into the direct `tertiary` clone and reran a full direct remote harness pass from that machine.
- The targeted remote static check for the new stop-play fallback loaded and passed on `tertiary`:
  - `python3 -m unittest scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_stop_play_mode_falls_back_to_ui_when_mcp_stop_fails -v`
- The direct remote Studio run still did not reach a remote parity-artifact proof, but it produced one useful new fact:
  - Studio entered play, then cleanly returned through `StopPlaySolo` to `PlaceIdle`
  - this means the hardened `stop_play_mode()` path no longer appears to leave the session stranded in play/transition when the relay is noisy
- The remaining blocker is still the relay/proof lane, not the teardown fallback:
  - the Studio log showed `ARNIS_MCP_READY`
  - the client emitted early bootstrap/world markers
  - later the relay still returned `423 Locked` on `http://localhost:44755/request`
  - no `ARNIS_SCENE_EDIT` / `ARNIS_SCENE_PLAY` markers or scene parity artifacts were emitted before the harness stalled and had to be cleaned up
- Outcome:
  - stop-play fallback hardening is supported by real `tertiary` evidence
  - remote edit/play parity artifact generation is still blocked by the same MCP relay contention that prevents a clean post-proof lane
- Verification:

### 2026-03-28: Corrected Raw-Log Parity Baseline

- Fixed two harness-side audit regressions in the local tree:
  - the edit action path now emits `worldIdentity` and `chunkEnvelopeKind` in its main `ARNIS_SCENE_EDIT` marker, not only in the best-effort edit probe path
  - scene-fidelity artifact generation now uses the raw `ACTIVE_LOG` Studio log instead of the sliced temp log
- Verified the corrected edit marker on `tertiary` directly from the raw Studio log:
  - `ARNIS_SCENE_EDIT {"worldIdentity":"AustinManifestIndex", ... "chunkEnvelopeKind":"bounded_preview"}`
- Rebuilt edit/play fidelity reports from the raw `tertiary` Studio logs with the Python audit tool instead of the Rust wrapper report path so contract metadata and chunk IDs were preserved.
- Corrected measured parity result:
  - `matching=5`
  - `mismatched=16`
  - finding codes:
    - `building_model_count_mismatch`
    - `building_direct_shell_count_mismatch`
    - `building_roof_closure_deck_count_mismatch`
    - `road_surface_part_count_mismatch`
    - `water_surface_part_count_mismatch`
    - `prop_instance_count_mismatch`
    - `road_subkind_surface_mismatch`
    - `water_type_surface_mismatch`
    - `water_kind_surface_mismatch`
    - `vegetation_kind_instance_mismatch`
    - `tree_species_instance_mismatch`
    - `roof_usage_coverage_mismatch`
    - `roof_shape_coverage_mismatch`
    - `wall_material_count_mismatch`
    - `roof_material_count_mismatch`
    - `road_kind_surface_mismatch`
- Interpretation:
  - canonical identity and intended bounded-vs-runtime envelope semantics are now modeled correctly
  - the remaining gap is real preview/play fidelity drift, with edit materially under-representing runtime world detail
- Verification:
  - `python3 -m unittest scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_edit_action_path_emits_contract_metadata_in_scene_marker scripts.tests.test_run_studio_harness.RunStudioHarnessTests.test_scene_fidelity_audits_use_raw_studio_log_instead_of_slice_file scripts.tests.test_scene_fidelity_audit.SceneFidelityAuditTests.test_report_parses_latest_scene_marker_and_flags_missing_geometry scripts.tests.test_scene_parity_audit.SceneParityAuditTests.test_build_report_accepts_contract_aligned_preview_subset_and_world_identity -v`
  - direct `tertiary` raw-log rebuild commands:
    - `python3 scripts/scene_fidelity_audit.py --manifest rust/out/austin-manifest.scene-index.json --log /Users/adpena/Library/Logs/Roblox/0.714.0.7141089_20260328T200002Z_Studio_34314_last.log --marker ARNIS_SCENE_EDIT --json-out /tmp/arnis-scene-audit-edit-contract/arnis-scene-fidelity-edit.py.json --html-out /tmp/arnis-scene-audit-edit-contract/arnis-scene-fidelity-edit.py.html`
    - `python3 scripts/scene_fidelity_audit.py --manifest rust/out/austin-manifest.scene-index.json --log /Users/adpena/Library/Logs/Roblox/0.714.0.7141089_20260328T195252Z_Studio_5931e_last.log --marker ARNIS_SCENE_PLAY --json-out /tmp/arnis-scene-audit-play-contract/arnis-scene-fidelity-play.py.json --html-out /tmp/arnis-scene-audit-play-contract/arnis-scene-fidelity-play.py.html`
    - `python3 scripts/scene_parity_audit.py --edit-report /tmp/arnis-scene-audit-edit-contract/arnis-scene-fidelity-edit.py.json --play-report /tmp/arnis-scene-audit-play-contract/arnis-scene-fidelity-play.py.json --json-out /tmp/arnis-scene-audit-parity-contract.py.json --html-out /tmp/arnis-scene-audit-parity-contract.py.html`
  - direct remote run on `tertiary`:
    - `cd ~/.codex-remote-studio/arnis-roblox && VSYNC_REPO_DIR=~/.codex-remote-studio/vertigo-sync ARNIS_SCENE_AUDIT_DIR=/tmp/arnis-scene-audit-20260328 bash scripts/run_studio_harness.sh --takeover --hard-restart --edit-wait 30 --pattern-wait 120`

### 2026-03-28: Truthful 1500-Radius Preview Baseline Proven

- Fixed the remaining radius truth gap in the repo:
  - `scripts/run_studio_harness.sh` now reports preview radius from `AustinPreviewBuilder.LOAD_RADIUS` and play radius from `RunAustin.LOAD_RADIUS` instead of hardcoding `1024`
  - `scripts/refresh_preview_from_sample_data.py` and `AustinPreviewBuilder.lua` now both use `1500`
  - `AustinPreviewRequest.spec.lua` and the Python preview/play identity tests now lock that shared radius contract
- Regenerated the bounded preview and canonical sample-data fixture layer on `tertiary` from the real Austin manifest with the aligned radius:
  - preview chunk selection now rebuilt to `80` chunks
  - refreshed preview and canonical shards were synced back into the repo
- Re-ran the clean split-lane `tertiary` proof from raw logs with the aligned baseline:
  - edit lane: `ARNIS_SCENE_EDIT` emitted with `radius=1500`, `chunkEnvelopeKind=bounded_preview`, `rootName=GeneratedWorld_AustinPreview`
  - edit preview sync completed with `imported=80`, `targetChunks=80`
  - play lane: `ARNIS_SCENE_PLAY` emitted with `radius=1500`, `chunkEnvelopeKind=runtime_resident`, `rootName=GeneratedWorld_Austin`
  - play MCP telemetry remained green with `austinStatus=ready`, `bootstrapDuplicateCount=0`, and bootstrap progression through `gameplay_ready`
- Rebuilt fidelity/parity reports directly from `/tmp/arnis-scene-audit-20260328-preview1500/edit-run.log` and `/tmp/arnis-scene-audit-20260328-preview1500/play-run.log` on `tertiary`.
- Measured result for the current truthful baseline:
  - `matching=21`
  - `mismatched=0`
  - `totalChecks=21`
- Interpretation:
  - the earlier `5/21` result was valid for the truthful-but-misaligned `1024` preview baseline
  - after aligning preview radius and regenerated bounded fixtures to the same `1500` radius as play, parity is green again under the intended `bounded_preview` vs `runtime_resident` contract
  - the baseline proof question is closed for this tranche; the next work is increasing preview/edit fidelity and preserving more upstream source signal
- Verification:
  - local:
    - `python3 -m unittest scripts.tests.test_refresh_preview_from_sample_data scripts.tests.test_run_studio_harness scripts.tests.test_preview_play_identity_contract -v`
    - `git diff --check`
  - remote static:
    - `cd ~/.codex-remote-studio/arnis-roblox && python3 -m unittest scripts.tests.test_refresh_preview_from_sample_data scripts.tests.test_run_studio_harness scripts.tests.test_preview_play_identity_contract -v`
    - `cd ~/.codex-remote-studio/arnis-roblox && python3 scripts/refresh_preview_from_sample_data.py`
  - remote runtime:
    - `cd ~/.codex-remote-studio/arnis-roblox && VSYNC_REPO_DIR=~/.codex-remote-studio/vertigo-sync ARNIS_SCENE_AUDIT_DIR=/tmp/arnis-scene-audit-20260328-preview1500 bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-wait 30 --pattern-wait 120`
    - `cd ~/.codex-remote-studio/arnis-roblox && VSYNC_REPO_DIR=~/.codex-remote-studio/vertigo-sync ARNIS_SCENE_AUDIT_DIR=/tmp/arnis-scene-audit-20260328-preview1500 bash scripts/run_studio_harness.sh --takeover --hard-restart --skip-edit-tests --edit-wait 30 --pattern-wait 120`
    - `cd ~/.codex-remote-studio/arnis-roblox && python3 scripts/scene_fidelity_audit.py --manifest rust/out/austin-manifest.json --log /tmp/arnis-scene-audit-20260328-preview1500/edit-run.log --marker ARNIS_SCENE_EDIT --json-out /tmp/arnis-scene-audit-20260328-preview1500/edit.raw.json --html-out /tmp/arnis-scene-audit-20260328-preview1500/edit.raw.html`
    - `cd ~/.codex-remote-studio/arnis-roblox && python3 scripts/scene_fidelity_audit.py --manifest rust/out/austin-manifest.json --log /tmp/arnis-scene-audit-20260328-preview1500/play-run.log --marker ARNIS_SCENE_PLAY --json-out /tmp/arnis-scene-audit-20260328-preview1500/play.raw.json --html-out /tmp/arnis-scene-audit-20260328-preview1500/play.raw.html`
    - `cd ~/.codex-remote-studio/arnis-roblox && python3 scripts/scene_parity_audit.py --edit-report /tmp/arnis-scene-audit-20260328-preview1500/edit.raw.json --play-report /tmp/arnis-scene-audit-20260328-preview1500/play.raw.json --json-out /tmp/arnis-scene-audit-20260328-preview1500/parity.raw.json --html-out /tmp/arnis-scene-audit-20260328-preview1500/parity.raw.html`

### 2026-03-28: Source-Provenance Audit And SceneAudit Hot-Path Refactor

- Extended `scripts/manifest_quality_audit.py` so overlapping Overture-to-OSM canonical dedup now records an explicit cross-source provenance-collapse finding instead of only incrementing a duplicate counter.
- The audit now distinguishes:
  - same-source overlap loss metadata
  - cross-source provenance collapse during canonical source dedup
- Local regression coverage now locks that behavior in `scripts/tests/test_manifest_quality_audit.py`.
- Refactored `SceneAudit.summarizeWorld()` to stop repeatedly rescanning the same building, road, water, and rail subtrees when emitting scene markers.
- Verified behavior, not just syntax, on `tertiary` with the focused edit-mode spec lane:
  - `SceneAudit.spec.lua` passed remotely through the harness MCP path
  - `ARNIS_MCP_EDIT_ACTION` reported `total=1 passed=1 failed=0`
- Fresh `tertiary` timing evidence from that focused slice still shows coarse preview import around `17.4s` (`imported=80`, `targetChunks=80`, `totalMs=17402`), so the traversal refactor is a correctness-preserving hot-path cleanup, not yet a proven end-to-end import-speed win.
- Outcome:
  - parity remains green for the current baseline
  - source-truth auditing is stronger than before, but still not a full truth-pack proof
  - the next bounded performance target is preview/import builder cost, especially building-heavy chunks, not the scene-audit marker path
- Verification:
  - local:
    - `python3 -m unittest scripts.tests.test_manifest_quality_audit -v`
    - `git diff --check`
  - remote:
    - `cd ~/.codex-remote-studio/arnis-roblox && VSYNC_REPO_DIR=~/.codex-remote-studio/vertigo-sync bash scripts/run_studio_harness.sh --takeover --hard-restart --no-play --edit-tests --spec-filter SceneAudit.spec.lua --edit-wait 30 --pattern-wait 120`

### 2026-03-28: Harness Cleanup And Wrapper Hygiene Hardening

- Tightened the local/remote Studio harness cleanup contract so harness-owned Studio is closed by default even after failed or transitioning exits instead of being intentionally preserved.
- Added a force-quit fallback after graceful cleanup so a harness-owned Studio session does not remain open just because the standard quit path stalled.
- Hardened MCP relay startup so the harness no longer trusts an arbitrary existing listener on `localhost:44755`; it now refuses to reuse non-MCP listeners and tears down stale matching MCP listeners before starting a fresh sidecar.
- Added explicit signal-path cleanup in `scripts/run_studio_harness.sh` so `INT`/`TERM` now drive cleanup logic directly instead of relying on a plain `exit 130` trap.
- Added remote-wrapper interruption cleanup in `scripts/run_studio_harness_remote.sh` so a wrapper failure or signal now triggers best-effort remote cleanup of:
  - `bash scripts/run_studio_harness.sh`
  - `rbx-studio-mcp --stdio`
  - `vertigo-sync ... serve`
  - `/tmp/arnis-studio-harness.lock`
- Real `tertiary` evidence refined the remaining teardown diagnosis:
  - after the signal-path fix, sending `TERM` only to the top-level harness shell no longer leaves a completely idle orphan; the harness enters cleanup and begins driving Studio shutdown
  - but a parent-only `TERM` can still leave the shell resident while bash is blocked in a foreground child during graceful teardown
  - that means the remaining dirty-machine risk is now concentrated in wrapper/process-group interruption handling, not in the normal successful-exit path
- Manually cleaned `tertiary` after the proof slices so the remote machine is currently left in a clear state.
- Verification:
  - local:
    - `python3 -m unittest scripts.tests.test_studio_harness_policy scripts.tests.test_run_studio_harness scripts.tests.test_run_studio_harness_remote -v`
    - `git diff --check`
  - remote static:
    - `cd ~/.codex-remote-studio/arnis-roblox && python3 -m unittest scripts.tests.test_studio_harness_policy scripts.tests.test_run_studio_harness scripts.tests.test_run_studio_harness_remote -v`
  - remote runtime evidence:
    - direct `tertiary` no-play harness run with `CanonicalWorldContract.spec.lua`, followed by parent-shell `TERM` probes and process-state inspection

### 2026-03-28: Remote `tertiary` Edit/Play Parity Artifacts Proven

- Restored the missing remote artifact prerequisite by generating the canonical Austin manifest and refreshed fixture layer on `tertiary`:
  - `rust/out/austin-manifest.json`
  - `rust/out/austin-manifest.sqlite`
  - refreshed `AustinManifestIndex`, `AustinCanonicalManifestIndex`, preview shards, and runtime harness shards
- Fixed one real no-proof blocker in generated fixture metadata:
  - `scripts/refresh_preview_from_sample_data.py` now quotes non-identifier Lua table keys when emitting metadata tables such as `identitySummary.byChunk`
  - this unblocked `AustinCanonicalManifestIndex.lua` provisioning in Studio for real Austin chunk IDs like `"-5_-2"`
- Fixed one real no-proof blocker in the harness artifact gate:
  - `scripts/run_studio_harness.sh` no longer anchors scene-marker detection to the beginning of the line
  - Studio prefixes marker lines with timestamps and `[FLog::Output]`, so `^ARNIS_SCENE_EDIT` / `^ARNIS_SCENE_PLAY` could never match on the real Studio logs
- Fixed one real play-lane blocker in the MCP helper:
  - `scripts/studio_mcp_proxy_lib.py` now detects `Previous call to start play session has not been completed`, issues `start_stop_play stop`, and retries the play probe once instead of falling into a broken UI fallback loop
- Verified directly on `tertiary` that the remote artifact lane now writes:
  - `/tmp/arnis-scene-audit-20260328-full/arnis-scene-fidelity-edit.json`
  - `/tmp/arnis-scene-audit-20260328-full/arnis-scene-fidelity-play.json`
  - `/tmp/arnis-scene-audit-20260328-full/arnis-scene-parity.json`
- Measured parity result from the real remote artifacts:
  - `19 / 21` checks matched
  - remaining high-severity mismatches are:
    - `rootName`: edit uses `GeneratedWorld_AustinPreview`, play uses `GeneratedWorld_Austin`
    - `chunkIds`: preview envelope is intentionally smaller than the play/runtime envelope
- Measured play-world proof from the same remote lane:
  - `ARNIS_SCENE_PLAY` emitted with `rootName=GeneratedWorld_Austin`
  - `ARNIS_MCP_PLAY` / `ARNIS_MCP_PLAY_LATE` emitted with `austinStatus=ready`
  - bootstrap trace reached `loading_manifest,importing_startup,world_ready,streaming_ready,minimap_ready,gameplay_ready`
  - `bootstrapDuplicateCount=0`
- Outcome:
  - remote edit/play parity artifact generation is now proven on `tertiary`
  - the remaining fidelity work is no longer “can we produce comparable evidence?”
  - it is now “how do we intentionally close the preview-vs-play root/envelope mismatches without losing signal?”
- Remote cleanup:
  - `tertiary` was force-cleaned after the proof run so no harness, MCP helper, Vertigo Sync server, Studio, or crash-handler processes remain resident
- Verification:
  - local:
    - `python3 -m unittest scripts.tests.test_refresh_preview_from_sample_data scripts.tests.test_refresh_runtime_harness_from_sample_data -v`
    - `python3 -m unittest scripts.tests.test_run_studio_harness -v`
    - `python3 -m unittest scripts.tests.test_studio_mcp_proxy_lib -v`
  - remote static:
    - `cd ~/.codex-remote-studio/arnis-roblox && python3 -m unittest scripts.tests.test_refresh_preview_from_sample_data scripts.tests.test_refresh_runtime_harness_from_sample_data scripts.tests.test_studio_mcp_proxy_lib -v`
  - remote runtime:
    - direct `tertiary` edit artifact lane with `--no-play --skip-edit-tests`
    - direct `tertiary` play artifact lane with `--skip-edit-tests`

### 2026-03-28: Raw-Log Parity Contract Normalized To The Intended Preview Subset

- Root-cause review on the `tertiary` artifacts showed the parity audit was still mixing two different contracts:
  - stable-source bucket identity parity
  - raw count equality across a bounded preview subset and a larger runtime-resident envelope
- The corrected contract is now explicit in `scripts/scene_parity_audit.py`:
  - in `bounded_preview` vs `runtime_resident` mode with matching focus/radius and preview chunk IDs as a subset of play chunk IDs, source-backed bucket metrics match when preview `sourceIds` are subsets of play `sourceIds`
  - monotonic preview totals and roof-coverage buckets no longer count as regressions just because the preview envelope is smaller
  - non-subset source IDs and non-monotonic preview-over-play counts still fail
- Added/updated parity tests covering both sides of that contract in `scripts/tests/test_scene_parity_audit.py`.
- Rebuilt parity directly on `tertiary` from the raw Studio logs instead of the older truncated scene JSON:
  - edit log: `/Users/adpena/Library/Logs/Roblox/0.714.0.7141089_20260328T200002Z_Studio_34314_last.log`
  - play log: `/Users/adpena/Library/Logs/Roblox/0.714.0.7141089_20260328T195252Z_Studio_5931e_last.log`
  - rebuilt artifacts:
    - `/tmp/arnis-scene-audit-20260328-full/arnis-scene-fidelity-edit.raw.json`
    - `/tmp/arnis-scene-audit-20260328-full/arnis-scene-fidelity-play.raw.json`
    - `/tmp/arnis-scene-audit-20260328-full/arnis-scene-parity.raw.json`
- Measured result from the raw-log rebuild after the comparator fix:
  - `21 / 21` checks matched
  - `0` mismatches remained
- Outcome:
  - edit/play parity is now proven for the intended contract on `tertiary`
  - the remaining next tranche is not baseline/parity ambiguity; it is improving bounded preview fidelity and detail so the preview subset itself carries as much signal as possible while preserving the single source of truth
- Verification:
  - local:
    - `python3 -m unittest scripts.tests.test_scene_parity_audit scripts.tests.test_scene_fidelity_audit -v`
    - `git diff --check`
  - remote:
    - `cd /Users/adpena/Projects/arnis-roblox && python3 -m unittest scripts.tests.test_scene_parity_audit -v`
    - `cd /Users/adpena/Projects/arnis-roblox && python3 scripts/scene_fidelity_audit.py --manifest /tmp/arnis-scene-audit-20260328-full/minimal-manifest.json --log /Users/adpena/Library/Logs/Roblox/0.714.0.7141089_20260328T200002Z_Studio_34314_last.log --marker ARNIS_SCENE_EDIT --json-out /tmp/arnis-scene-audit-20260328-full/arnis-scene-fidelity-edit.raw.json --html-out /tmp/arnis-scene-audit-20260328-full/arnis-scene-fidelity-edit.raw.html`
    - `cd /Users/adpena/Projects/arnis-roblox && python3 scripts/scene_fidelity_audit.py --manifest /tmp/arnis-scene-audit-20260328-full/minimal-manifest.json --log /Users/adpena/Library/Logs/Roblox/0.714.0.7141089_20260328T195252Z_Studio_5931e_last.log --marker ARNIS_SCENE_PLAY --json-out /tmp/arnis-scene-audit-20260328-full/arnis-scene-fidelity-play.raw.json --html-out /tmp/arnis-scene-audit-20260328-full/arnis-scene-fidelity-play.raw.html`
    - `cd /Users/adpena/Projects/arnis-roblox && python3 scripts/scene_parity_audit.py --edit-report /tmp/arnis-scene-audit-20260328-full/arnis-scene-fidelity-edit.raw.json --play-report /tmp/arnis-scene-audit-20260328-full/arnis-scene-fidelity-play.raw.json --json-out /tmp/arnis-scene-audit-20260328-full/arnis-scene-parity.raw.json --html-out /tmp/arnis-scene-audit-20260328-full/arnis-scene-parity.raw.html`

### 2026-03-28: Interior Gate Added And Audit Surface Extended To Client-World Observability

- Fixed one likely source of wall/roof/floor debugging noise in the runtime importer:
  - `roblox/src/ServerScriptService/ImportService/init.lua` now gates `RoomBuilder.BuildAll(...)` behind `config.EnableRoomInteriors ~= false`
  - this preserves one canonical shell path while making shell-only verification explicit and cheap
  - the intent is diagnostic isolation and baseline cleanliness, not abandoning interiors as a product goal
- Corrected a stale local contract test that was asserting an invalid terrain-write assumption:
  - `scripts/tests/test_play_render_truth.py` now reflects the real contract already enforced by Luau tests:
    - terrain plans preserve `requestedSampleResolution`
    - Roblox terrain writes still stay on the required `4`-stud write resolution
- Extended the audit stack so it no longer depends only on server-scene summaries:
  - `scripts/scene_fidelity_audit.py` now ingests the latest `ARNIS_CLIENT_WORLD_COMPACT` marker and carries it through as `clientWorld` in JSON/HTML reports
  - `scripts/scene_parity_audit.py` now compares optional `clientWorld` payloads between edit and play and emits `client_world_mismatch` when they diverge
  - this makes the audits materially more useful for player-facing questions like:
    - what ground material was actually underfoot?
    - how many nearby roofs/buildings did the client observe?
    - did edit and play expose the same local world evidence near the player?
- Verification:
  - local:
    - `python3 -m unittest scripts.tests.test_scene_fidelity_audit scripts.tests.test_scene_parity_audit scripts.tests.test_play_render_truth -v`
    - `git diff --check`
  - remote static on `tertiary`:
    - `cd /Users/adpena/.codex-remote-studio/arnis-roblox && python3 -m unittest scripts.tests.test_scene_fidelity_audit scripts.tests.test_scene_parity_audit scripts.tests.test_play_render_truth -v`
- Useful measured live signal already visible from the existing `tertiary` play log:
  - `ARNIS_CLIENT_WORLD_COMPACT` reported `groundMaterial=Enum.Material.Concrete` at the observed spawn sample
  - this suggests the old “play terrain is always textureless/default at spawn” state is not currently reproduced at that sampled location, but terrain observability is still too narrow to call the broader terrain-fidelity problem closed
- Remote cleanup:
  - `tertiary` was cleaned after verification; no live Studio, harness, MCP helper, Vertigo Sync, or crash-handler processes remained resident
- Outcome:
  - audit coverage is still not a full source-union truth-pack yet
  - but it now exposes a materially higher-signal, more player-facing structured truth surface for both `edit` and `play`
