# osm2world Building Geometry Port Status

Date: 2026-04-09
Status: Active

## Purpose

This is the only active rolling status and handoff document in `docs/superpowers/`.

The active design spec is:

- `docs/superpowers/specs/2026-04-09-osm2world-building-geometry-port-design.md`

The active implementation plan is:

- `docs/superpowers/plans/2026-04-09-osm2world-building-geometry-port.md`

The compact historical archive index is:

- `docs/superpowers/archive-index.md`

## Current Snapshot

- The April 9 osm2world building-geometry tranche is now the repo's sole active superpowers truth stack.
- The March 30 and April 6 superpowers doc stacks were fully implemented, folded into `docs/superpowers/archive-index.md`, and deleted.
- The March 28 canonical baseline status remains the only retained completed rolling handoff for foundational convergence context.
- The active tranche goal is unchanged: port osm2world's building geometry algorithms into Rust at 1:1 parity with only a narrow manifest handshake extension where needed.
- The tranche now carries one explicit runtime/exporter contract addition: optional `building.roofIncluded` marks when a precomputed `shellMesh` already contains roof geometry so Lua can skip duplicate roof generation.

## Verification Snapshot

### Local Static

- Verification for the April 9 docs-governance reset is captured in the 2026-04-09 status note below.

## Residual Gaps

- The implementation work described by the April 9 spec and plan is still ahead; this file currently establishes the governance and handoff surface only.
- Future implementation slices must append dated status notes here after meaningful debugging, testing, or remote proof work.

## Status Notes

### 2026-04-09: Superpowers Truth Stack Reset

- Created this rolling status file so the April 9 tranche has the required active status surface.
- Deleted the redundant March 30 and April 6 superpowers spec/plan/status files after preserving their high-signal outcomes in `docs/superpowers/archive-index.md`.
- Updated the docs guardrail test and the minimal operator docs that otherwise would have kept dead links into deleted superpowers files.
- Verification:
  - `python3 -m unittest scripts.tests.test_convergence_guardrails -v`
  - passed on 2026-04-09

### 2026-04-09: roofIncluded Exporter/Importer Contract

- Extended the manifest contract with optional `building.roofIncluded` so the Rust exporter can explicitly mark roof-inclusive `shellMesh` payloads.
- Updated the Lua importer to skip explicit runtime roof generation only when `roofIncluded == true` and a valid precomputed `shellMesh` is present, preserving backward compatibility for older walls-only mesh manifests.
- Updated schema, runtime contract tests, and chunk-schema validation for the new field.
- Verification:
  - `python3 -m unittest scripts.tests.test_austin_runtime_contract -v`
  - `cargo test --manifest-path rust/Cargo.toml -p arbx_roblox_export`
  - both passed on 2026-04-09

### 2026-04-09: Live Streaming Hardening + Remote Proof Diagnostics

- Pulled current live telemetry from Cloudflare and confirmed real movement/stationary churn instead of a wrapper-only problem. The worst `success_post_walk` sample carried `chunkThrashCount=37`, `movingThrashCount=16`, `stationaryThrashCount=21`, and large near/mid ring delta peaks.
- Hardened the live runtime path to default to `production_server` streaming settings outside Studio while preserving `local_dev` behavior for Studio/harness sessions.
- Reduced visible residency churn in `StreamingService` by:
  - deriving ring/building LOD from actual player distance instead of scheduler lookahead distance
  - only reimporting resident chunks upward in building detail, not downward
  - deferring resident chunk eviction on ring-nil / chunk-limit pressure to the existing cooldown sweep instead of unloading immediately inside the candidate loop
- Fixed the remote play probe failure path in `scripts/run_studio_harness.sh` so a non-zero probe exit no longer trips `set -e` before status classification.
- Synced the remote harness stdout log into the local artifact directory from `scripts/run_studio_harness_remote.sh`, eliminating the previous evidence gap when wrapper runs failed.
- Added startup-envelope recognition for roof-inclusive merged shell meshes by stamping `ArnisImportRoofIncluded` on building models and treating nearby merged shell meshes as roof evidence when appropriate. This aligns the new `roofIncluded` mesh path with the existing startup readiness contract.
- Fresh targeted remote proof still exits non-zero, but now leaves usable evidence:
  - synced stdout log: `/tmp/arnis-remote-studio-hardening-proof/arnis-remote-harness.stdout.log`
  - synced screenshot: `/tmp/arnis-studio-harness-swift-20260409-112004.png`
  - current blocker remains parity/startup-envelope failure: the run imports all 26 startup chunks but does not reach `streaming_ready` / `gameplay_ready`, and the screenshot still shows a broken sky-dominant runtime view.
