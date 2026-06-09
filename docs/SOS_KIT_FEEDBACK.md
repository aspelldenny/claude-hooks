# SOS-Kit Workflow Feedback — từ pilot `claude-hooks`

> **Mục đích:** ghi lại lỗi / mâu thuẫn / ma sát của **workflow sos-kit** (KHÔNG phải bug code claude-hooks) phát hiện trong lúc build, để bê sang `~/sos-kit` xử lý.
> **Nguồn:** downstream repo `claude-hooks` (adopt sos-kit spine 2026-06-09). Quản đốc (main session) ghi live.
> **Doctrine ref:** `~/sos-kit/docs/WORKFLOW_V2.2.md`.

---

## F-001 — Mâu thuẫn marker `worker-active`: contract nói RM, hook đòi TOUCH

**Severity:** Medium (block được Worker, nhưng Worker tự workaround được)
**Phát hiện:** P001 EXECUTE (2026-06-09)
**Loại:** Contract ⇄ hook mismatch (doc sai vs behavior).

**Triệu chứng:** `orchestrator-guard` PreToolUse hook **chặn Write ĐẦU TIÊN của Worker** vì thiếu marker `.sos-state/worker-active`. Worker phải tự `touch` marker giữa chừng mới code tiếp được (theo chính error message của `orchestrator-guard.sh`).

**Root cause (chính xác hoá sau khi đọc hook):** `orchestrator-guard.sh:24` **TỰ document đúng protocol**:
> "Quản đốc PHẢI `touch .sos-state/worker-active` TRƯỚC spawn Thợ, `rm -f` sau khi Thợ về"

→ Hook KHÔNG sai. Vấn đề là **SessionStart banner orchestrator contract** (artifact load vào context Quản đốc MỖI session) **thiếu bước này**. Banner chỉ ghi:
> "Marker: touch .sos-state/architect-active trước spawn architect; **rm -f trước spawn worker**."

Banner dạy *xoá architect-active* trước khi spawn worker, nhưng **KHÔNG** nhắc *touch worker-active*. Quản đốc đọc banner (không đọc `orchestrator-guard.sh:24`) → quên touch → Worker bị chặn Write đầu tiên. **Lệch giữa contract-được-surface (banner) và protocol-thật (hook comment).**

**Đề xuất fix (sos-kit side):** đồng bộ `touch/rm worker-active` (đối xứng architect-active) vào MỌI nơi surface orchestrator contract:
- `hooks/session-start-banner.sh` (đoạn "🤖 Orchestrator contract") — thêm dòng marker worker-active.
- `agents/orchestrator.md` (condensed contract Quản đốc đọc).
- `docs/ORCHESTRATION.md` (spec đầy đủ).
- Lý do chọn fix banner thay vì để Worker tự touch: giữ Quản đốc là **single owner** của marker lifecycle (đối xứng + không cho subagent tự cấp quyền write cho chính nó = giữ envelope rõ ràng).

---

## F-002 — Phiếu lifecycle tooling (`phieu` / `.phieu-counter`) không bootstrap khi adopt

**Severity:** Low (manual workaround dễ)
**Phát hiện:** P001 (2026-06-09)
**Loại:** Adopt gap (tooling thiếu sau `sos adopt`).

**Triệu chứng:** Template `phieu/TICKET_TEMPLATE.md` giả định phiếu được tạo qua shell function `phieu <slug>` (auto-fill ID từ `<project>/.phieu-counter` + tạo branch + worktree + move active→done qua `phieu-done`). Nhưng repo adopt KHÔNG có `.phieu-counter`, và `phieu`/`phieu-done` function không có sẵn trong session → toàn bộ vòng đời phiếu (đặt ID, ghi file, move sang `done/`) phải làm TAY.

**Hệ quả:** Worker commit code nhưng **để phiếu untracked** (`docs/ticket/P001-scaffold-cli.md` không vào git). Quản đốc phải tự `mv` sang `done/` + commit riêng. Không có cơ chế tự động.

**Đề xuất fix (sos-kit side):** `sos adopt` nên (a) seed `.phieu-counter` (start 000), (b) cài/symlink `phieu`+`phieu-done` shell function (hoặc tài liệu hoá rõ "downstream phải tự source"), HOẶC (c) ghi rõ trong `agents/worker.md` rằng Worker phải `git add` phiếu file vào commit của mình.

---

## F-003 — pre-commit [3/7] hardcode sai path `docs/CHANGELOG.md` (false-positive warning)

**Severity:** Low (chỉ warning vàng, KHÔNG block commit — nhưng gây nhiễu + xói lòng tin vào gate)
**Phát hiện:** P002 EXECUTE (2026-06-09), Thợ surface
**Loại:** Hook bug (path assumption lệch repo layout).

**Triệu chứng:** Mỗi commit có "code + phiếu thay đổi", pre-commit `[3/7] sos-kit v2 checks` in:
> ⚠️ Code + phiếu changed but DISCOVERIES.md or CHANGELOG.md not staged

dù CẢ HAI đã staged đúng. False positive.

**Root cause:** `hooks/pre-commit:190`:
```sh
CHANGELOG_STAGED=$(git diff --cached --name-only --diff-filter=AM | grep -E '^docs/CHANGELOG\.md$' || true)
```
Grep tìm `docs/CHANGELOG.md`, nhưng repo này (và CLAUDE.md DOCS GATE) để CHANGELOG ở **root** (`CHANGELOG.md`). Path không bao giờ khớp → `CHANGELOG_STAGED` luôn rỗng → nhánh `[ -z ... ]` (L191) luôn true → warning. Nghịch lý: chính `[2/7] docs-gate` (Rust binary) lại check CHANGELOG ở root đúng → 2 gate trong cùng chain bất đồng về vị trí CHANGELOG.

**Đề xuất fix (sos-kit side):**
- Sửa `hooks/pre-commit:190` cho khớp layout thật: hoặc `grep -E '^(docs/)?CHANGELOG\.md$'` (chấp nhận cả 2), hoặc đọc path canonical từ `.docs-gate.toml` (đã có `ticket_dir` — thêm `changelog_path`) thay vì hardcode.
- Đồng bộ giả định CHANGELOG location giữa `[2/7]` (docs-gate binary, root) và `[3/7]` (shell check, `docs/`) — hiện 2 gate lệch nhau.

---

<!-- Quản đốc: append F-NNN khi gặp ma sát workflow mới. Format: triệu chứng → mâu thuẫn → đề xuất fix sos-kit. -->
