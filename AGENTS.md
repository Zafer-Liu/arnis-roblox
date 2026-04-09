# AGENTS.md

This file is the operating manual for coding agents working in this repository.

## Mission

Build a performant, chunked Roblox world importer/export pipeline inspired by Arnis, while keeping
the data compiler and the Roblox runtime/editor responsibilities separate.

## Non-negotiable architecture rules

1. **Offline compile, online import**
   - External geodata retrieval belongs in Rust-side tooling or a server-side pipeline.
   - Roblox runtime/editor code consumes already-compiled manifests.

2. **Schema before behavior**
   - Manifest and config schema changes must be reflected in `specs/` first.
   - The repo treats `0.4.0` as the only supported manifest schema; older manifests fail fast.

3. **Chunk everything**
   - New systems must identify their chunk ownership explicitly.
   - No global, unbounded scene generation helpers.

4. **Idempotent imports**
   - Importing the same manifest/chunk twice should overwrite or reconcile, not duplicate.

5. **Performance beats ornament**
   - Prefer terrain, merged representations, and pooled instances.
   - Delay high-detail assets and interiors until shell import is stable and benchmarked.

6. **Deterministic output**
   - The same source input and config should produce the same manifest and equivalent scene graph.

## Current state (post-HD Pipeline)

The pipeline is complete and demo-ready. All builders are production-quality:

- **Schema 0.4.0 only**; older manifests are rejected at import time
- **ElevationEnrichmentStage** — DEM-derived Y for all features
- **EditableMesh merging** for buildings and roads
- **26 surface physics types** with real-world friction coefficients
- **5-phase day/night cycle** with lerped atmospheric transitions
- **25+ prop types**, **20+ building materials**, **25+ tree species**
- **Car + jetpack + parachute** gameplay with full physics and sound
- **Live minimap**, **loading screen**, **ambient soundscape**
- **Worldwide support** — any lat/lon bbox, auto-downloads elevation

## Sequence of work for agents

1. Read `docs/chunk_schema.md` for the manifest contract.
2. Read `roblox/src/ReplicatedStorage/Shared/WorldConfig.lua` for all config knobs.
3. Use `arbx_cli explain` for the full pipeline architecture.
4. Use `arbx_cli compile --help` for CLI options.
5. Run `cargo test --workspace` in `rust/` to verify the pipeline.
6. Builders are in `roblox/src/ServerScriptService/ImportService/Builders/`.
7. Gameplay is in `roblox/src/StarterPlayer/StarterPlayerScripts/` — specifically `VehicleController.client.lua` is the MONOLITH for all traversal systems (car, jetpack, parachute, wingsuit, grapple). Do NOT create sibling controllers; they will collide on key bindings. Edit in place or split+delete in the same commit.

## Automated iteration loop

- One command runs the full build → publish → remote-harness → telemetry → audit cycle: `bash scripts/auto_loop.sh`.
- This is the primary way to iterate without asking the user to rejoin the live game. Flip it after any runtime change and read the final `auto-loop finished: ... audit=PASS total=Xs` line.
- Producer: `TelemetryReporter.lua` POSTs bootstrap stats (phases, chunk fetch latency, import counts, errors) to Cloudflare on success/failure.
- Transport: `https://planetary.adpena.workers.dev/telemetry/run` (POST) + `GET /telemetry/latest?limit=N`.
- Consumer: `scripts/fetch_telemetry.py` (pretty-print / JSON / watch) and `scripts/live_stream_audit.py` (assertions + exit codes 0/1/2/3).
- Remote harness runs on tertiary (8GB M1 via Tailscale). `run_studio_harness_remote.sh` already handles rsync, sentinel-encoded ssh args, and SSD-redirected cargo target via `CARGO_TARGET_DIR=/Volumes/APDataStore/arnis/remote-studio/cargo-target`.

## Runtime footguns — MUST NOT regress

