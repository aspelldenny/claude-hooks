# Changelog

Format loosely follows Keep a Changelog.

## v0.9.0 — P010 architect-guard TRUE parity tarot (fix F-004) — 2026-06-09

- **P010**: architect-guard TRUE parity tarot: tool_name dispatch + Write/Edit allowlist branch (`docs/ticket/P*-*.md` only, deny TICKET_TEMPLATE) + Read/Glob superset (prisma/ + `*.prisma`/`*.sql` + Glob `path` root) + 2 block messages (read/write). Fix F-004.
  - **tool_name dispatch** (`src/hooks/mod.rs`): `architect_guard_decide` signature extended to 4-arg: `(tool_name, file_path, pattern, path)`. Dispatch on `tool_name`: `Read|Glob` → Read/Glob branch (3 candidates: file_path, pattern, path); `Write|Edit` → Write/Edit allowlist branch; `_` (None/unknown) → default ALLOW (real Claude Code payload always has tool_name — no-tool_name = default ALLOW is faithful tarot).
  - **Read/Glob superset** (`is_forbidden_for_read`): added `prisma/*|*/prisma/*` (oracle L43) and `*.prisma|*.sql` (oracle L47) to existing forbidden set. Added `path` (Glob search root) as 3rd candidate alongside `file_path` and `pattern`.
  - **Write/Edit branch** (new — `is_allowed_for_write`): deny `docs/ticket/TICKET_TEMPLATE.md` FIRST (explicit, defense-in-depth); allow `docs/ticket/P*-*.md|*/docs/ticket/P*-*.md`; else block. No-path → ALLOW (defensive, oracle L111).
  - **2 block messages** (verbatim oracle): `block_read` (Read/Glob violation) + `block_write` (Write/Edit violation). Both port byte-verbatim from tarot oracle L65-94.
  - **Marker GIỮ `.sos-state/architect-active`** (binary convention, F-005 marker-path unify defer).
  - **`src/io.rs`**: `HookPayload` +`tool_name: Option<String>` (top-level, `#[serde(default)]`); `ToolInput` +`path: Option<String>` (Glob root, `#[serde(default)]`).
  - **`src/serve.rs`**: `GuardInput` +`tool_name`+`path`; `architect_guard` tool passes 4-arg; `why_blocked` Read|Glob branch updated to pass tool_name.
  - **`scripts/architect-guard.sh`**: synced to tarot 119-line version with MARKER divergence `.sos-state/` (F-005 defer, noted in comment).
  - **Tests (Tension 1)**: updated 5 P002 cli.rs tests to add `"tool_name":"Read"/"Glob"` (real payload shape); updated `mcp_handshake.rs::run_architect_guard` helper. Added 11 P010 integration tests: Write/Edit cases (5) + Read/Glob superset (5) + dispatch-default no-tool_name test (1). Updated `decide_architect_guard_marker_src_blocks` in mcp_handshake.rs.
  - **Total tests: 104** (49 unit + 43 cli.rs + 12 mcp_handshake.rs) — all pass. Parity verified vs `bash ~/tarot/scripts/architect-guard.sh` (12/12 cases, exit code + stderr message verbatim).
  - Docs Gate (Tầng 1 — hook security-surface): `docs/ARCHITECTURE.md` architect-guard section updated (tool_name dispatch + Write/Edit allowlist + Read/Glob superset + 2 messages + why_blocked limitation note). stdin-JSON Harness updated (tool_name, path fields). MCP tool table updated.

## v0.8.0 — P008 ship prep: serverInfo.name fix + README + version bump — 2026-06-09

