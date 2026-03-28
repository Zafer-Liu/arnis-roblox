# Canonical Baseline And Single Source Of Truth Implementation Plan

Status: Completed

This plan is now historical as the completed baseline tranche.

The active follow-on plan is:

- `docs/superpowers/plans/2026-03-28-play-fidelity-and-observability.md`

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish one clean baseline for current repo truth by consolidating docs and handoff state, canonicalizing bootstrap and world-root ownership, extending preview-vs-play parity coverage, and proving the cleaned baseline through remote Studio validation on the operator-selected profile for this workstation, expected to be `tertiary`.

**Architecture:** Treat the current canonical code paths as the base, not the stale plan text. First normalize the documentation stack and status trail so only one active plan and one current status document remain authoritative. Then fix the two highest-value runtime single-source-of-truth drifts: bootstrap state ownership and world-root publication. Extend existing parity coverage instead of replacing it, then use the remote Studio harness as the authoritative runtime proof surface for edit and play-world lanes.

**Tech Stack:** Luau, Python unittest, Bash harness scripts, existing Roblox Studio remote harness wrapper, current canonical-world and convergence tests

This is the completed implementation plan for the canonical-baseline tranche.
For current execution, use the active play-fidelity stack instead; the `Execution Status` section below is preserved as historical progress, and the red/green step text under each task is execution history rather than a claim that the work is still pending.

## Execution Status

- 2026-03-28: Task 1 completed locally. Status trail created, stale March 26 plans marked historical, remote-profile docs normalized.
- 2026-03-28: Task 2 completed locally. Live bootstrap now uses `BootstrapStateMachine`, and runtime/harness tests enforce canonical bootstrap attempt/state-trace ownership.
- 2026-03-28: Task 3 completed locally. `WorldStateApplier` is the canonical world-root publisher; minimap mirrors only, and duplicate `RunAustin` publication was removed.
- 2026-03-28: Task 4 completed locally. Preview/play parity coverage now includes normalized `identitySummary` and `minimapBasis` metadata plus Luau parity assertions.
- 2026-03-28: Task 5 is complete for the current baseline. The isolated `tertiary` edit contract slice, the `tertiary` play-world slice, and the corrected raw-log parity proof are all green for the aligned `1500`-radius preview baseline (`21/21` matching).
- 2026-03-28: The parity proof topology is now explicit: use separate clean edit-only and play-only Studio runs on `tertiary`, then compare rebuilt raw-log reports. Do not treat mixed wrapper slices or sliced temp logs as authoritative for fidelity parity.
- 2026-03-28: The active follow-on tranche is now explicit: strengthen upstream source-truth preservation, raise bounded preview/edit fidelity, and target measured preview/import hotspots on `tertiary` from the proven baseline. Do not reopen baseline/parity proof work unless new evidence invalidates the `1500`-radius contract.

---

## File Structure

### Documentation and status consolidation

- Create: `docs/superpowers/status/2026-03-28-canonical-baseline-status.md`
- Modify: `docs/superpowers/plans/2026-03-26-play-preview-convergence.md`
- Modify: `docs/superpowers/plans/2026-03-26-play-preview-export-convergence-implementation.md`
- Modify: `docs/superpowers/specs/2026-03-28-canonical-baseline-and-single-source-of-truth-design.md`
- Modify: `docs/remote-studio-development.md`

### Bootstrap and world-root canonicalization

- Modify: `roblox/src/ServerScriptService/BootstrapAustin.server.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/BootstrapStateMachine.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/WorldStateApplier.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/MinimapService.lua`
- Modify: `roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua`
- Modify: `scripts/run_studio_harness.sh`
- Modify: `scripts/tests/test_austin_runtime_contract.py`
- Modify: `scripts/tests/test_run_studio_harness.py`

### Preview-vs-play parity coverage

- Modify: `roblox/src/ServerScriptService/Tests/CanonicalWorldParity.spec.lua`
- Create: `scripts/tests/test_preview_play_identity_contract.py`
- Modify: `scripts/tests/test_refresh_preview_from_sample_data.py`
- Modify: `scripts/tests/test_refresh_runtime_harness_from_sample_data.py`
- Modify: `scripts/refresh_preview_from_sample_data.py`
- Modify: `scripts/refresh_runtime_harness_from_sample_data.py`

### Remote validation and closure

- Modify: `docs/superpowers/status/2026-03-28-canonical-baseline-status.md`
- Modify as needed based on remote findings

## Task 1: Consolidate docs and create one current status trail

