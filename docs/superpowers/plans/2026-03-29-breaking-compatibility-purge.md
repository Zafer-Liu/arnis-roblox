# Breaking Compatibility Purge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove backward-compatibility and legacy-code paths so the repo supports only schema `0.4.0`, carries one canonical runtime truth path, and no longer preserves migration-era fixtures or docs.

**Architecture:** Keep `0.4.0` as the single canonical contract and delete code that exists only to translate, accept, or describe older schema versions. Execute the purge in bounded slices so contract breakage is explicit, tests move first, and the current proven edit/play runtime baseline remains green while migration-era shims are removed.

**Tech Stack:** Python unittest, Luau runtime/importer code, Rust CLI/export tooling, shell harness scripts, Markdown docs

---

## File Map

### Contract and schema enforcement

- Modify: `roblox/src/ReplicatedStorage/Shared/Version.lua`
  - keep `0.4.0` as the only supported Roblox-side schema constant
- Modify or delete: `roblox/src/ReplicatedStorage/Shared/Migrations.lua`
  - remove migration machinery for `0.1.0` / `0.2.0` / `0.3.0`
- Modify: `roblox/src/ReplicatedStorage/Shared/ChunkSchema.lua`
  - hard-fail on non-`0.4.0`
- Modify: `rust/crates/arbx_cli/src/main.rs`
  - remove legacy-version examples/tests and keep CLI/help text aligned to `0.4.0` only

### Fixture and artifact cleanup

- Modify or delete: `specs/generated/*.json`
  - remove legacy-version generated fixtures or rewrite them to `0.4.0`
- Modify: `specs/sample-chunk-manifest.json`
  - move to `0.4.0` or replace with a current-only fixture
- Modify: `docs/exporter-fixtures.md`
  - stop claiming `0.3.0` fixture output/support
- Modify: `scripts/generate_synthetic_manifest.py`
  - emit only `0.4.0`

### Tests

- Modify: `roblox/src/ServerScriptService/Tests/ChunkSchema.spec.lua`
- Modify or delete: `roblox/src/ServerScriptService/Tests/Migrations.spec.lua`
- Modify: `roblox/src/ServerScriptService/Tests/AuthoritativeOverwrite.spec.lua`
- Modify: `roblox/src/ServerScriptService/Tests/ImportChunkPlanKey.spec.lua`
- Modify: `roblox/src/ServerScriptService/Tests/RoadDetailGroups.spec.lua`
- Modify: `roblox/src/ServerScriptService/Tests/LandusePerformance.spec.lua`
- Modify: `roblox/src/ServerScriptService/Tests/RoadChunkPlanReuse.spec.lua`
- Modify: `roblox/src/ServerScriptService/Tests/Streaming.spec.lua`
- Modify: `roblox/src/ServerScriptService/Tests/PlacementHardening.spec.lua`
- Modify: `roblox/src/ServerScriptService/Tests/AustinPreviewTimeTravel.spec.lua`
- Modify: `scripts/tests/test_refresh_preview_from_sample_data.py`
  - remove migration-era acceptance expectations that are no longer valid
- Modify: `scripts/tests/test_refresh_runtime_harness_from_sample_data.py`
  - same cleanup for runtime seed refresh path
- Modify: `scripts/tests/test_json_manifest_to_sharded_lua.py`
  - keep only current-contract assertions
- Modify: `scripts/tests/test_scene_fidelity_audit.py`
  - ensure current-only fixtures remain sufficient
- Modify: `scripts/check_scaffold.py`
- Modify: `scripts/run_all_checks.py`
- Modify: `scripts/verify_generated_austin_assets.py`
- Modify: `scripts/tests/test_run_all_checks.py`
- Modify: `scripts/tests/test_generated_austin_assets.py`
- Modify/add: schema validation and CLI tests under existing Rust/Luau test surfaces
  - replace migration expectations with hard-fail expectations

### Runtime compatibility cleanup

- Inspect and modify only if direct legacy-compatibility evidence remains after Tasks 1-4:
  - `roblox/src/ServerScriptService/ImportService/MinimapService.lua`
  - `roblox/src/ServerScriptService/ImportService/RunAustin.lua`
  - `roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua`
  - `scripts/run_studio_harness.sh`
  - `scripts/scene_fidelity_audit.py`
  - `scripts/scene_parity_audit.py`

### Documentation truth

- Modify: `docs/chunk_schema.md`
- Modify: `docs/build-pipeline.md`
- Modify: `docs/architecture.md`
- Modify: `docs/remote-studio-development.md`
- Modify: `docs/superpowers/status/2026-03-28-play-fidelity-and-observability-status.md`
- Modify: `docs/superpowers/plans/2026-03-28-play-fidelity-and-observability.md`

