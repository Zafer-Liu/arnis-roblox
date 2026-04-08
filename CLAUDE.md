# CLAUDE.md

Kodex should read `AGENTS.md` first.

## What Kodex should optimize for

- preserve architecture boundaries
- improve determinism
- reduce per-chunk instance count
- prefer simple systems that can be benchmarked
- leave clean seams for future Arnis adapter work
- avoid catastrophic memory spikes; prefer bounded-memory inspection and generation paths for large manifests and exports

## Large artifact guardrails

- Never eagerly read known large manifests or exports with `Path.read_text()`, `json.load()`, `json.loads()`, or similar whole-file APIs when a streaming or indexed path is possible.
- Prefer shard/index metadata, streaming parsers, mmap-backed extraction, and bounded shell reads over whole-file scans.
- When introducing new large derived artifacts, prefer queryable/indexed formats such as SQLite or Parquet, or chunked text/binary layouts, instead of monolithic JSON.
- Add telemetry and fail-fast guardrails before a dev/test workflow can drift toward multi-GB resident memory; avoid OOMs by design, not by recovery.

## Convergence guardrails

- `arnis-roblox` owns canonical world truth, manifest semantics, and scene extraction adapters.
- `vertigo-sync` owns edit/full-bake orchestration and export-3d user-facing orchestration.
- Do not add new parallel preview/play/full-bake world-definition paths in `RunAustin.lua`, `AustinPreviewBuilder.lua`, `BootstrapAustin.server.lua`, or `AustinSpawn.lua`.
- Keep exactly one active `docs/superpowers/` spec, one active plan, and one active rolling status file for the whole repo. Treat every other superpowers doc as `Historical` or `Completed` context only.
- Treat historical `docs/superpowers/` plan/status/spec files as context unless the current task explicitly says they are active.
- When work is happening under an active spec or implementation plan, append dated status notes as debugging/verification slices complete so another agent can resume without reconstructing chat history.
- Keep remote Studio hosts and machine-specific paths in ignored local config or env, not in committed repo scripts.
- Treat `primary` and `tertiary` as local profile aliases only; the committed repo must stay portable across direct-dev and remote-executor machines. Prefer `tertiary` for remote Studio proof work when that lane is selected.

## Studio harness tool surface

- Use `scripts/run_studio_harness.sh` as the primary local Studio proof tool.
- Use `scripts/run_studio_harness_remote.sh` when the proof lane is remote and you need artifacts synced back locally.
- The harness already provides:
  - clean-place or fresh-template startup
  - edit-only or play-mode execution
  - isolated Luau spec runs via `--spec-filter`
  - edit/play screenshots via `--screenshot`
  - MCP probes, bootstrap/play telemetry, preview telemetry capture, and scene fidelity/parity audits
  - remote fetch of logs, screenshots, and preview/plugin-state artifacts
- If you need to inspect what Studio rendered on a remote host, prefer the harness screenshot artifact path first. Do not default to raw remote `screencapture` when the harness can capture and return the phase-specific image.
- Use these as the default invocation shapes:
  - `scripts/run_studio_harness.sh --play --screenshot --artifact-dir /tmp/arnis-studio-harness`
  - `scripts/run_studio_harness_remote.sh --host <alias> --play --screenshot --artifact-dir /tmp/arnis-remote-studio`
  - add `--edit-only` for edit-mode proof
  - add `--spec-filter <SpecName>` for isolated Luau repros
  - add `--play-wait <seconds>` when large manifests need more time to reach `gameplay_ready` (default 60s)
- When screenshot capture fails, inspect the sibling `*.capture.json` artifact before doing anything else. It records the capture method, stderr, and window/session diagnostics and is the repo’s authoritative failure breadcrumb for remote display issues.
- On `tertiary`, the committed remote screenshot path now prefers a GUI-session relay through the logged-in `Terminal` app when direct display capture is blocked. In a healthy remote visual proof, the sidecar may therefore show `capture_method="gui_terminal_display"` with `guiSessionRelay.method="terminal.command"`.
- Keep `primary` out of remote GUI automation. Use it only to trigger, poll, and sync artifacts back; the actual screenshot/capture work belongs on `tertiary`.
- For programmatic Studio viewport screenshots from SSH, use the ScreenCaptureKit Swift tool:
  - Compile: `swiftc scripts/capture_studio_sck.swift -parse-as-library -o /tmp/capture_sck -framework Cocoa -framework ScreenCaptureKit`
  - Run via GUI relay: create a `.command` file that activates Studio and runs `/tmp/capture_sck`, then `open` it from SSH
  - The `.command` file runs in Terminal.app's GUI session which has Screen Recording permission
  - Output: `/tmp/studio_sck.png` (full display capture at 2x retina resolution)
  - This is the ONLY method that works for capturing Studio viewport from SSH — all other methods (screencapture, Quartz, osascript) fail due to macOS security

## Automated iteration loop (as of 2026-04-08)