**Files:**
- Create: `docs/superpowers/status/2026-03-28-canonical-baseline-status.md`
- Modify: `docs/superpowers/plans/2026-03-26-play-preview-convergence.md`
- Modify: `docs/superpowers/plans/2026-03-26-play-preview-export-convergence-implementation.md`
- Modify: `docs/remote-studio-development.md`

- [ ] **Step 1: Write failing doc-guard assertions for the new baseline status trail**

Add or extend static tests so they assert:
- the status trail exists
- the stale March 26 convergence plans are marked historical or superseded near the top
- remote Studio docs describe operator-selected profiles and mention `tertiary` only as the local default for this workstation

- [ ] **Step 2: Run the focused static tests and verify failure**

Run:
```bash
python3 -m unittest \
  scripts.tests.test_convergence_guardrails \
  scripts.tests.test_run_studio_harness_remote -v
```

Expected: FAIL because the new status file and historical-plan markers do not exist yet.

- [ ] **Step 3: Implement the documentation cleanup**

Implement:
- one rolling baseline status file
- top-of-file historical/superseded markers in the stale March 26 plans
- a remote Studio doc wording cleanup that preserves profile aliases and notes `tertiary` as the current local default, not a committed repo dependency

- [ ] **Step 4: Re-run the focused tests and verify pass**

Run the same commands from Step 2.
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/status/2026-03-28-canonical-baseline-status.md \
  docs/superpowers/plans/2026-03-26-play-preview-convergence.md \
  docs/superpowers/plans/2026-03-26-play-preview-export-convergence-implementation.md \
  docs/remote-studio-development.md
git commit -m "docs: establish canonical baseline status trail"
```

## Task 2: Canonicalize live bootstrap state ownership and attempt identity

**Files:**
- Modify: `roblox/src/ServerScriptService/BootstrapAustin.server.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/BootstrapStateMachine.lua`
- Modify: `scripts/run_studio_harness.sh`
- Modify: `scripts/tests/test_austin_runtime_contract.py`
- Modify: `scripts/tests/test_run_studio_harness.py`

- [ ] **Step 1: Write the failing bootstrap-owner tests**

Add tests that assert:
- live bootstrap uses the canonical bootstrap-state owner instead of duplicating inline state handling
- `ArnisAustinBootstrapAttemptId` is published by the live runtime path
- harness probe extraction keys off the same canonical attempt-id and state-trace attributes

- [ ] **Step 2: Run the focused tests and verify failure**

Run:
```bash
python3 -m unittest \
  scripts.tests.test_austin_runtime_contract \
  scripts.tests.test_run_studio_harness -v
```

Expected: FAIL because live bootstrap still owns state inline and does not publish the canonical attempt-id path.

- [ ] **Step 3: Implement the minimal bootstrap canonicalization**

Implement one of:
- wire `BootstrapAustin.server.lua` to `BootstrapStateMachine.lua`

Do not leave both the module and a separate inline owner active.

- [ ] **Step 4: Re-run the focused tests and verify pass**

Run the same commands from Step 2.
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add roblox/src/ServerScriptService/BootstrapAustin.server.lua \
  roblox/src/ServerScriptService/ImportService/BootstrapStateMachine.lua \
  scripts/run_studio_harness.sh \
  scripts/tests/test_austin_runtime_contract.py \
  scripts/tests/test_run_studio_harness.py
git commit -m "fix: canonicalize live bootstrap state ownership"
```

## Task 3: Publish canonical world-root truth outside minimap startup

**Files:**
- Modify: `roblox/src/ServerScriptService/ImportService/WorldStateApplier.lua`
- Modify: `roblox/src/ServerScriptService/ImportService/MinimapService.lua`
- Modify: `roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua`
- Modify: `scripts/tests/test_austin_runtime_contract.py`
- Modify: `scripts/tests/test_run_studio_harness.py`

- [ ] **Step 1: Write the failing world-root publication tests**

Add tests that assert:
- world-root publication is owned outside minimap startup
- preview remains observable when minimap startup is disabled
- `ArnisMinimapWorldRootName` is mirrored only as backward compatibility, not as the canonical source

- [ ] **Step 2: Run the focused tests and verify failure**

Run:
```bash
python3 -m unittest \
  scripts.tests.test_austin_runtime_contract \
  scripts.tests.test_run_studio_harness -v
```

Expected: FAIL because world-root publication is still minimap-owned.

- [ ] **Step 3: Implement the minimal canonical world-root publisher**

Implement:
- one canonical world-root attribute published from the world-state/import owner
- compatibility mirroring for current minimap consumers
- direct probe consumption of the canonical attribute

- [ ] **Step 4: Re-run the focused tests and verify pass**