- **P008**: Release prep — 4 fixes before `cargo publish`. Phase 4 ship-prep DONE.
  - **`serverInfo.name` fix (MCP)**: handshake previously returned `"name":"rmcp"` (from `Implementation::from_build_env()` expanding at rmcp crate site). Fixed by replacing `#[tool_router(server_handler)]` with `#[tool_router]` + explicit `#[tool_handler] impl ServerHandler for HooksServer { fn get_info() ... }` returning `Implementation::new("claude-hooks", env!("CARGO_PKG_VERSION"))`. Handshake now returns `"name":"claude-hooks","version":"0.8.0"`. Tool routing unchanged — all 93 existing tests pass.
  - **`README.md`**: replaced 2-line stub with usable English docs: Install, CLI usage (5 subcmd + stdin JSON shape + exit codes + settings.json wire example), MCP mode (5 tools + `why_blocked` description + `.mcp.json` example), exit convention (fail-open default / `block-unsafe-merge` fail-CLOSED).
  - **`Cargo.toml` version**: `0.1.0` → `0.8.0` (CHANGELOG head was already v0.7.0; P008 = next bump). `env!("CARGO_PKG_VERSION")` in `get_info()` now resolves to "0.8.0" at compile time.
  - **`Cargo.toml` exclude**: added `exclude` list to slim crate package from ~778KB/101-files to ~148KB/25-files. Excludes `scripts/`, `docs/`, `phieu/`, `.sos-state/`, `.backup/`, `.claude/`, `.github/`, `hooks/pre-commit`, `hooks/pre-push`. `src/hooks/mod.rs` kept (require `hooks` exclude to be file-specific, NOT prefix glob). `tests/` kept (cargo publish verify). `cargo publish --dry-run` clean.
  - **`docs/ARCHITECTURE.md`**: serve section updated — `#[tool_router(server_handler)]` → `#[tool_router]` + explicit `ServerHandler::get_info()` explanation. Status line added to Overview: Phase 1-3 complete (P001–P007); Phase 4 P008 ship-prep; P009 wire-tarot follow-on.
  - **Total tests: 93** (49 unit + 32 cli.rs + 12 mcp_handshake.rs) — all pass unchanged (metadata-only change, no behavior delta).

## v0.7.0 — P007 `why_blocked` composite MCP router — 2026-06-09

- **P007**: Add `why_blocked` as 5th MCP tool in `src/serve.rs`. Phase 3 DONE (P006 + P007 complete).
  - **`why_blocked` composite router**: accepts `{ tool_name, tool_input }` (mirror Claude Code PreToolUse payload shape) → routes `tool_name` to matching hook per `.claude/settings.json` matchers → returns `{ hook, blocked, exit_code, reason }`. One call = full debug answer (which hook fired + block/allow + reason), no need to know hook routing manually.
  - **Routing (verbatim settings.json matchers):** `Read|Glob` → `architect_guard_decide`; `Edit|Write|MultiEdit|NotebookEdit` → `block_env_edit_decide`; `Bash` → `block_unsafe_merge_decide`; all other tool_names → `hook="none", blocked=false, exit_code=0, reason="no hook matches tool <name>"`.
  - **New structs in `src/serve.rs`:** `ToolInputArg { file_path?, pattern?, notebook_path?, command? }` (flat, reusable); `WhyBlockedInput { tool_name, tool_input: ToolInputArg }` (`#[serde(default)]` on `tool_input` for fail-open input); `WhyBlockedOutput { hook, blocked, exit_code, reason }`. Derive: `Deserialize + rmcp::schemars::JsonSchema + Default` (input), `Serialize + rmcp::schemars::JsonSchema` (output) — same pattern as 4 P006 structs.
  - **State-honesty:** `architect_guard` route reads `.sos-state/architect-active` marker from real fs; `block_unsafe_merge` route makes real `gh` shell calls (fail-CLOSED). No mocking — output reflects actual serve environment state.
  - **No new deps, no Cargo.toml changes.** rmcp/serde already present (P006).
  - `tests/mcp_handshake.rs` updated: (a) handshake smoke `mcp_serve_tools_list_returns_4_tools` → `_5_tools` (assert `why_blocked` present). (b) 7 new `why_blocked_*` routing tests via MCP `tools/call` (deterministic, no fs/network dependency): `Edit`+`.env.local`→blocked, `Edit`+`.env.example`→allowed, `Write`/`MultiEdit`/`NotebookEdit`+`.env.local`→blocked, `WebFetch`→hook=none, `Bash`+`echo hi`→hook=block_unsafe_merge+allowed.
  - **Total tests: 93** (49 unit + 32 cli.rs + 12 mcp_handshake.rs). 86 pre-P007 tests pass unchanged.
  - Docs Gate (Tầng 1 — MCP surface + security-debug): `docs/ARCHITECTURE.md` — serve section: 4→5 tools; added `why_blocked` row to MCP tool table; added routing table (`tool_name → hook`); added state-honesty notes; Module Structure + MCP Surface + Data Flow updated.

