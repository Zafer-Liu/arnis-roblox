# Play Fidelity And Observability Design

Date: 2026-03-28
Status: Active

This is the active design spec for the current fidelity/observability tranche.

Use the paired implementation plan and rolling status document as the current execution and handoff surfaces:

- `docs/superpowers/plans/2026-03-28-play-fidelity-and-observability.md`
- `docs/superpowers/status/2026-03-28-play-fidelity-and-observability-status.md`

## Goal

Raise player-visible play/edit fidelity and audit observability from the proven March 28 baseline without reopening canonical world-truth ownership or parity semantics.

## Problem Statement

The single-source-of-truth baseline is now proven, but several player-visible and operator-visible gaps remain:

- shaped roofs still needed explicit internal-support handling to avoid duplicate visible roof truth
- shell-mesh buildings were underrepresented in player-local telemetry, making “missing walls” reports ambiguous
- play/edit runtime truth is aligned, but the audit surface still needs stronger local support, enclosure, and roof-cover evidence
- preview/edit performance still shows a large end-to-end sync cost and building-heavy slow chunks
- the repo doc stack drifted because the completed baseline tranche was still labeled active

## Desired End State

The project should have:

1. one active docs stack for the fidelity/observability tranche
2. internal roof-closure support that does not count as visible roof truth
3. player-local telemetry that reflects shell-mesh building evidence, enclosure, and roof cover with less ambiguity
4. explicit measured residual gaps for walls, terrain, interiors, and performance hotspots

## In Scope

- docs-stack rollover from the completed baseline tranche
- roof-closure internal-support handling
- shell-mesh observability improvements in `WorldProbe`
- remote verification on `tertiary`
- measured tracking of remaining wall, terrain, interior, and hotspot work

## Out Of Scope

- full terrain representation redesign
- full interior-generation redesign
- full source-truth-pack implementation
- planet-scale streaming redesign

Those remain follow-on workstreams after this tranche.
