# PHIẾU P010: `architect-guard` TRUE parity tarot (fix F-004)

> **ID format:** P010. **Filename:** `docs/ticket/P010-architect-guard-true-parity.md`.
> **Branch:** `feat/P010-architect-guard-write-guard` (Quản đốc đã tạo + checkout). Snapshot `.backup/P010/`.

---

> **Loại:** Bugfix (capability gap — security guard)
> **Ưu tiên:** P1
> **Tầng:** 1 (security boundary — architect-guard = hook chặn/cho action; sai thì Architect drift không bị chặn = LAN tới mọi session. AUTO Tầng 1 dù diff nhỏ, theo CLAUDE.md "Security-surface → AUTO Tầng 1").
> **Ảnh hưởng:** `src/io.rs` (ToolInput + HookPayload), `src/hooks/mod.rs` (architect_guard_decide + wrapper), `src/serve.rs` (GuardInput + architect_guard MCP tool), `tests/cli.rs` (update test cũ + thêm Write/Edit cases), `tests/mcp_handshake.rs` (unit core), `scripts/architect-guard.sh` (đồng bộ oracle 119-dòng), `docs/ARCHITECTURE.md`, `CHANGELOG.md`, `docs/discoveries/P010.md`.
> **Dependency:** None (P009 wire-tarot blocked-on chính P010 này — P010 xong trước).

---

## Context

### Vấn đề hiện tại

F-004 (dogfood tarot P344, severity HIGH): binary `claude-hooks architect-guard` port từ oracle CŨ 86-dòng (`~/claude-hooks/scripts/architect-guard.sh`) — **chỉ guard `Read|Glob`**. Nhưng bản đang **deploy thật** ở tarot là oracle 119-dòng giàu hơn, **còn guard cả `Write|Edit`** (chặn Architect GHI source/CLAUDE.md/guides, chỉ allowlist `docs/ticket/P*-*.md`).

Runtime evidence (F-004 verified 2026-06-09): marker `.sos-state/architect-active` present + `Edit CLAUDE.md` → **binary exit 0 (ALLOW — sai)**; tarot bash → **exit 2 (BLOCK — đúng)**. Swap binary vào tarot = mất Write-guard câm lặng (silent loss of security guard).

### Giải pháp

Port TRUNG THÀNH bản **oracle TAROT 119-dòng** (`~/tarot/scripts/architect-guard.sh`) — đây là SPEC duy nhất, KHÔNG dùng bản cũ 86-dòng. 4 khác biệt phải đóng:

1. **tool_name dispatch** — đọc top-level `tool_name` (oracle L31, NGOÀI tool_input), `case`: `Read|Glob` vs `Write|Edit`. No-match → allow (default).
2. **Read/Glob branch superset** — check 3 path: `file_path`, `pattern`, **`path`** (Glob search root, oracle L36/99); `.md` early-allow (L103); `is_forbidden_for_read` THÊM `prisma/*|*/prisma/*` + `*.prisma|*.sql` (L43,47) so bản cũ.
3. **Write/Edit branch (MỚI, L109-115)** — no path → allow; `is_allowed_for_write` (L54-61): allow `docs/ticket/P*-*.md|*/docs/ticket/P*-*.md`, **deny `docs/ticket/TICKET_TEMPLATE.md`** (explicit, ưu tiên trước allow), else block_write.
4. **2 block message** — block_read (L66-76) cho Read/Glob, block_write (L83-92) cho Write/Edit, verbatim oracle.

**Divergence GIỮ NGUYÊN (không phải bug — port có chủ đích):**
- **Marker path:** GIỮ `.sos-state/architect-active` (binary convention, đã port P002). Oracle tarot dùng `.claude/.architect-active` (L22). KHÔNG đổi ở P010 — đây là F-005 marker-path, phiếu riêng, defer. Port = behavior parity (logic dispatch + allowlist + message), marker path GIỮ convention claude-hooks.
- **MultiEdit/NotebookEdit:** oracle tarot `case` CHỈ `Read|Glob` và `Write|Edit` (KHÔNG MultiEdit/NotebookEdit). Port faithful = dispatch chỉ 4 tool đó. `MultiEdit`/`NotebookEdit` → no-match → default allow (architect-guard không guard; chúng vẫn bị `block-env-edit` route khác fire). Quyết: **faithful** (không thêm MultiEdit vào allowlist branch) — xem Tension 4.

### Scope

- CHỈ F-004: tool_name dispatch + Read/Glob superset (prisma/sql/path) + Write/Edit branch + 2 message.
- KHÔNG đổi marker path (F-005 defer). KHÔNG đổi `why_blocked` routing core (Tension 3 = option a, bounded). KHÔNG đụng 3 hook khác.