## Task 1: Remove Roblox-Side Schema Migration Support

**Files:**
- Modify: `roblox/src/ReplicatedStorage/Shared/Migrations.lua`
- Modify: `roblox/src/ReplicatedStorage/Shared/ChunkSchema.lua`
- Modify: `roblox/src/ReplicatedStorage/Shared/Version.lua`
- Modify: `roblox/src/ServerScriptService/Tests/ChunkSchema.spec.lua`
- Modify or delete: `roblox/src/ServerScriptService/Tests/Migrations.spec.lua`
- Modify: `roblox/src/ServerScriptService/Tests/AuthoritativeOverwrite.spec.lua`
- Modify: `roblox/src/ServerScriptService/Tests/ImportChunkPlanKey.spec.lua`
- Modify: `roblox/src/ServerScriptService/Tests/RoadDetailGroups.spec.lua`
- Modify: `roblox/src/ServerScriptService/Tests/LandusePerformance.spec.lua`
- Modify: `roblox/src/ServerScriptService/Tests/RoadChunkPlanReuse.spec.lua`
- Modify: `roblox/src/ServerScriptService/Tests/Streaming.spec.lua`
- Modify: `roblox/src/ServerScriptService/Tests/PlacementHardening.spec.lua`
- Modify: `roblox/src/ServerScriptService/Tests/AustinPreviewTimeTravel.spec.lua`

- [ ] **Step 1: Write the failing tests**

Add or update Luau tests so manifests with `schemaVersion = "0.1.0"`, `"0.2.0"`, or `"0.3.0"` now fail instead of migrating.

Example assertion shape:

```lua
it("rejects pre-0.4.0 manifests instead of migrating them", function()
    local legacy = {
        schemaVersion = "0.3.0",
        meta = { metersPerStud = 1.0, chunkSizeStuds = 256 },
        chunks = {},
    }
    local ok, err = pcall(function()
        ChunkSchema.validateManifest(legacy)
    end)
    expect(ok).to.equal(false)
    expect(tostring(err)).to.contain("0.4.0")
end)
```

- [ ] **Step 2: Run the focused Luau/shared-schema tests to verify they fail**

Run:
```bash
python3 scripts/run_luau_tests.py \
  roblox/src/ServerScriptService/Tests/ChunkSchema.spec.lua \
  roblox/src/ServerScriptService/Tests/Migrations.spec.lua \
  roblox/src/ServerScriptService/Tests/AuthoritativeOverwrite.spec.lua \
  roblox/src/ServerScriptService/Tests/ImportChunkPlanKey.spec.lua \
  roblox/src/ServerScriptService/Tests/RoadDetailGroups.spec.lua \
  roblox/src/ServerScriptService/Tests/LandusePerformance.spec.lua \
  roblox/src/ServerScriptService/Tests/RoadChunkPlanReuse.spec.lua \
  roblox/src/ServerScriptService/Tests/Streaming.spec.lua \
  roblox/src/ServerScriptService/Tests/PlacementHardening.spec.lua \
  roblox/src/ServerScriptService/Tests/AustinPreviewTimeTravel.spec.lua
```

Expected: FAIL because migration-era acceptance still exists

- [ ] **Step 3: Write the minimal implementation**

- remove migration chaining logic from `Migrations.lua`, or reduce the module to a hard-fail helper if callers still require the module boundary
- update `ChunkSchema.lua` so non-`0.4.0` manifests fail immediately with a clear error
- keep `Version.lua` authoritative at `0.4.0`

- [ ] **Step 4: Run the focused tests to verify they pass**

