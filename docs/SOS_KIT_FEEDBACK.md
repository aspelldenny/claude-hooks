# Feedback ledger — pilot `claude-hooks` + dogfood tarot

> **Mục đích:** sổ findings handoff. Hai loại:
> - **F-001..F-003 — workflow/doctrine sos-kit** (build P001-P008) → bê sang `~/sos-kit`. (Bản handoff sạch: `docs/handoff/SOS_KIT_HANDOFF.md`.)
> - **F-004..F-007 — binary parity-gap / hardening** (dogfood tarot P344/P345 + `/security-review`) → phiếu fix trong claude-hooks BACKLOG.
> **Nguồn:** claude-hooks (adopt 2026-06-09) + dogfood tarot (2026-06-09).
> **⚠️ Numbering:** file này từng bị 2 nguồn (claude-hooks Quản đốc + tarot) đánh trùng F-004/005/006. **Đã reconcile về 1 scheme canonical bên dưới** — xem §Reconciliation cuối file cho mapping số tarot cũ.

---

## 📊 Status table (canonical)

| ID | Tóm tắt | Loại | Severity | Status |
|---|---|---|---|---|
| F-001 | Banner orchestrator-contract thiếu `touch worker-active` | workflow → sos-kit | Medium | 🔴 OPEN (sos-kit) |
| F-002 | Phiếu tooling (`phieu`/`.phieu-counter`) không bootstrap khi adopt | workflow → sos-kit | Low | 🔴 OPEN (sos-kit) |
| F-003 | pre-commit `[3/7]` hardcode sai path `docs/CHANGELOG.md` | workflow → sos-kit | Low | 🔴 OPEN (sos-kit) |
| F-004 | `architect-guard` thiếu Write/Edit branch (vs tarot 119-dòng) | binary parity-gap | HIGH | ✅ **CLOSED** (P010 `86446f1` + tarot P345 live-confirm) |
| F-005 | Marker path `.sos-state/` (binary) vs `.claude/` (tarot bash cũ) | binary↔env convention | Medium | ✅ **CLOSED** (tarot aligned `.sos-state/`; bash guard cũ đã chết) |
| F-006 | Binary vắng PATH → `block-unsafe-merge` (fail-CLOSED) lại fail-OPEN | deployment failure-mode | Medium-High | 🟡 OPEN — mitigated tarot-side; cần README pattern (đang làm) |
| F-007 | `is_allowed_for_write` path-traversal (`../` bypass allowlist) | security-hardening | Low-Medium | 🔴 OPEN (P011 candidate, sync 3 nơi) |

> **Non-issue (investigated, KHÔNG cấp số):** tarot từng lo `session-banner` doc-size hardcode `docs/CHANGELOG.md` bỏ sót root `CHANGELOG.md`. **Verified KHÔNG đúng** — binary check cả 3 path (`docs/CHANGELOG.md`, `docs/DISCOVERIES.md`, `CHANGELOG.md` root) tại `src/hooks/mod.rs:801-803` (faithful oracle L78). Không miss.

---

## F-001 — Banner orchestrator-contract THIẾU bước `touch worker-active`

**Severity:** Medium · **Loại:** doc(banner) ⇄ hook mismatch · **Phát hiện:** P001 EXECUTE.

`orchestrator-guard` chặn Write ĐẦU TIÊN của Worker vì thiếu `.sos-state/worker-active`. Root cause: `orchestrator-guard.sh:24` document đúng protocol (*"Quản đốc PHẢI touch worker-active trước spawn Thợ, rm sau"*), NHƯNG SessionStart banner (artifact Quản đốc đọc mỗi session) chỉ ghi *"touch architect-active... rm trước spawn worker"* — thiếu touch worker-active. Lệch contract-surface (banner) vs protocol-thật (hook comment).

**Fix (sos-kit):** đồng bộ `touch/rm worker-active` vào `hooks/session-start-banner.sh` (block "🤖 Orchestrator contract") + `agents/orchestrator.md` + `docs/ORCHESTRATION.md`. Giữ Quản đốc là single owner marker lifecycle.

## F-002 — Phiếu tooling không bootstrap khi `sos adopt`

**Severity:** Low · **Loại:** adopt gap · **Phát hiện:** P001.

`TICKET_TEMPLATE.md` giả định `phieu`/`phieu-done` shell function + `.phieu-counter`. Repo adopt không có → vòng đời phiếu (ID, ghi, archive) làm tay; Worker dễ để phiếu untracked.

**Fix (sos-kit):** `sos adopt` seed `.phieu-counter` + cài/symlink `phieu`/`phieu-done` (hoặc document "downstream tự source"); và/hoặc `agents/worker.md` dặn Worker `git add` phiếu vào commit.

## F-003 — pre-commit `[3/7]` hardcode sai path `docs/CHANGELOG.md`

**Severity:** Low (warning, không block) · **Loại:** hook bug · **Phát hiện:** P002.

`hooks/pre-commit:190` grep `^docs/CHANGELOG\.md$`, nhưng CHANGELOG ở **root** → luôn rỗng → warning "CHANGELOG not staged" mỗi commit dù đã stage. `[2/7]` (root) vs `[3/7]` (docs/) bất đồng vị trí.

**Fix (sos-kit):** `grep -E '^(docs/)?CHANGELOG\.md$'` hoặc đọc `changelog_path` từ `.docs-gate.toml`.

---

## F-004 — `architect-guard` thiếu Write/Edit branch ✅ CLOSED

**Severity:** HIGH · **Loại:** binary parity-gap · **Phát hiện:** dogfood tarot P344 CHALLENGE.