### Skills consulted

<!-- None. -->

---

## Task 0 — Verification Anchors

> Architect KHÔNG Read được `src/*.rs` (envelope). Mọi anchor code-level = `[needs Worker verify]` — Worker grep tại EXECUTE trước khi sửa. Oracle tarot `[verified]` (Architect đã Read `~/tarot/scripts/architect-guard.sh` — .sh repo khác, guard không chặn).

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Oracle tarot 119-dòng: `tool_name` top-level L31, `is_forbidden_for_read` L39-50 (+prisma +sql), `is_allowed_for_write` L54-61 (deny TICKET_TEMPLATE first, allow P*-*.md), `block_read` L63-78, `block_write` L80-94, `case` L96-116 chỉ `Read\|Glob` + `Write\|Edit` | Architect Read `~/tarot/scripts/architect-guard.sh` | ✅ [verified] toàn bộ — đây là SPEC |
| 2 | `ToolInput` (io.rs) hiện có `file_path, pattern, notebook_path, command` — THIẾU `path` | `grep -n "struct ToolInput" -A8 src/io.rs` | ⏳ [needs Worker verify] — Quản đốc feed: thiếu `path` |
| 3 | `HookPayload` (io.rs) hiện CHỈ wrap `tool_input` — THIẾU top-level `tool_name` | `grep -n "struct HookPayload" -A4 src/io.rs` | ⏳ [needs Worker verify] — Quản đốc feed: thiếu `tool_name` |
| 4 | `architect_guard_decide(file_path, pattern) -> Decision` tại `src/hooks/mod.rs:6` (chỉ Read/Glob) | `grep -n "fn architect_guard_decide" src/hooks/mod.rs` | ⏳ [needs Worker verify] — Quản đốc feed mod.rs:6 |
| 5 | CLI wrapper `architect_guard()` tại `mod.rs:79` đọc payload → gọi `_decide(file_path, pattern)` | `grep -n "fn architect_guard\b" src/hooks/mod.rs` | ⏳ [needs Worker verify] — Quản đốc feed mod.rs:79 |
| 6 | MCP tool `architect_guard` nhận `GuardInput { file_path?, pattern? }` tại `src/serve.rs` | `grep -n "struct GuardInput\|fn architect_guard" src/serve.rs` | ⏳ [needs Worker verify] — Quản đốc feed |
| 7 | `why_blocked` route: `Read\|Glob → architect_guard_decide`; `Edit\|Write\|MultiEdit\|NotebookEdit → block_env_edit_decide`; `Bash → block_unsafe_merge` | `grep -n "tool_name\|match" src/serve.rs` (why_blocked fn) | ⏳ [needs Worker verify] — Quản đốc feed |
| 8 | Integration test `architect-guard` hiện feed `{"tool_input":{"file_path":"..."}}` KHÔNG có `tool_name`, expect exit 2 | `grep -n "architect-guard\|tool_input" tests/cli.rs` | ⏳ [needs Worker verify] — đếm + liệt kê test cần sửa (xem Tension 1) |
| 9 | `Decision` struct `{ exit_code, blocked, reason }` (io.rs) — pattern dùng cho `_decide` return | `grep -n "struct Decision" src/io.rs` | ⏳ [needs Worker verify] — ARCHITECTURE.md xác nhận |
| 10 | `ALLOW=0`, `BLOCK=2` constants (io.rs) | `grep -n "const ALLOW\|const BLOCK" src/io.rs` | ⏳ [needs Worker verify] |

**❌/⚠️ note:** Anchor 2,3 (THIẾU `path`/`tool_name`) là gap PHẢI thêm — không phải sai assumption. Worker xác nhận cấu trúc hiện tại rồi thêm field.

### Pre-phiếu snapshot (Worker auto first-step)

> Worker EXECUTE FIRST ACTION: snapshot rollback point. (Quản đốc đã có `.backup/P010/` — Worker xác nhận tồn tại, nếu thiếu thì tạo.)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

> Schema: 1 turn = Worker Challenge + Architect Response. Cap = 3.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Quản đốc Challenge (orchestrator verify code-level + oracle tarot)