- Verification:
  - `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_run_studio_harness scripts.tests.test_run_studio_harness_remote -v`
  - `bash -n scripts/run_studio_harness.sh scripts/run_studio_harness_remote.sh`
  - `git diff --check`
  - `bash scripts/auto_loop.sh`
  - `ARNIS_REMOTE_STUDIO_ARTIFACT_DIR=/tmp/arnis-remote-studio-hardening-proof bash scripts/run_studio_harness_remote.sh --swift-screenshot -- --takeover --skip-edit-tests --play-wait 35 --pattern-wait 180 --screenshot /tmp/arnis-studio-harness.png`

### 2026-04-09: Runtime Import Crash Fix + Screenshot Authority Hardening

- Root-caused the remote `small-place` proof failure to runtime import, not manifest fetch. The harness log showed `RunAustin` loading the manifest and resolving the Austin anchor successfully, then dying on `part.DoubleSided = true` with `The current thread cannot write 'DoubleSided' (lacking capability Plugin)`.
- Hardened both building and road mesh builders to treat `DoubleSided` as best-effort in non-plugin runtime threads. The import now warns once per builder family instead of aborting the entire bootstrap.
- Preserved failure signal in `BootstrapAustin.server.lua`: uncaught runtime/import failures now surface as `Austin import failed — bootstrap halted.` instead of being misclassified as missing manifests.
- Reduced per-frame traversal telemetry churn by making `SharedState.publishClientCameraTelemetry` change-driven and switching the ready flag to `setPlayerAttributeIfChanged(...)` instead of unconditional `player:SetAttribute(...)` inside the render loop.
- Reworked the remote GUI screenshot relay to call `scripts/studio_ui_control.py capture-screenshot` from the logged-in Terminal session instead of blindly running whole-display `screencapture -x`. The relay now preserves the inner sidecar method and window/session diagnostics.
- Tightened `run_studio_harness_remote.sh` so a synced play screenshot only counts as authoritative when the sidecar reports `success=true` and `capture_method` of `window` or `rect`. Whole-display relay output remains diagnostic evidence, not parity proof.
- Updated `AGENTS.md` and `docs/remote-studio-development.md` to match that stricter proof contract.
- Fresh targeted remote proof on `tertiary` now clears the critical runtime blocker:
  - `RunAustin` imports the startup manifest successfully: `chunks=26 roads=295 buildings=64 props=90`
  - bootstrap reaches `gameplay_ready`
  - synced play screenshot sidecar reports `capture_method="rect"` with `guiSessionRelay.method="terminal.command"`
- Remaining proof gap after this slice:
  - the first harness `play world verdict (authoritative client)` still reports zero nearby buildings/roofs even though later `ARNIS_CLIENT_WORLD(_COMPACT)` markers during the same run show non-zero nearby building, roof, and wall counts. That is now a proof-timing/verdict issue, not an import/runtime crash.
- Verification:
  - `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_gui_session_capture scripts.tests.test_run_studio_harness scripts.tests.test_run_studio_harness_remote -v`
  - `bash -n scripts/run_studio_harness.sh scripts/run_studio_harness_remote.sh`
  - `git diff --check`
  - `ARNIS_REMOTE_STUDIO_ARTIFACT_DIR=/tmp/arnis-remote-studio-proof-v2 bash scripts/run_studio_harness_remote.sh --swift-screenshot -- --small-place --takeover --skip-edit-tests --play-wait 130 --pattern-wait 240 --screenshot /tmp/arnis-studio-harness.png`

### 2026-04-09: Authoritative World Verdict Salvage + WorldProbe Scan Trim

- Root-caused the remaining authoritative play-world verdict bug to harness parsing, not runtime telemetry. `ARNIS_CLIENT_WORLD_COMPACT` late-play lines in the Studio log are sometimes truncated mid-JSON because the payload carries long nearest-building name/source-id arrays; the old harness parser skipped those malformed lines and fell back to the last short parseable payload, which was often an early `world_ready` zero-building sample.
- Hardened `scripts/run_studio_harness.sh` in two ways:
  - `summarize_log()` now uses `ACTIVE_LOG` for authoritative client verdict extraction and keeps `LOG_SLICE_FILE` only for the human-readable tail summary.
  - `log_effective_play_world_state()` now prefers `gameplay_ready` client-world markers and can salvage the key proof fields (`worldRootName`, `worldRootExists`, `nearbyBuildingModels`, `nearbyRoofParts`, `overheadRoofParts`, `bootstrapState`) directly from truncated marker text when JSON decoding fails.
