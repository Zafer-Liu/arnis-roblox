# Remote Studio Development

Use this repo so Studio execution can move between machines without committing machine-specific hostnames, usernames, paths, or secrets.

Remote Studio validation is profile-based. On this workstation, `tertiary` is the preferred proof profile, but it is still only a local alias, not a committed repo dependency.

Treat `tertiary` as the default remote proof lane when you need cross-machine Studio validation. If wrapper transport is unhealthy, direct SSH into a git-backed remote `tertiary` clone is the authoritative fallback for that lane. For current proof state and active follow-on work, use the single active rolling status file instead of duplicating volatile results here:

- `docs/superpowers/status/2026-03-30-outdoor-fidelity-and-source-truth-status.md`

The March 30 outdoor fidelity and source-truth tranche is the current rolling status surface. Keep `tertiary` as the proof lane, but avoid repeating fast-moving proof claims in this operator doc.

## Rules

- Keep remote host aliases and machine-specific paths in `scripts/remote_studio_profiles.local.sh`, not in committed scripts.
- Use `scripts/remote_studio_profiles.example.sh` as the starting template.
- Treat `primary` and `tertiary` as profile aliases and machine roles, not as committed transport details.
- Prefer direct development on the chosen dev machine when possible.
- Use the remote harness wrapper when Studio must run on another machine from the one holding your current worktree; if wrapper transport is unhealthy, switch to direct SSH on a real remote `tertiary` git clone for the proof run.

## Direct Development On The Active Dev Machine

1. Run the machine bootstrap once:

```bash
bash scripts/setup_remote_dev_machine.sh
```

This standardizes the expected Homebrew toolchain (`tmux`, `mosh`, `uv`) and installs the shared tmux overlay used by Blink/Termius remote sessions.

2. Keep a normal clone or worktree of `arnis-roblox` on the current development machine.
3. Keep the adjacent `vertigo-sync` repo on that same machine.
4. Run the local harness there:

```bash
bash scripts/run_studio_harness.sh --no-play --edit-tests
```

That path is the simplest and should be the default when `primary` is acting as the active development machine.

## Remote Terminal Development

After `bash scripts/setup_remote_dev_machine.sh` has run on a remote macOS machine:

- use Tailscale/SSH aliases from your ignored local profile config
- use `mosh <profile>` from Blink or Termius when the machine has a working Tailscale install
- land in a persistent tmux session with a command like:

```bash
mosh primary -- tmux new -A -s dev
mosh tertiary -- tmux new -A -s dev
```

If Tailscale is installed but still needs an interactive admin or sign-in step, the setup script should report that explicitly instead of pretending the machine is ready.

## Remote Harness From Another Machine

1. Copy the template:

```bash
cp scripts/remote_studio_profiles.example.sh scripts/remote_studio_profiles.local.sh
```

2. Fill in your local SSH aliases or transport targets for the `primary` and `tertiary` profiles.
3. Run the remote wrapper with the chosen profile:

```bash
bash scripts/run_studio_harness_remote.sh --remote-profile tertiary -- --no-play --edit-tests
```

If the wrapper is unstable on the current workstation, run the harness directly on a git-backed remote clone over SSH instead and record that lane in the status file:

```bash
# Replace the placeholder paths with the real git-backed proof clone and the operator-owned
# vertigo-sync checkout or stage on the remote host.
ssh tertiary 'cd "/path/to/arnis-roblox" && VSYNC_REPO_DIR="/path/to/vertigo-sync" bash scripts/run_studio_harness.sh --no-play --edit-tests'
```

## Notes

- The committed wrapper supports profile aliases, but not committed real hostnames, usernames, IPs, or `.local` machine names.
- Remote stage paths default to `__REMOTE_HOME__/...` templates and expand on the remote machine.
- Fresh remote machines do not need pre-seeded sibling clones; the first synced run can seed the remote stage directly from the current worktree snapshot.
- Remote snapshot sync transfers tracked files and untracked non-ignored files only. Keep `.gitignore` current so generated `target`, `out`, `dist`, `build`, cache, and dependency trees never transfer into remote stages.
- The committed remote wrapper now also seeds `rust/out/austin-manifest.scene-index.json` into the staged clone when it is available locally or in the configured remote base clone; this keeps staged proof runs able to execute scene-audit tooling without bulk-syncing ignored manifest stores.
- The committed remote wrapper now syncs any harness-produced `/tmp/arnis-scene-fidelity-*.{json,html}` and `/tmp/arnis-scene-parity.{json,html}` artifacts back into the local artifact directory alongside the latest Studio log and screenshot attempts.
- The committed remote wrapper now clears that volatile proof-artifact set both on the remote host and inside the chosen local artifact directory before each run so proof-first sync cannot reuse stale `/tmp` outputs from an older session.
- The committed harness now treats play scene-audit artifacts as trusted only after an explicit validated live-play marker; edit-context MCP false positives should fail honestly instead of producing misleading play scene reports.
- The committed play proof lane now emits `ARNIS_SCENE_PLAY` from the live Austin runtime itself and builds play scene-fidelity artifacts from that runtime marker plus the authoritative client bootstrap trace, so remote proof no longer depends on `run_code` successfully binding to the live play session.
- The committed screenshot lane now goes through `scripts/studio_ui_control.py capture-screenshot`, which writes a `.capture.json` sidecar beside each attempted image. That sidecar records session/window state, the chosen capture method, attempt stderr, and any window-lookup error so remote display failures are explicit even when no PNG is produced.
- On `tertiary`, the committed remote screenshot lane now has a second-stage GUI-session relay through the logged-in `Terminal` desktop session. When that relay is active, the sidecar should report `capture_method="gui_terminal_display"` and `guiSessionRelay.method="terminal.command"` instead of the older `host_display_capture_blocked` failure.
- Keep GUI automation and screenshot capture on `tertiary`. `primary` should stay a control/sync machine only.
- Because remote stage sync intentionally excludes ignored generated outputs, a staged remote clone may not have compiled manifest summaries such as `rust/out/*.scene-index.json`; if a remote proof slice needs offline scene-audit regeneration after the Studio run, regenerate or seed that summary explicitly instead of assuming it is present.
- Do not disable SSH host-key verification in committed scripts. Accept or rotate host keys out-of-band on each operator machine before using a new remote profile.
- If a host has no local profile config, the wrapper should fail early with a clear configuration error instead of guessing.
- If you use direct SSH proof instead of the wrapper, verify that the target path is an actual git clone on the intended branch before treating it as the proof surface; do not rely on loose copied working folders as if they were the canonical remote repo.
- If you use direct SSH proof instead of the wrapper, provide an explicit `VSYNC_REPO_DIR` (or `VSYNC_BIN`) for that remote machine; do not assume an adjacent `~/Projects/vertigo-sync` checkout exists.
- If a remote Studio lane is selected as the current proof surface, verify Studio launch and MCP/helper readiness there first; do not assume a configured `tertiary` profile is automatically green beyond the specific proof slices already verified.
- Current `tertiary` proof state is intentionally tracked only in the rolling status file above.