Run the same Luau/shared-schema command  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add roblox/src/ReplicatedStorage/Shared/Migrations.lua roblox/src/ReplicatedStorage/Shared/ChunkSchema.lua roblox/src/ReplicatedStorage/Shared/Version.lua
git commit -m "refactor: drop Roblox schema migration support"
```

## Task 2: Tighten Rust CLI and Tooling to 0.4.0 Only

**Files:**
- Modify: `rust/crates/arbx_cli/src/main.rs`
- Modify: `scripts/generate_synthetic_manifest.py`
- Test: Rust CLI unit tests in `rust/crates/arbx_cli/src/main.rs`

- [ ] **Step 1: Write the failing tests**

Add or update Rust tests so:

- legacy example payloads are no longer treated as acceptable fixture inputs
- unsupported schema versions explicitly fail with `0.4.0`-only language

Example assertion shape:

```rust
#[test]
fn schema_index_rejects_non_040_manifest() {
    let legacy = r#"{ "schemaVersion": "0.3.0", "meta": {}, "chunks": [] }"#;
    let err = validate_manifest_schema_only(legacy).unwrap_err();
    assert!(err.to_string().contains("0.4.0"));
}
```

- [ ] **Step 2: Run the focused Rust tests to verify they fail**

Run: `cargo test --manifest-path rust/Cargo.toml -p arbx_cli -- --nocapture`  
Expected: FAIL in legacy-schema acceptance paths

- [ ] **Step 3: Write the minimal implementation**

- remove legacy schema examples and comparison cases in `main.rs`
- keep help text, explain text, and sample output aligned to `0.4.0`
- update `generate_synthetic_manifest.py` to emit only `0.4.0`

- [ ] **Step 4: Run the focused Rust tests to verify they pass**

Run: `cargo test --manifest-path rust/Cargo.toml -p arbx_cli -- --nocapture`  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rust/crates/arbx_cli/src/main.rs scripts/generate_synthetic_manifest.py
git commit -m "refactor: make CLI fixtures 0.4.0-only"
```

## Task 3: Purge Legacy Fixtures and Generated Samples

**Files:**
- Modify or delete: `specs/generated/*.json`
- Modify: `specs/sample-chunk-manifest.json`
- Modify: `docs/exporter-fixtures.md`
- Modify: `scripts/check_scaffold.py`
- Modify: `scripts/run_all_checks.py`
- Modify: `scripts/verify_generated_austin_assets.py`
- Modify: `scripts/tests/test_run_all_checks.py`
- Modify: `scripts/tests/test_generated_austin_assets.py`
- Test: Python tests that currently rely on legacy schema fixture files

- [ ] **Step 1: Write the failing tests**

Update tests to stop expecting legacy-version fixture artifacts and to require only `0.4.0` examples.

Example assertion shape:

```python
def test_sample_chunk_manifest_uses_current_schema_only():
    payload = json.loads(Path("specs/sample-chunk-manifest.json").read_text())
    assert payload["schemaVersion"] == "0.4.0"
```

- [ ] **Step 2: Run the focused Python tests to verify they fail**

Run:
```bash
python3 -m unittest scripts.tests.test_run_all_checks scripts.tests.test_generated_austin_assets -v
python3 scripts/check_scaffold.py
python3 scripts/verify_generated_austin_assets.py
```

Expected: FAIL because current fixtures still contain legacy schema versions or validation scripts still assume legacy support

- [ ] **Step 3: Write the minimal implementation**

- remove truly obsolete generated files that only exist for migration-era coverage
- rewrite remaining sample/spec fixtures to `0.4.0`
- update exporter fixture docs so they do not mention `0.3.0` as active output

- [ ] **Step 4: Run the focused Python tests to verify they pass**

Run the same commands  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add specs docs/exporter-fixtures.md scripts/check_scaffold.py scripts/run_all_checks.py scripts/verify_generated_austin_assets.py scripts/tests/test_run_all_checks.py scripts/tests/test_generated_austin_assets.py
git commit -m "refactor: remove legacy schema fixtures"
```

## Task 4: Clean Python Refresh and Sharding Paths of Compatibility Assumptions

**Files:**
- Modify: `scripts/refresh_preview_from_sample_data.py`
- Modify: `scripts/refresh_runtime_harness_from_sample_data.py`
- Modify: `scripts/json_manifest_to_sharded_lua.py`
- Test:
  - `scripts/tests/test_refresh_preview_from_sample_data.py`
  - `scripts/tests/test_refresh_runtime_harness_from_sample_data.py`
  - `scripts/tests/test_json_manifest_to_sharded_lua.py`

- [ ] **Step 1: Write the failing tests**

Replace migration-style success expectations with hard-fail expectations for non-`0.4.0` inputs.

Example assertion shape:

```python
def test_refresh_preview_rejects_non_040_schema():
    with self.assertRaises(SystemExit) as ctx:
        refresh_preview_from_manifest(legacy_manifest_path)
    self.assertIn("0.4.0", str(ctx.exception))
```

- [ ] **Step 2: Run the focused Python tests to verify they fail**

Run:
```bash
python3 -m unittest \
  scripts.tests.test_refresh_preview_from_sample_data \
  scripts.tests.test_refresh_runtime_harness_from_sample_data \
  scripts.tests.test_json_manifest_to_sharded_lua -v
