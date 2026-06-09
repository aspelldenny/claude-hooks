# BACKLOG — claude-hooks

> **Single source of truth cho "làm gì tiếp trên claude-hooks".**
> Live tracker. SessionStart hook surfaces Active sprint mỗi session mới. Pick item hoặc `/idea` để capture.
> **Vision:** `docs/PROJECT.md` (Rust binary thay ~418 dòng Bash hook, dual CLI + MCP). **Doctrine:** `~/sos-kit/docs/WORKFLOW_V2.2.md`.
>
> **Architect Rule 0:** chỉ viết phiếu cho item trong **Active sprint** (hoặc Sếp explicit promote từ Next). Không phiếu cho Open backlog / Park.

---

## 🔥 Active sprint: Phase 4 — Ship

> **Mục tiêu:** đóng gói + wire bản Rust vào tarot, thay 4 Bash hook gốc.
> **Kết thúc khi:** README/ARCHITECTURE polish + `cargo publish` dry-run clean + tarot chạy 1h session smoke clean với binary.
> **Promote:** 2026-06-09 (Phase 1+2+3 đã ship P001–P007, parity-verified — xem Done bên dưới).

- [ ] **[P008]** README + ARCHITECTURE polish + `cargo publish` dry-run. Gồm: fix `serverInfo.name` "rmcp"→"claude-hooks" (MCP get_info default), README usage (CLI 5 subcmd + MCP 5 tools), `cargo publish --dry-run` clean.
- [ ] **[P009]** Wire tarot — replace `tarot/scripts/{architect-guard,block-env-edit,block-unsafe-merge,session-start-banner}.sh` bằng `claude-hooks <subcmd>` trong `tarot/.claude/settings.json`. 1h session smoke clean.

---

## ✅ Phase 1+2+3 DONE — 4 hook port + MCP server (P001–P007)

> **Shipped 2026-06-09 (1 session, end-to-end).** 93/93 test, clippy clean, parity-verified vs Bash oracle. PR #1 (running build). Chi tiết từng phiếu → Recently shipped.

- [x] **Phase 1** — P001 scaffold CLI · P002 architect-guard · P003 block-env-edit. Parity 8/8 + 10/10 vs Bash.
- [x] **Phase 2** — P004 block-unsafe-merge (fail-CLOSED) · P005 session-banner (render, stdout byte-identical). 
- [x] **Phase 3** — P006 serve MCP (rmcp stdio + Decision-core refactor + 4 tool) · P007 why_blocked (5th composite tool, routing khớp settings.json). Real handshake verified.

---

## 💡 Open backlog (chưa thuộc sprint)

- [ ] **[SCOPE-DECISION — Sếp quyết]** **Hook canonical set đã mọc thêm từ vision (2026-05-28).** PROJECT.md nhắm **4 hook tarot** (architect-guard, block-env-edit, block-unsafe-merge, session-banner). Từ đó sos-kit canonical thêm: `orchestrator-guard.sh` · `no-code-on-default.sh` (P050) · `block-env-commit.sh` (P052) · `check-case-collision.sh` · pre-commit chain `[1/7]→[7/7]`. **Quyết định cần:** (a) Phase 1-4 giữ nguyên 4 hook gốc (MVP parity), hook mới = **Phase 5 follow-on** — em nghiêng cái này (giữ vision bounded, đừng nhắm bia di động); HAY (b) mở scope ngay = port cả bộ hiện tại. Bản Bash mới đã có sẵn trong `scripts/` (adopt copy) làm reference khi tới lượt.

---

## 🅿️ Park / nghĩ thêm

- [ ] Cross-platform Windows (git-bash) — Rust binary né được Bash shebang/`sed`/`python3` giòn. Grounding tươi: Sếp pull sos-kit sang Windows 2026-06-09. Có thể nâng priority nếu Windows thành môi trường thật.

---

## ✅ Recently shipped

- ✅ **[P007] why_blocked composite tool** (09/06/2026) — 5th MCP tool, routes tool_name→hook (verbatim settings.json), returns blocked/exit/reason. Commit `c5b3640`. Real handshake: Edit+.env.local→block_env_edit blocked. **→ Phase 3 DONE.**
- ✅ **[P006] serve MCP server** (09/06/2026) — rmcp 1.7 stdio + Decision-core refactor (4× `_decide` cores, CLI wrappers unchanged) + 4 hook tools. Real JSON-RPC handshake verified. 81 CLI tests unbroken.
- ✅ **[P005] session-banner port** (09/06/2026) — render hook: BACKLOG sprint block + doc size warn + cleanup nudge + advisory staleness + orchestrator contract. stdout always exit 0. Manual ISO→epoch (Hinnant). F-001 verbatim. 27 unit + 4 integration tests. **→ Phase 2 DONE.**
- ✅ **[P004] block-unsafe-merge port** (09/06/2026) — gh API + security-surface regex + APPROVE sentinel check. Fail-CLOSED divergence. 22 unit + 4 integration tests. Parity vs Bash.
- ✅ **[P002] architect-guard port** (09/06/2026) — marker gate + forbidden path set + .md allow. Commit `de05a9d`. Parity 8/8 vs Bash.
- ✅ **[P003] block-env-edit port** (09/06/2026) — regex `^\.env($|\.)` verbatim + .env.example allowlist + notebook fallback. Commit `42530a0`. Parity 10/10 vs Bash. **→ Phase 1 core DONE.**
- ✅ **[P001] Scaffold CLI** (09/06/2026) — clap derive 5-subcmd + stdin-JSON harness (`io.rs`, fail-open) + exit convention (0/2) + 8 verify-cò integration test. Commit `b216949`. Foundation cho P002–P006.
- ✅ **Kit adopt** (09/06/2026) — `sos adopt` hạ full sos-kit spine (agents + hooks + phieu + skills + gates) vào repo; `.sos-stack.toml` rust; `core.hooksPath=hooks` wired. Sẵn sàng code Phase 1.

---

## 📌 Quy tắc maintenance

1. Idea mới → `/idea` → Open backlog / Active sprint.
2. Phiếu xong → move xuống Recently shipped + ghi CHANGELOG.
3. Architect rule (cứng): không phiếu ngoài Active sprint.
4. Port doctrine: **trung thành Bash reference, KHÔNG redesign** (giống doc-rotate port caveat #2). Reference = `scripts/*.sh` đã copy.

---

*File LIVE. Sếp chỉnh trực tiếp. Architect/Worker chỉ ĐỌC khi đang viết phiếu.*
