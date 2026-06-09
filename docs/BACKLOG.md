# BACKLOG — claude-hooks

> **Single source of truth cho "làm gì tiếp trên claude-hooks".**
> Live tracker. SessionStart hook surfaces Active sprint mỗi session mới. Pick item hoặc `/idea` để capture.
> **Vision:** `docs/PROJECT.md` (Rust binary thay ~418 dòng Bash hook, dual CLI + MCP). **Doctrine:** `~/sos-kit/docs/WORKFLOW_V2.2.md`.
>
> **Architect Rule 0:** chỉ viết phiếu cho item trong **Active sprint** (hoặc Sếp explicit promote từ Next). Không phiếu cho Open backlog / Park.

---

## 🔥 Active sprint: Phase 1 — Scaffold + core 2 hook

> **Mục tiêu:** dựng CLI 5-subcmd (clap derive) + port 2 hook đơn giản nhất (`architect-guard`, `block-env-edit`) đạt CLI parity với bản Bash.
> **Kết thúc khi:** `cargo build` clean + `architect-guard` & `block-env-edit` cho cùng exit code + stderr như Bash counterpart trên 100% test fixtures (PROJECT.md Success #1).
> **Started:** 09/06/2026
> **Reference Bash (đã copy vào `scripts/` khi adopt — port từ đây, KHÔNG bịa logic):** `scripts/architect-guard.sh` · `scripts/block-env-edit.sh`.

- [ ] **[P001]** Scaffold CLI — `clap` derive, 5 subcmd registered (`architect-guard` · `block-env-edit` · `block-unsafe-merge` · `session-banner` · `serve`), stdin JSON parse harness (`serde_json`), exit-code convention (0 allow / 2 block) + stderr reason. **Verify-cò (P057 spirit):** mỗi subcmd stub trả exit hợp lệ + 1 integration test `assert_cmd` xác nhận CLI dispatch nổ.
- [ ] **[P002]** `architect-guard` subcmd — port `scripts/architect-guard.sh` (119 dòng): parse `tool_input.file_path` từ stdin JSON, check `.sos-state/architect-active` marker, block product-code Read/Glob khi architect active, exit 0/2 + reason. **Fire-test:** fixture set (architect-active + product file → exit 2; doc file → exit 0; no marker → exit 0).
- [ ] **[P003]** `block-env-edit` subcmd — port `scripts/block-env-edit.sh` (54 dòng): block `.env*` Edit/Write, allow `.env.example`, regex `^\.env($|\.)` verbatim. **Fire-test:** `.env` → 2, `.env.example` → 0, `.envrc` → 0 (mirror sos-kit P052 [O1.1] decision).

---

## 🎯 Next sprint: Phase 2 — Advanced 2 hook

> **Trigger:** Phase 1 xong (CLI scaffold + 2 hook parity).
> **Theme:** port 2 hook phức tạp (gh API + render).

- [ ] **[P004]** `block-unsafe-merge` subcmd — port `scripts/block-unsafe-merge.sh` (137 dòng): `gh pr diff` capture + security-surface regex + APPROVE sentinel check. Lane: Tầng 1 (security-surface). Reference: `block-unsafe-merge.sh:102-106` sentinel grep.
- [ ] **[P005]** `session-banner` subcmd — port `scripts/session-start-banner.sh` (108 dòng): render sprint + advisory staleness + runtime preflight cho SessionStart.

---

## 🌊 Future waves (cam kết low)

- [ ] **Phase 3 — MCP** (`serve` + `why_blocked`)
  - [ ] **[P006]** `serve` subcmd — `rmcp` stdio JSON-RPC, expose 4 hook above as MCP tools.
  - [ ] **[P007]** `why_blocked` composite tool — Sếp/Quản đốc gọi `mcp__claude_hooks__why_blocked --tool-call <json>` để debug lý do hook chặn (thay vì đọc Bash sed).
- [ ] **Phase 4 — Ship**
  - [ ] **[P008]** README + ARCHITECTURE polish + `cargo publish`.
  - [ ] **[P009]** Wire tarot — replace `tarot/scripts/{architect-guard,block-env-edit,block-unsafe-merge,session-start-banner}.sh`. 1h session smoke clean.

---

## 💡 Open backlog (chưa thuộc sprint)

- [ ] **[SCOPE-DECISION — Sếp quyết]** **Hook canonical set đã mọc thêm từ vision (2026-05-28).** PROJECT.md nhắm **4 hook tarot** (architect-guard, block-env-edit, block-unsafe-merge, session-banner). Từ đó sos-kit canonical thêm: `orchestrator-guard.sh` · `no-code-on-default.sh` (P050) · `block-env-commit.sh` (P052) · `check-case-collision.sh` · pre-commit chain `[1/7]→[7/7]`. **Quyết định cần:** (a) Phase 1-4 giữ nguyên 4 hook gốc (MVP parity), hook mới = **Phase 5 follow-on** — em nghiêng cái này (giữ vision bounded, đừng nhắm bia di động); HAY (b) mở scope ngay = port cả bộ hiện tại. Bản Bash mới đã có sẵn trong `scripts/` (adopt copy) làm reference khi tới lượt.

---

## 🅿️ Park / nghĩ thêm

- [ ] Cross-platform Windows (git-bash) — Rust binary né được Bash shebang/`sed`/`python3` giòn. Grounding tươi: Sếp pull sos-kit sang Windows 2026-06-09. Có thể nâng priority nếu Windows thành môi trường thật.

---

## ✅ Recently shipped

- ✅ **Kit adopt** (09/06/2026) — `sos adopt` hạ full sos-kit spine (agents + hooks + phieu + skills + gates) vào repo; `.sos-stack.toml` rust; `core.hooksPath=hooks` wired. Sẵn sàng code Phase 1.

---

## 📌 Quy tắc maintenance

1. Idea mới → `/idea` → Open backlog / Active sprint.
2. Phiếu xong → move xuống Recently shipped + ghi CHANGELOG.
3. Architect rule (cứng): không phiếu ngoài Active sprint.
4. Port doctrine: **trung thành Bash reference, KHÔNG redesign** (giống doc-rotate port caveat #2). Reference = `scripts/*.sh` đã copy.

---

*File LIVE. Sếp chỉnh trực tiếp. Architect/Worker chỉ ĐỌC khi đang viết phiếu.*