**Anchor verification (✅):**
- Oracle tarot ✅ `~/tarot/scripts/architect-guard.sh` 119-dòng = SPEC (Architect Read được; em verify L22 marker, L31 tool_name, L39-61 is_forbidden_for_read/is_allowed_for_write, L96-116 case dispatch, L63-94 2 message)
- HookPayload ✅ `src/io.rs:24-27` — chỉ `tool_input`, THIẾU top-level `tool_name` (P010 thêm)
- GuardInput ✅ `src/serve.rs:16-19` — `{file_path, pattern}`, thêm `tool_name`+`path`
- architect_guard_decide ✅ `src/hooks/mod.rs:6` — `(file_path, pattern)`, refactor thêm tool_name dispatch
- **Tension-1 confirmed:** test cũ `tests/cli.rs:35,44,85,99,113,127,141,155` KHÔNG có `tool_name` → strict dispatch làm chúng đổi BLOCK→ALLOW → PHẢI thêm `"tool_name":"Read"`. Real Claude Code payload luôn có tool_name → đúng hơn.

**Objections (Tầng 1):** None. Port trung thành oracle tarot 119-dòng. Tension 1-4 resolution hợp lý (T1 update test + assert no-tool_name→ALLOW intended; T2 GuardInput mở rộng; T3 why_blocked bounded + note; T4 MultiEdit faithful). Marker GIỮ `.sos-state/` (F-004 only, F-005 defer) — ghi rõ.

**Lưu ý cho Worker:** claude-hooks OWN `.claude/settings.json` dùng bash + matcher `Read|Glob` cho architect-guard (repo này dev-hook dùng bash, không phải binary — KHÔNG cần rewire). Consumer wiring (matcher `Read|Glob|Write|Edit`) document qua README/ARCHITECTURE.

**Status:** ✅ ACCEPTED V1 — no challenges.

### Final consensus
- Phiếu version: V1 (no debate turn needed)
- Total turns: 0
- Approved by: Quản đốc (Sếp uỷ quyền approval gate, 2026-06-09) — EXECUTE may begin

---

## Nhiệm vụ

### Task 1: Thêm `tool_name` (top-level) + `path` (tool_input) vào payload struct

**File:** `src/io.rs`

**Tìm:** struct `HookPayload` (hiện wrap `tool_input: ToolInput`) và struct `ToolInput` (hiện `file_path, pattern, notebook_path, command`). `[needs Worker verify]` — grep `struct HookPayload` + `struct ToolInput`.

**Thay bằng / Thêm:**
- `HookPayload`: thêm field top-level `tool_name: Option<String>` với `#[serde(default)]` (oracle L31 đọc top-level `"tool_name"`, NGOÀI tool_input). GIỮ `tool_input` nguyên.
- `ToolInput`: thêm field `path: Option<String>` với `#[serde(default)]` (oracle L36/99 — Glob search root). GIỮ 4 field cũ.

**Lưu ý:**
- Fail-open HARD giữ nguyên: empty/invalid JSON → `default()` (mọi Option = None). KHÔNG `unwrap`.
- `tool_name` đặt ở `HookPayload` (top-level payload), KHÔNG trong `ToolInput` — payload Claude Code thật: `{"tool_name":"Read","tool_input":{...}}`.
- Cập nhật ARCHITECTURE.md "stdin-JSON Harness" section (JSON shape + field list) — Docs Gate (Task 6).

### Task 2: Viết lại `architect_guard_decide` — tool_name dispatch + 2 branch

**File:** `src/hooks/mod.rs`

**Tìm:** `fn architect_guard_decide(file_path: Option<&str>, pattern: Option<&str>) -> Decision` (≈ mod.rs:6). `[needs Worker verify]`.

**Thay bằng / Thêm:** đổi signature thành nhận `tool_name`, `file_path`, `pattern`, `path`. Đề xuất:
```rust
pub fn architect_guard_decide(
    tool_name: Option<&str>,
    file_path: Option<&str>,
    pattern: Option<&str>,
    path: Option<&str>,
) -> Decision
```
Logic port verbatim oracle (giữ marker gate `.sos-state/architect-active` — KHÔNG đổi sang `.claude/`):

1. Marker gate: `.sos-state/architect-active` không tồn tại → `ALLOW` (Decision exit 0, blocked false). GIỮ NGUYÊN logic P002.
2. `match tool_name`:
   - `Some("Read") | Some("Glob")` → **Read/Glob branch**: với mỗi candidate trong `[file_path, pattern, path]` (skip None/empty): strip `./`; nếu `ends_with(".md")` → continue (early-allow); nếu `is_forbidden_for_read(candidate)` → `block_read(original_candidate)`. Hết loop không block → ALLOW.
   - `Some("Write") | Some("Edit")` → **Write/Edit branch**: nếu `file_path` None/empty → ALLOW (oracle L111 defensive). Nếu `!is_allowed_for_write(file_path)` → `block_write(file_path)`. Else ALLOW.
   - `_` (None hoặc tool khác) → ALLOW (oracle default, case không match — đây là lý do Tension 1: test cũ thiếu tool_name sẽ rơi vào đây = ALLOW).