## v0.6.0 — P006 serve MCP server + Decision-core refactor — 2026-06-09

- **P006**: Implement `serve` subcmd as real MCP server (rmcp 1.7 stdio JSON-RPC). Phase 3 DONE.
  - **Decision-core refactor** (`src/hooks/mod.rs`): all 4 hook functions split into `*_decide(inputs) -> Decision` (logic, no IO) + thin CLI wrapper (`-> i32`, reads stdin, prints reason, returns exit_code). `Decision { exit_code: i32, blocked: bool, reason: Option<String> }` added to `src/io.rs`. CLI behavior BẮT BIẾN — 81 pre-P006 tests pass unchanged.
  - **`src/serve.rs`**: `HooksServer { tool_router: ToolRouter<Self> }` with `#[tool_router(server_handler)]` macro (rmcp). 4 `#[tool]` sync methods calling `*_decide`/`render_banner`, returning `Json<DecisionOutput>` / `Json<BannerOutput>`. `run() -> i32`: `tokio::runtime::Builder::new_current_thread().enable_all()` + `block_on(serve(transport::stdio()).waiting())`. Returns ALLOW (0).
  - **rmcp features added**: `macros` added to rmcp dep in Cargo.toml (was `server`, `transport-io`). Required for `#[tool_router]`/`#[tool]` proc-macros. No new crate added.
  - **Tokio runtime**: `new_current_thread().enable_all()`. No `time` or `rt-multi-thread` features needed — rmcp stdio does not use tokio timer/multi-thread internally.
  - **Transport framing**: newline-delimited JSON (rmcp async_rw codec). MCP sequence: `initialize` → `notifications/initialized` → tool calls. Server exits on stdin EOF.
  - **`io::block` removed**: unused after Decision-core refactor (was `pub fn block(reason: &str) -> i32` in io.rs). `*_decide` functions build reason strings directly; CLI wrappers `eprintln!` them. Not a public API breakage (binary crate).
  - `tests/mcp_handshake.rs` (new, 5 tests): (a) 4 Decision-core unit tests via CLI shim (architect_guard_decide: no-marker/marker paths; block_env_edit_decide: .env.local block / .env.example allow). (b) 1 MCP handshake smoke: spawn `claude-hooks serve`, write `initialize` + `notifications/initialized` + `tools/list` over stdin, close stdin, assert stdout contains all 4 tool names + `"jsonrpc"` key.
  - **Total tests: 86** (49 unit + 32 cli.rs + 5 mcp_handshake.rs). Pre-P006 baseline 81 = all pass.
  - Docs Gate (Tầng 1 — MCP surface + security-surface Decision-core): `docs/ARCHITECTURE.md` — `serve` stub→real; new `serve` section (Decision-core table, MCP tool table, transport/framing, tokio setup, rmcp features); Module Structure + Data Flow updated; per-hook sections updated with Decision-core note.

## v0.5.0 — P005 session-banner port — 2026-06-09

