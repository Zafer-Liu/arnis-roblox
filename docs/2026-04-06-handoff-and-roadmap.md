# 2026-04-06 Handoff And Roadmap

## Purpose

This document is the practical handoff for the current remote proof, route-parity, and planetary-realism work.

It is intentionally not a replacement for the governing superpowers docs. It is the operator-facing bridge between:

- what is already complete
- what is currently active
- what proof commands and artifacts are authoritative
- what the next engineer should do next

## Current State

- Branch: `main`
- Repo state at handoff: clean
- Remote state at handoff: the standard remote wrapper path on `tertiary` is green for the active route-catalog play proof lane
- Latest pushed commit for this tranche: `9893c2f3` `Harden remote wrapper proof completion on tertiary`
- Senior review: passed, `APPROVE`

## Executive Summary

The biggest infrastructure risk from the previous tranche is gone:

- the standard remote wrapper no longer depends on a fragile live SSH parent
- the wrapper now launches a detached remote harness runner, polls remote status, and syncs proof artifacts from remote state
- the remote screenshot lane is green through the tertiary-local GUI-session relay
- the route-catalog play-fidelity lane is green through the same wrapper path
- the last live runtime blocker in this slice, a plugin-only `Model.LevelOfDetail` write during play import, was fixed at the source in `BuildingBuilder`

The practical result is that the repo now has a stable, repeatable remote proof lane on `tertiary` for the active route-catalog play slice, including:

- synced PNG screenshot
- synced screenshot sidecar
- synced play-fidelity JSON/HTML

## Governing Docs

### Completed tranche

- Design: [2026-03-30-outdoor-fidelity-and-source-truth-design.md](/Users/adpena/Projects/arnis-roblox/docs/superpowers/specs/2026-03-30-outdoor-fidelity-and-source-truth-design.md)
- Plan: [2026-03-30-outdoor-fidelity-and-source-truth.md](/Users/adpena/Projects/arnis-roblox/docs/superpowers/plans/2026-03-30-outdoor-fidelity-and-source-truth.md)
- Rolling status: [2026-03-30-outdoor-fidelity-and-source-truth-status.md](/Users/adpena/Projects/arnis-roblox/docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md)

Status:
- this tranche is effectively complete for the route-proof/harness lane
- it produced the stable proof infrastructure the current sprint now relies on

### Active tranche

- Design: [2026-04-06-planetary-realism-sprint-design.md](/Users/adpena/Projects/arnis-roblox/docs/superpowers/specs/2026-04-06-planetary-realism-sprint-design.md)
- Plan: [2026-04-06-planetary-realism-sprint.md](/Users/adpena/Projects/arnis-roblox/docs/superpowers/plans/2026-04-06-planetary-realism-sprint.md)
- Rolling status: [2026-04-06-planetary-realism-sprint-status.md](/Users/adpena/Projects/arnis-roblox/docs/superpowers/status/2026-04-06-planetary-realism-sprint-status.md)

Status:
- active
- some builder/material work is already landed
- proof widening and remaining visual-quality work are still open

### OMX planning surfaces still worth reading

- PRD: [prd-outdoor-fidelity-route-play-parity.md](/Users/adpena/Projects/arnis-roblox/.omx/plans/prd-outdoor-fidelity-route-play-parity.md)
- Test spec: [test-spec-outdoor-fidelity-route-play-parity.md](/Users/adpena/Projects/arnis-roblox/.omx/plans/test-spec-outdoor-fidelity-route-play-parity.md)

Status:
- historical but still useful for understanding how route play parity was closed
- the core route play-parity problem described there is no longer the live blocker

## Progress Against Plans

### March 30 outdoor fidelity and source-truth plan

Progress summary:
- route-scoped edit/play parity was burned down
- route-catalog proof metadata is stable
- bounded route-slice parity/fidelity artifacts were built
- non-visual landmark/roof proof surfaces were built
- remote screenshot capture was hardened through the GUI-session relay
- the standard remote wrapper path is now green on the active route-catalog play slice

