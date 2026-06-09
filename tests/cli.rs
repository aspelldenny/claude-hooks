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
#[test]
fn p002_marker_src_file_blocked() {
    let temp = make_temp_dir("case1");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_input":{"file_path":"src/main.rs"}}"#)
        .assert()
        .code(2);
    cleanup(&temp);
}

/// Case 2: marker present + README.md -> exit 0 (.md always allowed)
#[test]
fn p002_marker_md_file_allowed() {
    let temp = make_temp_dir("case2");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_input":{"file_path":"README.md"}}"#)
        .assert()
        .code(0);
    cleanup(&temp);
}

/// Case 3: marker present + pattern src/**/*.rs -> exit 2 (path via pattern fallback)
#[test]
fn p002_marker_pattern_src_blocked() {
    let temp = make_temp_dir("case3");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_input":{"pattern":"src/**/*.rs"}}"#)
        .assert()
        .code(2);
    cleanup(&temp);
}

/// Case 4: marker present + docs/x.txt -> exit 0 (default allow, not in forbidden set)
#[test]
fn p002_marker_docs_txt_allowed() {
    let temp = make_temp_dir("case4");
    place_marker(&temp);
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_input":{"file_path":"docs/x.txt"}}"#)
        .assert()
        .code(0);
    cleanup(&temp);
}

/// Case 5: NO marker + src/main.rs -> exit 0 (marker gate allows all when no marker)
#[test]
fn p002_no_marker_src_allowed() {
    let temp = make_temp_dir("case5");
    // deliberately do NOT place marker
    bin()
        .arg("architect-guard")
        .env("CLAUDE_PROJECT_DIR", &temp)
        .write_stdin(r#"{"tool_input":{"file_path":"src/main.rs"}}"#)
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
