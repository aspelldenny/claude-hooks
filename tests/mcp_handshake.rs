// P006/P007: MCP handshake smoke test + Decision-core unit tests.
//
// Smoke: spawn `claude-hooks serve`, send initialize + tools/list over stdin (newline-delimited
// JSON-RPC, rmcp stdio framing), assert stdout contains 5 tool names + valid JSON-RPC.
//
// Unit: call *_decide fns via CLI binary shim (integration tests cannot import from binary crate).
// Exit code 0=ALLOW / 2=BLOCK — consistent with 81 existing tests.

// ── Decision-core unit tests (via CLI binary shim) ────────────────────────────

/// architect_guard_decide: no marker → ALLOW regardless of src/ path
#[test]
fn decide_architect_guard_no_marker_allows_src() {
    let temp = make_temp_dir("ag_no_marker");
    let d = run_architect_guard(Some("src/main.rs"), None, Some(&temp));
    let _ = std::fs::remove_dir_all(&temp);
    assert!(!d.blocked, "expected ALLOW, got blocked with: {:?}", d.reason);
    assert_eq!(d.exit_code, 0);
}

/// architect_guard_decide: marker present + src/ path → BLOCK
#[test]
fn decide_architect_guard_marker_src_blocks() {
    let temp = make_temp_dir("ag_marker_src");
    place_marker(&temp);
    let d = run_architect_guard(Some("src/main.rs"), None, Some(&temp));
    let _ = std::fs::remove_dir_all(&temp);
    assert!(d.blocked, "expected BLOCK (exit 2)");
    assert_eq!(d.exit_code, 2);
    assert!(d.reason.is_some());
}

/// block_env_edit_decide: .env.local → BLOCK
#[test]
fn decide_block_env_edit_local_blocks() {
    let d = run_block_env_edit(Some(".env.local"), None);
    assert!(d.blocked, "expected BLOCK for .env.local");
    assert_eq!(d.exit_code, 2);
}

/// block_env_edit_decide: .env.example → ALLOW
#[test]
fn decide_block_env_edit_example_allows() {
    let d = run_block_env_edit(Some(".env.example"), None);
    assert!(!d.blocked, "expected ALLOW for .env.example");
    assert_eq!(d.exit_code, 0);
}

// ── MCP handshake smoke ───────────────────────────────────────────────────────

