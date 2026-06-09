# Handoff → sos-kit — workflow/doctrine findings từ pilot claude-hooks

> **Nguồn:** repo `claude-hooks` (downstream, adopt sos-kit spine 2026-06-09). Phát hiện khi build P001–P008 + dogfood.
> **Phạm vi file này:** CHỈ 3 finding thuộc **workflow/doctrine sos-kit** — sửa ở `~/sos-kit`, hạ xuống downstream qua sync.
> **KHÔNG thuộc file này:** F-004..F-007 (bug/hardening của binary `claude-hooks` + tarot oracle) → xem brief tarot riêng.
> **Bản đầy đủ (mọi finding, có evidence):** `docs/SOS_KIT_FEEDBACK.md` trong claude-hooks.

---

## F-001 — Banner orchestrator-contract THIẾU bước `touch worker-active`

**Severity:** Medium · **Loại:** doc ⇄ hook mismatch (contract surface lệch behavior thật).

**Vấn đề:** `orchestrator-guard.sh:24` (comment) tự document đúng protocol — *"Quản đốc PHẢI `touch .sos-state/worker-active` TRƯỚC spawn Thợ, `rm -f` sau"*. Nhưng **SessionStart banner** (`session-start-banner.sh` đoạn "🤖 Orchestrator contract") — artifact load vào context Quản đốc MỖI session — chỉ ghi:
> "Marker: touch .sos-state/architect-active trước spawn architect; rm -f trước spawn worker."

→ Banner dạy *rm architect-active* trước spawn worker, KHÔNG nhắc *touch worker-active*. Quản đốc đọc banner (không đọc comment trong hook) → quên touch → `orchestrator-guard` chặn **Write đầu tiên của Worker**. (Đã trúng thật ở claude-hooks P001 — Worker phải tự touch giữa chừng.)

**Fix (đồng bộ MỌI nơi surface contract):**
- `hooks/session-start-banner.sh` — thêm dòng marker worker-active vào block "🤖 Orchestrator contract".
- `agents/orchestrator.md` (condensed contract).
- `docs/ORCHESTRATION.md` (spec đầy đủ).
- Giữ Quản đốc là **single owner** của marker lifecycle (đối xứng architect-active: touch trước spawn, rm sau về).

---

## F-002 — Phiếu lifecycle tooling (`phieu` / `.phieu-counter`) không bootstrap khi `sos adopt`

**Severity:** Low · **Loại:** adopt gap (tooling thiếu sau adopt).

**Vấn đề:** `phieu/TICKET_TEMPLATE.md` giả định phiếu tạo qua shell function `phieu <slug>` (auto ID từ `<project>/.phieu-counter` + branch + worktree, `phieu-done` move active→done/). Repo adopt KHÔNG có `.phieu-counter`, function `phieu`/`phieu-done` không có trong session → cả vòng đời phiếu (đặt ID, ghi file, archive) làm TAY. Hệ quả: Worker dễ để phiếu untracked (claude-hooks P001 gặp — phải `git add` phiếu tay).

**Fix:** `sos adopt` nên (a) seed `.phieu-counter` (start 000); (b) cài/symlink `phieu`+`phieu-done` (hoặc tài liệu hoá rõ "downstream tự source"); và/hoặc (c) ghi trong `agents/worker.md` rằng Worker phải `git add` phiếu file vào commit của mình.

---

## F-003 — pre-commit `[3/7]` hardcode sai path `docs/CHANGELOG.md` (false-positive)

**Severity:** Low (chỉ warning vàng, KHÔNG block — nhưng nhiễu + xói lòng tin gate). · **Loại:** hook bug (path assumption lệch layout).

**Vấn đề:** `hooks/pre-commit:190`:
```sh
CHANGELOG_STAGED=$(git diff --cached --name-only --diff-filter=AM | grep -E '^docs/CHANGELOG\.md$' || true)
```
Grep tìm `docs/CHANGELOG.md`, nhưng repo này (và CLAUDE.md DOCS GATE) để CHANGELOG ở **root** (`CHANGELOG.md`). → luôn rỗng → warning "CHANGELOG.md not staged" mỗi commit có code+phiếu, dù đã stage. Nghịch lý: `[2/7] docs-gate` (Rust binary) check CHANGELOG ở root ĐÚNG → 2 gate trong cùng chain bất đồng vị trí CHANGELOG.

**Fix:**
- Sửa `hooks/pre-commit:190` → `grep -E '^(docs/)?CHANGELOG\.md$'` (chấp nhận cả 2), HOẶC đọc path canonical từ `.docs-gate.toml` (đã có `ticket_dir` — thêm `changelog_path`) thay vì hardcode.
- Đồng bộ giả định CHANGELOG location giữa `[2/7]` (root) và `[3/7]` (`docs/`).

---

*3 finding trên đều REPRODUCIBLE ở claude-hooks. Evidence chi tiết + line refs: `docs/SOS_KIT_FEEDBACK.md`.*
