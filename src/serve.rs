// MCP server — rmcp 1.7 stdio JSON-RPC, 4 hook tools (P006).
// Decision-core fns live in hooks:: module; this file only maps them to MCP tool output.

use rmcp::{tool_router, tool, ServiceExt, transport};
use rmcp::handler::server::wrapper::{Parameters, Json};
use rmcp::handler::server::tool::ToolRouter;
use rmcp::schemars;
use serde::{Serialize, Deserialize};
use crate::hooks;
use crate::io::Decision;

// ── Input structs ─────────────────────────────────────────────────────────────

#[derive(Deserialize, rmcp::schemars::JsonSchema, Default)]
struct GuardInput {
    file_path: Option<String>,
    pattern: Option<String>,
}

#[derive(Deserialize, rmcp::schemars::JsonSchema, Default)]
struct EnvEditInput {
    file_path: Option<String>,
    notebook_path: Option<String>,
}

#[derive(Deserialize, rmcp::schemars::JsonSchema, Default)]
struct MergeInput {
    command: Option<String>,
}

#[derive(Deserialize, rmcp::schemars::JsonSchema, Default)]
struct EmptyInput {}

// ── Output structs ────────────────────────────────────────────────────────────

#[derive(Serialize, rmcp::schemars::JsonSchema)]
struct DecisionOutput {
    blocked: bool,
    exit_code: i32,
    reason: Option<String>,
}

impl From<Decision> for DecisionOutput {
    fn from(d: Decision) -> Self {
        Self { blocked: d.blocked, exit_code: d.exit_code, reason: d.reason }
    }
}

#[derive(Serialize, rmcp::schemars::JsonSchema)]
struct BannerOutput {
    banner: String,
}

// ── Server ────────────────────────────────────────────────────────────────────

#[allow(dead_code)] // tool_router is accessed by macro-generated ServerHandler impl
struct HooksServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router(server_handler)]
impl HooksServer {
    #[tool(
        name = "architect_guard",
        description = "Check Architect envelope: block Read/Glob to source paths when architect-active marker present. Returns block decision + reason."
    )]
    fn architect_guard(&self, Parameters(i): Parameters<GuardInput>) -> Json<DecisionOutput> {
        Json(hooks::architect_guard_decide(i.file_path.as_deref(), i.pattern.as_deref()).into())
    }

    #[tool(
        name = "block_env_edit",
        description = "Check if Edit/Write to a .env* file (not .env.example) should be blocked."
    )]
    fn block_env_edit(&self, Parameters(i): Parameters<EnvEditInput>) -> Json<DecisionOutput> {
        Json(hooks::block_env_edit_decide(i.file_path.as_deref(), i.notebook_path.as_deref()).into())
    }

    #[tool(
        name = "block_unsafe_merge",
        description = "Check if a `gh pr merge` command targets a security surface without an APPROVE review. May report gh-unavailable (fail-closed)."
    )]
    fn block_unsafe_merge(&self, Parameters(i): Parameters<MergeInput>) -> Json<DecisionOutput> {
        Json(hooks::block_unsafe_merge_decide(i.command.as_deref()).into())
    }

    #[tool(
        name = "session_banner",
        description = "Render the SessionStart banner text (sprint + advisory staleness + orchestrator contract)."
    )]
    fn session_banner(&self, Parameters(_): Parameters<EmptyInput>) -> Json<BannerOutput> {
        Json(BannerOutput { banner: hooks::render_banner() })
    }
}

impl HooksServer {
    fn new() -> Self {
        Self { tool_router: Self::tool_router() }
    }
}

/// Dispatch entry: build tokio current-thread runtime, serve stdio until client close.
pub fn run() -> i32 {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("serve: runtime build failed: {e}");
            return crate::io::ALLOW;
        }
    };
    rt.block_on(async {
        match HooksServer::new().serve(transport::stdio()).await {
            Ok(svc) => { let _ = svc.waiting().await; }
            Err(e) => eprintln!("serve: handshake failed: {e}"),
        }
    });
    crate::io::ALLOW
}
