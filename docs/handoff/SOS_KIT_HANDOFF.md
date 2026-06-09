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

# 🧭 DECISION cho sos-kit — doctrine adopt binary `claude-hooks` vào downstream

> **Đây là quyết định của sos-kit** (nó sở hữu adoption doctrine cho MỌI downstream, tarot chỉ là repo đầu). claude-hooks (binary) đã xong + sẽ KHÔNG đổi cho việc này. Phần wrapper/distribution sống ở **tầng adopt của sos-kit**, không phải trong binary. Dưới đây là toàn cảnh + option để sos-kit cân, KHÔNG chốt sẵn.

## Bối cảnh
- `claude-hooks` = 4 hook (architect-guard, block-env-edit, block-unsafe-merge, session-banner), Rust, parity-verified vs bash oracle, v0.9.0, shippable.
- Cài 1 lần (`cargo install`) → **binary độc lập, chạy KHÔNG cần Rust** (Rust chỉ cần lúc build).
- claude-hooks repo đã **dogfood**: 4 hook của nó giờ chạy bằng binary (settings.json). `orchestrator-guard` GIỮ bash (không phải hook claude-hooks).

## Vấn đề cốt lõi cần doctrine (F-006)
"Binary có thể VẮNG trên PATH" (máy mới chưa chạy install) chia 2 loại hook:
- **3 hook fail-OPEN** (architect-guard, block-env-edit, session-banner): vắng → exit 127 → harness allow → mà đằng nào chúng cũng fail-open theo thiết kế → **KHÔNG thêm rủi ro** → wire binary trực tiếp OK.
- **1 hook fail-CLOSED** (block-unsafe-merge, gác merge security): vắng → exit 127 → harness allow → **gác bảo mật mở toang IM LẶNG**. Đây là case nguy hiểm DUY NHẤT cần doctrine.

## Option space (sos-kit chọn pattern canonical cho fail-CLOSED hook)
| # | Pattern | Ưu | Nhược |
|---|---|---|---|
| 1 | **Bash-fallback wrapper** — ưu tiên binary, vắng thì chạy bash oracle (luôn có) | Gác **vẫn làm việc** khi vắng binary; không fail-open; không phiền | Phải **nuôi bash parity** + bash fallback **mục âm thầm nếu không test** (bài học F-005 — gác bash tarot từng chết) |
| 2 | **Fail-closed shim** — `command -v claude-hooks \|\| exit 2` | Đơn giản, an toàn mặc định, không nuôi bash | **Chặn cả merge hợp lệ** tới khi cài binary (phiền) |
| 3 | **Prebuilt binary** (GitHub Releases / Homebrew / cargo-binstall) | Máy consumer **KHÔNG cần Rust** — tải file chạy về; bỏ ma sát "phải compile" | Vẫn có thể vắng nếu chưa tải; nên ghép với (1)/(2) |
| 4 | **setup-dev enforce** — 1 lệnh bootstrap cài binary mỗi máy | Đảm bảo có mặt | Không cứu nếu **ai đó quên chạy** → ghép (1)/(2) làm dây an toàn |

## Khung gợi ý (để sos-kit cân — KHÔNG phải lệnh)
- **3 hook fail-open:** wire binary trực tiếp. Xong.
- **1 hook fail-closed:** cần pattern an toàn-khi-vắng. (1) wrapper giữ gác sống; (2) shim đơn giản hơn. sos-kit chọn theo: có muốn nuôi bash dài hạn không.
- **Distribution:** cân nhắc mạnh **option 3 (prebuilt)** — "bắt mọi máy cài Rust" mới là rào cản adopt thật, không phải cái wrapper. Prebuilt gỡ rào đó.
- Dù chọn gì: **bash oracle phải parity-synced với binary** (sos-kit ship cả 2 nếu giữ fallback) — Rust thêm feature (P010 Write/Edit, P011 `..`) thì bash phải theo, không thì fallback hành xử khác âm thầm.

## Liên hệ findings khác
- F-004/F-005 đã CLOSED (tarot P345 live). F-007 (path-traversal `..`) = hardening low-pri, sync 3 nơi khi làm (binary + 2 oracle) — nếu sos-kit giữ bash fallback, nhớ áp `..`-guard cho cả bash.
- README claude-hooks (`## Exit convention` → Deployment caveat) có mô tả vấn đề + 2 pattern tham khảo, nhưng **quyết định canonical là ở đây (sos-kit)**.

---

*3 workflow finding (F-001/002/003) + decision deployment trên đều REPRODUCIBLE / grounded ở claude-hooks. Evidence + line refs đầy đủ: `docs/SOS_KIT_FEEDBACK.md`.*