1. **Roblox `HttpService:RequestAsync` restricted headers**: never set `Accept-Encoding`, `User-Agent`, `Host`, `Origin`, `Referer`, `Content-Length`, `Transfer-Encoding`, `Connection`, `Via`. Engine throws `Header "X" is not allowed!` and aborts. Only `Accept`, `Content-Type`, custom `X-*`.
2. **`chunk.originStuds`** is a JSON table with **lowercase** `{x, y, z}` — not a Vector3. All builders must use `local ox = originStuds.X or originStuds.x or 0` (dual-case fallback). Search for this pattern before adding a new builder.
3. **Lua/Luau closure scope**: locals are only visible AFTER declaration. A closure defined earlier in the same block that references a later-declared local resolves it as nil (upvalue/global lookup). Declare locals BEFORE closures that need them.
4. **`ssh host bash -s -- "$@"`** drops empty-string positional args. Use the `__EMPTY__` sentinel encode/decode helpers already in `run_studio_harness_remote.sh`.
5. **`set -euo pipefail` + `grep` inside `$(...)`** silently kills the script when grep finds nothing. Every grep command substitution must `|| true` inside.
6. **Never `source ~/.zshrc` from bash** — zsh-only syntax crashes bash and the error trips `set -e` before `|| true` runs. Use `zsh -c 'source ~/.zshrc; printf "%s" "$VAR"'` to extract env vars.
7. **`ChunkSchema.validateManifest`** now accepts manifests with only `chunkRefs` (no inline `chunks`) so lazy streaming sources aren't forced through the embedded fallback. Do not add an `assert(#manifest.chunks > 0)` back.

## Non-Negotiable Execution Policy

- Default to multi-hour autonomous execution behavior whenever substantial work remains.
- Work in long uninterrupted bursts and batch multiple related product slices into each turn instead of stopping after one neat local checkpoint.
- Keep changes internally bounded and reviewable, but do not stop reporting back after every small fix if the broader bundled burndown is still progressing.
- Use minimal worker orchestration; prefer a few well-bounded workers over large swarms.
- Proactively clean stale Codex and `rbx-studio-mcp` processes when they create file-descriptor pressure or destabilize the session.
- Do not stop for convenience. Only stop for a real blocker, an explicit safety constraint, or when remote proof on `tertiary` is the next required step.
- Do not emit tranche summaries after every small fix. Keep going until a substantial bundled burndown is complete.

## Change discipline

For every meaningful code change:

- update or add a test if there is a harness for that area
- update docs if the contract changed
- keep exactly one active `docs/superpowers/` spec, one active plan, and one active rolling status file for the whole repo; all other superpowers docs must be marked `Historical` or `Completed`
- if a spec or implementation plan is active, append a dated status note after any meaningful debugging/verification slice that changes the next agent's understanding, especially after remote Studio runs
- keep remote Studio host aliases, usernames, and machine-specific paths in ignored local config or env, never in committed scripts
- treat `primary` and `tertiary` as local profile aliases only; direct development may happen on either machine, and the committed repo must not depend on a specific hostname or pre-seeded sibling clones
- avoid introducing new dependencies without a concrete payoff
- prefer small, reviewable steps over giant speculative rewrites
- never eagerly load known large artifacts into memory; avoid `Path.read_text()`, `json.load()`, `json.loads()`, or whole-file slurps on multi-MB/GB manifests when a bounded-memory path exists
- for large-file inspection, prefer shard/index metadata, streaming parsers, mmap-backed extraction, `rg -m`, `head`, `tail`, or other bounded reads over full scans that materialize the entire file
- when defining new large intermediate/export formats, prefer chunked/indexed layouts and queryable containers such as SQLite or Parquet over monolithic JSON blobs
- add telemetry or explicit guardrails before any dev/test workflow can plausibly exceed roughly 4 GB resident memory; fail early with a clear error instead of risking OOM
- zero per-frame allocations in render loops
- all lerps must be dt-scaled (frame-rate independent)
- all sounds must fade (no audio pops)
- all UI transitions must use TweenService (no snaps)

## Convergence guardrails

