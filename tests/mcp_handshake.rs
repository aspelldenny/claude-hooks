// P006: MCP handshake smoke test + Decision-core unit tests.
//
// Smoke: spawn `claude-hooks serve`, send initialize + tools/list over stdin (newline-delimited
// JSON-RPC, rmcp stdio framing), assert stdout contains 4 tool names + valid JSON-RPC.
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
/// assert stdout contains all 4 expected tool names in valid JSON-RPC response.
///
/// Framing: rmcp stdio uses newline-delimited JSON (one JSON object per line).
/// Sequence: initialize request → notifications/initialized → tools/list → close stdin.
/// The server exits when stdin reaches EOF (transport close → waiting() returns).
#[test]
fn mcp_serve_tools_list_returns_4_tools() {
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

    // Assert 4 tool names in stdout (from tools/list response)
    for tool in &["architect_guard", "block_env_edit", "block_unsafe_merge", "session_banner"] {
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
