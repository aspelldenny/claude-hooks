# SOS-Kit Workflow Feedback — từ pilot `claude-hooks`

> **Mục đích:** handoff findings cho upstream xử lý. Hai loại:
> - **F-001..F-003 — workflow/doctrine sos-kit** (phát hiện lúc build P001-P008): bê sang `~/sos-kit`.
> - **F-004+ — claude-hooks binary parity-gap** (phát hiện khi dogfood binary vào tarot, phiếu `~/tarot/docs/ticket/P344-adopt-claude-hooks.md`): bê vào `docs/BACKLOG.md` claude-hooks (phiếu fix mới). Đây là bug/gap của binary này, KHÔNG phải workflow.
> **Nguồn:** downstream repo `claude-hooks` (adopt sos-kit spine 2026-06-09) + dogfood tarot (2026-06-09). Quản đốc ghi live.
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

---

## F-004 — `architect-guard` binary thiếu Write/Edit branch — mất Write-guard khi swap

**Severity:** HIGH — security guard gap (silent failure)
**Phát hiện:** P344 CHALLENGE + EXECUTE (2026-06-09), tarot dogfood
**Loại:** Binary capability gap (binary thiếu feature bash oracle có).

**Triệu chứng:** Khi swap `bash scripts/architect-guard.sh` → `claude-hooks architect-guard`, Write/Edit to `src/`, `CLAUDE.md`, `docs/BACKEND_GUIDE.md`, `CHANGELOG.md` (tất cả ngoài allowlist) **đều exit 0** (ALLOW) thay vì exit 2 (BLOCK). Architect drift không bị chặn.

**Root cause:** `~/claude-hooks/src/hooks/mod.rs` hàm `architect_guard_decide` chỉ nhận `file_path` và pattern. Struct `ToolInput` (io.rs) KHÔNG có `tool_name` field. Không có Write/Edit branch. Binary chỉ guard `Read|Glob` (README:30 đúng).

**Bash oracle behavior (file `scripts/architect-guard.sh`):**
- `L54-61` `is_allowed_for_write()`: allowlist chỉ `docs/ticket/P*-*.md` (phiếu active)
- `L109-115` `case Write|Edit`: áp allowlist → block mọi file ngoài phiếu → exit 2
- `L80-94` `block_write()`: message "Architect không được sửa source/docs/CLAUDE.md"

**Runtime test (P344 CHALLENGE):**
- Marker `.sos-state/architect-active` present + `Edit CLAUDE.md` → **binary exit 0** (ALLOW — sai)
- Same + `bash scripts/architect-guard.sh` → **bash exit 2** (BLOCK — đúng)

**Đề xuất fix:** Thêm `tool_name: Option<String>` field vào `ToolInput` struct (io.rs). Trong `architect_guard_decide`: if `tool_name == "Write" | "Edit" | "MultiEdit" | "NotebookEdit"` → apply `is_allowed_for_write()` allowlist (chỉ `docs/ticket/P*-*.md`). Đây là Write-guard THẬT, không phải no-op.

**Impact for tarot P344:** `bash scripts/architect-guard.sh` GIỮ NGUYÊN cho hook này. Sẽ swap khi F-004 fix ở claude-hooks.

---

## F-005 — Thiếu cơ chế fail-CLOSED khi binary vắng PATH

**Severity:** HIGH — merge gate có thể fail-OPEN trên máy mới
**Phát hiện:** P344 EXECUTE (2026-06-09), anchor #15 analysis
**Loại:** Binary availability gap (fail-OPEN vs fail-CLOSED semantic mismatch).

**Triệu chứng:** Nếu `claude-hooks` binary không có trên PATH (máy mới chưa chạy `setup-dev.sh`), Claude Code gọi `claude-hooks block-unsafe-merge` → shell exit 127 (command not found). Claude Code treat exit 127 như exit 0 (allow) → `block-unsafe-merge` **fails OPEN** — PR merge không có SECURITY_REVIEW APPROVE vẫn lọt qua.

**Tác động:** `block-unsafe-merge` là fail-CLOSED gate (merge security review). Khi binary vắng → từ fail-CLOSED biến thành fail-OPEN silently. Fail mode hoàn toàn im lặng — không có warning, không có log.

**Đề xuất fix (2 option):**
- **(A) Shim presence-check:** `setup-dev.sh` cài shim `block-unsafe-merge` → shell wrapper mà:
  ```bash
  #!/usr/bin/env bash
  # Presence-check shim
  if ! command -v claude-hooks >/dev/null 2>&1; then
    echo "⛔ BLOCKED: claude-hooks binary not found. Run scripts/setup-dev.sh first." >&2
    exit 2  # fail-CLOSED
  fi
  exec claude-hooks block-unsafe-merge "$@"
  ```
  Shim đặt ở `scripts/block-unsafe-merge-shim.sh` → `.claude/settings.json` gọi shim thay vì binary trực tiếp.
