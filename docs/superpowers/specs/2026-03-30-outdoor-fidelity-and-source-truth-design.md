# Outdoor Fidelity And Source-Truth Design

Date: 2026-03-30
Status: Active

This design defines the active outdoor fidelity and source-truth tranche after the March 28 baseline and play-fidelity work.

Use the paired implementation plan and rolling status document as the active execution and handoff surfaces for this tranche:

- `docs/superpowers/plans/2026-03-30-outdoor-fidelity-and-source-truth.md`
- `docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md`

This repo enforces one active superpowers truth stack only:

- one active spec
- one active implementation plan
- one active rolling status file

All other superpowers docs are historical or completed context only.

The deleted historical backlog is summarized in:

- `docs/superpowers/archive-index.md`

## Goal

Strengthen outdoor fidelity and upstream source-truth preservation from the current clean baseline without introducing a second world-definition path, silent source trampling, or stale proof/documentation surfaces.

## Problem Statement

The repo is in a cleaner state than before:

- edit/play parity is proven on the current bounded baseline
- the active runtime/world-root ownership path is canonical
- player-local fidelity telemetry is materially stronger
- the repo now supports a hard `0.4.0` schema contract without migration-era compatibility

But the next highest-value gaps are still outdoors-first:

- terrain still shows boxiness, material flatness, and limited player-visible nuance
- outdoor-heavy chunks still dominate preview/edit hotspot cost
- the current downstream audits are good at parity and render drift, but they still do not prove a full “no signal loss, no trampling” story across OSM, Overpass, and Overture before canonicalization
- some high-signal outdoor-local experience metrics exist, but remote proof and artifact regeneration remain narrower than they should be
- the docs stack needs a clean rollover so the active tranche reflects the current outdoor-first priorities without drift

## Desired End State

The project should have:

1. one active docs stack for the outdoor fidelity and source-truth tranche
2. a bounded source truth-pack and audit surface that records outdoor source contribution, overlap, collapse, and dropped semantics before canonical collapse
3. stronger canonical outdoor manifest semantics and/or metadata where required to improve terrain, landuse, water, roads, vegetation, and outdoor structure-shell fidelity
4. compact, flaggable outdoor telemetry that makes edit/play proof and iteration faster instead of noisier
5. `tertiary`-verified outdoor proof slices that measure real player-visible improvement and hotspot behavior without local Studio runs on this machine
6. a runtime streaming engine contract that treats the offline compiler as canonical truth and the runtime as a budget-enforcing JIT residency scheduler

## Scope

### In Scope

- outdoor source-truth preservation and auditability for OSM, Overpass, and Overture-derived features
- bounded truth-pack artifacts and audits for terrain, roads, water, vegetation, landuse, and outdoor structure-shell semantics
- downstream audit improvements that join outdoor truth-pack, scene fidelity, and scene parity evidence instead of leaving them as isolated reports
- outdoor-local telemetry and harness/report controls so specific signal families can be enabled selectively
- measured outdoor fidelity work driven by the new audit evidence, especially terrain/material/detail complaints and outdoor-heavy hotspot costs
- a first alpha runtime streaming-engine tranche for the Austin sample with explicit residency budgets, prefetch/eviction behavior, and ring telemetry
- `tertiary` edit/play proof slices for the outdoor tranche
- docs rollover and continuous status maintenance for the active tranche

### Out Of Scope

- a full interior traversal/generation redesign
- replacing the offline compiler with a runtime-only world-definition path
- a new parallel preview/play/full-bake world-definition path
- broad aesthetic stylization that is not justified by retained source semantics or measured player-visible fidelity gains

Planet-scale streaming remains the long-term goal, but the first alpha tranche should prove its architecture on one bounded Austin sample before widening scope.

## Architecture

The tranche keeps one canonical pipeline and makes its truth surfaces explicit:

1. `source union -> truth-pack audit -> canonical manifest -> edit/play consumers -> scene/parity audits`
2. the truth-pack records provenance, overlap, retained fields, dropped fields, and merge/collapse decisions before canonical collapse
3. the canonical manifest remains the only runtime/render source of truth
4. edit and play stay downstream consumers of the same canonical truth
5. audits compose rather than duplicate:
   - truth-pack audit answers “what did the sources say and what changed?”
   - scene fidelity audit answers “what did edit or play actually produce?”
   - scene parity audit answers “do edit and play agree about the same canonical truth?”

For streaming, the pipeline becomes:

6. `source union -> truth-pack audit -> canonical tile database -> runtime streaming scheduler -> resident scene`
7. the offline compiler remains authoritative for canonical world truth, tile semantics, and auditability
8. the runtime streaming engine is a JIT residency layer, not a second compiler:
   - it decides what to prefetch, keep resident, downgrade, or evict
   - it must not invent a parallel world-definition path
9. dev tooling such as Vertigo Sync may accelerate iteration and observability, but it must not own runtime semantics or budget policy

## Execution Rules

### No-Trampling Rule

If two sources disagree or overlap, the repo must not silently flatten that disagreement away before the truth-pack records it. Canonicalization may choose one render/runtime representation, but the audit layer must retain the fact that an overlap, collapse, or dropped semantic occurred.

### Selective Telemetry Rule

Outdoor telemetry and report generation must support explicit signal families so iteration stays fast and token-efficient. The initial expected families are:

