# Architecture — claude-hooks

## Overview

`claude-hooks` is a Rust binary replacing ~418 lines of Bash hot-path hooks for Claude Code.
It runs in two modes: **CLI** (invoked by Claude Code PreToolUse hooks) and **MCP** (stdio JSON-RPC server for debug tooling).
The binary name is `claude-hooks`; subcommands are kebab-case.

**Status:** Phase 1-3 complete (P001–P007). Phase 4 ship-prep (P008): README + publish-ready (v0.8.0, serverInfo.name fixed). Phase 4 wire-tarot = P009. P010: architect-guard TRUE parity tarot (F-004 fix, v0.9.0).

## CLI Surface

### Subcommands (5)

| Subcommand | Clap variant | Status | Port target |
|---|---|---|---|
| `architect-guard` | `Cmd::ArchitectGuard` | real (P002) | — |
| `block-env-edit` | `Cmd::BlockEnvEdit` | real (P003) | — |
| `block-unsafe-merge` | `Cmd::BlockUnsafeMerge` | real (P004) | — |
| `session-banner` | `Cmd::SessionBanner` | real (P005) | — |
| `serve` | `Cmd::Serve` | real (P006) | — |

Kebab-case names are auto-derived by clap from PascalCase variants (verified P001: no `#[command(name=...)]` needed for clap 4.6).

### `architect-guard` (P002 real implementation, P010 TRUE parity tarot)

Ports `scripts/architect-guard.sh` 1:1 (tarot oracle 119-line). Fires on `Read`/`Glob`/`Write`/`Edit` PreToolUse calls.

**Pipeline:**

1. Resolve repo root from `CLAUDE_PROJECT_DIR` env (fallback: cwd).
2. **Marker gate:** if `.sos-state/architect-active` does not exist → `ALLOW` (not running as Architect). GIỮ `.sos-state/` (F-005 marker-path unify defer — oracle tarot uses `.claude/.architect-active`).
3. **tool_name dispatch** (P010, oracle L96-116):
   - `Read` | `Glob` → **Read/Glob branch** (steps 4-6 below).
   - `Write` | `Edit` → **Write/Edit branch** (steps 7-8 below).
   - None / other → **default ALLOW** (real Claude Code payload always has tool_name; absent = not a guarded call).