- Confirmed that the new parser recovers the correct settled verdict from the real failing remote log:
  - old behavior selected `world_ready` with `nearbyBuildingModels=0`
  - new behavior recovers `gameplay_ready` with non-zero nearby building/roof counts from the same `*_last.log`
- Landed one bounded runtime perf reduction in `WorldProbe.client.lua`:
  - added squared-radius constants for nearby-building / named-building / wall / overhead-roof checks
  - removed `Vector2.new(...).Magnitude` allocation churn from the deep structure scan while preserving emitted rounded distance fields where payloads still need them
- Current residual issue after this slice is now remote harness lifecycle on `tertiary`, not proof parsing:
  - one fresh `proof-v4` wrapper run returned non-zero without syncing the final authoritative harness artifacts back locally
  - direct remote inspection showed leftover run-specific harness processes (`PGID 29281`) still alive after wrapper exit; they were terminated manually to restore a clean proof surface
- Verification:
  - `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_gui_session_capture scripts.tests.test_run_studio_harness scripts.tests.test_run_studio_harness_remote -v`
  - `bash -n scripts/run_studio_harness.sh scripts/run_studio_harness_remote.sh`
  - `git diff --check`
  - local replay of the salvaged-world parser against `/tmp/arnis-remote-studio-proof-v3/0.716.0.7160873_20260409T172359Z_Studio_b79f6_last.log`

### 2026-04-09: Remote Wrapper Lifecycle Recovery + Minimap Heading Repaint Split

- Root-caused the `proof-v4` remote wrapper failure to lifecycle/state management in `scripts/run_studio_harness_remote.sh`, not another runtime or proof-parser regression:
  - the wrapper could treat remote status as `missing` if the PGID file disappeared before the exit file landed
  - on non-zero wrapper exit, it disarmed `REMOTE_HARNESS_ACTIVE` too early, so trap cleanup could not reap orphaned remote harness shells
- Hardened the remote wrapper by:
  - extending `remote_harness_status()` to recognize `running_orphaned` via targeted stage-local process discovery when PGID/exit files are temporarily absent
  - extending `stop_remote_harness_if_active()` to kill targeted orphaned stage-local harness shells even when the PGID file is gone
  - forcing `stop_remote_harness_if_active()` before non-zero wrapper exit so bad status transitions cannot strand remote harness processes
- Fresh `proof-v5` confirms the wrapper/harness/proof stack now works end to end on a clean `tertiary` surface:
  - authoritative client bootstrap trace: `valid`
  - authoritative client world verdict: `worldRootExists=True nearbyBuildingModels=6 nearbyRoofParts=41 overheadRoofParts=8`
  - authoritative play screenshot sidecar: `capture_method="rect"` with `guiSessionRelay.method="terminal.command"`
  - wrapper exits cleanly after the bounded cleanup tail
  - post-run remote process check shows no remaining stage-local harness shells beyond the operator's current `pgrep` check
- Landed a separate client-performance slice in `MinimapController.client.lua`:
  - cached the north-up base raster in a second buffer
  - split base-map rerender invalidation from heading-only overlay refresh
  - removed full chunk reraster on heading-only camera turns while preserving the north-up map contract and rotating player-heading overlay
- Verification:
  - `python3 -m unittest scripts.tests.test_minimap_runtime_contract -v`
  - `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_gui_session_capture scripts.tests.test_run_studio_harness scripts.tests.test_run_studio_harness_remote -v`
  - `bash -n scripts/run_studio_harness.sh scripts/run_studio_harness_remote.sh`
  - `git diff --check`
  - `ARNIS_REMOTE_STUDIO_ARTIFACT_DIR=/tmp/arnis-remote-studio-proof-v5 bash scripts/run_studio_harness_remote.sh --swift-screenshot -- --small-place --takeover --skip-edit-tests --play-wait 130 --pattern-wait 240 --screenshot /tmp/arnis-studio-harness.png`

### 2026-04-09: Ambient Tag Cache Hardening