- `terrain`
- `roads`
- `water`
- `vegetation`
- `structures`
- `hotspots`
- `player_local`

Default runs should stay compact. Deep runs should opt into richer outdoor markers only when needed.

### Single-Source Runtime Rule

Outdoor fidelity improvements must land in shared builders/import paths or shared manifest semantics. Do not add play-only or edit-only visual patches that bypass the canonical manifest truth.

Fidelity, runtime streaming, and engine behavior must converge on the same production path: as visual quality improves, it should continue to work through the same runtime residency, streaming, and loader contracts rather than relying on a special high-fidelity path.

### Runtime Budget Rule

The runtime streaming engine must treat `estimatedMemoryCost` as the authoritative residency budget and `chunk count` as a secondary guardrail.

The first alpha contract must expose and enforce:

- estimated resident memory by ring
- resident chunk counts by ring
- inflight chunk counts/costs
- queued prefetch work
- explicit eviction reasons

If `estimatedMemoryCost` and `chunk count` disagree, the memory budget wins.

### Proof Rule

- local machine: static/schema/Python/Rust verification only
- `tertiary`: all Studio edit/play experiential proof
- docs update after every proof slice that changes the repo’s active truth

Success for this tranche means:

- stronger outdoor audit coverage before canonical collapse
- measured outdoor fidelity improvements on `tertiary`
- explicit hotspot quantification for outdoor-heavy chunks
- no stale competing truth in specs, plans, or status docs

### Clean-Break Rule

Do not add new backward-compatibility shims. If an older outdoor/audit path is redundant, delete it instead of layering new behavior on top.

### Alpha-First Rule

Prefer production-hardened hyperscale architecture from the beginning over temporary compatibility layers:

- no legacy streaming mode preserved for safety
- no compatibility wrappers for old runtime packaging assumptions
- Austin is the first alpha sample, but the runtime contract should be written as if it must scale to real-time planetary streaming

## Delivery Tracks

### Track A: Outdoor Truth-Pack And Audit Depth

Add a bounded pre-canonical truth-pack and audit surface for outdoor features first. The initial priority order is:

1. terrain and landuse semantics
2. roads, paths, and surface roles
3. water features and water-type semantics
4. vegetation/detail provenance
5. outdoor building-shell/material semantics where they affect outdoor truth

This track should make the repo answerable on:

- what each source contributed
- what was merged or collapsed
- what semantics were dropped
- whether a render/runtime simplification is backed by audit evidence or is premature stylization

### Track B: Outdoor Fidelity And Hotspots

Use the truth-pack and existing runtime telemetry to drive targeted outdoor improvements:

- terrain geometry/material/detail fidelity
- play-vs-edit terrain/material drift if present
- outdoor shell/roof/floor artifacts that affect the player’s exterior experience
- preview/edit hotspot costs on outdoor-heavy chunks

This track should improve what the player sees on `tertiary` without inventing a second truth path.

### Track C: Remote Proof And Docs

Keep `tertiary` as the only Studio proof surface for this tranche. Each meaningful proof run must update the active plan/status/doc stack immediately if it changes what the repo should believe.

### Track D: Runtime Streaming Engine

Add the first alpha runtime streaming-engine tranche on top of the canonical tile database:

1. explicit near/mid/far residency rings
2. hard per-ring budgets:
   - estimated memory cost
   - chunk count
3. movement-aware prefetch:
   - bias toward forward motion and heading
   - favor likely next-needed chunks over symmetric radius loading
4. explicit eviction policy:
   - lowest-priority ring first
   - farthest distance
   - lowest recency/value
   - highest cost for lowest value
5. runtime telemetry proving:
   - resident memory by ring
   - resident chunk counts by ring
   - queue depth
   - prefetch hit/miss rates
   - eviction reasons
   - import time by ring/class

The first alpha does not need a brand-new runtime container format yet; it may continue to use the current Rust-generated Lua shards while the scheduler and residency contract are hardened. The next packaging redesign should follow the scheduler, not precede it.

## Testing And Verification

### Local Static

- Python tests for truth-pack extraction, audit findings, telemetry gating, and downstream report carry-through
- Rust tests for source-preservation and canonical-outdoor contract changes
- shell/static checks for harness/report path changes

### Remote `tertiary`

- focused edit-mode proof slices for outdoor reducers/specs
- focused play-mode proof slices for player-visible outdoor experience and telemetry capture
- audit artifact generation/rebuild from remote raw logs or committed seed manifests where ignored outputs are absent
- direct runtime-emitter and residency-budget measurements against the bounded Austin sample

No Studio execution should run on this machine for this tranche.

## Risks

- the truth-pack can become too large or too expensive if it is not bounded and query-oriented
- telemetry can become noisy or token-heavy if signal families are not explicit and selective
- terrain fidelity changes can become stylized or divergent if they outrun source-preservation evidence
- docs can drift again if status updates lag behind remote proof

## Recommendation

Start with an audit-first outdoor tranche:

1. build the bounded outdoor truth-pack and audit layer
2. use it to drive the highest-signal outdoor fidelity and hotspot fixes
3. add the first alpha runtime streaming engine with explicit residency budgets and scheduler telemetry
4. prove the result on `tertiary`

This is the cleanest way to improve player-visible outdoors while also raising confidence that the pipeline is not silently losing or trampling source truth.