**Helper `is_forbidden_for_read(p)`** (port `is_forbidden_for_read` oracle L39-50, strip `./` đầu): true nếu match BẤT KỲ:
- prefix `src/ lib/ app/ pkg/`; segment `*/src/* */lib/* */app/* */pkg/*`; `crates/*/src/*`
- **`prisma/` prefix + `*/prisma/*` segment (MỚI vs bản cũ — oracle L43)**
- test: prefix `tests/ test/ __tests__/`; segment `*/tests/* */test/*`
- build prefix: `node_modules/ target/ dist/ build/ .next/ .nuxt/ .svelte-kit/`
- ext: `.rs .ts .tsx .js .jsx .py .go .java .cpp .c .h .hpp`
- **`.prisma .sql` (MỚI vs bản cũ — oracle L47)**

`[needs Worker verify]` — nếu P002 đã có `is_forbidden_for_read` riêng hoặc inline trong `_decide`, Worker tái dùng + THÊM 2 dòng prisma/sql; nếu chưa tách helper, tách ra (giữ behavior cũ + thêm 2 nhóm). Glob segment-vs-prefix khớp ARCHITECTURE.md bảng (starts_with vs contains).

**Helper `is_allowed_for_write(p)`** (MỚI — port oracle L54-61, strip `./`): thứ tự QUAN TRỌNG:
- `p == "docs/ticket/TICKET_TEMPLATE.md"` → return **false** (deny, explicit, check TRƯỚC).
- `p` match `docs/ticket/P*-*.md` HOẶC `*/docs/ticket/P*-*.md` → return **true**.
- else → false.

**Lưu ý:**
- `P*-*.md` glob: prefix literal `docs/ticket/P`, có ít nhất 1 `-` sau, đuôi `.md`. Worker chọn cách match (regex `^(.*/)?docs/ticket/P[^/]*-[^/]*\.md$` hoặc starts_with+contains+ends_with) — `[needs Worker verify]` cách impl, miễn parity oracle case glob. Lưu ý `TICKET_TEMPLATE.md` KHÔNG khớp `P*-*.md` (không bắt đầu `P`) → deny explicit là defense-in-depth oracle giữ; PORT NGUYÊN check deny-first.
- `crates/*/src/*` đã có ở bản cũ (ARCHITECTURE bảng) — GIỮ.
- Decision return: dùng `Decision { exit_code, blocked, reason }` pattern (Task 9 anchor). block_read/block_write set `exit_code=BLOCK, blocked=true, reason=Some(msg)`.

### Task 3: 2 block message verbatim oracle (block_read / block_write)

**File:** `src/hooks/mod.rs`

**Tìm:** message hiện tại của architect-guard (1 message Read/Glob, port P002 từ oracle cũ L66-76 — `[needs Worker verify]` nội dung hiện tại).

**Thay bằng / Thêm:** 2 message riêng, verbatim oracle tarot:

`block_read` (oracle L65-76, cho Read/Glob) — placeholder `$violator` = candidate gốc (pre-strip):
```
🚫 Architect envelope violation (Read/Glob)

Architect cannot read source code: {violator}

What to do instead: write a Task 0 anchor in the phiếu.
Example:
  | # | Assumption | Verify by | Result |
  | 1 | <claim about {violator}> | grep ... {violator} | ⏳ TO VERIFY |

Worker (separate subagent) will grep-verify it for you. The constraint IS the feature.
```

`block_write` (oracle L82-92, cho Write/Edit) — `{violator}` = file_path:
```
🚫 Architect envelope violation (Write/Edit)

Architect cannot Write/Edit: {violator}

Architect's Write allowlist (per architect.md line 32):
  - docs/ticket/P*-*.md  (phiếu files only)

Everything else (src/, CLAUDE.md, BACKLOG.md, CHANGELOG.md, guides) belongs to Worker.
If a phiếu needs to update those files, encode it as a Worker Task in the phiếu.
```

**Lưu ý:**
- Verbatim = khớp byte oracle (emoji 🚫, dấu cách, xuống dòng). Worker đối chiếu trực tiếp `~/tarot/scripts/architect-guard.sh` L65-94 khi viết.
- Message đi stderr (Decision.reason → CLI wrapper `eprintln!`). KHÔNG stdout.
- block_read dùng `{violator}` 3 chỗ (dòng "cannot read", và 2 chỗ trong bảng example).

### Task 4: CLI wrapper `architect_guard()` đọc `tool_name` + `path` từ payload

**File:** `src/hooks/mod.rs`

