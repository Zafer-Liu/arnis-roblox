# Canonical Baseline And Single Source Of Truth Design

Date: 2026-03-28
Status: Completed

This design is now historical as the completed March 28 baseline tranche.

The later outdoor fidelity and source-truth design is:

- `docs/superpowers/specs/2026-03-30-outdoor-fidelity-and-source-truth-design.md`

This is the completed design spec for the canonical-baseline tranche.
For later execution and handoff, use the March 30 outdoor fidelity stack instead:

- `docs/superpowers/specs/2026-03-30-outdoor-fidelity-and-source-truth-design.md`
- `docs/superpowers/plans/2026-03-30-outdoor-fidelity-and-source-truth.md`
- `docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md`

## Goal

Establish one clean, current baseline for `arnis-roblox` so future work on convergence, fidelity, audit coverage, export, and planet-scale streaming starts from a single documented truth instead of a mixture of shipped code, stale plans, and implicit runtime behavior.

## Problem Statement

The repo has real convergence progress in code, but the working state is not yet clean:

- canonical manifest-family convergence and anti-drift guardrails are implemented
- several plan documents still describe already-shipped work as pending
- preview and play still diverge in a few important ownership paths:
  - anchor semantics
  - bootstrap state ownership
  - world-root publication
  - minimap-driven observability
  - play-only world-state layering
- the audit path is stronger than before, but it still compares manifest buckets to scene buckets instead of preserving full source-to-scene truth
- streaming/subplan scheduling is regionally credible, but not yet a planet-scale serving architecture

The immediate risk is not simply missing features. The risk is proceeding from a dirty baseline and losing confidence in which contracts are canonical.

## Desired End State

The project should have:

1. one canonical documentation stack for current truth
2. one canonical preview/play world contract
3. one authoritative remote Studio validation flow for this machine setup
4. one explicit backlog of measured open gaps after baseline cleanup

This design does not try to solve every remaining fidelity or planet-scale problem in one step. It creates the clean baseline required to solve them without reintroducing drift.

## Scope

### In Scope

- streamline and normalize baseline documentation
- mark stale plans as historical or superseded
- define one current convergence status document
- define one current implementation plan for the baseline tranche
- remove remaining preview/play single-source-of-truth drift in the highest-value runtime ownership paths
- add executable parity coverage where the repo currently relies on indirect contract tests
- define explicit local-vs-remote verification lanes
- treat remote Studio profile `tertiary` as the default Studio execution surface for this workstation setup, without making the committed repo depend on a specific host

### Out Of Scope

- full export convergence implementation
- full source-truth preservation redesign across OSM, Overpass, and Overture
- full truth-pack scene comparator implementation
- full planet-scale corpus serving
- broad visual restyling or speculative builder rewrites

Those remain follow-on workstreams after the baseline is trusted.

## Canonical Documentation Stack

After this baseline tranche, the repo should have one clearly documented current-truth stack with these required layers:

### 1. Stable contract documents

- `docs/chunk_schema.md`
- `docs/vertigo-sync-boundary.md`
- `AGENTS.md`
- `CLAUDE.md`
- runtime/config contracts already encoded in source where appropriate
- executable ownership guardrails in `scripts/tests/test_convergence_guardrails.py`

These documents describe stable ownership and schema semantics.

### 2. One active baseline design spec

- this spec document

This document explains the architecture and sequencing of the cleanup tranche.

### 3. One active implementation plan

- a single plan created from this spec

This plan becomes the execution source of truth for the current tranche.

### 4. One rolling status / handoff document

- a single status file under `docs/superpowers/` that records:
  - what has landed
  - what was verified locally
  - what still requires remote Studio validation
  - the measured open gaps after each meaningful slice

### 5. Historical plans

Older plan files remain in the repo, but they must no longer read like live execution truth.

They must be one of:

- explicitly marked historical
- explicitly marked superseded
- or summarized and linked from the rolling status doc

Important rule:

- no historical plan may continue to advertise already-landed work as “Expected: FAIL” without a clear superseded note at the top

## Canonical Runtime Truth

The baseline must make preview and play share one world-truth contract as far as possible before any consumer-specific policy diverges.

### A. Manifest and envelope truth

The canonical manifest family remains the full-bake Austin manifest family already owned by `CanonicalWorldContract`.

For the same shared envelope definition:

- preview and play must resolve the same chunk set
- preview and play must resolve the same source-feature identity set
- preview and play must resolve the same canonical anchor inputs

For this baseline, source-feature identity means:

- manifest-side normalized feature IDs grouped by feature family and chunk membership
- runtime-observed feature identity via builder-emitted source IDs such as `ArnisImportSourceId` where available
- parity comparisons may compare identity sets per family rather than raw instance counts

### B. Anchor truth

Current code still allows drift between:

- preview bounded-envelope anchor resolution
- runtime-specific spawn anchor recomputation

The baseline must tighten this boundary:

- one canonical anchor contract is resolved first
- any play-specific spawn adjustment must be an explicit policy transform on top of that same canonical anchor, not a second source of anchor truth

### C. Bootstrap state truth

The repo already contains a dedicated bootstrap state-machine module, but live bootstrap still duplicates state handling inline.

The baseline requires:

- one bootstrap state owner
- one attempt identity path
- one set of workspace attributes for monotonic readiness

Important implementation rule:

- either wire `BootstrapAustin.server.lua` to the canonical bootstrap-state module and publish the same attempt-id/readiness attributes live
- or explicitly replace/remove that module and update harness/tests in the same change