**Triệu chứng (lúc mở):** binary `architect-guard` chỉ guard `Read|Glob`. Tarot deploy bản 119-dòng còn guard `Write|Edit` (`is_allowed_for_write` allowlist `docs/ticket/P*-*.md`, deny `TICKET_TEMPLATE.md`). Swap → mất Write-guard im lặng (Edit CLAUDE.md → binary exit 0 vs bash exit 2).

**Root cause:** parity "8/8" verified vs oracle CỦA claude-hooks (86-dòng), KHÔNG phải bản tarot deploy (119-dòng). Scope-drift oracle.

**✅ FIXED — P010** (`86446f1`): thêm `tool_name` dispatch + nhánh Write/Edit + Read/Glob superset (prisma/sql/path) + 2 message + sync `scripts/architect-guard.sh` repo này. Parity 12/12 vs tarot bash (Quản đốc claude-hooks).

**✅ CONFIRMED LIVE — tarot P345 (PR #630):** tarot swap `bash architect-guard.sh` → `claude-hooks architect-guard`. Parity 9/9 + live-fire 5/5. **Subagent propagation confirmed** — Explore subagent Read `src/middleware.ts` (marker set) → chặn thật với `🚫 Architect envelope violation (Read/Glob)`. Gác fire trên subagent ở production.

## F-005 — Marker path `.sos-state/` vs tarot `.claude/.architect-active` ✅ CLOSED

**Severity:** Medium · **Loại:** binary↔env convention · **Phát hiện:** dogfood tarot P344 (RISK-3).

Binary đọc `.sos-state/architect-active` (mod.rs + scripts/architect-guard.sh:25); tarot bash cũ đọc `.claude/.architect-active` (tarot:22).

**✅ RESOLVED consumer-side — tarot P345:** tarot orchestrator contract ĐÃ dùng `.sos-state/` sẵn → **bash guard tarot ĐANG CHẾT** (đọc `.claude/.architect-active` không ai tạo → 100% allow). Swap sang binary = **sửa 1 gác đã chết**. Convention `.sos-state/` của binary ĐÚNG; tarot aligned. (Binary-side không cần đổi gì.)

## F-006 — Binary vắng PATH → `block-unsafe-merge` fail-OPEN 🟡 OPEN

**Severity:** Medium-High · **Loại:** deployment failure-mode · **Phát hiện:** P344 anchor #15.

`.claude/settings.json` gọi `claude-hooks block-unsafe-merge`. Binary chưa cài (máy mới) → shell exit 127 → Claude Code coi non-block → ALLOW. Nghịch lý: hook thiết kế fail-CLOSED, nhưng vắng-PATH lại fail-OPEN ở tầng ngoài, im lặng.

**Trạng thái:** tarot mitigate bằng `scripts/setup-dev.sh` (ép `cargo install`) + GIỮ `block-unsafe-merge` bash (bash luôn có) thay vì swap. Xin claude-hooks **document pattern fail-CLOSED wrapper**.

**Fix (đang làm — README):** document pattern shim cho hook stakes cao:
```bash
command -v claude-hooks >/dev/null 2>&1 || { echo "BLOCKED: claude-hooks not installed" >&2; exit 2; }
exec claude-hooks block-unsafe-merge "$@"
```
→ downstream wrap hook fail-CLOSED qua shim, hoặc giữ bash tới khi shim sẵn. (Binary self-check PATH = option phụ, nhưng binary-vắng thì chính nó không chạy được để self-check → shim/wrapper là đúng tầng.)

## F-007 — `is_allowed_for_write` path-traversal (`../` bypass) 🔴 OPEN

**Severity:** Low-Medium · **Loại:** security-hardening (diverge oracle) · **Phát hiện:** `/security-review` PR #2 (Giám sát, advisory — verdict vẫn APPROVE).

`docs/ticket/P010-x.md/../../CLAUDE.md` → bắt đầu `docs/ticket/P`, chứa `-`, kết `.md` → **allowlist PASS** → Architect ghi CLAUDE.md trá hình phiếu. **Faithful bash oracle** (`case docs/ticket/P*-*.md` cũng dính) → claude-hooks binary + `scripts/architect-guard.sh` + tarot oracle ĐỀU dính. Thực tế thấp (Claude Code gửi path normalized) nhưng là 1 gác.

**Fix (P011 — hardening, sync 3 nơi):** deny-fast `p.contains("..")` TRƯỚC allowlist trong `is_allowed_for_write` (binary) + `scripts/architect-guard.sh` + feed tarot. Cân nhắc áp luôn `is_forbidden_for_read`. ⚠️ Divergence-có-chủ-đích khỏi oracle (improve, không phải port faithful).

---

## 📌 Reconciliation — mapping số tarot cũ → canonical

File này từng có 2 scheme. Mapping để reference tarot cũ không dangling:
- tarot **F-004** (architect-guard Write/Edit) = canonical **F-004** ✅ (trùng nghĩa).
- tarot **F-005** (fail-CLOSED binary vắng PATH) = canonical **F-006**.
- tarot **F-006** (session-banner doc-size hardcode) = **non-issue** (verified, binary check root CHANGELOG đúng — xem note §Status table).
- claude-hooks **F-005** (marker path) = canonical **F-005** (giữ).
- claude-hooks **F-006** (fail-open) = canonical **F-006** (trùng tarot F-005, gộp).
- claude-hooks **F-007** (path-traversal) = canonical **F-007** (giữ).

## 📌 Version label (resolved)

`Cargo.toml` từng = 0.8.0 dù HEAD chứa P010/F-004. **FIXED** `e9013ee`: bump → **0.9.0**. `claude-hooks --version` = 0.9.0; consumer pin/detect F-004 đúng.

---

<!-- Append F-NNN theo canonical scheme. F-001..003 workflow→sos-kit; F-004+ binary. Format: triệu chứng → evidence (line) → fix. Đừng đánh trùng số — check Status table trước. -->
