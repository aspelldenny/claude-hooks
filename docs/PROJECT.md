# PROJECT — claude-hooks

> **Status:** Bootstrap (2026-05-28). Cargo skeleton + vision only.
> **Full setup deferred:** post-pilot retrospective of `advisory-inbox` (Workflow v2.1 pilot, ETA ~3 days). At that point: port v2.1 doctrine + agents + skills symlink + CI workflow from sos-kit golden template (which itself updates from pilot lessons).
> **Workflow v2.1 spec:** `~/sos-kit/docs/WORKFLOW_V2.1.md` (durable doctrine).

---

## Vision (1 câu)

Rust binary thay **4 Bash hot-path hook** (~418 dòng) trong `tarot/.claude/settings.json` PreToolUse + SessionStart wiring — `architect-guard`, `block-env-edit`, `block-unsafe-merge`, `session-banner` — dual mode **CLI** (Claude Code hook gọi command) + **MCP** (em hoặc Sếp gọi `mcp__claude_hooks__why_blocked` debug).

---

## Why this exists

Tarot hiện có 4 Bash hook hot path fire MỌI Claude Code session turn:

| File | LOC | Trigger | Việc |
|------|-----|---------|------|
| `scripts/architect-guard.sh` | 119 | PreToolUse Read/Glob/Write/Edit | Block Architect ngoài envelope (sed regex JSON parse, no jq cross-platform) |
| `scripts/block-env-edit.sh` | 54 | PreToolUse Edit/Write | Block edit `.env*` (allow `.env.example`) |
| `scripts/block-unsafe-merge.sh` | 137 | PreToolUse Bash | Block `gh pr merge` không có `/security-review APPROVE` |
| `scripts/session-start-banner.sh` | 108 | SessionStart | Show sprint + advisory staleness + runtime state preflight |

**~418 dòng Bash + sed regex** = LLM phải đọc mỗi lần debug hook chặn lý do gì. Bash quote escape + sed cross-platform compat fragile. JSON parse qua Python inline → 5-line `python3 -c "import json..."`.

Rust binary `claude-hooks` replace toàn bộ với:
- Deterministic JSON parse (`serde_json`)
- Compile-time error catch
- 1 subcmd test = `cargo test` (vs Bash echo + grep manual)
- MCP mode: em (Claude Code orchestrator) call `mcp__claude_hooks__why_blocked --tool-call <json>` để debug lý do hook chặn, KHÔNG phải đọc 100-dòng Bash sed regex.

---

## Scope cứng

### IN scope (Phase 1-3)

**5 subcommand:**

1. **`architect-guard`** — read JSON tool_input từ stdin, check architect-active marker, exit 0/2 với reason
2. **`block-env-edit`** — block `.env*` Edit/Write (allow `.env.example` + `.runtime-env.allowlist`)
3. **`block-unsafe-merge`** — block `gh pr merge <N>` chưa có security review APPROVE comment
4. **`session-banner`** — render SessionStart banner (sprint + advisory + runtime preflight)
5. **`serve`** — MCP server stdio JSON-RPC, expose 4 above + 1 composite `why_blocked` (Sếp debug hook reason)

### OUT scope (NOT building)

- Multi-language hook support (Python/Ruby/etc.) — Bash hooks chỉ chạy 1 language Sếp dùng
- Plugin architecture cho custom hooks — 1 binary 1 job
- Web UI dashboard config — `.claude/settings.json` đủ
- Auto-fix violations — block + return reason, KHÔNG silent fix
- Telemetry — KHÔNG ship usage data

---

## Success criteria

