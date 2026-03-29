# 2026-03-29 Breaking Compatibility Purge Design

## Goal

Deliberately simplify the repository by removing legacy and backward-compatibility code paths instead
of preserving them behind shims. The cleaned baseline should prefer one canonical contract, one code
path, one fixture family, and one documentation truth surface.

This is a breaking change by design.

## Decision

Keep manifest/schema contract `0.4.0` as the only supported version.

Do not combine this cleanup with a `0.5.0` semantic change. A schema bump would add churn without
making the codebase cleaner unless manifest meaning is also changing. The cleanest path is:

- support exactly one manifest/schema contract: `0.4.0`
- delete migrations and compatibility branches for older versions
- rewrite or remove legacy fixtures and examples
- make import/export/audit tooling fail loudly on non-`0.4.0` inputs

## Why This Is Cleaner

This avoids two forms of unnecessary complexity:

1. migration-era complexity
   - code that exists only to translate `0.1.0` / `0.2.0` / `0.3.0`
   - test fixtures that exercise contracts the repo no longer wants to carry
   - docs that describe historical support as active behavior

2. mixed-truth complexity
   - fallback diagnostics that look canonical
   - multiple accepted fixture families
   - CLI/help text that implies broader compatibility than the repo should guarantee

The result should be a narrower and more honest codebase: one truth, no historical baggage.

## Non-Goals

- no new manifest semantics
- no opportunistic architecture rewrite unrelated to compatibility removal
- no partial “soft deprecation” period
- no preservation of old sample/generated manifests just for nostalgia or migration coverage

## Scope

### 1. Contract purge

Remove compatibility surfaces that accept or translate older manifest/schema versions.

Targets include:

- Roblox-side migration modules and callers
- migration-specific tests
- Rust CLI/example/test inputs that still validate or compare legacy schema versions
- importer/validator branches that imply old versions are still supported

Desired end state:

- `0.4.0` is the only accepted manifest schema version
- non-`0.4.0` inputs fail immediately with a clear error

### 2. Fixture and generated artifact purge

Remove or rewrite artifacts that preserve old schema families when they are no longer part of the
supported contract.

Targets include:

- `specs/generated/*` legacy schema fixtures
- older sample manifests used only for migration tests
- exporter fixtures and docs that still claim `0.3.0` output

Desired end state:

- committed fixtures represent the current contract only
- generated examples no longer imply support for obsolete schema versions

### 3. Documentation purge

Update canonical docs to reflect the break explicitly and remove stale compatibility claims.

Required doc surfaces:

- `docs/chunk_schema.md`
- CLI/help documentation that still references older schema families
- active superpowers plan/status docs if they mention compatibility as current policy

Desired end state:

- docs say older schemas are unsupported
- there is no active doc drift about auto-migration or compatibility guarantees that no longer exist

### 4. Runtime compatibility purge

Remove compatibility mirrors and fallback paths that exist only for migration-era coexistence,
provided they are not required by the current proven edit/play baseline.

Rule:

- if a path exists only to preserve old consumers, delete it
- if a path is still required by the current canonical edit/play proof lane, keep it until the
  consuming surface is replaced in the same tranche

This prevents “cleanup” from accidentally breaking the currently proven baseline while still pushing
the runtime toward a single source of truth.

## Architecture Rules For This Cleanup

- One supported schema contract: `0.4.0`
- One canonical runtime world-truth path
- One fixture family for supported inputs
- One active documentation truth per topic
- Fail loudly instead of silently migrating or guessing

## Implementation Strategy

Execute in bounded slices rather than a giant repo-wide rewrite.

### Slice A: schema acceptance and migration removal

- remove migration machinery
- tighten validators and tests to hard-fail on older schema versions

### Slice B: fixture and example cleanup

- rewrite or remove obsolete fixtures and generated examples
- update tests to stop relying on old manifest families

### Slice C: runtime/fallback cleanup

- remove migration-era fallback paths that are no longer necessary
- preserve only the canonical play/edit proof path

### Slice D: documentation and verification cleanup

- rewrite docs to match the new break
- run local verification and targeted `tertiary` proof slices
- update rolling status docs so no stale compatibility claims remain

## Error Handling

After this purge, invalid compatibility inputs should fail clearly:

- unsupported schema version errors must name the received version and the required `0.4.0`
- tooling should not auto-upgrade, auto-coerce, or silently drop unsupported compatibility data
- partial/filtered telemetry runs must remain explicitly marked as partial and must not masquerade
  as full fidelity audits

## Testing Strategy

The cleanup is complete only when tests prove both removal and preservation boundaries.

Required verification classes:

- unit tests asserting non-`0.4.0` inputs hard-fail
- fixture/tests updated to current-only contract
- contract tests proving canonical runtime truth still holds
- scene audit/parity tests still green on the current baseline
- targeted `tertiary` runs for any runtime cleanup that touches the proven edit/play path

## Risks

### 1. Hidden consumers of compatibility paths

Some compatibility branches may still be serving current dev workflows or harnesses indirectly.

Mitigation:

- remove them in bounded slices
- verify each slice before moving on

### 2. Historical fixture assumptions embedded in tests

Legacy fixtures may be wired into more places than expected.

Mitigation:

- replace tests with current-contract fixtures rather than mass deleting blindly

### 3. Documentation drift during the break

This repo has multiple plan/spec/status surfaces, so cleanup can create stale claims quickly.

Mitigation:

- update canonical docs in the same tranche as code removal

## Definition Of Done

This cleanup is done when all of the following are true:

- only schema `0.4.0` is supported
- legacy migration code is removed
- obsolete compatibility fixtures/examples are removed or rewritten
- canonical docs no longer claim auto-migration or old-version support
- runtime truth surfaces no longer carry migration-era shims that are not needed
- local verification is green
- any touched runtime proof lanes remain green on `tertiary`