```

Expected: FAIL because refresh/sharding paths still preserve older-version assumptions

- [ ] **Step 3: Write the minimal implementation**

- remove compatibility branches that tolerate older schema families
- make refresh/sharding fail clearly on non-`0.4.0`
- keep emitted Lua index/schema text current-only

- [ ] **Step 4: Run the focused tests to verify they pass**

Run the same unittest command  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add scripts/refresh_preview_from_sample_data.py scripts/refresh_runtime_harness_from_sample_data.py scripts/json_manifest_to_sharded_lua.py scripts/tests/test_refresh_preview_from_sample_data.py scripts/tests/test_refresh_runtime_harness_from_sample_data.py scripts/tests/test_json_manifest_to_sharded_lua.py
git commit -m "refactor: remove legacy schema assumptions from refresh tooling"
```

## Task 5: Inventory Remaining Runtime Legacy-Compatibility Evidence

**Files:**
- Inspect and modify as needed:
  - `roblox/src/ServerScriptService/ImportService/MinimapService.lua`
  - `roblox/src/ServerScriptService/ImportService/RunAustin.lua`
  - `roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua`
  - `scripts/run_studio_harness.sh`
  - `scripts/scene_fidelity_audit.py`
  - `scripts/scene_parity_audit.py`
- Test:
  - `scripts/tests/test_austin_runtime_contract.py`
  - `scripts/tests/test_run_studio_harness.py`
  - `scripts/tests/test_scene_fidelity_audit.py`
  - `scripts/tests/test_scene_parity_audit.py`

- [ ] **Step 1: Write the failing tests**

First prove that a runtime file still contains a direct pre-`0.4.0` compatibility branch before touching it.

Examples:

```python
def test_runtime_file_contains_direct_legacy_manifest_compatibility_branch():
    self.assertRegex(text, r"0\\.1\\.0|0\\.2\\.0|0\\.3\\.0|migration|legacy schema")
```

```python
def test_runtime_cleanup_inventory_is_empty_after_tasks_1_to_4():
    self.assertEqual(legacy_hits, [])
```

- [ ] **Step 2: Run the focused runtime/audit tests to verify they fail**

Run:
```bash
python3 -m unittest \
  scripts.tests.test_austin_runtime_contract \
  scripts.tests.test_run_studio_harness \
  scripts.tests.test_scene_fidelity_audit \
  scripts.tests.test_scene_parity_audit -v
```

Expected: FAIL only if direct legacy-compatibility evidence remains

- [ ] **Step 3: Write the minimal implementation**

- delete only branches whose sole purpose is accepting, translating, or masking pre-`0.4.0` manifest inputs
- do not remove canonical preview/play/full-bake routing, canonical materialization fallback resolution, or current proof-lane behavior
- if no direct legacy-compatibility evidence remains, skip implementation and mark the task complete as a verified no-op

- [ ] **Step 4: Run the focused runtime/audit tests to verify they pass**

Run the same unittest command  
Expected: PASS

- [ ] **Step 5: Verify on the configured remote profile if runtime truth changed**

Run targeted remote proof lanes only if touched runtime code affects play/edit truth, using the
existing remote harness/profile entrypoint rather than hardcoded host paths.

Expected: current canonical proof lane remains green

- [ ] **Step 6: Commit**

```bash
git add roblox/src/ServerScriptService/ImportService/MinimapService.lua roblox/src/ServerScriptService/ImportService/RunAustin.lua roblox/src/StarterPlayer/StarterPlayerScripts/WorldProbe.client.lua scripts/run_studio_harness.sh scripts/scene_fidelity_audit.py scripts/scene_parity_audit.py scripts/tests/test_austin_runtime_contract.py scripts/tests/test_run_studio_harness.py scripts/tests/test_scene_fidelity_audit.py scripts/tests/test_scene_parity_audit.py
git commit -m "refactor: remove remaining runtime legacy compatibility shims"
```

## Task 6: Rewrite Canonical Docs and Status Surfaces

**Files:**
- Modify: `docs/chunk_schema.md`
- Modify: `docs/build-pipeline.md`
- Modify: `docs/architecture.md`
- Modify: `docs/remote-studio-development.md`
- Modify: `docs/superpowers/plans/2026-03-28-play-fidelity-and-observability.md`
- Modify: `docs/superpowers/status/2026-03-28-play-fidelity-and-observability-status.md`

- [ ] **Step 1: Write the failing doc-truth tests or assertions where available**

Where text-based tests exist, add/update them. Otherwise create a small focused verification step that greps for stale claims across the active docs, status, and repo-level operating manuals.

Example command:

