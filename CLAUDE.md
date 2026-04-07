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
- When screenshot capture fails, inspect the sibling `*.capture.json` artifact before doing anything else. It records the capture method, stderr, and window/session diagnostics and is the repo’s authoritative failure breadcrumb for remote display issues.
- On `tertiary`, the committed remote screenshot path now prefers a GUI-session relay through the logged-in `Terminal` app when direct display capture is blocked. In a healthy remote visual proof, the sidecar may therefore show `capture_method="gui_terminal_display"` with `guiSessionRelay.method="terminal.command"`.
- Keep `primary` out of remote GUI automation. Use it only to trigger, poll, and sync artifacts back; the actual screenshot/capture work belongs on `tertiary`.
- For programmatic Studio viewport screenshots from SSH, use the ScreenCaptureKit Swift tool:
  - Compile: `swiftc scripts/capture_studio_sck.swift -parse-as-library -o /tmp/capture_sck -framework Cocoa -framework ScreenCaptureKit`
  - Run via GUI relay: create a `.command` file that activates Studio and runs `/tmp/capture_sck`, then `open` it from SSH
  - The `.command` file runs in Terminal.app's GUI session which has Screen Recording permission
  - Output: `/tmp/studio_sck.png` (full display capture at 2x retina resolution)
  - This is the ONLY method that works for capturing Studio viewport from SSH — all other methods (screencapture, Quartz, osascript) fail due to macOS security

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