- **(B) claude-hooks self-check PATH:** binary detect nếu mình bị gọi qua stale PATH → print warning + exit 2.
- **(C) document + setup-dev mitigation** (current tarot approach — giữ bash thay vì swap, vì bash luôn có sẵn). Acceptable nếu fail-CLOSED gate không swap sang binary.

**Impact for tarot P344:** Đây là lý do tarot giữ `bash scripts/block-unsafe-merge.sh` thay vì swap — bash không có FAIL-OPEN gap vì bash luôn có. Sẽ swap khi có cơ chế fail-CLOSED đảm bảo.

---

## F-006 (low) — `session-banner` doc-size check hardcode `docs/CHANGELOG.md`

**Severity:** Low — false-positive warning (không block, gây nhiễu)
**Phát hiện:** P344 EXECUTE (2026-06-09), tarot dogfood
**Loại:** Hardcode path assumption lệch repo layout.

**Triệu chứng:** `claude-hooks session-banner` in "doc size warning: docs/CHANGELOG.md (Xk > 40k threshold)". Trong tarot, CHANGELOG ở `docs/CHANGELOG.md` nên khớp. Nhưng nếu repo có CHANGELOG ở root (`CHANGELOG.md`) — lệch F-003 pattern — banner sẽ miss file thật và không warn. Session có false sense of security.

**Confirm tarot:** `docs/CHANGELOG.md` khớp hardcode path → warning fires đúng cho tarot. KHÔNG miss.

**Nhưng nếu repo khác dùng root `CHANGELOG.md`** (như `~/claude-hooks` chính nó — `CHANGELOG.md` ở root, không phải `docs/`): banner miss, không warn → false sense "doc nhỏ, ok".

**Đề xuất fix:** Đọc path từ `.doc-rotate.toml` `changelog_path` (nếu có) thay vì hardcode `docs/CHANGELOG.md`. Fallback: check cả `CHANGELOG.md` (root) + `docs/CHANGELOG.md`. Đồng nhất với F-003 logic.

---

## F-001 — Confirm từ tarot dogfood (P344)

**Status:** CONFIRMED — tarot trực tiếp gặp marker path lệch khi P344 CHALLENGE.

**Confirm detail:** tarot `architect-guard.sh:22` dùng `.claude/.architect-active`. Binary `mod.rs:13` dùng `.sos-state/architect-active`. Tarot xử lý bằng cách giữ bash (vì RISK-1 F-004 đã force giữ bash anyway), nên RISK-3 marker moot cho P344. Nhưng khi F-004 fix → swap architect-guard sang binary → marker path sẽ cần align. Đề xuất: binary docs đã nói `.sos-state/` → orchestrator contract tarot update touch `.sos-state/architect-active` (khi ready). F-001 root cause đúng như mô tả.

## F-004 — `architect-guard` THIẾU nhánh Write/Edit guard (parity-gap vs tarot deployed hook)

**Severity:** HIGH (silent loss of a real security guard nếu swap binary vào tarot)
**Phát hiện:** dogfood tarot P344 CHALLENGE (2026-06-09), Quản đốc verify trực tiếp
**Loại:** Binary parity-gap — port dựa trên oracle CŨ/NHỎ hơn bản tarot chạy thật.

**Triệu chứng:** binary `claude-hooks architect-guard` chỉ guard `Read|Glob` (chặn Architect ĐỌC source). Bản `architect-guard.sh` tarot đang deploy **còn guard cả `Write|Edit`** — chặn Architect GHI source/CLAUDE.md/guides, chỉ allowlist `docs/ticket/P*-*.md`.

**Evidence (verified 2026-06-09):**
- `~/tarot/scripts/architect-guard.sh` = **119 dòng**: có `is_allowed_for_write()` (L54-61, allowlist chỉ `docs/ticket/P*-*.md`, deny `TICKET_TEMPLATE.md`) + `case Write|Edit` (L109-115) gọi `block_write` exit 2.
- `~/claude-hooks/scripts/architect-guard.sh` (oracle em port từ) = **86 dòng**: CHỈ Read/Glob, KHÔNG có `tool_name`/Write/Edit branch.
- `src/hooks/mod.rs::architect_guard_decide(file_path, pattern)` — KHÔNG nhận `tool_name`, không có Write/Edit branch. `ToolInput` không có field `tool_name`.
- Runtime: marker present + `Edit CLAUDE.md` → **binary exit 0** (ALLOW); tarot bash → **exit 2** (BLOCK).

