use serde::Deserialize;

/// Fields are intentionally forward-declared for P002+ hook implementations.
#[allow(dead_code)]
#[derive(Debug, Default, Deserialize)]
pub struct ToolInput {
    pub file_path: Option<String>,
    pub pattern: Option<String>,
    pub notebook_path: Option<String>,
}

/// Wrapper for Claude Code PreToolUse JSON payload.
/// Fields are intentionally forward-declared for P002+ hook implementations.
#[allow(dead_code)]
#[derive(Debug, Default, Deserialize)]
pub struct HookPayload {
    #[serde(default)]
    pub tool_input: ToolInput,
}

/// Read stdin, parse JSON. FAIL-OPEN: empty stdin / invalid JSON -> Default (empty).
/// Mirrors scripts/architect-guard.sh:44 + scripts/block-env-edit.sh:23,35 (anchors #6,#7).
pub fn read_payload() -> HookPayload {
    use std::io::Read;
    let mut buf = String::new();
    if std::io::stdin().read_to_string(&mut buf).is_err() {
        return HookPayload::default();
    }
    let buf = buf.trim();
    if buf.is_empty() {
        return HookPayload::default();
    }
    serde_json::from_str(buf).unwrap_or_default()
}

pub const ALLOW: i32 = 0;
/// Exit code 2 = block (reason -> stderr).
pub const BLOCK: i32 = 2;

/// Block with reason printed to stderr. Returns BLOCK so caller can return it.
pub fn block(reason: &str) -> i32 {
    eprintln!("{reason}");
    BLOCK
}