Run the same commands from Step 2.
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add roblox/src/ServerScriptService/ImportService/WorldStateApplier.lua \
  roblox/src/ServerScriptService/ImportService/MinimapService.lua \
  roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua \
  scripts/tests/test_austin_runtime_contract.py \
  scripts/tests/test_run_studio_harness.py
git commit -m "fix: publish canonical world root outside minimap startup"
```

## Task 4: Extend preview-vs-play parity coverage for source identity and minimap basis

**Files:**
- Create: `scripts/tests/test_preview_play_identity_contract.py`
- Modify: `roblox/src/ServerScriptService/Tests/CanonicalWorldParity.spec.lua`
- Modify: `scripts/tests/test_refresh_preview_from_sample_data.py`
- Modify: `scripts/tests/test_refresh_runtime_harness_from_sample_data.py`
- Modify: `scripts/refresh_preview_from_sample_data.py`
- Modify: `scripts/refresh_runtime_harness_from_sample_data.py`

- [ ] **Step 1: Write the failing parity-extension tests**

Add tests that assert a shared envelope contract yields:
- the same manifest-side source-feature identity sets by family and chunk membership
- the same canonical minimap/static transform basis inputs before runtime-only policy differences

Preserve existing Luau parity assertions for chunk IDs and canonical anchor values.

- [ ] **Step 2: Run the focused tests and verify failure**

Run:
```bash
python3 -m unittest \
  scripts.tests.test_preview_play_identity_contract \
  scripts.tests.test_refresh_preview_from_sample_data \
  scripts.tests.test_refresh_runtime_harness_from_sample_data -v
```

Expected: FAIL because the identity/minimap-basis parity surface does not yet exist.

- [ ] **Step 3: Implement the minimal parity-surface changes**

Implement:
- normalized identity summaries in the fixture refresh paths
- parity assertions without redefining manifest truth
- no second preview/play world-definition path

- [ ] **Step 4: Re-run the focused tests and verify pass**

Run the same commands from Step 2.
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add scripts/tests/test_preview_play_identity_contract.py \
  roblox/src/ServerScriptService/Tests/CanonicalWorldParity.spec.lua \
  scripts/tests/test_refresh_preview_from_sample_data.py \
  scripts/tests/test_refresh_runtime_harness_from_sample_data.py \
  scripts/refresh_preview_from_sample_data.py \
  scripts/refresh_runtime_harness_from_sample_data.py
git commit -m "test: extend preview play parity coverage"
```

## Task 5: Prove the cleaned baseline on the remote Studio profile

**Files:**
- Modify: `docs/superpowers/status/2026-03-28-canonical-baseline-status.md`
- Modify as needed based on findings

- [ ] **Step 1: Run local static verification**

Run:
```bash
python3 -m unittest \
  scripts.tests.test_convergence_guardrails \
  scripts.tests.test_austin_runtime_contract \
  scripts.tests.test_run_studio_harness \
  scripts.tests.test_run_studio_harness_remote \
  scripts.tests.test_preview_play_identity_contract \
  scripts.tests.test_refresh_preview_from_sample_data \
  scripts.tests.test_refresh_runtime_harness_from_sample_data -v
cargo test --manifest-path rust/Cargo.toml --workspace
```

Expected: PASS

- [ ] **Step 2: Run remote edit validation on the selected profile**

For this workstation setup, default to `tertiary`:

```bash
bash scripts/run_studio_harness_remote.sh --remote-profile tertiary -- \
  --no-play --edit-tests --takeover --hard-restart \
  --spec-filter CanonicalWorldContract.spec.lua

bash scripts/run_studio_harness_remote.sh --remote-profile tertiary -- \
  --no-play --edit-tests --takeover --hard-restart \
  --spec-filter CanonicalWorldParity.spec.lua
```

Expected: PASS

- [ ] **Step 3: Run remote play-world validation on the selected profile**

```bash
bash scripts/run_studio_harness_remote.sh --remote-profile tertiary -- \
  --play --takeover --hard-restart --edit-wait 30 --pattern-wait 120
```

Expected:
- bootstrap attempt identity present
- monotonic runtime state trace present
- non-empty world root
- no obvious startup-vs-streaming regression markers

- [ ] **Step 4: Record a dated status note**

Append to:
`docs/superpowers/status/2026-03-28-canonical-baseline-status.md`

Include:
- local verification commands and outcomes
- remote edit/play commands and outcomes
- measured residual gaps after the baseline pass

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/status/2026-03-28-canonical-baseline-status.md
git commit -m "docs: record canonical baseline verification status"
```
