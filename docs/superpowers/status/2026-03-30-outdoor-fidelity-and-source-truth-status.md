# Outdoor Fidelity And Source-Truth Status

Date: 2026-03-30
Status: Active

## Purpose

This is the rolling status and handoff document for the active outdoor fidelity and source-truth tranche.

The active design spec is:

- `docs/superpowers/specs/2026-03-30-outdoor-fidelity-and-source-truth-design.md`

The active implementation plan is:

- `docs/superpowers/plans/2026-03-30-outdoor-fidelity-and-source-truth.md`

## Current Snapshot

- Inherited from the March 28 baseline and archived play-fidelity tranche: edit/play parity, canonical runtime ownership, and player-local observability are already proven.
- The compatibility purge is in place: the repo treats `0.4.0` as the supported manifest schema and does not keep older manifest compatibility active.
- The active tranche is now the March 30 outdoor fidelity and source-truth stack.
- The earlier March 28 play-fidelity tranche is historical context only.

## Verification Snapshot

### Local Static

- `rg -n "^Status: Active$" docs/superpowers/specs docs/superpowers/plans docs/superpowers/status`
  - passed on 2026-03-30
  - verified that the March 30 spec/plan/status stack was the only set of docs carrying active markers after the rollover
- `git diff --check`
  - passed on 2026-03-30
  - verified the Task 1 documentation rollover landed cleanly

### Remote `tertiary`

No verification recorded yet.

## Residual Gaps

- Outdoor fidelity still needs dedicated work on terrain detail, shell nuance, and player-visible exterior realism.
- Outdoor hotspots still need tighter measurement so preview/edit cost can be traced at chunk scope instead of only at the whole-run level.
- Source-truth preservation still needs explicit proof across upstream source union, canonical collapse, and downstream audits.

## Status Notes

### 2026-04-01: Task 2 Narrowed To The First Honest Truth-Pack Slice

- A focused codebase audit confirmed that the current pipeline only retains truthful pre-canonical source-union data inside the Overpass/live adapter path plus the Overture building merge seam.
- The original Task 2 wording overclaimed generic compile-wide truth-pack coverage.
- The active plan now narrows the first slice to:
  - `OverpassAdapter`
  - `LiveOverpassAdapter`
  - Overpass-derived retained features
  - Overture building candidates
  - Overture-to-OSM collapse rows
- `FileSourceAdapter`, synthetic adapters, and post-canonical export-only seams remain out of scope for this first truth-pack slice until raw lineage is preserved there.