What is done enough to treat as complete:
- wrapper proof for the active route-catalog step-0 play lane
- screenshot artifact sync
- route-runtime-backed play-fidelity artifact sync
- route-window parity on the previously proven bounded slices

What remains only as follow-on widening, not as a blocker:
- rerun equivalent proof on additional route windows and non-route modes
- optionally suppress the remaining non-blocking MCP plugin localhost noise

### April 6 planetary realism sprint plan

Progress summary from the active sprint docs plus the latest repo state:

- Task 1 terrain satellite material consumption: partially/meaningfully advanced in prior sprint status
- Task 2 road data consumption: materially advanced in prior sprint status
- Task 3 building material diversity: materially advanced in prior sprint status
- Task 4 water color and building LOD: materially advanced; this handoff slice also hardened the runtime LOD path so it no longer crashes play import
- Task 5 hero PBR surfaces: still open
- Task 6 rooftop gameplay surfaces: partially advanced, still open for widening/polish
- Task 7 full verification and tertiary proof: active and now much more realistic because the wrapper lane is stable

Practical interpretation:
- the sprint is no longer blocked on proof infrastructure
- remaining work is mostly product-facing visual and material quality work, plus widened proof coverage

## Latest Green Proof

### Authoritative wrapper proof

Artifact directory:
- `/tmp/arnis-remote-studio-wrapper-green4`

Important synced files:
- [0.715.1.7151119_20260406T183944Z_Studio_9e719_last.log](/tmp/arnis-remote-studio-wrapper-green4/0.715.1.7151119_20260406T183944Z_Studio_9e719_last.log)
- [arnis-studio-harness-play.png](/tmp/arnis-remote-studio-wrapper-green4/arnis-studio-harness-play.png)
- [arnis-studio-harness-play.capture.json](/tmp/arnis-remote-studio-wrapper-green4/arnis-studio-harness-play.capture.json)
- [arnis-scene-fidelity-play.json](/tmp/arnis-remote-studio-wrapper-green4/arnis-scene-fidelity-play.json)
- [arnis-scene-fidelity-play.html](/tmp/arnis-remote-studio-wrapper-green4/arnis-scene-fidelity-play.html)

Key facts from that run:
- wrapper exit: `0`
- wrapper tail behavior: bounded intentionally after proof completion
- screenshot sidecar:
  - `success=true`
  - `capture_method="gui_terminal_display"`
  - `guiSessionRelay.method="terminal.command"`
  - `blocker_reason=null`
- play-fidelity report:
  - `manifestSourceKind="route_catalog"`
  - `findings=[]`

### What that proves

- the detached remote-wrapper ownership model works
- the GUI-session screenshot relay works in the standard wrapper lane, not just the direct harness lane
- route-catalog play proof and synced fidelity artifacts now survive the full remote wrapper path

## Key Technical Changes In The Last Tranche

### `scripts/run_studio_harness_remote.sh`

Why it changed:
- the old wrapper could lose the proof run when the live SSH parent dropped or ended awkwardly

What changed:
- remote harness runs are now launched detached
- remote status is tracked through:
  - `.arnis-remote-harness.stdout.log`
  - `.arnis-remote-harness.exit`
  - `.arnis-remote-harness.pgid`
- proof detection now works from synced remote stdout instead of a fragile live stream
- cleanup tail is bounded after proof completion

### `scripts/run_studio_harness.sh`

Why it changed:
- detached wrapper-managed remote runs needed a way to opt out of the parent watchdog without weakening the direct local safety path

What changed:
- `ARNIS_PARENT_WATCHDOG` now gates the parent watchdog
- direct/default harness behavior stays safe
- detached remote wrapper runs can disable the SSH-parent kill path intentionally

### `roblox/src/ServerScriptService/ImportService/Builders/BuildingBuilder.lua`

Why it changed:
- play/runtime import on `tertiary` was failing because `Model.LevelOfDetail` was being written from a non-plugin-capable runtime path