- Reduced recurring client ambience overhead in `AmbientSoundscape.client.lua` by replacing per-tick `CollectionService:GetTagged(...)` scans with cached `Road` and water-surface part sets.
- The cache is now:
  - seeded once at startup from the tagged sets
  - maintained incrementally through `GetInstanceAddedSignal(...)` / `GetInstanceRemovedSignal(...)`
  - cleaned opportunistically when cached parts lose their parent
- This keeps the near-road / near-water behavior unchanged while removing repeated tagged-array allocations from the `0.3s` ambience loop.
- Verification:
  - `python3 -m unittest scripts.tests.test_ambient_soundscape_runtime_contract -v`
  - `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_ambient_soundscape_runtime_contract scripts.tests.test_minimap_runtime_contract scripts.tests.test_gui_session_capture scripts.tests.test_run_studio_harness scripts.tests.test_run_studio_harness_remote -v`
  - `bash -n scripts/run_studio_harness.sh scripts/run_studio_harness_remote.sh`
  - `git diff --check`

### 2026-04-09: Ambient Footstep Throttle + WorldProbe Micro-Trim

- Landed one more bounded ambience slice:
  - footstep surface-material raycasts in `AmbientSoundscape.client.lua` are now throttled to `0.12s`
  - per-frame play/stop decisions and playback-speed updates remain live, so movement responsiveness is preserved while removing unnecessary every-frame ground-material probes
- Landed one more small WorldProbe micro-optimization without changing markers or payload shape:
  - removed an unused `math.sqrt` from the building descendant scan
  - tightened roof-closure detection to reuse the already-lowercased part name path
- Verification:
  - `python3 -m unittest scripts.tests.test_play_audio_assets scripts.tests.test_ambient_soundscape_runtime_contract -v`
  - `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_ambient_soundscape_runtime_contract scripts.tests.test_minimap_runtime_contract scripts.tests.test_play_audio_assets scripts.tests.test_gui_session_capture scripts.tests.test_run_studio_harness scripts.tests.test_run_studio_harness_remote -v`
  - `bash -n scripts/run_studio_harness.sh scripts/run_studio_harness_remote.sh`
  - `git diff --check`

### 2026-04-09: Play-Focused Proof Fast-Path Recovery

- Root-caused the next proof-lane stall in the play-focused harness path:
  - the shell could miss the immediate client-proof fast path and then fall through into probe paths that were intended only as fallback
  - the later `run_probe_best_effort "play" 8` call was still running under `set -e`, so a non-zero best-effort probe could kill the main harness shell while the background Studio-log pipe kept streaming, leaving no summary or cleanup signal
- Hardened `scripts/run_studio_harness.sh` by:
  - adding a short `wait_for_authoritative_client_play_proof` poll before deciding to invoke any play probe in the play-focused branch
  - making the fallback best-effort play probe genuinely best-effort with an explicit continue-on-failure log path
  - preserving the new client-proof fast path so play-focused runs can skip both MCP and best-effort probes once the required client markers are present
- Fresh `proof-v9` on `tertiary` now confirms the branch behaves correctly end to end:
  - `skipping play-mode MCP probe because authoritative client proof is already present`
  - `skipping redundant play MCP probe after successful authoritative play proof`
  - authoritative client world verdict: `worldRootExists=True nearbyBuildingModels=6 nearbyRoofParts=41 overheadRoofParts=8`
  - authoritative client perf verdict: `avgFrameTimeMs=21.28 p99FrameTimeMs=107.85 maxFrameTimeMs=139.49 fps=47`
  - authoritative play screenshot sidecar: `capture_method="rect"`
  - harness reaches `main harness flow complete; exiting` and `cleanup starting exit_code=0`
  - post-run remote process check shows no remaining stage-local harness shells
- Verification:
  - `python3 -m unittest scripts.tests.test_austin_runtime_contract scripts.tests.test_ambient_soundscape_runtime_contract scripts.tests.test_minimap_runtime_contract scripts.tests.test_play_audio_assets scripts.tests.test_gui_session_capture scripts.tests.test_run_studio_harness scripts.tests.test_run_studio_harness_remote -v`
  - `bash -n scripts/run_studio_harness.sh scripts/run_studio_harness_remote.sh`
  - `git diff --check`
  - `ARNIS_REMOTE_STUDIO_ARTIFACT_DIR=/tmp/arnis-remote-studio-proof-v9 bash scripts/run_studio_harness_remote.sh --swift-screenshot -- --small-place --takeover --skip-edit-tests --play-wait 130 --pattern-wait 240 --screenshot /tmp/arnis-studio-harness.png`