**Tìm:** `fn architect_guard()` (≈ mod.rs:79) đọc payload → gọi `architect_guard_decide(file_path, pattern)`. `[needs Worker verify]`.

**Thay bằng / Thêm:** đọc thêm `payload.tool_name` (top-level) + `payload.tool_input.path`, truyền 4 arg vào `_decide`:
```rust
let p = read_payload();
let d = architect_guard_decide(
    p.tool_name.as_deref(),
    p.tool_input.file_path.as_deref(),
    p.tool_input.pattern.as_deref(),
    p.tool_input.path.as_deref(),
);
// eprintln!(reason) nếu Some + return d.exit_code  (pattern P006 giữ nguyên)
```
**Lưu ý:** GIỮ pattern wrapper P006 (read_payload fail-open → _decide → eprintln reason nếu Some → return exit_code). Chỉ đổi arg list.

### Task 5: MCP tool `architect_guard` — GuardInput thêm `tool_name` + `path` (Tension 2)

**File:** `src/serve.rs`

**Tìm:** `struct GuardInput { file_path?, pattern? }` + `#[tool] fn architect_guard(...)`. `[needs Worker verify]`.

**Thay bằng / Thêm:** GuardInput thêm 2 field optional:
```rust
struct GuardInput {
    tool_name: Option<String>,  // MỚI — dispatch Read/Glob vs Write/Edit
    file_path: Option<String>,
    pattern: Option<String>,
    path: Option<String>,       // MỚI — Glob search root
}
```
tool method gọi `architect_guard_decide(tool_name, file_path, pattern, path)` (4 arg mới).

**Lưu ý:**
- **Quyết (Tension 2):** GuardInput nhận `tool_name` + `path` (mở rộng input hiện có, KHÔNG tách struct mới) — giữ MCP tool mirror đúng CLI `_decide` signature. Lý do: 1 nguồn sự thật cho dispatch logic, debug `why_blocked` chính xác.
- Nếu MCP client cũ gọi `architect_guard` KHÔNG có `tool_name` → `None` → branch default ALLOW (honest: marker-less hoặc tool-name-less = không guard, giống CLI). Ghi vào Discovery + ARCHITECTURE state-honesty note.
- MCP handshake test phải vẫn assert **5 tool** (không thêm/bớt tool, chỉ đổi input schema của 1 tool).

### Task 6: Đồng bộ oracle repo này — `scripts/architect-guard.sh` lên 119-dòng tarot

**File:** `scripts/architect-guard.sh` (repo claude-hooks)

**Tìm:** file hiện tại 86-dòng (oracle cũ, chỉ Read/Glob). `[needs Worker verify]` line count.

**Thay bằng / Thêm:** copy NỘI DUNG oracle tarot `~/tarot/scripts/architect-guard.sh` (119-dòng) **NHƯNG ĐỔI 1 dòng**: `MARKER_FILE` GIỮ `.sos-state/architect-active` (KHÔNG dùng `.claude/.architect-active` của tarot L22). Đây là oracle-reference cho binary repo này — phải khớp behavior binary (marker `.sos-state/`).

**Lưu ý:**
- Lý do đồng bộ: tránh F-004 tái diễn — oracle repo này phải = behavior deploy. Đây là spec cho regression test tương lai.
- **Divergence duy nhất giữa scripts/ repo này và oracle tarot = MARKER_FILE path** (`.sos-state/` vs `.claude/`). Ghi comment trong script: `# MARKER divergence from tarot oracle: .sos-state/ (binary convention) — F-005 marker-path unify is separate phiếu`.
- Worker đối chiếu binary behavior vs `bash scripts/architect-guard.sh` (đã đổi marker) — 2 cái PHẢI khớp exit code mọi case verify-cò.

### Task 7: Update test cũ (Tension 1) + thêm Write/Edit + superset cases

**File:** `tests/cli.rs` (integration) + `tests/mcp_handshake.rs` (unit core nếu có architect_guard_decide test)

