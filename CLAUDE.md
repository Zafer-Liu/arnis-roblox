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