4. **(Read/Glob branch)** Check 3 candidates: `tool_input.file_path`, `tool_input.pattern`, `tool_input.path` (Glob search root). For each non-empty candidate:
5. Strip leading `./`. If ends with `.md` → skip (early-allow; docs are Architect's domain).
6. **`is_forbidden_for_read`** — `BLOCK` (block_read message) if any match:

   | Group | Pattern | Rust check |
   |---|---|---|
   | Source dirs (prefix) | `src/*`, `lib/*`, `app/*`, `pkg/*` | `starts_with` |
   | Source dirs (segment) | `*/src/*`, `*/lib/*`, `*/app/*`, `*/pkg/*` | `contains` |
   | crates subcrate src | `crates/*/src/*` | `starts_with("crates/")` + `contains("/src/")` |
   | Prisma dirs (P010 NEW) | `prisma/*`, `*/prisma/*` | `starts_with` / `contains` |
   | Test dirs (prefix) | `tests/*`, `test/*`, `__tests__/*` | `starts_with` |
   | Test dirs (segment) | `*/tests/*`, `*/test/*` | `contains` |
   | Build artifacts (prefix only) | `node_modules/*`, `target/*`, `dist/*`, `build/*`, `.next/*`, `.nuxt/*`, `.svelte-kit/*` | `starts_with` |
   | Code extensions | `*.rs *.ts *.tsx *.js *.jsx *.py *.go *.java *.cpp *.c *.h *.hpp` | `ends_with` |
   | DB schema extensions (P010 NEW) | `*.prisma *.sql` | `ends_with` |

   All candidates pass without hit → `ALLOW`.

7. **(Write/Edit branch)** No `file_path` (empty/missing) → `ALLOW` (defensive, oracle L111).
8. **`is_allowed_for_write`** — `BLOCK` (block_write message) if NOT in allowlist:

   | Check | Order | Rule |
   |---|---|---|
   | `docs/ticket/TICKET_TEMPLATE.md` | FIRST — explicit deny | Template is reference, not a phiếu |
   | `docs/ticket/P*-*.md` or `*/docs/ticket/P*-*.md` | ALLOW | Phiếu files only |
   | Everything else | BLOCK | src/, CLAUDE.md, BACKLOG.md, CHANGELOG.md, guides → Worker's domain |

**Exit codes:** `0` (ALLOW), `2` (BLOCK).

**2 block messages** (stderr, verbatim oracle tarot L65-94):
- `block_read` (Read/Glob): `🚫 Architect envelope violation (Read/Glob)` + path + Task 0 anchor instructions.
- `block_write` (Write/Edit): `🚫 Architect envelope violation (Write/Edit)` + path + allowlist reminder.

**P006/P010 Decision-core:** `architect_guard_decide(tool_name, file_path, pattern, path) -> Decision` (4-arg, P010). `architect_guard() -> i32` is the thin CLI wrapper (reads stdin, extracts `p.tool_name` + `p.tool_input.{file_path,pattern,path}`, calls `_decide`, prints reason, returns exit_code).

**Fail-open:** fail-OPEN (marker absent / no tool_name / no paths → ALLOW). KHÔNG fail-CLOSED (opposite of block_unsafe_merge).

**state-honesty note (MCP):** MCP tool `architect_guard` + `why_blocked` Read/Glob route both read `.sos-state/architect-active` from the fs of the serve environment. If serve runs without marker, Read/Glob calls return ALLOW (correct — no architect active).

### `block-env-edit` (P003 — real implementation)

Ports `scripts/block-env-edit.sh` 1:1. Fires on every `Edit`/`Write` PreToolUse call. Guards against secret leak (`.env*` files contain API keys, DB credentials, webhook tokens).

**Pipeline (8 steps):**

1. Read stdin payload via `read_payload()` (fail-open). **Note:** env-fallback `CLAUDE_TOOL_INPUT` (oracle L16-20) intentionally not ported — Claude Code always pipes stdin; see `docs/discoveries/P003.md` for rationale.
2. Empty payload (empty stdin) → `ALLOW` (fail-open via steps 3-4).
3. Parse path: `tool_input.file_path` (priority), fallback `tool_input.notebook_path` (NotebookEdit). **No `pattern` field** — this hook does not handle Glob.
4. No path parsed → `ALLOW` (fail-open).
5. **Basename:** take last `/`-delimited segment of path. (`/a/b/.env` → `.env`).
6. **Allowlist:** basename `== ".env.example"` → `ALLOW` (template, no real secrets).
7. **Block regex:** basename matches `^\.env($|\.)` → `io::block(msg)` (stderr) → `BLOCK` (exit 2).
   - Matches: `.env`, `.env.local`, `.env.production`, `.env.staging`, …
   - Does NOT match: `.envrc`, `.environment`, `config.yaml`, …
8. Else → `ALLOW`.

**Exit codes:** `0` (ALLOW), `2` (BLOCK).

**Block message** (stderr, verbatim oracle L46-59): `⛔ BLOCKED: Edit/Write tới <full-path> bị chặn.` + secret-leak rationale + valid alternatives + override instructions.

**P006 Decision-core:** `block_env_edit_decide(file_path, notebook_path) -> Decision`. CLI wrapper: `block_env_edit() -> i32`.

### `session-banner` (P005 — real implementation)

Ports `scripts/session-start-banner.sh` 1:1. Fires on `SessionStart`. Renders an informational banner from file/git state.

**Key differences from the 3 block hooks:**
- **Reads stdout** (`println!`), NOT stderr. Banner is displayed to Sếp at session start.
- **ALWAYS exit 0** — render hook, informational. Any failure (no BACKLOG, fs error, git fail) → fail-open, silent. This is the **OPPOSITE** of `block_unsafe_merge` (fail-CLOSED). Do NOT change to fail-closed.
- **Does NOT read stdin** — does not call `read_payload()`. Renders from file/git state (oracle never `cat`s stdin).

**Render pipeline (10 steps):**

1. Resolve repo root from `CLAUDE_PROJECT_DIR` env (fallback: cwd). All paths joined to root via `PathBuf::join` (no real `chdir`).
2. Read `docs/BACKLOG.md` — missing → `return ALLOW` (silent).
3. `find_sprint_block(content)` — find first `^## .*Active sprint` header (fallback: first `^## ` with `fallback_used=true`). No `^## ` → `return ALLOW` (silent).
4. `count_items(block)` — count `^- [ ]` (open) and `^- [x]` (done, lowercase x only).
5. **Main banner** (stdout, verbatim oracle L58-71): `━`×60 lines, `🏠 Sếp's project — Active sprint status`, sprint block (first 25 lines), sprint count, optional fallback note.
6. **Doc size warn** (oracle L73-92): check `docs/CHANGELOG.md`, `docs/DISCOVERIES.md`, `CHANGELOG.md` for > 40960 bytes. `doc_size_warns([(rel_path, bytes)])` → `📏 Doc size warning:` + 4-space indented lines (verbatim oracle L85: `⚠️  {doc} ({kb}k > 40k threshold) — gọi thợ trim…`).
7. **Phiếu cleanup nudge** (oracle L94-138): scan `docs/ticket/` (fallback `phieu/active/`) for `P*.md` with non-placeholder `Approved by Chủ nhà:` line. Run `git branch --merged main` via `Command::new("git")` + args vec (NOT `sh -c`) → check if any merged branch matches `/{phieu_id}-` → `🧹 Cleanup nudge:` + 4-space indented nudges.
8. **Advisory staleness** (oracle L140-171, only if `docs/security/advisory-inbox.md` exists): read `.advisory-scan-state`, parse `"last_scan_at"` JSON or legacy raw ISO. `staleness_days(iso, now_epoch)` (injected epoch) → `staleness_category` → Critical (🚨 >= 7 days) / Warn (⚠️  3-6 days) / Silent (0-2 days or negative).
9. **Orchestrator contract + Architect Rule 0** (oracle L173-188, verbatim including bug F-001 — see below).
10. `return ALLOW`.

**Date computation:** manual ISO→epoch (Howard Hinnant days-from-civil, ~10 lines, public domain). No `chrono`/`time` dep. `staleness_days(iso, now_epoch)` takes injected `now_epoch` for deterministic unit tests. Verified: `2026-06-09T00:00:00Z` → 1780963200 matches `date -j -f "%Y-%m-%dT%H:%M:%SZ"`.

**Git shelling:** `git branch --merged main` via `std::process::Command::new("git").args([…])`. Fail → empty (silent skip nudge). NOT `sh -c`.

**Bug F-001 (verbatim port, do NOT fix here):** Oracle L178 `Marker:` line is missing `touch .sos-state/worker-active`. Port doctrine requires verbatim copy; fix must go upstream to sos-kit canonical `.sh` + orchestrator.md + ORCHESTRATION.md simultaneously. Text now lives in 2 places: `scripts/session-start-banner.sh` and `src/hooks/mod.rs`.

**Exit codes:** `0` (ALLOW) only — never exits 2.

**P006 Decision-core:** `render_banner() -> String` contains the full render pipeline. CLI wrapper `session_banner() -> i32` calls `print!("{}", render_banner())` + returns ALLOW. F-001 verbatim bug preserved in `render_banner()`.

**Pure helpers (unit-testable, no fs/git/clock):**
- `find_sprint_block(backlog: &str) -> Option<(String, String, bool)>`
- `count_items(block: &str) -> (usize, usize)`
- `staleness_days(iso: &str, now_epoch: i64) -> Option<i64>`
- `staleness_category(days: i64) -> Staleness`
- `doc_size_warns(docs: &[(&str, u64)]) -> Vec<String>`

### `block-unsafe-merge` (P004 — real implementation)

Ports `scripts/block-unsafe-merge.sh` 1:1. Fires on every `Bash` PreToolUse call. Guards against merging PRs that touch security surface without a `/security-review APPROVE` comment.

**DIVERGENCE — FAIL-CLOSED (intentional, do not change):** When `gh pr diff` fails or returns empty, this hook returns `BLOCK` (exit 2). This is the **opposite** of `architect-guard`, `block-env-edit`, and `session-banner`, which all fail-open (exit 0). Rationale: an unverifiable merge of an unknown security surface must be treated as unsafe. Any future hook inheriting this pattern must explicitly document fail-closed intent.

**Pipeline (8 steps):**

1. Read stdin payload → `tool_input.command`. Empty/missing → `ALLOW` (fail-open). **Note:** env-fallback `CLAUDE_TOOL_INPUT` intentionally not ported (same decision as P002/P003).
2. `parse_merge_pr(command)` — matches `gh pr merge\s+\d+` (Rust regex ≈ oracle L41 `[[:space:]]+[0-9]+`). No match → `ALLOW`. Known limitation: branch-only form `gh pr merge --merge` (no number) bypasses hook (oracle L15-16, intentional).
3. `extract_skip_marker(command)` — matches `[security-review-skip:<reason>]`. Match → print override warning to stderr (verbatim oracle L52-53) → `ALLOW`.
4. Read `SECURITY_SURFACE_EXTRA` env var (optional, per-repo pattern extension).
5. **gh call #1:** `gh pr diff <PR> --name-only`. Fail OR empty stdout → **FAIL-CLOSED BLOCK** (exit 2, verbatim oracle L71-81 message). DO NOT change to fail-open.
6. `touches_security_surface(diff_files, extra)` — two-branch check:
   - **(a)** base pattern (oracle L60 verbatim) or extended pattern matches any filename.
   - **(b)** pattern does NOT match but a `^\.env` non-example file is present (catches `.env.local`, `.env.staging`, etc. that `\.env[^.]` misses). No match in either branch → `ALLOW`.
7. **gh call #2:** `gh pr view <PR> --json comments --jq '.comments[].body'`. Fail → empty string (fail-open, oracle L96).
8. `verdict_is_approve(comments)`:
   - `NoBlock` (no `<!-- security-review-start -->` marker) → `BLOCK` (oracle L120-137, verbatim message including literal `$(gh pr view …)` instruction for user, NOT executed by hook).
   - `Approve` (`^Verdict: APPROVE` within 50 lines of marker) → `ALLOW`.
   - `NeedsReview` (marker present but verdict is not APPROVE) → `BLOCK` (oracle L105-116, includes `$VERDICT_LINE` interpolation).

**Exit codes:** `0` (ALLOW), `2` (BLOCK).

**P006 Decision-core:** `block_unsafe_merge_decide(command) -> Decision`. gh-shelling is inside `_decide` (core makes real gh calls). MCP context: gh may fail when serve env differs from hook env → fail-CLOSED returns `blocked=true, reason="gh unavailable"` (honest, NOT fake ALLOW). CLI wrapper: `block_unsafe_merge() -> i32`. Note: override marker returns `Decision { exit_code: ALLOW, blocked: false, reason: Some(warning_msg) }` — CLI wrapper prints warning via eprintln for any non-None reason.

**Security-surface base pattern (oracle L60 — verbatim, do not modify):**
```
src/|schema\.(prisma|sql)|migrations?/|nginx/|docker-compose.*\.yml|Dockerfile|\.env[^.]|middleware\.|lib/auth/|\.claude/agents/security-|docs/security/|scripts/security-gate|scripts/check-(hardcoded|runtime)-secrets|hooks/pre-commit
```

**Override marker:** `[security-review-skip:<reason>]` in command → hook logs warning to stderr and allows merge. Intended for docs-only PRs where pattern false-positives.

### `serve` (P006/P007 — real implementation)

MCP server: exposes the 4 hook decision functions + 1 composite router as JSON-RPC tools over stdio (rmcp 1.7).

**Decision-core refactor (P006):** all 4 hook functions were refactored to split decision logic from IO.
Each hook now has two layers:

| Layer | Function | Reads stdin? | Prints? | Returns |
|---|---|---|---|---|
| Core (`_decide`) | `architect_guard_decide(tool_name, file_path, pattern, path)` (P010 4-arg) | No | No | `Decision { exit_code, blocked, reason }` |
| CLI wrapper | `architect_guard()` | Yes (`read_payload()`) | Yes (stderr) | `i32` (exit code) |
| Core | `block_env_edit_decide(file_path, notebook_path)` | No | No | `Decision` |
| CLI wrapper | `block_env_edit()` | Yes | Yes (stderr) | `i32` |
| Core | `block_unsafe_merge_decide(command)` | No | No (gh-shelling yes) | `Decision` |
| CLI wrapper | `block_unsafe_merge()` | Yes | Yes (stderr) | `i32` |
| Core | `render_banner()` | No | No | `String` (full banner) |
| CLI wrapper | `session_banner()` | No | Yes (stdout) | `i32` |

`Decision` struct (`src/io.rs`): `{ exit_code: i32, blocked: bool, reason: Option<String> }`. CLI wrappers map to: `eprintln!(reason)` if `Some` + return `exit_code`. MCP tools map to `DecisionOutput { blocked, exit_code, reason }` in JSON.

**CLI parity invariant:** the 81 pre-P006 tests pass unchanged — Decision-core refactor is mechanical (move logic, not change behavior).

**MCP server (`src/serve.rs`):**
- Struct `HooksServer { tool_router: ToolRouter<Self> }` with `#[tool_router]` macro on the impl block.
- 5 `#[tool]` methods (sync): 4 direct hook wrappers (P006) + 1 composite router `why_blocked` (P007).
- `run() -> i32`: builds `tokio::runtime::Builder::new_current_thread().enable_all()` runtime, `block_on(HooksServer::new().serve(transport::stdio()).await?.waiting().await)`. Returns `ALLOW` (0) always.
- rmcp features used: `server`, `transport-io`, `macros`. `#[tool_router]` registers tool methods; `#[tool_handler] impl ServerHandler` wires tool routing + overrides `get_info()` (P008: returns `name="claude-hooks"`, `version=env!("CARGO_PKG_VERSION")`). Previously `#[tool_router(server_handler)]` auto-generated `ServerHandler` using rmcp build env → name was "rmcp"; explicit impl corrects this.

**5 MCP tools:**

| Tool | Input | Output | Behavior |
|---|---|---|---|
| `architect_guard` | `{ tool_name?, file_path?, pattern?, path? }` | `DecisionOutput` | Marker gate + tool_name dispatch + forbidden/allowed path check (real fs read). P010: +tool_name +path. |
| `block_env_edit` | `{ file_path?, notebook_path? }` | `DecisionOutput` | `.env*` check (not `.env.example`) |
| `block_unsafe_merge` | `{ command? }` | `DecisionOutput` | PR merge check (real gh shell calls, fail-CLOSED) |
| `session_banner` | `{}` | `{ banner: String }` | Full banner from fs/git state of serve env |
| `why_blocked` | `{ tool_name, tool_input? }` | `WhyBlockedOutput` | Composite router: routes by tool_name → fires matching hook (P007) |

**`why_blocked` composite router (P007):**

Accepts the full PreToolUse tool-call shape `{"tool_name":"Read","tool_input":{"file_path":"src/x.rs"}}` and routes to the matching hook based on `tool_name`, mirroring `.claude/settings.json` PreToolUse matchers. Returns `{ hook, blocked, exit_code, reason }`.

**Routing table (verbatim `.claude/settings.json` matchers):**

| `tool_name` | Hook fired | Decision fn called |
|---|---|---|
| `Read`, `Glob` | `architect_guard` | `architect_guard_decide(tool_name, file_path, pattern, path=None)` (P010: 4-arg) |
| `Edit`, `Write`, `MultiEdit`, `NotebookEdit` | `block_env_edit` | `block_env_edit_decide(file_path, notebook_path)` |
| `Bash` | `block_unsafe_merge` | `block_unsafe_merge_decide(command)` |
| any other | `none` | — (returns `blocked=false, exit_code=0`) |

**Tension 3 / why_blocked limitation (P010):** In tarot deploy, `Write`/`Edit` fires BOTH `architect_guard` (Write/Edit allowlist check) AND `block_env_edit` (.env* check) as separate hooks. `why_blocked` routes `Edit|Write` to `block_env_edit` ONLY (bounded, P010 = option a). To debug architect_guard Write/Edit block via MCP, call `architect_guard` tool directly with `tool_name="Write"/"Edit"`. Multi-hook `why_blocked` routing = finding for future phiếu.

**State-honesty notes:**
- `architect_guard` route reads `.sos-state/architect-active` marker from the fs of the environment where `serve` runs. If serve runs outside an architect-active context (marker absent), `Read`/`Glob` will return ALLOW — this is correct and honest behavior. Callers should be aware of this when using `why_blocked` for debug.
- `block_unsafe_merge` route makes real `gh` shell calls and is fail-CLOSED. If the serve environment lacks `gh` auth or network access, `Bash + gh pr merge <N>` commands will return `blocked=true, reason="gh unavailable"`. This is intentional — honest, not fake ALLOW.
- `session_banner`/`render_banner` is NOT routed by `why_blocked` — banner is not a block/allow decision. Any tool_name that would map to banner falls through to the `none` branch (allowed).

**Output struct `WhyBlockedOutput`:** `{ hook: String, blocked: bool, exit_code: i32, reason: Option<String> }`. `hook` = `"architect_guard" | "block_env_edit" | "block_unsafe_merge" | "none"`.

**Transport + framing:** `transport::stdio()` → `(tokio::io::Stdin, tokio::io::Stdout)`. Framing: newline-delimited JSON (one JSON object per line). Client must send `initialize` → `notifications/initialized` → tool calls. Server exits when stdin closes (`waiting()` returns on transport close).

**Tokio runtime:** `new_current_thread().enable_all()`. Cargo.toml tokio features: `rt`, `macros`, `io-std` (no `time` or `rt-multi-thread` needed for rmcp stdio).

**Note:** `.mcp.json` wiring (registering server as MCP provider) is P009/smoke, not P006.

### stdin-JSON Harness (`src/io.rs`)

Claude Code PreToolUse hooks pass a JSON payload on stdin:

```json
{ "tool_name": "Read", "tool_input": { "file_path": "...", "pattern": "...", "path": "...", "notebook_path": "...", "command": "..." } }
```

`HookPayload` top-level fields (`#[serde(default)]`):
- `tool_name: Option<String>` — added P010. Used by `architect-guard` for tool_name dispatch. Real Claude Code payload always includes this; absent → default ALLOW in dispatch.

`ToolInput` fields (all `Option<String>`, `#[serde(default)]`):
- `file_path` — used by `architect-guard` (Read path, and Write/Edit path) and `block-env-edit`.
- `pattern` — used by `architect-guard` (Glob pattern candidate, Read/Glob branch).
- `path` — added P010. Used by `architect-guard` (Glob search root candidate, Read/Glob branch).
- `notebook_path` — used by `block-env-edit` (NotebookEdit fallback).
- `command` — added P004. Used by `block-unsafe-merge`. Bash-tool payload field.

The harness (`read_payload()`) reads all stdin and parses via serde_json.
**Fail-open semantics (HARD):** empty stdin / invalid JSON / missing fields → `HookPayload::default()` (all `Option` fields are `None`). No `unwrap()`/`expect()` panics on parse path. Mirrors `scripts/architect-guard.sh` and `scripts/block-env-edit.sh`.

### Exit-Code Convention

| Code | Meaning | Usage |
|---|---|---|
| `0` (`ALLOW`) | Allow action to proceed | All stubs + fail-open fallback |
| `2` (`BLOCK`) | Block action, reason on stderr | P002+ real hook logic |

Block reason is written to **stderr only** (not stdout). Constants `ALLOW` and `BLOCK` are defined in `src/io.rs`. All P001 stubs return `ALLOW`. `process::exit(code)` is called in `main`; hook functions return `i32` (not self-exit, enabling unit tests).

## Module Structure

```
src/
  main.rs        -- clap entry + dispatch (thin)
  io.rs          -- shared stdin harness + exit constants + Decision struct
  hooks/
    mod.rs       -- 4 hooks: *_decide() core + CLI wrapper fn (P006 refactor)
  serve.rs       -- MCP server: HooksServer + 5 #[tool] methods + tokio runtime (P007: +why_blocked)
tests/
  cli.rs         -- 81 integration tests (assert_cmd, CLI parity P002-P005)
  mcp_handshake.rs -- 12 tests: Decision-core unit + MCP handshake smoke (5-tool assert, P007 routing)
```

## MCP Surface

`serve` subcommand: stdio JSON-RPC server (rmcp 1.7). Exposes 5 MCP tools: 4 direct hook wrappers (P006) + `why_blocked` composite router (P007). See `serve` section above for full detail including routing table and state-honesty notes.

## Data Flow

```
Claude Code PreToolUse trigger
  -> claude-hooks <subcmd>  (stdin = JSON payload)
     -> clap parse subcommand
     -> dispatch to hook fn (CLI wrapper, -> i32)
        -> read_payload() [fail-open]  (not session-banner)
        -> *_decide(parsed_inputs) -> Decision  [P006 refactor]
           -> hook logic (marker gate + path match + gh-shell)
        -> eprintln!(reason) if Some  [CLI wrapper]
     -> process::exit(code)

MCP path (P006):
  Claude MCP client -> stdio JSON-RPC
     -> HooksServer.serve(transport::stdio())
     -> #[tool] method -> *_decide(params) -> Json<DecisionOutput>
     -> JSON-RPC response
```

`architect-guard` (P002): real logic. `block-env-edit` (P003): real logic. `block-unsafe-merge` (P004): real logic (gh-shelling, fail-CLOSED). `session-banner` (P005): real logic (render from fs/git state, no stdin, stdout, always exit 0). `serve` (P006): real MCP server (rmcp 1.7 stdio, 4 hook tools via Decision-core). `why_blocked` (P007): composite router tool — routes tool_name → matching `*_decide` per `.claude/settings.json` matchers. All hooks refactored to `*_decide + wrapper` pattern (P006). Phase 3 DONE (P006 + P007). Phase 4 (P008): `serverInfo.name` fixed ("rmcp" → "claude-hooks") via explicit `#[tool_handler] impl ServerHandler`; v0.8.0 publish-ready.

## Bash Reference (oracle)

Port doctrine: 1:1 from `scripts/` canonical copies. Do not redesign behavior.

| Rust subcmd | Bash oracle |
|---|---|
| `architect-guard` | `scripts/architect-guard.sh` |
| `block-env-edit` | `scripts/block-env-edit.sh` |
| `block-unsafe-merge` | `scripts/block-unsafe-merge.sh` |
| `session-banner` | `scripts/session-start-banner.sh` |
