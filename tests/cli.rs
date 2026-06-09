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