/// Smoke: spawn `claude-hooks serve`, send initialize + tools/list JSON-RPC (newline-delimited),
/// assert stdout contains all 5 expected tool names in valid JSON-RPC response.
///
/// Framing: rmcp stdio uses newline-delimited JSON (one JSON object per line).
/// Sequence: initialize request → notifications/initialized → tools/list → close stdin.
/// The server exits when stdin reaches EOF (transport close → waiting() returns).
#[test]
fn mcp_serve_tools_list_returns_5_tools() {
    use std::process::{Command, Stdio};
    use std::io::Write;
    use std::time::Duration;

    let binary = assert_cmd::cargo::cargo_bin("claude-hooks");

    let mut child = Command::new(&binary)
        .arg("serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn claude-hooks serve");

    {
        let stdin = child.stdin.as_mut().expect("stdin pipe");

        // MCP JSON-RPC initialize (newline-delimited framing)
        let init_req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"smoke-test","version":"1.0"}}}"#;
        let init_notif = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let tools_req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;

        writeln!(stdin, "{init_req}").expect("write initialize");
        writeln!(stdin, "{init_notif}").expect("write initialized notification");
        writeln!(stdin, "{tools_req}").expect("write tools/list");
    }
    // Drop stdin → EOF → server transport closes → waiting() returns
    drop(child.stdin.take());

    // Wait for process to exit with timeout
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();
    loop {
        match child.try_wait().expect("try_wait") {
            Some(_) => break,
            None => {
                if start.elapsed() > timeout {
                    child.kill().ok();
                    panic!("serve timed out after {timeout:?}");
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }

    let output = child.wait_with_output().expect("collect output");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Assert 5 tool names in stdout (from tools/list response, P007: +why_blocked)
    for tool in &["architect_guard", "block_env_edit", "block_unsafe_merge", "session_banner", "why_blocked"] {
        assert!(
            stdout.contains(tool),
            "tool '{tool}' not found in stdout.\nFull stdout:\n{stdout}"
        );
    }

    // Assert valid JSON-RPC envelope
    assert!(
        stdout.contains("\"jsonrpc\""),
        "no jsonrpc key in stdout.\nFull stdout:\n{stdout}"
    );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_temp_dir(name: &str) -> std::path::PathBuf {
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("claude-hooks-p006-{name}-{pid}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn place_marker(temp: &std::path::Path) {
    let state = temp.join(".sos-state");
    std::fs::create_dir_all(&state).expect("create .sos-state");
    std::fs::write(state.join("architect-active"), "").expect("write marker");
}

struct Decision {
    exit_code: i32,
    blocked: bool,
    reason: Option<String>,
}

fn run_architect_guard(
    file_path: Option<&str>,
    pattern: Option<&str>,
    project_dir: Option<&std::path::Path>,
) -> Decision {
    use std::process::{Command, Stdio};
    use std::io::Write;

    let binary = assert_cmd::cargo::cargo_bin("claude-hooks");
    let payload = match (file_path, pattern) {
        (Some(fp), _) => format!(r#"{{"tool_input":{{"file_path":"{fp}"}}}}"#),
        (None, Some(p)) => format!(r#"{{"tool_input":{{"pattern":"{p}"}}}}"#),
        (None, None) => "{}".to_owned(),
    };

    let mut cmd = Command::new(&binary);
    cmd.arg("architect-guard")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    if let Some(dir) = project_dir {
        cmd.env("CLAUDE_PROJECT_DIR", dir);
    }

    let mut child = cmd.spawn().expect("spawn architect-guard");
    child.stdin.as_mut().unwrap().write_all(payload.as_bytes()).ok();
    drop(child.stdin.take());

    let out = child.wait_with_output().expect("wait");
    let code = out.status.code().unwrap_or(1);
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_owned();
    Decision {
        exit_code: code,
        blocked: code == 2,
        reason: if stderr.is_empty() { None } else { Some(stderr) },
    }
}

fn run_block_env_edit(file_path: Option<&str>, notebook_path: Option<&str>) -> Decision {
    use std::process::{Command, Stdio};
    use std::io::Write;

    let binary = assert_cmd::cargo::cargo_bin("claude-hooks");
    let payload = match (file_path, notebook_path) {
        (Some(fp), _) => format!(r#"{{"tool_input":{{"file_path":"{fp}"}}}}"#),
        (None, Some(np)) => format!(r#"{{"tool_input":{{"notebook_path":"{np}"}}}}"#),
        (None, None) => "{}".to_owned(),
    };

    let mut child = Command::new(&binary)
        .arg("block-env-edit")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn block-env-edit");

    child.stdin.as_mut().unwrap().write_all(payload.as_bytes()).ok();
    drop(child.stdin.take());

    let out = child.wait_with_output().expect("wait");
    let code = out.status.code().unwrap_or(1);
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_owned();
    Decision {
        exit_code: code,
        blocked: code == 2,
        reason: if stderr.is_empty() { None } else { Some(stderr) },
    }
}

// ── P007: why_blocked routing unit tests (via MCP tools/call) ────────────────
//
// Strategy: spawn serve, call tools/call why_blocked with deterministic inputs.
// Cases chosen to avoid fs-marker or network dependency:
//   - Edit + .env.local  → hook="block_env_edit", blocked=true  (pure path-regex, no fs)
//   - Edit + .env.example → hook="block_env_edit", blocked=false (allowlist, no fs)
//   - Write/.MultiEdit/.NotebookEdit + .env.local → hook="block_env_edit", blocked=true
//   - WebFetch (unknown) → hook="none", blocked=false
//   - Bash + "echo hi"  → hook="block_unsafe_merge", blocked=false (no gh pr merge pattern,
//                          returns ALLOW before any gh shell call — deterministic)
//
// Note: Read/Glob → architect_guard_decide reads .sos-state/architect-active fs marker.
// Result is environment-dependent (see phiếu §3b + Discovery). Covered by existing
// decide_architect_guard_* tests; not duplicated here to avoid nondeterminism.

/// Spawn serve, send initialize + tools/call why_blocked, collect stdout.
fn call_why_blocked(tool_name: &str, file_path: Option<&str>, command: Option<&str>) -> String {
    use std::process::{Command, Stdio};
    use std::io::Write;
    use std::time::Duration;

    let binary = assert_cmd::cargo::cargo_bin("claude-hooks");

    // Build tool_input JSON
    let tool_input = match (file_path, command) {
        (Some(fp), _) => format!(r#"{{"file_path":"{fp}"}}"#),
        (_, Some(cmd)) => format!(r#"{{"command":"{cmd}"}}"#),
        _ => "{}".to_owned(),
    };

    let mut child = Command::new(&binary)
        .arg("serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn claude-hooks serve");

    {
        let stdin = child.stdin.as_mut().expect("stdin pipe");
        let init_req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"why-blocked-test","version":"1.0"}}}"#;
        let init_notif = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let call_req = format!(
            r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"why_blocked","arguments":{{"tool_name":"{tool_name}","tool_input":{tool_input}}}}}}}"#
        );
        writeln!(stdin, "{init_req}").expect("write initialize");
        writeln!(stdin, "{init_notif}").expect("write initialized");
        writeln!(stdin, "{call_req}").expect("write tools/call");
    }
    drop(child.stdin.take());

    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();
    loop {
        match child.try_wait().expect("try_wait") {
            Some(_) => break,
            None => {
                if start.elapsed() > timeout {
                    child.kill().ok();
                    panic!("serve timed out");
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }

    let output = child.wait_with_output().expect("collect output");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// why_blocked: Edit + .env.local → hook="block_env_edit", blocked=true
#[test]
fn why_blocked_edit_env_local_blocked() {
    let stdout = call_why_blocked("Edit", Some(".env.local"), None);
    assert!(stdout.contains("block_env_edit"),
        "expected hook=block_env_edit in stdout:\n{stdout}");
    assert!(stdout.contains("\"blocked\":true"),
        "expected blocked=true in stdout:\n{stdout}");
}

/// why_blocked: Edit + .env.example → hook="block_env_edit", blocked=false (allowlist)
#[test]
fn why_blocked_edit_env_example_allowed() {
    let stdout = call_why_blocked("Edit", Some(".env.example"), None);
    assert!(stdout.contains("block_env_edit"),
        "expected hook=block_env_edit in stdout:\n{stdout}");
    assert!(stdout.contains("\"blocked\":false"),
        "expected blocked=false in stdout:\n{stdout}");
}

/// why_blocked: Write + .env.local → hook="block_env_edit", blocked=true (Write routes same as Edit)
#[test]
fn why_blocked_write_env_local_blocked() {
    let stdout = call_why_blocked("Write", Some(".env.local"), None);
    assert!(stdout.contains("block_env_edit"),
        "expected hook=block_env_edit in stdout:\n{stdout}");
    assert!(stdout.contains("\"blocked\":true"),
        "expected blocked=true in stdout:\n{stdout}");
}

/// why_blocked: MultiEdit + .env.local → hook="block_env_edit", blocked=true
#[test]
fn why_blocked_multiedit_env_local_blocked() {
    let stdout = call_why_blocked("MultiEdit", Some(".env.local"), None);
    assert!(stdout.contains("block_env_edit"),
        "expected hook=block_env_edit in stdout:\n{stdout}");
    assert!(stdout.contains("\"blocked\":true"),
        "expected blocked=true in stdout:\n{stdout}");
}

/// why_blocked: NotebookEdit + .env.local → hook="block_env_edit", blocked=true
#[test]
fn why_blocked_notebook_edit_env_local_blocked() {
    let stdout = call_why_blocked("NotebookEdit", Some(".env.local"), None);
    assert!(stdout.contains("block_env_edit"),
        "expected hook=block_env_edit in stdout:\n{stdout}");
    assert!(stdout.contains("\"blocked\":true"),
        "expected blocked=true in stdout:\n{stdout}");
}

/// why_blocked: unknown tool_name → hook="none", blocked=false
#[test]
fn why_blocked_unknown_tool_none() {
    let stdout = call_why_blocked("WebFetch", None, None);
    assert!(stdout.contains("\"hook\":\"none\""),
        "expected hook=none in stdout:\n{stdout}");
    assert!(stdout.contains("\"blocked\":false"),
        "expected blocked=false in stdout:\n{stdout}");
    assert!(stdout.contains("no hook matches tool WebFetch"),
        "expected reason about WebFetch in stdout:\n{stdout}");
}

/// why_blocked: Bash + non-merge command → hook="block_unsafe_merge", blocked=false
/// (parse_merge_pr returns None for "echo hi" → ALLOW before any gh shell call)
#[test]
fn why_blocked_bash_non_merge_allowed() {
    let stdout = call_why_blocked("Bash", None, Some("echo hi"));
    assert!(stdout.contains("block_unsafe_merge"),
        "expected hook=block_unsafe_merge in stdout:\n{stdout}");
    assert!(stdout.contains("\"blocked\":false"),
        "expected blocked=false in stdout:\n{stdout}");
}