Inline bootstrap state duplication in the top-level runtime script is not acceptable after cleanup.

### D. World-root publication

World-root discovery must not depend on minimap startup.

The baseline requires:

- one canonical world-root publication path usable by preview, play, minimap, probes, and harness tooling
- one canonical world-root attribute name owned outside minimap startup
- backward compatibility for `ArnisMinimapWorldRootName` during migration
- `WorldProbe`, minimap client logic, and harness consumers must consume canonical world-root publication directly

Recommended shape:

- canonical publisher: the runtime/import world-state owner, not minimap startup
- canonical attribute: a world-root attribute published independently of minimap enablement
- compatibility path: mirror `ArnisMinimapWorldRootName` until all current consumers have switched

### E. World-state and lighting ownership

Play and preview must not have separate implicit world-state owners unless that difference is intentional and documented as policy.

The baseline should move toward:

- shared world-state application for environment truth
- explicit, narrow play-only layers for gameplay or interaction semantics

## Verification Lanes

The baseline should define explicit validation lanes instead of one mixed bag of checks.

### 1. `local-static`

Runs on this machine without Studio:

- Python tests
- Rust tests
- static contract checks
- schema/docs guardrails

Purpose:

- prove code and docs are internally coherent before remote Studio is involved

### 2. `remote-edit`

Runs on the operator-selected remote Studio profile.

For this workstation setup, default to `tertiary`.

- edit-mode canonical-world specs
- preview stability checks
- non-play minimap/edit observation checks

Purpose:

- prove preview/edit baseline is stable and non-janky

### 3. `remote-play-world`

Runs on the operator-selected remote Studio profile.

For this workstation setup, default to `tertiary`.

- play-mode world-fidelity validation
- bootstrap/readiness validation
- startup-vs-streaming reconciliation checks
- world-root and minimap-runtime observation checks

Purpose:

- prove play-mode world truth converges with preview for the same envelope

### 4. `remote-play-gameplay`

Runs on the operator-selected remote Studio profile.

For this workstation setup, default to `tertiary`.

- vehicle / jetpack / parachute / camera / audio checks

Purpose:

- isolate gameplay regressions from world-fidelity validation

Important rule:

- world correctness and gameplay validation must not share the same authoritative pass

## Missing Proof Surfaces To Add In The Baseline

### 1. Extend preview-vs-play parity coverage

The repo already has canonical parity coverage in Luau for canonical family/materialization, shared bounded envelopes, anchor values, and chunk IDs. The baseline must extend that coverage to the remaining runtime-facing assertions.

The baseline plan must preserve the existing parity assertions for:

- shared bounded-envelope chunk IDs
- canonical anchor values and basis

The baseline plan must add the still-missing runtime-facing coverage for:

- source-feature identity
- minimap payload basis or equivalent static transform inputs

### 2. Bootstrap canonicalization

The repo must stop relying on partial string-presence tests for live bootstrap semantics.

The baseline plan must verify:

- `BootstrapAustin.server.lua` uses the canonical bootstrap state owner
- attempt identity is actually published live
- harness expectations match the shipped runtime path

### 3. World-root observability

The baseline plan must ensure:

- probes, minimap, and harness logic rely on the same published world-root truth
- preview does not become unobservable just because minimap is intentionally disabled

## Sequencing After Baseline

Once the baseline is complete and verified, follow-on work should proceed in this order:

1. play-vs-edit fidelity and jank elimination
2. source-truth preservation upgrades
3. truth-pack and richer source-to-scene audit tooling
4. streaming/chunking improvements for larger traversal scopes
5. export convergence
6. global corpus and serving architecture

This ordering is deliberate:

- convergence before visual expansion
- source preservation before stylization
- regional/runtime truth before planet-scale serving

## Open Gaps The Baseline Must Preserve As Explicit Backlog

The baseline does not erase these known gaps. It records them cleanly:

- preview/play anchor-policy drift still needs removal
- minimap redraw jank likely remains a first post-baseline fidelity target
- Overture overlap enrichment is still lossy
- Overpass relation and crossing semantics are still narrower than desired
- centroid-based vertical sampling still loses truth for large features
- audit tooling still lacks full source-to-scene truth-pack comparison
- streaming still assumes the full chunk-ref catalog is in memory
- manifest/schema docs still need cleanup around memory-cost and subplan contract details
- export contract/code remains largely unstarted

## Acceptance Criteria

The baseline tranche is complete when all of the following are true:

- one current baseline spec exists
- one current implementation plan exists
- one rolling status/handoff document exists
- stale convergence plans are clearly marked historical or superseded
- preview/play anchor and bootstrap ownership drift is reduced to one documented canonical path
- world-root publication is canonical and minimap-independent
- preview-vs-play parity coverage exists for the shared envelope contract
- local static verification passes
- the selected remote validation profile for this workstation setup, expected to be `tertiary`, passes edit and play-world validation on the cleaned baseline
- remaining open problems are documented as measured backlog, not hidden inside stale plan text

## Recommended Next Step

Write a single implementation plan for the canonical-baseline tranche with tasks in this order:

1. documentation and handoff consolidation
2. bootstrap/world-root canonicalization
3. preview-vs-play parity test surface
4. remote edit/play-world verification using the selected operator profile, expected here to be `tertiary`
5. residual-gap status update

Only after that baseline is proven should the next plan tackle minimap jank, source-truth preservation, audit truth packs, or planet-scale serving.