- **P005**: Port `session-banner` subcmd 1:1 from `scripts/session-start-banner.sh` (188 lines). Render hook: reads file/git state → prints banner to **stdout** → always exit 0. Phase 2 DONE.
  - `src/hooks/mod.rs`: replaced stub `session_banner()` with full 10-step render pipeline + 5 pure helpers: `find_sprint_block`, `count_items`, `staleness_days` (manual ISO→epoch, Hinnant days-from-civil, no chrono/time dep), `staleness_category`, `doc_size_warns`.
  - **Date strategy:** manual ISO→epoch (Howard Hinnant public-domain algorithm, ~10 lines). No new dep added. Verified: `2026-06-09T00:00:00Z` → 1780963200 matches `date -j` output. `staleness_days(iso, now_epoch)` injects `now_epoch` for deterministic unit tests.
  - **Render hook fail-OPEN**: session-banner ALWAYS returns ALLOW (exit 0), every failure branch is silent. OPPOSITE of `block_unsafe_merge` (fail-CLOSED). Do not confuse.
  - **stdin NOT read**: `read_payload()` not called. Banner renders from fs/git state only. (Opposite of 3 block hooks.)
  - **Banner text VERBATIM including bug F-001** (oracle L178): `Marker:` line missing `touch .sos-state/worker-active`. Port doctrine: verbatim copy — fix must go upstream sos-kit updating `.sh` + Rust + orchestrator.md + ORCHESTRATION.md atomically. Text now lives in 2 places.
  - **git shelling**: `git branch --merged main` via `Command::new("git").args([…])`, NOT `sh -c`. Fail → empty (silent skip).
  - `tests/cli.rs`: 4 new P005 integration fixtures (CLAUDE_PROJECT_DIR isolation, no `tempfile` dep): with-BACKLOG, no-BACKLOG, no-H2, fallback-header.
  - `src/hooks/mod.rs`: 27 unit tests for all 5 pure helpers (deterministic, no fs/git/clock).
  - Docs Gate (Tầng 1 — orchestration-surface): `docs/ARCHITECTURE.md` — session-banner stub→real; new section with full render pipeline, fail-open/stdout/no-stdin divergence note, date manual-epoch note, F-001 note, pure helpers list; Data Flow updated.

## v0.4.0 — P004 block-unsafe-merge port — 2026-06-09

- **P004**: Port `block-unsafe-merge` subcmd 1:1 from `scripts/block-unsafe-merge.sh`. Security-surface guard: blocks `gh pr merge <N>` when the PR touches security-surface files without a `/security-review APPROVE` comment.
  - `src/io.rs`: added `#[serde(default)] pub command: Option<String>` to `ToolInput` — additive (P002/P003 payloads parse OK via `serde(default)`). Shared harness change; P006/P007 may reuse.
  - `src/hooks/mod.rs`: replaced stub `block_unsafe_merge()` with full 8-step pipeline: stdin parse via `command` field, `parse_merge_pr` (numbered form only), `extract_skip_marker` (override), `SECURITY_SURFACE_EXTRA` env extend, gh call #1 (`gh pr diff --name-only`), `touches_security_surface` (two-branch), gh call #2 (`gh pr view --json comments`), `verdict_is_approve`. Verbatim oracle messages (tiếng Việt + `⛔`/`⚠️`).
  - **FAIL-CLOSED divergence (intentional):** gh diff fail/empty → `BLOCK` exit 2 (oracle L68-83). This is the **opposite** of `architect-guard`/`block-env-edit`/`session-banner` (all fail-open). Do NOT change to fail-open — this design is the hook's core security property.
  - `security_surface`: two-branch check — (a) base pattern `\.env[^.]` etc., (b) `^\.env` non-example fallback catches `.env.local` and similar that pattern (a) misses.
  - `tests/cli.rs`: 4 new P004 integration test fixtures (gh-free paths): non-merge command, override marker, empty stdin, branch-only bypass.
  - `src/hooks/mod.rs`: 22 unit tests for all 4 pure helpers: `parse_merge_pr` (5 cases), `extract_skip_marker` (3), `touches_security_surface` (10), `verdict_is_approve` (4).
  - Docs Gate (Tầng 1 — security-surface): `docs/ARCHITECTURE.md` — `block-unsafe-merge` status stub→real; new section with pipeline, fail-CLOSED note, security-surface pattern; stdin-JSON Harness updated with `command` field; Data Flow updated.
  - **Conscious parity gap:** env-fallback `CLAUDE_TOOL_INPUT` (oracle L24-28) not ported — same decision as P002/P003. See `docs/discoveries/P004.md`.