**Root cause:** parity em "verified 8/8" là so với oracle `scripts/architect-guard.sh` CỦA repo claude-hooks (86-dòng) — nhưng đó KHÔNG phải bản tarot deploy (119-dòng, giàu hơn). Scope-drift giữa 2 bản oracle (PROJECT.md đã ghi nhận "vision nhắm 4 hook tarot" nhưng copy oracle lúc adopt là bản cũ hơn).

**Đề xuất fix (claude-hooks BACKLOG — phiếu mới, gọi tạm P010):**
- Thêm `tool_name` vào `ToolInput` (io.rs) + nhánh Write/Edit trong `architect_guard_decide`: nếu `tool_name ∈ {Write,Edit,MultiEdit}` → áp `is_allowed_for_write` allowlist (chỉ `docs/ticket/P*-*.md`), else block exit 2.
- Port từ oracle TAROT (119-dòng) làm spec, KHÔNG oracle cũ. Cập nhật `scripts/architect-guard.sh` repo này cho khớp tarot (đồng bộ oracle).
- `why_blocked` routing đã map `Edit|Write|MultiEdit|NotebookEdit → block_env_edit` — sau fix cần thêm architect_guard cũng nhận Write/Edit (2 hook cùng fire trên Edit/Write ở tarot: architect-guard chặn Architect-ghi-source, block-env-edit chặn .env).

---

## F-005 — Marker path lệch: binary `.sos-state/architect-active` vs tarot bash `.claude/.architect-active`

**Severity:** Medium (architect-guard không fire đúng lúc nếu orchestrator touch sai path)
**Phát hiện:** dogfood tarot P344 (RISK-3), verified
**Loại:** Binary ⇄ deployed-env convention mismatch.

**Evidence:**
- `src/hooks/mod.rs` (+ `scripts/architect-guard.sh:25`): marker = `.sos-state/architect-active`.
- `~/tarot/scripts/architect-guard.sh:22`: `MARKER_FILE=".claude/.architect-active"`.
→ Orchestrator tarot `touch .claude/.architect-active` thì binary (tìm `.sos-state/`) KHÔNG thấy → guard không fire.

**Đề xuất fix:** chốt 1 path canonical (sos-kit dùng `.sos-state/` ở các hook khác → binary đúng hướng). Khi adopt tarot: hoặc update orchestrator contract tarot touch `.sos-state/architect-active`, hoặc binary đọc cả 2 path 1 nhịp transition. Quyết ở phiếu adopt (P344 side, resolution A/B/C).

---

## F-006 — FAIL-OPEN GAP: binary vắng PATH làm `block-unsafe-merge` (fail-CLOSED) lại fail-OPEN