What changed:
- `Model.LevelOfDetail` writes are now guarded through a tiny `pcall` helper
- this fixes the real runtime import failure without moving the behavior to a wrapper-only exception path

## Verification At Handoff

Local verification that was green before handoff:

```bash
python3 -m unittest \
  scripts.tests.test_run_studio_harness \
  scripts.tests.test_run_studio_harness_remote \
  scripts.tests.test_gui_session_capture \
  scripts.tests.test_route_slice_parity_artifacts \
  scripts.tests.test_studio_workflow \
  scripts.tests.test_austin_runtime_contract -v

bash -n scripts/run_studio_harness.sh scripts/run_studio_harness_remote.sh

git diff --check
```

Observed result:
- Python tests: `220` passing
- shell syntax: passing
- diff check: clean

Review artifact:
- [claude-senior-engineer-code-review-requested-review-only-this-tranc-2026-04-06T18-46-00-801Z.md](/Users/adpena/Projects/arnis-roblox/.omx/artifacts/claude-senior-engineer-code-review-requested-review-only-this-tranc-2026-04-06T18-46-00-801Z.md)

Review verdict:
- `APPROVE`

## Recommended Next Steps

### Immediate next steps

1. Widen the now-green remote wrapper proof beyond `step-0 active`.
   Re-run the same wrapper lane for:
   - `step-1 active`
   - `step-1 retain`
   - `step-0 prefetch`

2. Re-prove the non-route wrapper modes.
   Specifically:
   - play-focused non-route
   - edit-only
   - isolated `--spec-filter`

3. Use the stable wrapper lane to drive real scene-quality burndown again.
   Highest-value candidates:
   - facade richness
   - roof/profile clarity
   - material feel
   - street-level readability

### Medium-term sprint work

4. Continue the active planetary realism sprint through the remaining open tracks:
   - hero PBR surfaces
   - widened rooftop gameplay surfaces
   - ring-budget-aware fidelity scaling
   - widened tertiary proof coverage

5. Tighten non-blocking proof noise.
   The `localhost:44755` MCP plugin connection-refused chatter is still visible in play-focused logs. It no longer blocks proof, but it should be made quieter or better classified.

### Release-readiness follow-up

6. After the widened proof matrix is green, run one more senior review pass focused on:
   - wrapper robustness across modes
   - log clarity / artifact completeness
   - whether any now-dead fallback branches should be simplified

## Commands To Reuse

### Standard active proof command

```bash
ARNIS_REMOTE_STUDIO_ARTIFACT_DIR=/tmp/arnis-remote-studio-wrapper-green4 \
ARNIS_TELEMETRY_FAMILIES=terrain,roads,structures,player_local \
bash scripts/run_studio_harness_remote.sh \
  --remote-host 100.65.24.39 -- \
  --takeover \
  --hard-restart \
  --skip-edit-tests \
  --play-wait 35 \
  --pattern-wait 180 \
  --screenshot /tmp/arnis-studio-harness.png \
  --route-catalog PlanetaryRouteBundle.route-catalog \
  --route-lane active \
  --route-step-index 0
```

### Quick artifact inspection

```bash
find /tmp/arnis-remote-studio-wrapper-green4 -maxdepth 2 -type f | sort
```

### Review the latest synced play report

```bash
python3 - <<'PY'
from pathlib import Path
import json
path = Path('/tmp/arnis-remote-studio-wrapper-green4/arnis-scene-fidelity-play.json')
data = json.loads(path.read_text())
print(data.get('manifestSourceKind'))
print(data.get('findings'))
PY
```

## What Not To Re-Learn

- Do not put GUI automation or screenshot ownership back onto `primary`.
- Do not treat direct `screencapture` failure on `tertiary` as the main visual lane anymore; the harness GUI-session relay is the real path.
- Do not rely on live SSH parent ownership for remote proof lifecycle.
- Do not fix runtime-only property failures in the wrapper if the underlying builder/runtime can be corrected directly.

## Known Non-Blocking Noise