## v0.3.0 — P003 block-env-edit port — 2026-06-09

- **P003**: Port `block-env-edit` subcmd 1:1 from `scripts/block-env-edit.sh`. Security-surface guard: blocks Edit/Write to `.env*` files (except `.env.example`) to prevent secret leak into prompt/context/log.
  - `src/hooks/mod.rs`: replaced stub `block_env_edit()` with 8-step logic: stdin parse, `file_path`/`notebook_path` fallback (no `pattern`), basename extraction via `rsplit('/')`, `.env.example` allowlist, regex `^\.env($|\.)` block check, verbatim oracle block message (tiếng Việt + `⛔`), exit 0/2.
  - Added `use regex::Regex;` — dep already present (`Cargo.toml:20 regex = "1"`), no new dep added.
  - `tests/cli.rs`: 10 new P003 fire-test fixtures (P057 verify-cò) — no isolation needed (no global marker). Covers: `.env`, `.env.example`, `.envrc`, `.env.local`, `.env.production`, `/some/dir/.env` (basename), `config.yaml`, `notebook_path` fallback, empty stdin, `.environment`.
  - Docs Gate (Tầng 1 — security-surface): `docs/ARCHITECTURE.md` — `block-env-edit` status stub→real; new section `block-env-edit (P003)` with full pipeline; Data Flow note updated.
  - **Conscious parity gap:** env-fallback `CLAUDE_TOOL_INPUT` (oracle L16-20) not ported — Claude Code always pipes stdin; env-fallback would require shared `io.rs` harness change (Tầng 1, out of scope). Same decision as P002. See `docs/discoveries/P003.md`.

## v0.2.0 — P002 architect-guard port — 2026-06-09

- **P002**: Port `architect-guard` subcmd 1:1 from `scripts/architect-guard.sh`.
  - `src/hooks/mod.rs`: replaced stub `architect_guard()` with 8-step logic: `CLAUDE_PROJECT_DIR` repo-root resolution, marker gate (`.sos-state/architect-active`), stdin path parse (`file_path` priority / `pattern` fallback), fail-open (no path → ALLOW), `./` strip, `.md` allow, forbidden pattern check (source dirs, test dirs, build artifacts, extensions), block with verbatim oracle message.
  - `src/io.rs`: removed `#[allow(dead_code)]` from `BLOCK` and `block()` — now used by `architect_guard`.
  - `tests/cli.rs`: 6 new P002 fire-test fixtures (P057 verify-cò). Isolation via `CLAUDE_PROJECT_DIR` env pointing to unique temp dir per test (no real `.sos-state/` touched).
  - Docs Gate (Tầng 1 — security-surface): `docs/ARCHITECTURE.md` updated (architect-guard status stub→real, forbidden set, marker gate, exit semantics, pipeline detail).

## v0.1.0 — P001 scaffold CLI — 2026-06-09

- **P001**: Scaffold CLI 5-subcommand + stdin-JSON harness + exit-code convention.
  - `src/main.rs`: clap derive entry point dispatching to 5 subcmds (`architect-guard`, `block-env-edit`, `block-unsafe-merge`, `session-banner`, `serve`). Hook entries return `i32`; `main` calls `process::exit`.
  - `src/io.rs`: `HookPayload`/`ToolInput` serde model (all fields `Option`); `read_payload()` fail-open (empty stdin / invalid JSON → `Default`); `ALLOW=0` / `BLOCK=2` constants; `block(reason)` stderr helper.
  - `src/hooks/mod.rs`: 4 stub hook functions return `ALLOW`; `architect_guard` + `block_env_edit` wire `read_payload()` harness.
  - `src/serve.rs`: MCP stub, prints `"serve: not yet implemented (P006)"` to stderr, exits 0.
  - `tests/cli.rs`: 8 integration tests (verify-cò P057) — all pass.
  - Docs Gate: `docs/ARCHITECTURE.md` CLI surface section added.

## v0.0.0 — sos adopt — 2026-06-09

- Spine retrofitted via `sos adopt`.