**Severity:** Medium-High (gác merge-security câm lặng nếu máy mới chưa cài binary)
**Phát hiện:** dogfood tarot P344 (anchor #15), reasoning
**Loại:** Deployment failure-mode (không phải code logic).

**Triệu chứng:** `.claude/settings.json` gọi `claude-hooks block-unsafe-merge`. Nếu binary CHƯA cài (máy mới, `cargo install` chưa chạy) → shell `command not found` → **exit 127** ≠ 2 → Claude Code coi là non-block → ALLOW. Nghịch lý: `block-unsafe-merge` thiết kế fail-CLOSED (gh fail → block), nhưng binary-vắng-PATH lại fail-OPEN ở tầng ngoài (trước cả khi code chạy).

**Đề xuất fix (defense-in-depth):**
- (a) `scripts/setup-dev.sh` bootstrap ép `cargo install` (P344 Task 3 đã làm phía tarot).
- (b) Cân nhắc: settings.json wrap `command -v claude-hooks || { echo "BLOCKED: claude-hooks not installed" >&2; exit 2; }` cho riêng hook fail-CLOSED — nhưng đó là deployer concern, không phải binary. Document trong README "Install required; absent binary = merge-gate fails open."
- (c) README đã có Install section; thêm warning rõ về fail-open-when-absent cho block-unsafe-merge.

---

## F-007 — `is_allowed_for_write` path-traversal: allowlist bypass qua `../`

**Severity:** Low-Medium (defense-in-depth gap trong security guard; thực tế thấp vì Claude Code gửi path normalized)
**Phát hiện:** `/security-review` PR #2 (Giám sát, 2026-06-09) — advisory observation, KHÔNG phải INV flag (verdict APPROVE)
**Loại:** Security-hardening — diverge oracle (oracle cũng dính → fix phải đồng bộ 3 nơi).

**Triệu chứng:** `is_allowed_for_write` (Write/Edit allowlist của architect-guard, thêm ở P010) cho qua path có `../` traversal nếu nó khớp pattern phiếu:
- Input `docs/ticket/P010-x.md/../../CLAUDE.md` → bắt đầu `docs/ticket/P`, chứa `-`, kết `.md` → **allowlist PASS** → Architect ghi được `CLAUDE.md` trá hình phiếu.

**Evidence:** `src/hooks/mod.rs::is_allowed_for_write` — check `strip_prefix("docs/ticket/P")` + `contains('-')` + `ends_with(".md")`, KHÔNG reject `..`. **Faithful với bash oracle** (`case docs/ticket/P*-*.md)` glob cũng match path traversal) → cả `~/claude-hooks/scripts/architect-guard.sh` + `~/tarot/scripts/architect-guard.sh` đều dính.

**Mitigating:** Claude Code PreToolUse gửi path đã normalize trong thực tế → khả năng thấp. Nhưng đây là 1 GÁC — defense-in-depth đáng làm.

**Đề xuất fix (phiếu hardening MỚI — gọi tạm P011, ĐỒNG BỘ 3 nơi):**
- `is_allowed_for_write`: deny-fast nếu `p.contains("..")` TRƯỚC khi check allowlist (binary).
- Đồng bộ guard `..` vào `scripts/architect-guard.sh` (claude-hooks oracle) + feed tarot update `~/tarot/scripts/architect-guard.sh`.
- Cân nhắc áp luôn cho `is_forbidden_for_read` (path traversal đọc source cũng nên chặn) — Sếp/Architect quyết scope.
- ⚠️ Đây là divergence-có-chủ-đích khỏi oracle hiện tại (hardening) — KHÔNG phải port faithful; cần ghi rõ trong phiếu là "improve oracle, sync downstream".

---

## ✅ F-004 — DOGFOOD CONFIRMED → CLOSE (tarot P345, 2026-06-09)

**Status:** RESOLVED — binary `architect-guard` Write/Edit guard (P010 fix, commit `86446f1`) verified live tại tarot. **claude-hooks có thể ĐÓNG F-004.**

**Evidence (Quản đốc tarot, 2026-06-09):**
- **Parity 9/9** vs bash oracle: Edit `src/lib/auth/x.ts`→2 · Write `src/`→2 · Edit `docs/ticket/P999-x.md`→0 · Edit `TICKET_TEMPLATE.md`→2 (deny) · Edit `CLAUDE.md`→2 · Read `src/`→2 · Read `docs/`→0 · Glob `src/**`→2 · Read `prisma/schema.prisma`→2.
- **Live-fire 5/5** đúng command `.claude/settings.json` gọi (`claude-hooks architect-guard`).
- **Subagent propagation CONFIRMED** — đây là điểm quyết định: spawn subagent (Explore) với marker `.sos-state/architect-active` SET → subagent Read `src/middleware.ts` bị chặn với message `🚫 Architect envelope violation (Read/Glob)`. PreToolUse hook FIRE thật trên subagent → gác vai architect KHÔNG phải sân khấu.
- tarot `.claude/settings.json` đã swap `bash scripts/architect-guard.sh` → `claude-hooks architect-guard` (tarot P345, PR #630).

## ✅ F-005 — RESOLVED consumer-side (tarot P345)

Marker path `.sos-state/architect-active` (binary) vs `.claude/.architect-active` (bash cũ): tarot orchestrator contract đã dùng `.sos-state/` sẵn → **bash guard tarot ĐANG CHẾT** (đọc `.claude/.architect-active` không ai tạo → 100% allow). Swap sang binary = sửa gác chết. **Convention `.sos-state/` của binary là ĐÚNG**; tarot aligned. F-005 đóng được.

## 📌 Version label — đề nghị bump

`Cargo.toml = 0.8.0` nhưng đã chứa F-004 (commit `86446f1` trong HEAD). Handoff/brief gọi "v0.9.0" → **bump version tag** để consumer pin đúng (tarot reinstall thấy `--version` vẫn 0.8.0, dễ tưởng chưa có F-004).

## ⏳ F-006 — vẫn mở (advisory, low-pri)

tarot mitigate fail-OPEN bằng `scripts/setup-dev.sh` (ép `cargo install`). Đề nghị claude-hooks **document pattern fail-CLOSED wrapper** (`command -v claude-hooks || exit 2`) cho hook stakes cao (vd block-unsafe-merge) — để downstream chọn swap an toàn. tarot hiện GIỮ block-unsafe-merge bash vì lý do này.

---

<!-- Quản đốc: append F-NNN khi gặp ma sát mới. F-001..003 = workflow sos-kit; F-004+ = binary parity-gap/hardening (dogfood + review). Format: triệu chứng → evidence (cite line) → đề xuất fix. -->