- The full `build → publish → remote-harness → telemetry → audit` cycle is one command: `bash scripts/auto_loop.sh`. Closed end-to-end; run it and read the final single-line summary.
- Producer side: `ServerScriptService/ImportService/TelemetryReporter.lua` POSTs structured bootstrap stats to `https://planetary.adpena.workers.dev/telemetry/run` on success/failure. Wired into `BootstrapAustin.server.lua` via `RecordPhase` inside `setBootstrapState` and `Report({status=...})` at both success and failure exits. The entire `RunAustin.run` call is wrapped in `pcall` so uncaught builder exceptions still emit structured failure telemetry.
- Transport: Cloudflare Worker `POST /telemetry/run` (stores in TELEMETRY KV, 30-day TTL, rolling 100-entry index at `telemetry:index`) and `GET /telemetry/latest?limit=N`.
- Consumer side: `scripts/fetch_telemetry.py` (pretty-print or `--json`, `--watch`, `--since`) and `scripts/live_stream_audit.py` (assertions with per-record + aggregate p95 checks, exit codes 0/1/2/3).
- The auto-loop is the primary way to iterate without asking the user to rejoin — run it after any runtime change and read the next telemetry record.

## Runtime footguns discovered 2026-04-08 (must not regress)

1. **Roblox HttpService:RequestAsync restricted headers**: do NOT set `Accept-Encoding`, `User-Agent`, `Host`, `Origin`, `Referer`, `Content-Length`, `Transfer-Encoding`, `Connection`, `Via`. The engine throws `Header "X" is not allowed!` and aborts the whole request. Only set `Accept`, `Content-Type`, and custom `X-*` headers. Roblox auto-negotiates gzip internally.
2. **chunk.originStuds is a JSON table with lowercase keys** (`{x, y, z}`), not a `Vector3`. Builders that read it MUST use the dual-case pattern `local ox = originStuds.X or originStuds.x or 0`. Applies to `RoadBuilder`, `BuildingBuilder`, `WaterBuilder`, `PropBuilder`, `TerrainBuilder`, and any future builder. This was a latent bug under the embedded-SampleData path that the lazy `external_url` fetcher exposed.
3. **Closures referencing `local` declared later in the same block capture nothing**: in Lua/Luau, locals are only in scope for code AFTER the declaration. Define closures AFTER the locals they need. Caught in `StreamingService.appendStreamingWorkItems` where the inner closure referenced `chunkRef` before it was declared, crashing with `attempt to index nil with 'id'`.
4. **`ssh host bash -s -- ` drops empty-string positional args** during remote command construction. Use the `__EMPTY__` sentinel encode/decode pattern (already implemented in `run_studio_harness_remote.sh`). Any new positional arg added to the ssh call MUST go through `sanitize_positional` on the outer side and `decode_positional` on the inner side.
5. **`set -euo pipefail` + grep in command substitution silently kills the script** when grep finds no match. Every `$(grep ...)` in shell scripts that may legitimately find nothing MUST append `|| true` inside the substitution. Caught in `auto_loop.sh` publish-version parsing.
6. **Do NOT `source ~/.zshrc` from bash**. Zsh-only syntax crashes bash parsers even with `|| true` (the error trips `set -e` before the handler runs). Use `zsh -c 'source ~/.zshrc; printf "%s" "$VAR"'` to extract a single env var into bash.
7. **ChunkSchema.validateManifest must accept chunkRefs-only manifests** (no inline `chunks` array) so lazy streaming sources aren't forced through the embedded fallback.

## Client traversal system (pre-existing, do not re-implement)

- `roblox/src/StarterPlayer/StarterPlayerScripts/VehicleController.client.lua` (~2866 lines) is the monolith owning **all** traversal systems: car (V), jetpack (J), parachute (P), wingsuit, grapple hook. It has been shipping since commit `a35b88d1` ("AAA-grade car + jetpack + parachute with physics, particles, camera, HUD") and evolved through many polish passes (see `efc35c0d`, `a62fd6e0`, `a399799e`, `30a8ef99`).
- If asked to add or fix jetpack / parachute / wingsuit / grapple / car behavior, **edit this file in place**. Do NOT create sibling controllers — they collide on key bindings (J and P are owned).
- A modular split into `CarController` / `JetpackController` / `ParachuteController` / `WingsuitController` / `GrappleController` is planned but the split MUST delete the old sections in the SAME commit, not layer new files on top.

## Immediate tasks Kodex can safely take on

1. Replace placeholder terrain import with a real voxel writer path.
2. Add chunk unload/reload with reference counting or authoritative overwrite.
3. Extend the Rust sample exporter so it can emit multiple adjacent chunks.
4. Build a stronger profiler/reporting pass in Roblox and Rust.

## What Kodex should not do early

- add interiors
- add live HTTP geodata calls to Roblox
- add flashy UI before importer correctness
- overfit the manifest to one city or one upstream source
