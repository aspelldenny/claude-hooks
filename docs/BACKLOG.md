# BACKLOG — claude-hooks

> **Single source of truth cho "làm gì tiếp trên claude-hooks".**
> Live tracker. SessionStart hook surfaces Active sprint mỗi session mới. Pick item hoặc `/idea` để capture.
> **Vision:** `docs/PROJECT.md` (Rust binary thay ~418 dòng Bash hook, dual CLI + MCP). **Doctrine:** `~/sos-kit/docs/WORKFLOW_V2.2.md`.
>
> **Architect Rule 0:** chỉ viết phiếu cho item trong **Active sprint** (hoặc Sếp explicit promote từ Next). Không phiếu cho Open backlog / Park.

---

## ✅ Phase 4 — Ship — DONE (2026-06-09)

> **Mục tiêu (đạt):** đóng gói + wire bản Rust vào tarot, thay 4 Bash hook gốc.
> **Kết quả:** P008 ship-prep ✅ · P010 architect-guard true parity ✅ · P009 wire-tarot ✅ (superseded by tarot P345 — 3/4 hook swapped live). cargo publish dry-run clean, version 0.9.0. **Dự án claude-hooks: shippable + adopted.**
> Open hardening còn lại (low-pri, không gấp): **P011** (F-007 path-traversal) — xem Open backlog.

- [x] **[P008]** ✅ README + ARCHITECTURE polish + ship prep. → shipped `20b455f`: serverInfo "rmcp"→"claude-hooks" (explicit ServerHandler), README usable, version 0.1.0→0.8.0, package 778KB→148KB, `cargo publish --dry-run` clean. 93/93 test.
- [x] **[P010]** ✅ `architect-guard` TRUE parity tarot (fix **F-004**) — tool_name dispatch + Write/Edit guard (allowlist `docs/ticket/P*-*.md`, deny `TICKET_TEMPLATE.md`) + Read/Glob superset (prisma/sql/path) + 2 message + sync oracle. → shipped `86446f1` (104 test, **parity 12/12 vs tarot bash oracle**). Marker GIỮ `.sos-state/` (F-005 defer).
- [x] **[P009]** ✅ CLOSED — **superseded bởi tarot P345** (PR #630). Việc wire do tarot tự sở hữu, không phải claude-hooks chạy. Live: tarot swap **3/4 hook** → binary (`session-banner`, `architect-guard`, `block-env-edit`); `block-unsafe-merge` CỐ Ý giữ bash (F-006 fail-open vắng-PATH — chờ shim, pattern đã có README). claude-hooks không còn việc cho P009.

---

## ✅ Phase 1+2+3 DONE — 4 hook port + MCP server (P001–P007)

> **Shipped 2026-06-09 (1 session, end-to-end).** 93/93 test, clippy clean, parity-verified vs Bash oracle. PR #1 (running build). Chi tiết từng phiếu → Recently shipped.

- [x] **Phase 1** — P001 scaffold CLI · P002 architect-guard · P003 block-env-edit. Parity 8/8 + 10/10 vs Bash.
- [x] **Phase 2** — P004 block-unsafe-merge (fail-CLOSED) · P005 session-banner (render, stdout byte-identical). 
- [x] **Phase 3** — P006 serve MCP (rmcp stdio + Decision-core refactor + 4 tool) · P007 why_blocked (5th composite tool, routing khớp settings.json). Real handshake verified.

---

## 💡 Open backlog (chưa thuộc sprint)

- [ ] **[P011]** Hardening F-007 — `is_allowed_for_write` (+cân nhắc `is_forbidden_for_read`) deny-fast path chứa `..` TRƯỚC allowlist. Low-pri (xác suất thật thấp — Claude Code gửi path normalized; KHÔNG có gì đang gãy). Divergence-có-chủ-đích khỏi oracle (improve, không port faithful) → sync 3 nơi: binary `is_allowed_for_write` + `scripts/architect-guard.sh` + feed tarot oracle. ~30 phút.
- [ ] **[P012]** **Trust gate port từ thanhtra v1.2** (12/06/2026 — Sếp duyệt ghi backlog, "để anh tính tiếp"). Threat model: binary này chạy BÊN TRONG session Claude Code của user trên mỗi tool call → release/PR bị tráo = code độc trong agent của mọi người cài. Immutable releases đã bật (12/06) chặn nhánh tráo-release; gate này chặn nhánh PR-content ("Rules File Backdoor" class — payload prose/Unicode vô hình, không phải code). **Oracle = thanhtra v1.2:** `thanhtra/core/trust.py` (3 lớp deterministic: hidden-unicode Tags U+E0000–E007F/zero-width/bidi · auto-exec configs · injection-marker; regex không bị prompt-inject) + `scripts/validate-trust.py` self-test + self-scan + baseline `tests/trust-baseline.json` + SECURITY.md invariants + workflow gate. **Port notes, KHÔNG copy nguyên xi:** (a) repo này ship `.claude/settings.json` + `.mcp.json` + `hooks/` CÓ CHỦ ĐÍCH → auto-exec check phải chuyển sang baseline-diff (đổi hooks = FAIL tới khi review) thay vì hard-fail như thanhtra; (b) U+FEFF literal `phieu/DISCOVERY_PROTOCOL.md:196` (quote kỹ thuật BOM) sẽ trip hidden-unicode → escape trước khi bật gate; (c) SECURITY.md threat model riêng (hooks được phép làm gì, không bao giờ fetch URL runtime, không giấu output khỏi user). Làm SAU sos-kit port (cùng item bên đó) rồi copy pattern — 2 repo share phieu/ sync. ~1 session.
- [ ] **[SCOPE-DECISION — Sếp quyết]** **Hook canonical set đã mọc thêm từ vision (2026-05-28).** PROJECT.md nhắm **4 hook tarot** (architect-guard, block-env-edit, block-unsafe-merge, session-banner). Từ đó sos-kit canonical thêm: `orchestrator-guard.sh` · `no-code-on-default.sh` (P050) · `block-env-commit.sh` (P052) · `check-case-collision.sh` · pre-commit chain `[1/7]→[7/7]`. **Quyết định cần:** (a) Phase 1-4 giữ nguyên 4 hook gốc (MVP parity), hook mới = **Phase 5 follow-on** — em nghiêng cái này (giữ vision bounded, đừng nhắm bia di động); HAY (b) mở scope ngay = port cả bộ hiện tại. Bản Bash mới đã có sẵn trong `scripts/` (adopt copy) làm reference khi tới lượt.

---

## 🅿️ Park / nghĩ thêm

- [ ] Cross-platform Windows (git-bash) — Rust binary né được Bash shebang/`sed`/`python3` giòn. Grounding tươi: Sếp pull sos-kit sang Windows 2026-06-09. Có thể nâng priority nếu Windows thành môi trường thật.

---

## ✅ Recently shipped

- ✅ **[P010] architect-guard TRUE parity tarot** (09/06/2026) — fix F-004 (dogfood): tool_name dispatch + Write/Edit guard + prisma/sql superset + sync oracle. Commit `86446f1`. Parity 12/12 vs tarot 119-line bash. Binary giờ thay được architect-guard tarot.
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