- Play-focused runs can still produce repeated localhost `44755` connection-refused chatter from the MCP plugin.
- In the current green route-catalog wrapper proof, that noise is non-blocking.
- If a future engineer sees that noise again, they should not assume the proof failed unless:
  - the wrapper exits nonzero
  - proof artifacts are missing
  - the log lacks authoritative proof markers

## Session 2 Progress (Later 2026-04-06)

A second agent session continued the planetary realism sprint, shipping 24 commits.

### What Was Shipped

**Data-faithful Lua builder rendering:**
- TerrainBuilder: 15-material satellite palette as PRIMARY source (slope fallback only when absent)
- RoadBuilder: sidewalk enum curbs, layer stacking, subkind material differentiation
- BuildingBuilder: facadeStyle, roofLevels, roof material hash diversification (usage-based fallback preserved)
- WaterBuilder: per-body color + kind-specific visuals (river/lake/pond/wetland)
- RailBuilder: kind-specific material/thickness
- PropBuilder: conifer species trigger needleleaved canopy

**Infrastructure:**
- Building LOD (Model.LevelOfDetail=Automatic)
- Atmosphere depth config (WorldConfig knobs, additive on phase presets)
- Ring-based transparency (additive, reversible via ArnisBaseTransparency)
- Zero-alloc frame profiler (ARNIS_CLIENT_PERF marker)
- Rooftop gameplay surfaces (parapets, equipment variety, 3+ level threshold)

**Rust pipeline enrichment:**
- `road.name` from OSM
- `building.cladding` from OSM building:cladding
- `building.roof_direction` + `building.roof_angle` from OSM
- `road.sidewalk_surface` from OSM
- Overture roof_shape extraction
- `arbx_cli audit-signal` subcommand (signal preservation measurement)

**Code quality:** 3-pass senior review, all issues resolved.

### In-Flight (Agents Running at Session End)

Two agents in isolated worktrees (both instructed to COMMIT):
1. RoadBuilder: consuming road.name (street labels), sidewalkSurface, lane marking geometry
2. BuildingBuilder: consuming cladding, roofDirection, roofAngle

When these return: merge one at a time → verify tests → push to origin/main → then cleanup.

### Test Counts at Session End

| Suite | Count |
|-------|-------|
| Python | 236 |
| Rust | 213 |

### Updated Recommended Next Steps

1. **Merge in-flight agents** (road labels/markings + building cladding/roof orientation)
2. **Recompile Austin** with enriched pipeline on tertiary
3. **Run audit-signal** on real manifest — quantify signal preservation
4. **Run profiler on tertiary** — establish frame time baseline
5. **Tertiary visual proof** — screenshot at street level + aerial
6. **Satellite imagery tile pipeline in Rust** — ESRI tiles at compile time (biggest visual leap)
7. **Multi-city validation** — compile Tokyo
8. **Performance optimization** — cut frames based on profiler data
9. **Style resolver** — formalize Layer 3 of canonical-feature-style-contract

### Principles Learned

1. Never trample source signal — express data faithfully, invent only for genuinely missing data
2. Everything performance-critical in Rust — Python for offline tooling only
3. Measure before shipping — profile on tertiary, know the frame time cost
4. Commit before cleanup — never clean worktrees with uncommitted changes
5. Free data sources only — ESRI, OSM, Overpass, Overture, AWS Terrain Tiles
6. 4TB SSD for caching — no disk pressure
7. Testing on tertiary only via vertigo-sync
8. What would Gabe Newell do — ship the frame, not the feature

## Handoff Readiness

This work is ready for handoff for the current tranche.

That means:
- code is pushed
- repo is clean
- review passed
- proof artifacts exist
- the active route-catalog wrapper lane is green
- 24 additional commits from the realism sprint are on origin/main

It does not mean the entire broader realism/fidelity program is finished. The next engineer should treat this as:

- proof infrastructure: stable enough to build on
- current sprint: still active, substantial builder/material work landed
- next job: merge in-flight agents, recompile Austin, run profiler, then resume visual quality + satellite imagery work