- `arnis-roblox` owns canonical world truth, manifest semantics, and scene extraction adapters.
- `vertigo-sync` owns edit/full-bake orchestration and export-3d user-facing orchestration.
- Do not add new parallel preview/play/full-bake world-definition paths in `RunAustin.lua`, `AustinPreviewBuilder.lua`, `BootstrapAustin.server.lua`, or `AustinSpawn.lua`.
- If this boundary changes, update `scripts/tests/test_convergence_guardrails.py` in the same change.

## Roblox-specific guardrails

- Studio plugin code must remain optional.
- Runtime modules should not depend on plugin-only APIs.
- Keep anything that mutates `Workspace.GeneratedWorld` behind an importer or chunk loader service.
- When a feature is not ready, fail loudly with a TODO and a clear message.

## Studio harness capabilities

- Prefer `scripts/run_studio_harness.sh` and `scripts/run_studio_harness_remote.sh` for Studio proof work instead of ad hoc desktop automation.
- The harness already supports:
  - clean-place or fresh-template launch
  - edit-only and play-mode runs
  - `--spec-filter` isolated Luau spec execution
  - edit/play screenshots via `--screenshot` with phase-specific outputs such as `/tmp/arnis-studio-harness-edit.png` and `/tmp/arnis-studio-harness-play.png`
  - MCP probes and ordered bootstrap/play telemetry capture
  - preview telemetry artifacts and scene fidelity/parity audits
  - remote artifact sync back to the local machine through `scripts/run_studio_harness_remote.sh`
- When you need a visual proof on a remote Studio machine, use the harness screenshot/artifact path first. Do not treat raw SSH `screencapture` as the authoritative capture lane when the harness can capture and sync the image itself.
- Preferred screenshot workflow:
  - local proof: `scripts/run_studio_harness.sh --play --screenshot --artifact-dir /tmp/arnis-studio-harness`
  - remote proof: `scripts/run_studio_harness_remote.sh --host <alias> --play --screenshot --artifact-dir /tmp/arnis-remote-studio`
  - edit-only screenshot: add `--edit-only`
  - isolated repro: add `--spec-filter <SpecName>`
- Screenshot artifacts now include a sibling `*.capture.json` sidecar with the capture method, stderr, and window/session diagnostics. Treat that sidecar as the first source of truth when a screenshot is missing or blank.
- On `tertiary`, remote screenshot capture may fall back to a GUI-session relay through the logged-in `Terminal` app when direct host capture is blocked. That relay should preserve `guiSessionRelay.method="terminal.command"` in the sibling `*.capture.json`, but only `capture_method="window"` or `capture_method="rect"` counts as authoritative visual proof. Whole-display fallback is diagnostic only.
- Keep screenshot ownership on `tertiary`. `primary` should remain a thin control/sync node only; do not move GUI automation or long-running capture loops onto `primary`.
- For programmatic Studio viewport screenshots from SSH on `tertiary`:
  - Use `scripts/capture_studio_sck.swift` (ScreenCaptureKit, macOS 14+)
  - Compile once: `swiftc scripts/capture_studio_sck.swift -parse-as-library -o /tmp/capture_sck -framework Cocoa -framework ScreenCaptureKit`
  - Run via `.command` file through `open` (inherits Terminal.app's Screen Recording permission)
  - This is the only reliable method — `screencapture`, Quartz CGWindowListCreateImage, and `osascript do shell script` all fail from SSH due to macOS security
- For remote runs, prefer the wrapper-managed artifact directory and synced outputs over manual SSH inspection. The wrapper already pulls back the Studio log, scene-fidelity artifacts, screenshots, and screenshot diagnostics when they exist.

## Rust-specific guardrails

- Keep exporter crates dependency-light until contracts settle.
- Avoid entangling domain types with source-adapter specifics.
- Keep upstream Arnis integration behind an adapter boundary instead of smearing it across the repo.

## Done criteria

A change is production-ready when all of the following are true:

- `cargo test --workspace` passes (31+ tests)
- the Roblox importer consumes the manifest without errors
- all features render correctly at the configured quality profile
- repeated imports are idempotent (no duplicate content)
- no per-frame allocations in any render loop
- all transitions smooth (TweenService, dt-scaled lerps)
- no TODO/placeholder comments in shipped code
