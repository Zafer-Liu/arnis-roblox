#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[2]
PROJECTS_ROOT = ROOT.parent.parent.parent if ROOT.parent.name == ".worktrees" else ROOT.parent
REMOTE_HARNESS_PATH = ROOT / "scripts" / "run_studio_harness_remote.sh"


def resolve_vertigo_sync_root() -> Path:
    result = subprocess.run(
        ["git", "-C", str(ROOT), "rev-parse", "--git-common-dir"],
        check=True,
        capture_output=True,
        text=True,
    )
    common_git_dir = Path(result.stdout.strip()).resolve()
    if common_git_dir.name == ".git":
        repo_root = common_git_dir.parent
    else:
        repo_root = common_git_dir.parents[2]
    return repo_root.parent / "vertigo-sync"


class RunStudioHarnessRemoteTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.text = REMOTE_HARNESS_PATH.read_text(encoding="utf-8")

    def test_uses_profile_based_remote_configuration_without_baked_host_defaults(self) -> None:
        self.assertIn('REMOTE_PROFILE="${ARNIS_REMOTE_STUDIO_PROFILE:-tertiary}"', self.text)
        self.assertIn('LOCAL_REMOTE_CONFIG="$ROOT_DIR/scripts/remote_studio_profiles.local.sh"', self.text)
        self.assertIn('EXAMPLE_REMOTE_CONFIG="$ROOT_DIR/scripts/remote_studio_profiles.example.sh"', self.text)
        self.assertIn('resolve_profile_value', self.text)
        self.assertNotIn('primary.local', self.text)
        self.assertNotIn('/Users/adpena/Projects/.codex-remote-studio', self.text)

    def test_syncs_local_arnis_and_vsync_snapshots_to_remote_stage(self) -> None:
        self.assertIn('LOCAL_ARNIS_DIR="$ROOT_DIR"', self.text)
        self.assertIn('git -C "$ROOT_DIR" rev-parse --git-common-dir', self.text)
        self.assertIn('LOCAL_VSYNC_DIR="${VSYNC_REPO_DIR:-$DEFAULT_VSYNC_DIR}"', self.text)
        self.assertIn("render_rsync_remote_path()", self.text)
        self.assertIn('RSYNC_REMOTE_ARNIS_DIR="$(render_rsync_remote_path "$REMOTE_ARNIS_DIR")"', self.text)
        self.assertIn('RSYNC_REMOTE_VSYNC_DIR="$(render_rsync_remote_path "$REMOTE_VSYNC_DIR")"', self.text)
        self.assertIn('sync_repo_snapshot "$LOCAL_ARNIS_DIR" "$REMOTE_ARNIS_DIR" "$RSYNC_REMOTE_ARNIS_DIR"', self.text)
        self.assertIn('sync_repo_snapshot "$LOCAL_VSYNC_DIR" "$REMOTE_VSYNC_DIR" "$RSYNC_REMOTE_VSYNC_DIR"', self.text)
        self.assertIn('git -C "$repo_dir" ls-files -z --cached --others --exclude-standard', self.text)
        self.assertIn('rsync -a --from0 --files-from="$manifest"', self.text)
        self.assertIn('reset_remote_stage_dir "$remote_dir"', self.text)
        self.assertIn('if git -C "$source_dir" rev-parse --is-inside-work-tree >/dev/null 2>&1; then', self.text)

    def test_remote_seed_prefers_existing_remote_repos_when_present(self) -> None:
        self.assertIn('REMOTE_ARNIS_BASE="$(resolve_profile_value ARNIS_REMOTE_STUDIO_BASE_ARNIS', self.text)
        self.assertIn('REMOTE_VSYNC_BASE="$(resolve_profile_value ARNIS_REMOTE_STUDIO_BASE_VSYNC', self.text)
        self.assertIn('seed_stage "$remote_arnis_base" "$remote_arnis_dir"', self.text)
        self.assertIn('seed_stage "$remote_vsync_base" "$remote_vsync_dir"', self.text)
        self.assertIn('git -C "$source_dir" rev-parse --is-inside-work-tree >/dev/null 2>&1', self.text)

    def test_remote_seed_supports_cold_remote_bootstrap_without_seed_repos(self) -> None:
        self.assertIn('if [[ ! -d "$source_dir" ]]; then', self.text)
        self.assertIn('return 0', self.text)

    def test_no_sync_requires_existing_remote_stage_with_clear_validation(self) -> None:
        self.assertIn('ensure_remote_stage_ready()', self.text)
        self.assertIn('missing remote arnis stage', self.text)
        self.assertIn('missing remote vertigo-sync stage', self.text)
        self.assertIn('re-run without --no-sync', self.text)
        self.assertIn('"$remote_root/arnis-roblox"|"$remote_root/vertigo-sync"', self.text)
        self.assertIn('refusing to reset unexpected remote stage path', self.text)

    def test_runs_same_remote_harness_with_remote_vsync_binary(self) -> None:
        self.assertIn('needs_vsync_build()', self.text)
        self.assertIn('cleanup_remote_harness()', self.text)
        self.assertIn("REMOTE_HARNESS_ACTIVE=0", self.text)
        self.assertIn('trap \'cleanup_remote_harness\' EXIT INT TERM', self.text)
        self.assertIn('REMOTE_HARNESS_PGID_FILE="$REMOTE_ARNIS_DIR/.arnis-remote-harness.pgid"', self.text)
        self.assertIn('REMOTE_HARNESS_LOCK_DIR="$REMOTE_ARNIS_DIR/.arnis-studio-harness.lock"', self.text)
        self.assertIn('kill -TERM -- "-$remote_harness_pgid"', self.text)
        self.assertIn('rm -rf "$remote_harness_lock_dir"', self.text)
        self.assertNotIn('pkill -f "$remote_arnis_dir/scripts/run_studio_harness.sh" || true', self.text)
        self.assertNotIn('pkill -f "$remote_vsync_target_dir/debug/vsync serve" || true', self.text)
        self.assertIn('if needs_vsync_build "$remote_vsync_dir" "$remote_vsync_target_dir"; then', self.text)
        self.assertIn('CARGO_TARGET_DIR="$remote_vsync_target_dir"', self.text)
        self.assertIn('cargo build --manifest-path "$remote_vsync_dir/Cargo.toml" --bin vsync >/dev/null', self.text)
        self.assertIn('"$repo_dir/build.rs"', self.text)
        self.assertIn('cd "$remote_arnis_dir"', self.text)
        self.assertIn('HARNESS_LOCK_DIR="$remote_harness_lock_dir"', self.text)
        self.assertIn('VSYNC_REPO_DIR="$remote_vsync_dir"', self.text)
        self.assertIn('VSYNC_BIN="$remote_vsync_target_dir/debug/vsync"', self.text)
        self.assertIn('bash "$remote_arnis_dir/scripts/run_studio_harness.sh" "$@"', self.text)
        self.assertIn('remote_harness_pgid="$(ps -o pgid= "$remote_harness_pid"', self.text)
        self.assertIn("REMOTE_HARNESS_ACTIVE=1", self.text)
        self.assertIn("REMOTE_HARNESS_ACTIVE=0", self.text)

    def test_fetches_remote_logs_and_screenshots_back_locally(self) -> None:
        self.assertIn('LOCAL_ARTIFACT_DIR="${ARNIS_REMOTE_STUDIO_ARTIFACT_DIR:-/tmp/arnis-remote-studio}"', self.text)
        self.assertIn("sync_remote_artifacts()", self.text)
        self.assertIn('remote_latest_log="$(ssh "$REMOTE_HOST" ', self.text)
        self.assertIn('rsync -a "$REMOTE_HOST:$remote_latest_log" "$LOCAL_ARTIFACT_DIR/"', self.text)
        self.assertIn('/tmp/arnis-studio-harness-edit.png', self.text)
        self.assertIn('/tmp/arnis-studio-harness-play.png', self.text)
        self.assertIn('/tmp/arnis-preview-plugin-state.json', self.text)

    def test_clears_volatile_remote_tmp_artifacts_before_each_run(self) -> None:
        self.assertIn("reset_remote_proof_artifacts()", self.text)
        self.assertIn('/tmp/arnis-scene-fidelity-edit.json', self.text)
        self.assertIn('/tmp/arnis-scene-fidelity-play.json', self.text)
        self.assertIn('/tmp/arnis-scene-parity.json', self.text)
        self.assertIn('/tmp/arnis-studio-harness-play.png', self.text)
        self.assertIn('rm -f "$artifact_path"', self.text)
        self.assertIn('reset_remote_proof_artifacts', self.text)

    def test_proof_first_wrapper_mirrors_remote_output_and_starts_early_sync_after_authoritative_signal(self) -> None:
        self.assertIn('REMOTE_SESSION_OUTPUT_LOG="$(mktemp -t arnis-remote-harness-output)"', self.text)
        self.assertIn('ssh "$REMOTE_HOST" \'bash -s\' -- "$SYNC_STAGE" "$REMOTE_ARNIS_DIR" "$REMOTE_VSYNC_DIR" "$REMOTE_VSYNC_TARGET_DIR" "${HARNESS_ARGS[@]}"', self.text)
        self.assertIn('> >(tee "$REMOTE_SESSION_OUTPUT_LOG") 2>&1', self.text)
        self.assertIn("remote_proof_signal_detected()", self.text)
        self.assertIn('play bootstrap trace verdict \\(authoritative client bootstrap marker\\): valid', self.text)
        self.assertIn('if [[ $proof_signal_seen -eq 0 ]] && remote_proof_signal_detected; then', self.text)
        self.assertIn('sync_remote_artifacts || true', self.text)

    def test_proof_first_wrapper_bounds_cleanup_tail_after_main_flow_completion(self) -> None:
        self.assertIn('PROOF_SYNC_TAIL_TIMEOUT_SECONDS="${ARNIS_REMOTE_STUDIO_TAIL_TIMEOUT_SECONDS:-20}"', self.text)
        self.assertIn("remote_completion_signal_detected()", self.text)
        self.assertIn('main harness flow complete; exiting', self.text)
        self.assertIn('completion_signal_seen_at=0', self.text)
        self.assertIn('if [[ $completion_signal_seen_at -eq 0 ]] && remote_completion_signal_detected; then', self.text)
        self.assertIn('if (( now_epoch - completion_signal_seen_at >= PROOF_SYNC_TAIL_TIMEOUT_SECONDS )); then', self.text)
        self.assertIn('echo "[remote-harness] bounded remote cleanup tail exceeded ${PROOF_SYNC_TAIL_TIMEOUT_SECONDS}s after proof completion; stopping wrapper wait" >&2', self.text)
        self.assertIn('kill -TERM "$remote_ssh_pid" >/dev/null 2>&1 || true', self.text)

    def test_seeds_manifest_summary_and_fetches_scene_audit_artifacts(self) -> None:
        self.assertIn('LOCAL_MANIFEST_SUMMARY_PATH="$LOCAL_ARNIS_DIR/rust/out/austin-manifest.scene-index.json"', self.text)
        self.assertIn('REMOTE_MANIFEST_SUMMARY_PATH="$REMOTE_ARNIS_DIR/rust/out/austin-manifest.scene-index.json"', self.text)
        self.assertIn('sync_optional_file "$LOCAL_MANIFEST_SUMMARY_PATH" "$REMOTE_MANIFEST_SUMMARY_PATH"', self.text)
        self.assertIn('seed_remote_optional_file_from_base "$REMOTE_ARNIS_BASE/rust/out/austin-manifest.scene-index.json" "$REMOTE_MANIFEST_SUMMARY_PATH"', self.text)
        self.assertIn('/tmp/arnis-studio-harness-edit.capture.json', self.text)
        self.assertIn('/tmp/arnis-studio-harness-play.capture.json', self.text)
        self.assertIn('/tmp/arnis-scene-fidelity-edit.json', self.text)
        self.assertIn('/tmp/arnis-scene-fidelity-edit.html', self.text)
        self.assertIn('/tmp/arnis-scene-fidelity-play.json', self.text)
        self.assertIn('/tmp/arnis-scene-fidelity-play.html', self.text)
        self.assertIn('/tmp/arnis-scene-parity.json', self.text)
        self.assertIn('/tmp/arnis-scene-parity.html', self.text)

    def test_supports_remote_profile_host_and_root_flags(self) -> None:
        self.assertIn('--remote-profile PROFILE', self.text)
        self.assertIn('--remote-host HOST', self.text)
        self.assertIn('--remote-root PATH', self.text)
        self.assertIn('--no-sync', self.text)
        self.assertIn('--route-catalog PlanetaryRouteBundle.route-catalog', self.text)
        self.assertIn('--route-lane active', self.text)
        self.assertIn('--route-step-index 0', self.text)

    def test_example_profile_template_exists(self) -> None:
        template_path = ROOT / "scripts" / "remote_studio_profiles.example.sh"
        self.assertTrue(template_path.exists(), "expected remote studio profile example template")
        template_text = template_path.read_text(encoding="utf-8")
        self.assertIn("ARNIS_REMOTE_STUDIO_HOST_PRIMARY", template_text)
        self.assertIn("ARNIS_REMOTE_STUDIO_HOST_TERTIARY", template_text)
        self.assertNotIn("primary.local", template_text)
        self.assertNotIn("tertiary.local", template_text)

    def test_remote_studio_docs_keep_profiles_generic_and_cover_cold_start_bootstrap(self) -> None:
        docs_text = (ROOT / "docs" / "remote-studio-development.md").read_text(encoding="utf-8")
        self.assertIn("Direct Development On The Active Dev Machine", docs_text)
        self.assertIn("Fresh remote machines do not need pre-seeded sibling clones", docs_text)
        self.assertIn("tracked files and untracked non-ignored files", docs_text)
        self.assertIn("profile aliases", docs_text)
        self.assertNotIn("primary.local", docs_text)
        self.assertNotIn("tertiary.local", docs_text)

    def test_gitignore_blocks_generated_artifacts_from_git_aware_sync(self) -> None:
        arnis_gitignore = (ROOT / ".gitignore").read_text(encoding="utf-8")
        vertigo_sync_gitignore = (resolve_vertigo_sync_root() / ".gitignore").read_text(encoding="utf-8")
        for text in (arnis_gitignore, vertigo_sync_gitignore):
            with self.subTest(gitignore=text[:32]):
                self.assertTrue("**/target/" in text or "**/target" in text)
                self.assertIn("**/out/", text)
                self.assertIn("**/build/", text)
                self.assertIn("**/dist/", text)
                self.assertIn("**/.venv/", text)
                self.assertIn("**/node_modules/", text)


if __name__ == "__main__":
    unittest.main()
