use assert_cmd::Command;

fn bin() -> Command {
    Command::cargo_bin("claude-hooks").unwrap() // binary name verified via anchor #8
}

#[test]
fn architect_guard_empty_stdin_allows() {
    bin().arg("architect-guard").write_stdin("").assert().code(0);
}

#[test]
fn block_env_edit_empty_stdin_allows() {
    bin().arg("block-env-edit").write_stdin("").assert().code(0);
}

#[test]
fn block_unsafe_merge_resolves() {
    bin().arg("block-unsafe-merge").assert().code(0);
}

#[test]
fn session_banner_resolves() {
    bin().arg("session-banner").assert().code(0);
}

#[test]
fn serve_resolves() {
    bin().arg("serve").assert().code(0);
}

#[test]
fn harness_parses_valid_json_no_panic() {
    bin()
        .arg("architect-guard")
        .write_stdin(r#"{"tool_input":{"file_path":"x"}}"#)
        .assert()
        .code(0);
}

#[test]
fn harness_invalid_json_fails_open() {
    bin()
        .arg("architect-guard")
        .write_stdin("{not valid json")
        .assert()
        .code(0); // fail-open -- NO panic, NO exit 2
}

#[test]
fn unknown_subcommand_errors() {
    bin().arg("nonexistent-cmd").assert().failure(); // clap rejects, exit != 0
}

// ── P002 fire-test fixtures (P057 verify-cò) ─────────────────────────────────
//
// Isolation: each test sets CLAUDE_PROJECT_DIR to a unique temp dir.
// Marker .sos-state/architect-active is created/omitted inside that temp dir.
// This prevents any test from touching the real .sos-state/ (which holds
// worker-active for the active session) and avoids parallel-test races.

fn make_temp_dir(name: &str) -> std::path::PathBuf {
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("claude-hooks-test-{name}-{pid}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn place_marker(temp: &std::path::Path) {
    let state = temp.join(".sos-state");
    std::fs::create_dir_all(&state).expect("create .sos-state");
    std::fs::write(state.join("architect-active"), "").expect("write marker");
}

fn cleanup(dir: &std::path::Path) {
    let _ = std::fs::remove_dir_all(dir); // best-effort
}

/// Case 1: marker present + src/main.rs -> exit 2 (BLOCK)
/// P010: added "tool_name":"Read" — real Claude Code payload always has tool_name (Tension 1).
#[test]
fn p002_marker_src_file_blocked() {
    let temp = make_temp_dir("case1");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_name":"Read","tool_input":{"file_path":"src/main.rs"}}"#)
        .assert()
        .code(2);
    cleanup(&temp);
}

/// Case 2: marker present + README.md -> exit 0 (.md always allowed)
/// P010: added "tool_name":"Read" for accurate payload (Tension 1). Still exit 0 (.md early-allow).
#[test]
fn p002_marker_md_file_allowed() {
    let temp = make_temp_dir("case2");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_name":"Read","tool_input":{"file_path":"README.md"}}"#)
        .assert()
        .code(0);
    cleanup(&temp);
}

/// Case 3: marker present + pattern src/**/*.rs -> exit 2 (Glob pattern forbidden)
/// P010: added "tool_name":"Glob" — real Claude Code Glob payload (Tension 1).
#[test]
fn p002_marker_pattern_src_blocked() {
    let temp = make_temp_dir("case3");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_name":"Glob","tool_input":{"pattern":"src/**/*.rs"}}"#)
        .assert()
        .code(2);
    cleanup(&temp);
}

/// Case 4: marker present + docs/x.txt -> exit 0 (default allow, not in forbidden set)
/// P010: added "tool_name":"Read" for accurate payload (Tension 1). Still exit 0.
#[test]
fn p002_marker_docs_txt_allowed() {
    let temp = make_temp_dir("case4");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_name":"Read","tool_input":{"file_path":"docs/x.txt"}}"#)
        .assert()
        .code(0);
    cleanup(&temp);
}

/// Case 5: NO marker + src/main.rs -> exit 0 (marker gate allows all when no marker)
/// P010: added "tool_name":"Read" for accurate payload (Tension 1). Still exit 0 (no marker).
#[test]
fn p002_no_marker_src_allowed() {
    let temp = make_temp_dir("case5");
    // deliberately do NOT place marker
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_name":"Read","tool_input":{"file_path":"src/main.rs"}}"#)
        .assert()
        .code(0);
    cleanup(&temp);
}

/// Case 6: marker present + empty stdin -> exit 0 (fail-open: no path parsed)
#[test]
fn p002_marker_empty_stdin_allowed() {
    let temp = make_temp_dir("case6");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin("")
        .assert()
        .code(0);
    cleanup(&temp);
}

// ── P003 fire-test fixtures (P057 verify-cò) ─────────────────────────────────
//
// block-env-edit does NOT depend on a global marker — no CLAUDE_PROJECT_DIR setup needed.
// Each fixture feeds stdin JSON and asserts exit code only.

/// Case 1: .env (basename) -> regex match ^\.env$ -> exit 2 (BLOCK)
#[test]
fn p003_dot_env_blocked() {
    bin()
        .arg("block-env-edit")
        .write_stdin(r#"{"tool_input":{"file_path":"/x/.env"}}"#)
        .assert()
        .code(2);
}

/// Case 2: .env.example -> allowlist (before regex) -> exit 0
#[test]
fn p003_dot_env_example_allowed() {
    bin()
        .arg("block-env-edit")
        .write_stdin(r#"{"tool_input":{"file_path":".env.example"}}"#)
        .assert()
        .code(0);
}

/// Case 3: .envrc -> regex does NOT match ('rc' is neither '$' nor '.') -> exit 0
/// This is the easiest case to get wrong: must port ($|\.) correctly.
#[test]
fn p003_dot_envrc_allowed() {
    bin()
        .arg("block-env-edit")
        .write_stdin(r#"{"tool_input":{"file_path":".envrc"}}"#)
        .assert()
        .code(0);
}

/// Case 4: .env.local -> dot after 'env' matches \. -> exit 2
#[test]
fn p003_dot_env_local_blocked() {
    bin()
        .arg("block-env-edit")
        .write_stdin(r#"{"tool_input":{"file_path":".env.local"}}"#)
        .assert()
        .code(2);
}

/// Case 5: .env.production -> dot after 'env' matches \. -> exit 2
#[test]
fn p003_dot_env_production_blocked() {
    bin()
        .arg("block-env-edit")
        .write_stdin(r#"{"tool_input":{"file_path":".env.production"}}"#)
        .assert()
        .code(2);
}

/// Case 6: /some/dir/.env -> basename .env -> regex match -> exit 2 (verify Step 5 basename)
#[test]
fn p003_absolute_path_dot_env_blocked() {
    bin()
        .arg("block-env-edit")
        .write_stdin(r#"{"tool_input":{"file_path":"/some/dir/.env"}}"#)
        .assert()
        .code(2);
}

/// Case 7: config.yaml -> non-env -> no match -> exit 0
#[test]
fn p003_config_yaml_allowed() {
    bin()
        .arg("block-env-edit")
        .write_stdin(r#"{"tool_input":{"file_path":"config.yaml"}}"#)
        .assert()
        .code(0);
}

/// Case 8: notebook_path fallback (NotebookEdit) -> x/.env -> basename .env -> exit 2
#[test]
fn p003_notebook_path_fallback_blocked() {
    bin()
        .arg("block-env-edit")
        .write_stdin(r#"{"tool_input":{"notebook_path":"x/.env"}}"#)
        .assert()
        .code(2);
}

/// Case 9: empty stdin -> fail-open -> exit 0
#[test]
fn p003_empty_stdin_allowed() {
    bin()
        .arg("block-env-edit")
        .write_stdin("")
        .assert()
        .code(0);
}

/// Case 10 (optional bonus): .environment -> 'ironment' after 'env' is letters, not '$'/'.'
/// -> regex does NOT match -> exit 0 (same family as .envrc)
#[test]
fn p003_dot_environment_allowed() {
    bin()
        .arg("block-env-edit")
        .write_stdin(r#"{"tool_input":{"file_path":".environment"}}"#)
        .assert()
        .code(0);
}

// ── P004 fire-test fixtures (P057 verify-cò) ─────────────────────────────────
//
// Integration tests cover only gh-FREE paths (oracle L31, L42, L47, L54).
// gh-dependent paths (fail-CLOSED BLOCK, verdict paths) require network/auth
// and must be verified manually against `bash scripts/block-unsafe-merge.sh`.

/// Case 1: command `echo hi` -> not a merge command -> exit 0
#[test]
fn p004_non_merge_command_allows() {
    bin()
        .arg("block-unsafe-merge")
        .write_stdin(r#"{"tool_input":{"command":"echo hi"}}"#)
        .assert()
        .code(0);
}

/// Case 2: override marker present -> warning stderr + exit 0
#[test]
fn p004_override_marker_allows() {
    bin()
        .arg("block-unsafe-merge")
        .write_stdin(r#"{"tool_input":{"command":"gh pr merge 9 [security-review-skip:test]"}}"#)
        .assert()
        .code(0);
}

/// Case 3: empty stdin -> no command -> exit 0 (fail-open, oracle L31)
#[test]
fn p004_empty_stdin_allows() {
    bin()
        .arg("block-unsafe-merge")
        .write_stdin("")
        .assert()
        .code(0);
}

/// Case 4: `gh pr merge --merge` (no number) -> parse_merge_pr returns None -> exit 0
/// (known limitation / bypass, oracle L15-16, intentional)
#[test]
fn p004_branch_only_form_allows() {
    bin()
        .arg("block-unsafe-merge")
        .write_stdin(r#"{"tool_input":{"command":"gh pr merge --merge"}}"#)
        .assert()
        .code(0);
}

// ── P005 fire-test fixtures (P057 verify-cò) ─────────────────────────────────
//
// Integration tests for session-banner. Isolation via CLAUDE_PROJECT_DIR pointing
// to a manually created temp dir (no `tempfile` dep — same pattern as P002).
// Git/advisory paths are NOT tested here (git-state-dependent → manual verification).

fn make_banner_temp(name: &str) -> std::path::PathBuf {
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("claude-hooks-banner-{name}-{pid}"));
    std::fs::create_dir_all(&dir).expect("create banner temp dir");
    dir
}

/// Case 1: BACKLOG with Active sprint + items → stdout contains banner markers + exit 0
#[test]
fn p005_banner_with_backlog_shows_markers() {
    let temp = make_banner_temp("case1");
    let docs = temp.join("docs");
    std::fs::create_dir_all(&docs).expect("create docs/");
    std::fs::write(
        docs.join("BACKLOG.md"),
        "## 🔥 Active sprint: Test\n- [ ] item one\n- [ ] item two\n- [x] done item\n",
    )
    .expect("write BACKLOG");

    let out = bin()
        .arg("session-banner")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .output()
        .expect("run binary");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("🏠 Sếp's project"), "missing home emoji header");
    assert!(
        stdout.contains("📊 Active sprint:"),
        "missing sprint count line"
    );
    assert!(
        stdout.contains("🤖 Orchestrator contract"),
        "missing orchestrator contract block"
    );
    assert!(
        stdout.contains("📌 Architect Rule 0"),
        "missing architect rule 0"
    );

    let _ = std::fs::remove_dir_all(&temp);
}

/// Case 2: no BACKLOG.md → stdout empty (silent) + exit 0
#[test]
fn p005_no_backlog_silent() {
    let temp = make_banner_temp("case2");
    // Create docs/ dir but no BACKLOG.md
    std::fs::create_dir_all(temp.join("docs")).expect("create docs/");

    let out = bin()
        .arg("session-banner")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .output()
        .expect("run binary");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.is_empty(), "expected empty stdout, got: {stdout:?}");

    let _ = std::fs::remove_dir_all(&temp);
}

/// Case 3: BACKLOG with no ^## headings → stdout empty + exit 0
#[test]
fn p005_backlog_no_h2_silent() {
    let temp = make_banner_temp("case3");
    let docs = temp.join("docs");
    std::fs::create_dir_all(&docs).expect("create docs/");
    std::fs::write(docs.join("BACKLOG.md"), "# Only H1\nSome prose\n### H3 only\n")
        .expect("write BACKLOG");

    let out = bin()
        .arg("session-banner")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .output()
        .expect("run binary");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.is_empty(), "expected empty stdout for no-h2, got: {stdout:?}");

    let _ = std::fs::remove_dir_all(&temp);
}

/// Case 4: fallback header (no "Active sprint") → stdout contains fallback note
#[test]
fn p005_fallback_header_shows_note() {
    let temp = make_banner_temp("case4");
    let docs = temp.join("docs");
    std::fs::create_dir_all(&docs).expect("create docs/");
    std::fs::write(docs.join("BACKLOG.md"), "## Foo\n- [ ] work\n")
        .expect("write BACKLOG");

    let out = bin()
        .arg("session-banner")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .output()
        .expect("run binary");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("📌 Treating \"Foo\" as Active sprint"),
        "missing fallback note, got: {stdout}"
    );

    let _ = std::fs::remove_dir_all(&temp);
}

// ── P010 fire-test fixtures (P057 verify-cò, TRUE parity tarot) ──────────────
//
// All tests with marker use isolated CLAUDE_PROJECT_DIR (same P002 pattern).
// Verify-cò matrix: Read/Glob forbidden + Write/Edit allowlist + dispatch-default.

// ── P010 Write/Edit guard ─────────────────────────────────────────────────────

/// Write src/foo.ts → marker present → exit 2 (block_write: not in allowlist)
#[test]
fn p010_write_src_file_blocked() {
    let temp = make_temp_dir("p010_w1");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_name":"Write","tool_input":{"file_path":"src/foo.ts"}}"#)
        .assert()
        .code(2);
    cleanup(&temp);
}

/// Edit CLAUDE.md → marker present → exit 2 (block_write: not in allowlist)
#[test]
fn p010_edit_claude_md_blocked() {
    let temp = make_temp_dir("p010_w2");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_name":"Edit","tool_input":{"file_path":"CLAUDE.md"}}"#)
        .assert()
        .code(2);
    cleanup(&temp);
}

/// Write docs/ticket/P010-x.md → marker present → exit 0 (phiếu allowlist)
#[test]
fn p010_write_phieu_file_allowed() {
    let temp = make_temp_dir("p010_w3");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_name":"Write","tool_input":{"file_path":"docs/ticket/P010-x.md"}}"#)
        .assert()
        .code(0);
    cleanup(&temp);
}

/// Write docs/ticket/TICKET_TEMPLATE.md → marker present → exit 2 (explicit deny)
#[test]
fn p010_write_ticket_template_blocked() {
    let temp = make_temp_dir("p010_w4");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_name":"Write","tool_input":{"file_path":"docs/ticket/TICKET_TEMPLATE.md"}}"#)
        .assert()
        .code(2);
    cleanup(&temp);
}

/// Edit with no file_path (empty tool_input) → marker present → exit 0 (defensive allow, oracle L111)
#[test]
fn p010_edit_no_path_allowed() {
    let temp = make_temp_dir("p010_w5");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_name":"Edit","tool_input":{}}"#)
        .assert()
        .code(0);
    cleanup(&temp);
}

// ── P010 Read/Glob superset (prisma, sql, path root) ─────────────────────────

/// Glob pattern src/** → marker present → exit 2
#[test]
fn p010_glob_pattern_src_blocked() {
    let temp = make_temp_dir("p010_r1");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_name":"Glob","tool_input":{"pattern":"src/**"}}"#)
        .assert()
        .code(2);
    cleanup(&temp);
}

/// Glob path prisma/ (Glob search root) → marker present → exit 2 (prisma/ new P010)
#[test]
fn p010_glob_path_prisma_blocked() {
    let temp = make_temp_dir("p010_r2");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_name":"Glob","tool_input":{"path":"prisma/"}}"#)
        .assert()
        .code(2);
    cleanup(&temp);
}

/// Read prisma/schema.prisma → marker present → exit 2 (.prisma ext new P010)
#[test]
fn p010_read_prisma_schema_blocked() {
    let temp = make_temp_dir("p010_r3");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_name":"Read","tool_input":{"file_path":"prisma/schema.prisma"}}"#)
        .assert()
        .code(2);
    cleanup(&temp);
}

/// Read db/x.sql → marker present → exit 2 (.sql ext new P010)
#[test]
fn p010_read_sql_file_blocked() {
    let temp = make_temp_dir("p010_r4");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_name":"Read","tool_input":{"file_path":"db/x.sql"}}"#)
        .assert()
        .code(2);
    cleanup(&temp);
}

/// Read README.md → marker present → exit 0 (.md early-allow)
#[test]
fn p010_read_readme_md_allowed() {
    let temp = make_temp_dir("p010_r5");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_name":"Read","tool_input":{"file_path":"README.md"}}"#)
        .assert()
        .code(0);
    cleanup(&temp);
}

// ── P010 dispatch default (no tool_name → ALLOW, faithful tarot oracle L118) ──

/// NO tool_name in payload + src/foo.ts + marker → exit 0
/// This is INTENDED behavior: real Claude Code payload always has tool_name.
/// Payloads without tool_name fall to case default → ALLOW (oracle case default).
/// Test name documents this is NOT a bug — it is faithful tarot port.
#[test]
fn p010_no_tool_name_allows_even_src_path() {
    let temp = make_temp_dir("p010_d1");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_input":{"file_path":"src/foo.ts"}}"#)
        .assert()
        .code(0);
    cleanup(&temp);
}
