# CLAUDE.md — claude-hooks

> Đọc file này TRƯỚC khi làm bất cứ gì.
> `docs/PROJECT.md` = vision (cứng, scope IN/OUT). `docs/BACKLOG.md` = sprint hiện tại. `docs/ARCHITECTURE.md` = surface map. `CHANGELOG.md` = đã làm gì.
> **Workflow doctrine (single-source):** `~/sos-kit/docs/WORKFLOW_V2.2.md`. Repo này là downstream sos-kit (kit hạ vào qua `sos adopt` 2026-06-09).

## Vai trò repo này

Rust binary `claude-hooks` thay **~418 dòng Bash hot-path hook** (`architect-guard` · `block-env-edit` · `block-unsafe-merge` · `session-banner`) + 1 MCP `serve` (`why_blocked` debug). Dual mode **CLI** (Claude Code hook gọi) + **MCP** (Sếp/Quản đốc gọi debug). Xem `docs/PROJECT.md` Scope cứng.

## Port doctrine (CỨNG — KHÔNG quên giữa build)

1. **Trung thành, KHÔNG redesign.** Mỗi subcmd port 1:1 từ bản Bash trong `scripts/` (adopt đã copy bản canonical). Cùng exit code + stderr message. Cấm "tiện tay cải tiến logic" — đồng bộ/maintainability, không phải đổi hành vi.
2. **Bash reference = oracle.** `scripts/{architect-guard,block-env-edit,block-unsafe-merge,session-start-banner}.sh` là spec. CLI parity = thắng (PROJECT.md Success #1).
3. **Verify-cò (P057).** Phiếu sinh subcmd mới → BẮT BUỘC kèm fire-test fixtures (exit code + stderr) trong cùng phiếu. Build cò ≠ cò sống.
4. **Scope bounded.** Hook canonical sos-kit đã mọc thêm (orchestrator-guard, no-code-on-default, block-env-commit, check-case-collision) — đó là **Phase 5 follow-on**, KHÔNG kéo vào Phase 1-4 (xem BACKLOG Open backlog SCOPE-DECISION). Đừng nhắm bia di động.

## Stack

- Rust edition 2024, MSRV 1.85 · clap 4 derive · serde/serde_json · regex · tokio (chỉ MCP) · rmcp 1.7 · anyhow/thiserror · assert_cmd/predicates (CLI test).
- Build: `cargo build` · Test: `cargo test --all` · Lint: `cargo clippy -- -D warnings`.
- `.sos-stack.toml` = rust (manifest Cargo.toml, lock Cargo.lock).

## Layer model (3-role + Quản đốc)

Quản đốc (main session, điều phối) → Kiến trúc sư (phiếu, docs-only) → Thợ (code). Specialist: Trinh sát (`/advisory-scan`), Giám sát (`/security-review`). State machine: DRAFT → CHALLENGE → [RESPOND ⇄ CHALLENGE] → APPROVAL_GATE → EXECUTE. Handbook: `.claude/agents/*.md`.

## Hook chain (pre-commit — adopt copy bản canonical sos-kit)

`hooks/pre-commit` `[1/7]`→`[7/7]`: type-check (cargo) · docs-gate · BACKLOG/Discovery · security-gate · case-collision · no-code-on-default · block-env-commit. `core.hooksPath=hooks` đã wire. PreToolUse guards trong `.claude/settings.json`.

## ⛔ DOCS GATE Tầng 1 — code change BẮT BUỘC update docs

| Code change | Target doc |
|---|---|
| Subcmd add/đổi behavior (`src/`) | `docs/ARCHITECTURE.md` + `CHANGELOG.md` + (nếu đổi CLI contract) `README.md` |
| Hook logic đổi (exit code / block reason) | `CHANGELOG.md` + fixture test (Tầng 1 — đây là security-surface) |
| MCP tool add (`serve`) | `docs/ARCHITECTURE.md` MCP section + `.mcp.json` nếu schema đổi |
| Cargo dep add | `CHANGELOG.md` (advisory-scan rescans `.sos-stack.toml`) |

Security-surface (hook = chặn/cho action) → AUTO Tầng 1. Worker ghi "Tầng 1 docs updated: <list>" trong Discovery.

## Definition of Done

1. Code chạy — `cargo build` + `cargo test` clean, `cargo clippy` không warning.
2. Không file rác / `dbg!` / unused.
3. CHANGELOG entry cho task.
4. Docs liên quan updated (bảng trên).
5. Fire-test fixtures cho subcmd mới PASS (P057).

Thiếu bước nào = CHƯA XONG.

## Language

- Nói tiếng Việt với Sếp (em / anh). Code comment + commit message tiếng Anh.
- Public docs (README, PROJECT) English-friendly (repo có thể `cargo publish`).