```bash
rg -n "Automatically migrated|backward compatibility|0.3.0 manifest JSON|0.1.0|0.2.0|0.3.0" docs/exporter-fixtures.md docs/chunk_schema.md docs/build-pipeline.md docs/architecture.md docs/remote-studio-development.md docs/superpowers/plans/2026-03-28-play-fidelity-and-observability.md docs/superpowers/status/2026-03-28-play-fidelity-and-observability-status.md specs/sample-chunk-manifest.json specs/generated rust/crates/arbx_cli/src/main.rs
```

Expected before cleanup: matches still exist in active truth surfaces

- [ ] **Step 2: Rewrite the docs**

- `docs/chunk_schema.md` must state that older schemas are unsupported
- remove active-language compatibility claims from build/architecture/operator docs
- reframe the March 28 plan/status docs as historical context instead of active instructions
- append a dated status note explaining the break and the new repo truth

- [ ] **Step 3: Run the doc-truth verification**

Run:

```bash
rg -n "Automatically migrated|backward compatibility|migration notes|Status: Active|active truth surface|0\.1\.0|0\.2\.0|0\.3\.0|--remote-host tertiary|~/.codex-remote-studio|schema migrations" AGENTS.md CLAUDE.md docs/chunk_schema.md docs/build-pipeline.md docs/architecture.md docs/remote-studio-development.md docs/superpowers/plans/2026-03-28-play-fidelity-and-observability.md docs/superpowers/status/2026-03-28-play-fidelity-and-observability-status.md
```

Expected: no stale compatibility claims in active surfaces

- [ ] **Step 4: Commit**

```bash
git add docs/chunk_schema.md docs/build-pipeline.md docs/architecture.md docs/remote-studio-development.md docs/superpowers/plans/2026-03-28-play-fidelity-and-observability.md docs/superpowers/status/2026-03-28-play-fidelity-and-observability-status.md
git commit -m "docs: remove stale compatibility claims"
```

## Task 7: Final Verification and Integration

**Files:**
- No new code targets; verify the repo state after all prior tasks

- [ ] **Step 1: Run the full local verification suite**

Run:

```bash
bash -n scripts/run_studio_harness.sh
python3 -m unittest \
  scripts.tests.test_run_all_checks \
  scripts.tests.test_generated_austin_assets \
  scripts.tests.test_refresh_preview_from_sample_data \
  scripts.tests.test_refresh_runtime_harness_from_sample_data \
  scripts.tests.test_json_manifest_to_sharded_lua \
  scripts.tests.test_run_studio_harness \
  scripts.tests.test_austin_runtime_contract \
  scripts.tests.test_scene_fidelity_audit \
  scripts.tests.test_scene_parity_audit \
  scripts.tests.test_preview_telemetry_summary -v
python3 scripts/check_scaffold.py
python3 scripts/verify_generated_austin_assets.py
python3 scripts/run_luau_tests.py \
  roblox/src/ServerScriptService/Tests/ChunkSchema.spec.lua \
  roblox/src/ServerScriptService/Tests/Migrations.spec.lua \
  roblox/src/ServerScriptService/Tests/AuthoritativeOverwrite.spec.lua \
  roblox/src/ServerScriptService/Tests/ImportChunkPlanKey.spec.lua \
  roblox/src/ServerScriptService/Tests/RoadDetailGroups.spec.lua \
  roblox/src/ServerScriptService/Tests/LandusePerformance.spec.lua \
  roblox/src/ServerScriptService/Tests/RoadChunkPlanReuse.spec.lua \
  roblox/src/ServerScriptService/Tests/Streaming.spec.lua \
  roblox/src/ServerScriptService/Tests/PlacementHardening.spec.lua \
  roblox/src/ServerScriptService/Tests/AustinPreviewTimeTravel.spec.lua
cargo test --manifest-path rust/Cargo.toml --workspace
git diff --check
```

Expected: all pass

- [ ] **Step 2: Run targeted `tertiary` verification for any touched runtime truth**

Run the existing remote harness/profile entrypoint against the configured `tertiary` profile for
the touched runtime verification lanes.

If runtime truth changed materially, also run the narrow Studio proof lane through that same
profile-based flow.

- [ ] **Step 3: Update the rolling status doc with final measured truth**

Append a dated note to:

- `docs/superpowers/status/2026-03-28-play-fidelity-and-observability-status.md`

Include:

- compatibility purge landed
- `0.4.0` is the only supported schema contract
- any runtime proof result from `tertiary`
- any residual follow-up work that remains

- [ ] **Step 4: Final commit**

```bash
git add docs/superpowers/status/2026-03-28-play-fidelity-and-observability-status.md
git commit -m "refactor: complete compatibility purge"
```