**Tìm:** mọi test `architect-guard` feed payload KHÔNG có `tool_name` (Anchor #8, `[needs Worker verify]` — đếm + liệt kê). VD `{"tool_input":{"file_path":"src/main.rs"}}` expect exit 2.

**Thay bằng / Thêm:**
- **Update test cũ:** thêm `"tool_name":"Read"` vào payload các test Read-forbidden cũ (`{"tool_name":"Read","tool_input":{"file_path":"src/main.rs"}}`). Lý do: payload Claude Code THẬT luôn có tool_name → test đúng hơn. Test `.md` allow + marker-absent allow cũng thêm tool_name.
- **Thêm test Write/Edit** (verify-cò mới):
  - `{"tool_name":"Write","tool_input":{"file_path":"src/foo.ts"}}` + marker → exit 2 (block_write)
  - `{"tool_name":"Edit","tool_input":{"file_path":"CLAUDE.md"}}` + marker → exit 2
  - `{"tool_name":"Write","tool_input":{"file_path":"docs/ticket/P010-x.md"}}` + marker → exit 0 (allowlist)
  - `{"tool_name":"Write","tool_input":{"file_path":"docs/ticket/TICKET_TEMPLATE.md"}}` + marker → exit 2 (deny explicit)
  - `{"tool_name":"Edit","tool_input":{}}` + marker (no path) → exit 0 (defensive allow)
- **Thêm test Read/Glob superset:**
  - `{"tool_name":"Glob","tool_input":{"pattern":"src/**"}}` + marker → exit 2
  - `{"tool_name":"Glob","tool_input":{"path":"prisma/"}}` + marker → exit 2 (path = Glob root, prisma forbidden mới)
  - `{"tool_name":"Read","tool_input":{"file_path":"prisma/schema.prisma"}}` + marker → exit 2 (.prisma mới)
  - `{"tool_name":"Read","tool_input":{"file_path":"db/x.sql"}}` + marker → exit 2 (.sql mới)
  - `{"tool_name":"Read","tool_input":{"file_path":"README.md"}}` + marker → exit 0 (.md allow)
- **Thêm test dispatch-default:**
  - `{"tool_input":{"file_path":"src/main.rs"}}` (NO tool_name) + marker → exit 0 (case default — đây là behavior MỚI, document rõ trong test name VD `architect_guard_no_tool_name_allows`)
  - marker absent + bất kỳ → exit 0.

**Lưu ý:**
- **Tension 1 RESOLUTION:** test cũ KHÔNG tool_name + expect-2 sẽ VỠ sau dispatch (rơi default ALLOW). Worker PHẢI: (1) thêm `tool_name` vào test cũ để giữ expect-2, (2) thêm 1 test riêng khẳng định no-tool_name → ALLOW là intended (không phải bug). Liệt kê chính xác test nào sửa vào Discovery.
- Các test KHÔNG-architect-guard (block-env-edit, block-unsafe-merge, session-banner, serve) KHÔNG được vỡ. Regression: tổng test count tăng (thêm cases), không giảm.

### Task 8: CHANGELOG + ARCHITECTURE (Docs Gate Tầng 1)

**File:** `CHANGELOG.md` (root) + `docs/ARCHITECTURE.md`

**Thêm:**
- `CHANGELOG.md`: entry P010 dưới v0.9.0 — "architect-guard TRUE parity tarot: tool_name dispatch + Write/Edit allowlist branch (`docs/ticket/P*-*.md` only, deny TICKET_TEMPLATE) + Read/Glob superset (prisma/ + *.prisma/*.sql + Glob `path` root) + 2 block messages (read/write). Fix F-004. Marker GIỮ `.sos-state/` (F-005 defer)."
- `docs/ARCHITECTURE.md`:
  - Section `### architect-guard (P002 …)` → cập nhật: thêm bước tool_name dispatch (Read|Glob vs Write|Edit), Write/Edit allowlist bảng, Read/Glob forbidden bảng THÊM `prisma/` row + `.prisma .sql` ext + `path` candidate, 2 message (block_read/block_write). Cập nhật `_decide` signature 4-arg.
  - Section "stdin-JSON Harness" → JSON shape thêm top-level `tool_name` + `tool_input.path`; field list thêm `path` (Glob root, architect-guard).
  - Bảng "5 MCP tools" → `architect_guard` input đổi thành `{ tool_name?, file_path?, pattern?, path? }`.
  - `why_blocked` routing table + state-honesty: thêm note Tension 3 (xem Task 9 / Constraint 6) — Write/Edit ở tarot fire CẢ architect-guard LẪN block-env-edit, nhưng `why_blocked` GIỮ route đơn `Edit|Write → block_env_edit` (bounded P010), limitation documented.

**Lưu ý:** README — kiểm tra mô tả architect-guard (README:30 "chỉ Read/Glob" theo F-004). NẾU README mô tả guard scope → cập nhật "Read/Glob + Write/Edit allowlist". `[needs Worker verify]` README có dòng đó không; nếu có → sửa (Docs Gate), nếu không → ghi "README không mô tả scope, không cần sửa" vào Discovery.

### Task 9: Discovery report `docs/discoveries/P010.md`

**File:** `docs/discoveries/P010.md`

**Thêm:** per-phiếu discovery (P038 pattern) — ghi rõ:
- Tarot-oracle diff (4 khác biệt) đóng hết chưa, byte-parity message OK chưa.
- **Tension 1-4 resolution:**
  1. Test cũ thiếu tool_name: list test đã sửa (thêm tool_name) + test mới no-tool_name→ALLOW.
  2. GuardInput thêm tool_name+path (mở rộng, không tách struct).
  3. why_blocked GIỮ route đơn (bounded) — limitation: trên Write/Edit chỉ debug block_env_edit, KHÔNG debug architect_guard. → finding/phiếu sau (why_blocked multi-hook).
  4. MultiEdit/NotebookEdit: faithful (không guard ở architect-guard branch) — divergence vs why_blocked routing đã note.
- Marker GIỮ `.sos-state/architect-active` — F-005 marker-path unify DEFER (note phiếu sau).
- Parity vs `bash ~/tarot/scripts/architect-guard.sh` (đã đổi marker `.sos-state/`): kết quả đối chiếu mọi case verify-cò.
- Assumptions Task 0 — CORRECT/WRONG với file:line.
- Docs updated (list) / Tier escalations (None hoặc ghi).

Append 1-dòng index vào `docs/DISCOVERIES.md`.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/io.rs` | Task 1: `HookPayload.tool_name` + `ToolInput.path` (Option, serde default) |
| `src/hooks/mod.rs` | Task 2-4: `_decide` 4-arg + dispatch + `is_forbidden_for_read` (+prisma/sql) + `is_allowed_for_write` (mới) + 2 message + wrapper |
| `src/serve.rs` | Task 5: GuardInput +tool_name +path; architect_guard tool gọi 4-arg |
| `tests/cli.rs` | Task 7: update test cũ (+tool_name) + thêm Write/Edit + superset + dispatch-default cases |
| `tests/mcp_handshake.rs` | Task 7: unit `_decide` cases mới nếu có; assert vẫn 5 MCP tool |
| `scripts/architect-guard.sh` | Task 6: lên 119-dòng tarot, MARKER GIỮ `.sos-state/` |
| `CHANGELOG.md` | Task 8: entry P010 v0.9.0 |
| `docs/ARCHITECTURE.md` | Task 8: architect-guard section + harness + MCP tool schema + why_blocked note |
| `docs/discoveries/P010.md` | Task 9: discovery (+ index DISCOVERIES.md) |
| `README.md` | Task 8: NẾU mô tả guard scope (verify) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `src/hooks/mod.rs` (3 hook khác) | `block_env_edit_decide`, `block_unsafe_merge_decide`, `render_banner` KHÔNG đổi behavior |
| `src/serve.rs` `why_blocked` core routing | GIỮ route đơn `Read\|Glob→architect_guard`, `Edit\|Write\|MultiEdit\|NotebookEdit→block_env_edit`, `Bash→block_unsafe_merge` (Tension 3 = bounded; chỉ đổi GuardInput schema, KHÔNG đổi routing) |
| `.claude/settings.json` | PreToolUse matchers architect-guard (Read/Glob/Write/Edit) — verify khớp dispatch; KHÔNG đổi trong P010 trừ khi matcher thiếu Write/Edit (nếu thiếu → ghi Discovery, có thể cần follow-on) |
| `tests/cli.rs` (block-env-edit, merge, banner) | KHÔNG vỡ |

---

## Luật chơi (Constraints)

1. **Port TRUNG THÀNH oracle TAROT 119-dòng** — KHÔNG redesign, KHÔNG "tiện tay cải tiến". Cùng exit code (0/2) + cùng stderr message (verbatim) như `bash ~/tarot/scripts/architect-guard.sh` (sau khi đổi marker `.sos-state/`).
2. **Marker GIỮ `.sos-state/architect-active`** — KHÔNG đổi sang `.claude/`. F-005 marker-path = phiếu riêng (defer). Divergence này có chủ đích, ghi Discovery.
3. **Fail-open HARD giữ nguyên** — empty/invalid stdin → default (None) → marker gate hoặc dispatch default → ALLOW. KHÔNG panic. (architect-guard = fail-OPEN, KHÁC block-unsafe-merge fail-CLOSED.)
4. **MultiEdit/NotebookEdit faithful** — dispatch CHỈ `Read|Glob` + `Write|Edit` (oracle). MultiEdit/NotebookEdit → default ALLOW ở architect-guard (vẫn bị block-env-edit route riêng). KHÔNG thêm vào allowlist branch.
5. **`is_allowed_for_write` deny-TICKET_TEMPLATE check TRƯỚC allow-P*-*.md** — thứ tự oracle L57-58. (Defense-in-depth: TICKET_TEMPLATE không khớp `P*-*.md` anyway, nhưng port nguyên check explicit.)
6. **why_blocked BOUNDED (Tension 3 = option a)** — KHÔNG đổi why_blocked routing core thành multi-hook. GIỮ route đơn, ghi limitation note (Write/Edit debug chỉ thấy block_env_edit, không thấy architect_guard). why_blocked multi-hook = finding/phiếu sau.
7. **MCP vẫn đúng 5 tool** — chỉ đổi input schema `architect_guard`, KHÔNG thêm/bớt tool. Handshake test assert 5.
8. **Verify-cò P057 bắt buộc** — fixtures exit-code + stderr trong cùng phiếu (Task 7). Build cò ≠ cò sống.

---

## Nghiệm thu

### Automated
- [ ] `cargo build` clean
- [ ] `cargo test --all` clean — test cũ (sau khi +tool_name) + test mới PASS, 3 hook khác KHÔNG vỡ
- [ ] `cargo clippy -- -D warnings` không warning

### Manual Testing — Verify-cò (P057), marker `.sos-state/architect-active` PRESENT

| Payload | Expect exit |
|---|---|
| `{"tool_name":"Read","tool_input":{"file_path":"src/foo.ts"}}` | 2 (block_read) |
| `{"tool_name":"Read","tool_input":{"file_path":"foo.md"}}` | 0 (.md allow) |
| `{"tool_name":"Glob","tool_input":{"pattern":"src/**"}}` | 2 |
| `{"tool_name":"Glob","tool_input":{"path":"prisma/"}}` | 2 (Glob root, prisma mới) |
| `{"tool_name":"Read","tool_input":{"file_path":"prisma/schema.prisma"}}` | 2 (.prisma mới) |
| `{"tool_name":"Read","tool_input":{"file_path":"x.sql"}}` | 2 (.sql mới) |
| `{"tool_name":"Write","tool_input":{"file_path":"src/foo.ts"}}` | 2 (block_write) |
| `{"tool_name":"Edit","tool_input":{"file_path":"CLAUDE.md"}}` | 2 (block_write) |
| `{"tool_name":"Write","tool_input":{"file_path":"docs/ticket/P010-x.md"}}` | 0 (allowlist) |
| `{"tool_name":"Write","tool_input":{"file_path":"docs/ticket/TICKET_TEMPLATE.md"}}` | 2 (deny explicit) |
| `{"tool_name":"Edit","tool_input":{}}` (no path) | 0 (defensive) |
| `{"tool_input":{"file_path":"src/foo.ts"}}` (NO tool_name) | 0 (dispatch default — intended) |

- [ ] **Đối chiếu `bash ~/tarot/scripts/architect-guard.sh`** (sau Task 6 đổi marker `.sos-state/`) cùng payload — exit code + stderr message KHỚP binary. (Quản đốc verify thật ở nghiệm thu.)
- [ ] block_read vs block_write stderr verbatim oracle L65-94.

### Marker ABSENT
- [ ] Mọi payload (kể cả Write src/) → exit 0 (marker gate). Verify 1-2 case.

### Regression
- [ ] block-env-edit / block-unsafe-merge / session-banner / serve tests PASS
- [ ] MCP handshake: 5 tool; `architect_guard` nhận `tool_name`+`path` (gọi không tool_name → ALLOW honest)
- [ ] `why_blocked` routing KHÔNG đổi (Read|Glob→architect_guard, Edit|Write→block_env_edit)

### Docs Gate (Tầng 1 — security-surface AUTO)
- [ ] `CHANGELOG.md` — entry P010 (v0.9.0)
- [ ] `docs/ARCHITECTURE.md` — architect-guard section (dispatch + Write/Edit allowlist + Read/Glob superset prisma/sql/path + 2 message) + harness (tool_name/path) + MCP tool schema + why_blocked limitation note
- [ ] `README.md` — cập nhật nếu mô tả guard scope (verify)
- [ ] Worker ghi "Tầng 1 docs updated: <list>" trong Discovery

### Discovery Report
- [ ] `docs/discoveries/P010.md`: tarot-oracle diff đóng hết, Tension 1-4 resolution, marker GIỮ `.sos-state` (F-005 defer note), parity vs tarot bash, assumptions CORRECT/WRONG (file:line), docs updated list, tier escalations
- [ ] 1-dòng index vào `docs/DISCOVERIES.md`

### Branch / commit
- [ ] Branch `feat/P010-architect-guard-write-guard` (đã checkout). `git add` cả phiếu (F-002). ĐỪNG push/PR — Quản đốc xử.