1. **CLI parity:** mỗi subcmd cho cùng exit code + stderr message như Bash counterpart cho 100% test fixtures
2. **MCP mode:** `serve` expose 5 tools, JSON-RPC handshake clean, schema validate input
3. **Test:** `cargo test --all` ≥ 30 tests pass (1 test per hook × 5-7 case mỗi cái)
4. **Binary size:** `< 5 MB` release build (strip + lto)
5. **Performance:** hook fire < 50ms cold start (vs Bash + python3 invocation ~200-400ms hiện tại)
6. **Tarot smoke test:** install vào tarot, replace 4 Bash file, session 1h chạy clean

---

## Tech Stack

- Rust edition 2024, MSRV 1.85
- clap 4.x derive
- serde + serde_json (parse hook JSON từ stdin)
- regex (sed regex equivalent — pattern matching)
- tokio (chỉ cho MCP `serve`)
- rmcp 1.7.0 (MCP server)
- anyhow + thiserror
- assert_cmd + predicates (CLI integration tests)

---

## Roadmap (placeholder — refine post-pilot retrospective)

### Phase 1 — Hook scaffold + core 2

- P001 scaffold CLI (clap derive, 5 subcmd registered)
- P002 `architect-guard` subcmd (parse marker file + path forbidden check)
- P003 `block-env-edit` subcmd (pattern allowlist)

### Phase 2 — Advanced 2

- P004 `block-unsafe-merge` subcmd (gh pr diff capture + security surface regex + APPROVE check)
- P005 `session-banner` subcmd (sprint + advisory + runtime preflight)

### Phase 3 — MCP

- P006 `serve` subcmd (rmcp stdio + 5 tools)
- P007 `why_blocked` composite tool (debug helper)

### Phase 4 — Ship

- P008 README + ARCHITECTURE polish + `cargo publish`
- P009 install in tarot — replace 4 Bash scripts trong `tarot/scripts/`

---

## Constraints

- KHÔNG fetch network (hooks chạy local, no telemetry)
- KHÔNG depend OS-specific (cross-platform macOS + Linux)
- KHÔNG break `.claude/settings.json` hook config schema
- KHÔNG silent fix violations — fail loud với reason

---

## Notes for future session resume

- Bootstrap commit này CHỈ ship Cargo skeleton + this PROJECT.md
- Full Workflow v2.1 doctrine port (CLAUDE.md + docs/RULES.md + .claude/agents + .tools/runtime-env.allowlist + .github/workflows/ci.yml + hooks scripts + skills symlink) sẽ ship **POST-PILOT** sau khi `advisory-inbox` ship Phase 1-4 xong + retrospective forge v2.2 nếu cần
- Em hoặc Sếp KHÔNG bắt đầu phiếu code trong repo này TRƯỚC khi pilot xong

---

## Resume brief from tarot orchestrator (2026-06-05)

**Gate ĐÃ MỞ:** pilot `advisory-inbox` ship Phase 1-4 (CHANGELOG P012) + retrospective viết xong (`~/sos-kit/docs/retro/WORKFLOW_V2.2_RETRO_advisory-inbox.md` + v2.3). Điều kiện "đợi pilot + retro" đã thoả → repo này **bắt đầu code được** với doctrine v2.2/v2.3.

**NHƯNG không gấp (Quản đốc + Sếp chốt 2026-06-05):** 4 Bash hook nó định thay đang **CHẠY TỐT** — `block-unsafe-merge` chặn đúng PR #603 (security-surface chưa review), `session-banner` cảnh báo advisory stale. Đây là **refactor đồng bộ/maintainability**, KHÔNG fix đau hiện tại → ưu tiên thấp hơn Tier 0 (cho tool đã ship chạy) + Tier 1 (bash bootstrap installer) + Tier 1.5 (adopt-poisoned-repo flow). claude-hooks nằm **Tier 2**, build khi gom "bộ kit hoàn chỉnh".

**Wiring tarot (Phase 4):** replace `tarot/scripts/{architect-guard,block-env-edit,block-unsafe-merge,session-start-banner}.sh`. Brief tarot-side: `~/tarot/docs/BACKLOG.md §🦀 Rust toolchain`.
