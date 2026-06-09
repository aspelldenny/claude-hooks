# PHIẾU P003: `block-env-edit` subcmd (port 1:1 từ Bash oracle)

> **ID:** P003 · **Filename:** `docs/ticket/P003-block-env-edit.md`
> **Branch:** `feat/P001-scaffold-cli` (Phase 1 bundle — KHÔNG tạo branch mới; xem Branch note cuối phiếu)

---

> **Loại:** Feature (port)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — hook = security-surface: `.env*` = secret leak guard. CLAUDE.md DOCS GATE: "Security-surface → AUTO Tầng 1". Sai regex/exit = secret `.env` lọt qua Claude tool VÀO prompt/context/log. 54 dòng LOC nhỏ KHÔNG hạ Tầng.)
> **Ảnh hưởng:** `src/hooks/mod.rs` (thay stub `block_env_edit()`), `tests/cli.rs` (fire-test fixtures)
> **Dependency:** P001 (harness `src/io.rs` + stub) + P002 (đã gỡ `#[allow(dead_code)]` khỏi `BLOCK`/`block`). Cả hai đã ship.

---

## Context

### Vấn đề hiện tại

`src/hooks/mod.rs::block_env_edit()` hiện là stub trả `ALLOW` vô điều kiện (P001 scaffold):

```rust
pub fn block_env_edit() -> i32 {
    let _payload = io::read_payload(); // real logic in P003
    ALLOW
}
```

Hook Bash oracle `scripts/block-env-edit.sh` (64 dòng) là cổng chặn Edit/Write tới `.env*` files (trừ `.env.example`) — chống secret thật (API keys, DB credentials, webhook tokens) leak vào prompt/context/log qua Claude tool call. Cần port logic này 1:1 sang Rust để đạt CLI parity (PROJECT.md Success #1 · BACKLOG Active sprint Phase 1 — **đây là phiếu CUỐI của Phase 1**).

### Giải pháp

Thay body stub bằng logic port TRUNG THÀNH từ oracle (CLAUDE.md §Port doctrine #1-2: cùng exit code + semantics, KHÔNG redesign). Pipeline theo oracle:

1. **Đọc input:** `read_payload()` (harness P001 đọc stdin). Quyết định port env-fallback `CLAUDE_TOOL_INPUT` — xem Task 1 + Constraint 6 (điểm Worker DISCOVERY).
2. **Input rỗng → `ALLOW`** (fail-open, oracle L23).
3. **Parse path:** `tool_input.file_path`, None → fallback `tool_input.notebook_path` (oracle L29-32, NotebookEdit). **KHÔNG dùng `pattern`** (khác P002 — block-env-edit không xử Glob).
4. **Không path → `ALLOW`** (oracle L35).
5. **BASE = basename(path)** (oracle L38) — chỉ tên file cuối: `/a/b/.env` → `.env`.
6. **Allowlist:** `BASE == ".env.example"` → `ALLOW` (oracle L41).
7. **Block nếu BASE khớp regex `^\.env($|\.)`** (oracle L44) → stderr message nguyên văn oracle L46-59 (gồm `⛔`) → `BLOCK` (exit 2).
8. **Else → `ALLOW`** (oracle L64).

### Scope

- CHỈ sửa `src/hooks/mod.rs` (thân hàm `block_env_edit`) + `tests/cli.rs` (thêm fire-test).
- KHÔNG sửa `src/io.rs` (API đã đủ — chỉ import thêm nếu cần), `src/main.rs` (dispatch không đổi), `Cargo.toml` (KHÔNG thêm dep — `regex` ĐÃ có sẵn, xem Task 0 #11 + Constraint 5).

---

## Task 0 — Verification Anchors

> Oracle (`scripts/block-env-edit.sh`): Architect ĐỌC ĐƯỢC file `.sh` (guard không chặn) → `[verified]` line-thật.
> src/ (io.rs API, hooks/mod.rs stub, Cargo.toml dep): Architect KHÔNG đọc được `src/*.rs` (guard chặn) → `[Quản-đốc-fed, Worker verify]`. P001/P002 Discovery + `docs/ARCHITECTURE.md` corroborate io.rs API.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Đọc input: stdin nếu `! -t 0`, fallback env `CLAUDE_TOOL_INPUT` | Oracle L16-20 | ✅ `[verified]` L16-20 |
| 2 | Input rỗng → exit 0 (fail-open) | Oracle L23 `[ -z "$INPUT" ] && exit 0` | ✅ `[verified]` L23 |
| 3 | Path parse: `file_path` ưu tiên, fallback `notebook_path` (NotebookEdit). KHÔNG dùng `pattern` | Oracle L29-32 | ✅ `[verified]` L29-32 (sed `file_path` → sed `notebook_path`) |
| 4 | Không path → exit 0 | Oracle L35 `[ -z "$FILE_PATH" ] && exit 0` | ✅ `[verified]` L35 |
| 5 | `BASE = basename(FILE_PATH)` | Oracle L38 `BASE=$(basename "$FILE_PATH")` | ✅ `[verified]` L38 |
| 6 | Allowlist `BASE == ".env.example"` → exit 0 | Oracle L41 | ✅ `[verified]` L41 |
| 7 | Block regex `^\.env($|\.)` | Oracle L44 `grep -qE '^\.env($\|\.)'` | ✅ `[verified]` L44 |
| 8 | Block message (tiếng Việt, gồm `⛔`) → stderr → exit 2 | Oracle L45-60 heredoc + `exit 2` | ✅ `[verified]` L45-60 (text verbatim trong Task 2) |
| 9 | Else → exit 0 | Oracle L64 `exit 0` | ✅ `[verified]` L64 |
| 10 | `read_payload() -> HookPayload`; `tool_input: ToolInput{ file_path, notebook_path: Option<String> }` (serde, `#[serde(default)]`) | Worker grep `pub fn read_payload\|pub struct ToolInput\|notebook_path` trong `src/io.rs` | ⏳ `[Quản-đốc-fed, Worker verify]` — ARCHITECTURE.md L59 + P001 Discovery corroborate |
| 11 | Dep `regex` có sẵn trong `Cargo.toml` (`regex = "1"`) | Worker grep `^regex` trong `Cargo.toml` `[dependencies]` | ⏳ `[Quản-đốc-fed, Worker verify]` — CLAUDE.md §Stack liệt kê `regex`; nếu MISSING → DISCOVERY + dùng string-match fallback (xem Task 1 Lưu ý) |
| 12 | `block_env_edit()` stub hiện trả `ALLOW`, import `use crate::io::{self, ALLOW}` (đầu mod.rs); `BLOCK`/`block` đã dùng (P002 gỡ `#[allow(dead_code)]`) | Worker đọc `src/hooks/mod.rs` đầu file + thân `block_env_edit` | ⏳ `[Quản-đốc-fed, Worker verify]` — Worker xác nhận import path `BLOCK`/`block` reachable; thêm vào `use` nếu cần |

**Lưu ý ❌-handling:** Nếu anchor #11 (regex dep) ❌ MISSING → Worker KHÔNG tự thêm dep (Tầng 1 contract đổi); ghi DISCOVERY + dùng string-match port (logic tương đương trong Task 1 Lưu ý) HOẶC escalate. Anchor #10/#12 ❌ → DISCOVERY_REPORT + hỏi Quản đốc API thật.

### Pre-phiếu snapshot (Worker auto first-step)

> Worker EXECUTE FIRST ACTION. Phase 1 bundle → snapshot vào `.backup/P003/`.

```bash
# Run from project root:
PHIEU_ID=P003
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/P003/"
```

Rollback (trong worktree only): `git reset --hard $(cat .backup/P003/main-head.txt)`. `.backup/` gitignored.

---

## Debate Log

> Auto-populated by Worker (CHALLENGE) + Architect (RESPOND). Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Quản đốc Challenge (orchestrator có Read src/ — verify #10/#11/#12 trực tiếp)

**Anchor verification (3/3 src ✅):**
- #10 ✅ `src/io.rs` — `read_payload()`, `ToolInput{file_path,pattern,notebook_path: Option<String>}` (ARCHITECTURE + P001/P002 corroborate)
- #11 ✅ `Cargo.toml:20` — `regex = "1"` có thật → dùng `regex::Regex` (KHÔNG rơi string-match fallback)
- #12 ✅ `src/hooks/mod.rs:1,90` — import `use crate::io::{self, ALLOW};`; `io::block` đã dùng qualified ở P002 (L90) → P003 dùng `io::block` qualified OK, không cần sửa `use`

**Objections (Tầng 1):** None. Port 1:1 trung thành oracle L29-64; regex verbatim `^\.env($|\.)`; `.envrc`/`.environment` edge có fixture. Quyết định #2 (KHÔNG port env-fallback `CLAUDE_TOOL_INPUT`): **chấp nhận** — oracle chỉ dùng env khi stdin tty (L16-20), Claude Code LUÔN pipe stdin nên nhánh đó không bao giờ fire trong production; architect-guard.sh cũng không có fallback này. Parity-gap negligible, đã ghi Discovery có ý thức.

**Status:** ✅ ACCEPTED V1 — no challenges.

### Final consensus
- Phiếu version: V1 (no debate turn needed)
- Total turns: 0
- Approved by: Quản đốc (Sếp uỷ quyền approval gate, 2026-06-09) — EXECUTE may begin

---

## Nhiệm vụ

### Task 1: Port `block_env_edit()` thân hàm

**File:** `src/hooks/mod.rs` `[Quản-đốc-fed, Worker verify path]`

**Tìm:** thân hàm stub `block_env_edit` (hiện trả `ALLOW` sau `let _payload = io::read_payload();`).

**Thay bằng:** logic port 1:1 oracle. Pseudocode (Worker viết Rust idiomatic, mirror style P002 `architect_guard`):

```
fn block_env_edit() -> i32 {
    let payload = io::read_payload();

    // Bước 3: file_path ưu tiên, fallback notebook_path. KHÔNG dùng pattern.
    let path = payload.tool_input.file_path
        .or(payload.tool_input.notebook_path);

    // Bước 4: không path → ALLOW (đã gộp input-rỗng vì payload rỗng → cả 2 field None)
    let path = match path {
        Some(p) if !p.is_empty() => p,
        _ => return ALLOW,
    };

    // Bước 5: BASE = basename. Lấy segment cuối sau '/'.
    let base = path.rsplit('/').next().unwrap_or(&path);

    // Bước 6: allowlist
    if base == ".env.example" {
        return ALLOW;
    }

    // Bước 7: regex ^\.env($|\.) — DÙNG regex::Regex (Constraint 5)
    let re = Regex::new(r"^\.env($|\.)").unwrap(); // pattern hằng → unwrap an toàn
    if re.is_match(base) {
        return io::block(<MESSAGE tiếng Việt verbatim oracle L46-59 — xem Task 2>);
    }

    // Bước 8: else
    ALLOW
}
```

**Lưu ý:**
- **Bước 2 (input rỗng → exit 0):** oracle check `[ -z "$INPUT" ]` TRƯỚC khi parse. Harness `read_payload()` fail-open: empty stdin → `HookPayload::default()` → cả `file_path` + `notebook_path` = `None` → rơi vào `return ALLOW` ở Bước 4. **Tương đương semantically** — không cần check input-rỗng riêng. Worker xác nhận `HookPayload::default()` cho `None` fields (ARCHITECTURE.md L63 confirm fail-open).
- **`basename` semantics:** oracle dùng `basename` shell. `path.rsplit('/').next()` cho cùng kết quả với path POSIX (`/some/dir/.env` → `.env`; `.env` → `.env`). Trailing-slash edge (`/a/b/`) hiếm với Edit/Write file_path — nếu Worker lo, dùng `std::path::Path::file_name()` cho robust; nhưng `rsplit('/')` đủ parity oracle. Worker quyết (Tầng 2 impl detail).
- **regex `unwrap()`:** pattern là string-literal hằng, compile không bao giờ fail → `unwrap()` chấp nhận được (KHÔNG phải user input). Mirror nếu codebase có convention khác (P002 không dùng regex). Cân nhắc `once_cell`/`LazyLock` nếu muốn tránh recompile mỗi call — nhưng hook chạy 1 lần/process nên KHÔNG cần optimize (Tầng 2, Worker quyết).
- **`pattern` field KHÔNG dùng** (khác P002): block-env-edit chỉ check file edit, không xử Glob. KHÔNG đọc `tool_input.pattern`.
- **Fallback regex dep MISSING (anchor #11 ❌):** nếu Worker grep KHÔNG thấy `regex` trong Cargo.toml → KHÔNG tự thêm dep (Tầng 1). Logic tương đương string-match: `base == ".env" || base.starts_with(".env.")`. Đây port đúng `^\.env($|\.)` (`$` sau env = đúng `.env`; `\.` sau env = `.env.` prefix). Ghi DISCOVERY nếu phải fallback. **Mặc định: dùng `regex` (CLAUDE.md §Stack confirm có).**

### Task 2: Block message (stderr verbatim oracle L46-59)

**File:** `src/hooks/mod.rs` (trong nhánh `re.is_match(base)` của Task 1)

**Thay bằng:** truyền chuỗi sau vào `io::block(...)`. **VERBATIM oracle L46-59** — KHÔNG dịch, KHÔNG đổi từ. `$FILE_PATH` → biến `path` thật (full path, KHÔNG phải `base`):

```
⛔ BLOCKED: Edit/Write tới {path} bị chặn.

Lý do: .env* file chứa secret thật (API keys, DB credentials, webhook tokens).
KHÔNG sửa qua Claude tool — risk leak vào prompt/context/log.

Cách hợp lệ:
  - Sửa .env.example (template, không secret thật)
  - Sếp paste secret thật vào .env tay (local-only edit)
  - Sửa qua SSH/console nếu là production env

Override (nếu thật sự cần Claude edit .env, hiếm):
  - Tạm rename .env → .env.draft, edit, rename back
  - Hoặc remove hook khỏi .claude/settings.json (PR review trước)
```

**Lưu ý:**
- `{path}` = full path nguyên gốc (oracle L46 dùng `$FILE_PATH`, KHÔNG phải `$BASE`). Worker dùng biến `path` (đã bind ở Task 1), KHÔNG `base`.
- `io::block(reason)` tự `eprintln!` + trả `BLOCK` (ARCHITECTURE.md L72). KHÔNG tự `process::exit` trong hook fn (unit-testable — mirror P002).
- Format chuỗi multi-line: Worker dùng `format!` với `{path}` hoặc raw-string + replace. Mirror cách P002 build block message.

### Task 3: Quyết định env-fallback `CLAUDE_TOOL_INPUT` (DISCOVERY point)

**File:** `src/hooks/mod.rs` (Bước 1 — TRƯỚC `read_payload`) HOẶC bỏ qua.

**Bối cảnh:** oracle L16-20 đọc stdin nếu `! -t 0`, fallback env `CLAUDE_TOOL_INPUT` khi stdin là tty. Harness P001 `read_payload()` CHỈ đọc stdin (không có env fallback).

**Quyết định Architect (default):** **KHÔNG port env-fallback trong P003.** Lý do:
1. Binary hook LUÔN nhận stdin từ Claude Code (PreToolUse spec gửi JSON qua stdin) → nhánh tty/env-fallback của oracle hiếm khi chạy thực tế.
2. Thêm env-fallback = đổi `io.rs` harness (shared API, dùng bởi P002/P004/P005) → Tầng 1 scope creep, KHÔNG thuộc phiếu này. P002 (`architect-guard`) cũng port mà KHÔNG có env-fallback — giữ nhất quán cross-hook.
3. Parity "đủ chặt cho hành vi thực": stdin path = path nóng, đã cover.

**Lưu ý (Worker DISCOVERY BẮT BUỘC ghi):** trong `docs/discoveries/P003.md` ghi rõ:
- "env-fallback `CLAUDE_TOOL_INPUT` (oracle L16-20) KHÔNG port — harness chỉ stdin. Lý do: hook luôn nhận stdin; env-fallback = shared-harness change (Tầng 1) ngoài scope. P002 cùng quyết định. Nếu Quản đốc/PARITY-test thấy cần → phiếu follow-on cho `io.rs` (cover cả P002/P003/P004/P005 cùng lúc)."
- Đây là điểm parity-gap CÓ Ý THỨC, KHÔNG phải sót. Nếu Worker bất đồng (muốn parity chặt) → raise [O] trong CHALLENGE.

### Task 4: Fire-test fixtures (P057 — BẮT BUỘC cùng phiếu)

**File:** `tests/cli.rs` `[Quản-đốc-fed, Worker verify path]`

**Thêm:** test cases mirror style P001/P002 (`assert_cmd` feed stdin JSON + assert exit code). **Test isolation:** block-env-edit KHÔNG phụ thuộc marker global (khác P002) → KHÔNG cần `CLAUDE_PROJECT_DIR` temp dir. Chỉ feed stdin + assert exit. Đơn giản hơn P002.

| # | Input (`tool_input`) | Expect exit | Lý do |
|---|---|---|---|
| 1 | `{"file_path":"/x/.env"}` | 2 | regex match `^\.env$` (basename `.env`) |
| 2 | `{"file_path":".env.example"}` | 0 | allowlist (Bước 6, trước regex) |
| 3 | `{"file_path":".envrc"}` | 0 | **regex KHÔNG match** — sau `env` là `rc`, không phải `$` hay `.`. CASE DỄ SAI NHẤT — port `($\|\.)` chuẩn |
| 4 | `{"file_path":".env.local"}` | 2 | dot sau env → `\.` match |
| 5 | `{"file_path":".env.production"}` | 2 | dot sau env → match |
| 6 | `{"file_path":"/some/dir/.env"}` | 2 | basename `/some/dir/.env` → `.env` → match (verify Bước 5) |
| 7 | `{"file_path":"config.yaml"}` | 0 | non-env → no match |
| 8 | `{"notebook_path":"x/.env"}` | 2 | fallback parse `notebook_path` (verify anchor #3 / Bước 3) |
| 9 | empty stdin (`""`) | 0 | fail-open (Bước 2/4 — payload rỗng → None → ALLOW) |

**Lưu ý:**
- Thêm: `.environment` → exit 0 (sau `env` là `ironment` = chữ, không `$`/`.`) — edge bonus củng cố regex anchor `($|\.)`. Optional nhưng nên có (cùng họ `.envrc`).
- Worker mirror helper/builder của test P001/P002 (đọc `tests/cli.rs` hiện có để reuse pattern, đừng tự chế).
- **14 test cũ (P001 8 + P002 6) KHÔNG được vỡ.** Chỉ THÊM, không sửa test cũ.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/hooks/mod.rs` | Task 1-3: thay thân stub `block_env_edit` bằng logic port (basename + allowlist + regex + block message). Thêm `use regex::Regex;` + import `BLOCK`/`block` nếu chưa reachable. |
| `tests/cli.rs` | Task 4: thêm 9 (+1 optional) fire-test fixtures. |
| `docs/ARCHITECTURE.md` | Docs Gate Tầng 1: block-env-edit "stub"→"real" (xem Nghiệm thu). |
| `CHANGELOG.md` | Entry P003. |
| `docs/discoveries/P003.md` + `docs/DISCOVERIES.md` | Discovery report (đặc biệt Task 3 env-fallback decision). |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `src/io.rs` | `read_payload`/`HookPayload`/`ToolInput`/`ALLOW`/`BLOCK`/`block` KHÔNG đổi. Chỉ import. |
| `src/main.rs` | Dispatch `block-env-edit` → `block_env_edit()` không đổi. |
| `Cargo.toml` | `regex` ĐÃ có (anchor #11) — KHÔNG thêm dep. Nếu MISSING → DISCOVERY, KHÔNG tự thêm (Tầng 1). |
| `src/hooks/mod.rs` các hook khác | `architect_guard` (P002) + 2 stub còn lại không đổi. |

---

## Luật chơi (Constraints)

1. **Port 1:1, KHÔNG redesign** (CLAUDE.md §Port doctrine #1). Cùng exit (0/2) + cùng semantics oracle. Block message VERBATIM (Task 2).
2. **Oracle = spec** (CLAUDE.md #2). Mọi nghi ngờ → đọc `scripts/block-env-edit.sh` line-thật, KHÔNG bịa.
3. **Verify-cò bắt buộc** (P057 / CLAUDE.md #3): fixture Task 4 trong CÙNG phiếu. Build cò ≠ cò sống — `cargo test` PASS mới xong.
4. **Scope bounded** (CLAUDE.md #4): chỉ port `block-env-edit`. KHÔNG kéo env-fallback harness change (Task 3), KHÔNG đụng hook khác.
5. **KHÔNG thêm Cargo dep.** `regex` đã có (anchor #11). Nếu Worker grep thấy MISSING → DISCOVERY + string-match fallback (Task 1 Lưu ý), KHÔNG tự `cargo add` (Tầng 1 contract đổi → phải qua phiếu).
6. **env-fallback decision = DISCOVERY có ý thức** (Task 3): KHÔNG port trong P003, ghi rõ lý do. Nếu Worker bất đồng → CHALLENGE [O].
7. **Hook fn trả `i32`, KHÔNG self-`exit`** (ARCHITECTURE.md L72): unit-testable. `process::exit` chỉ ở `main`.
8. **14 test cũ KHÔNG vỡ** (chỉ thêm fixture mới).

---

## Nghiệm thu

### Automated
- [ ] `cargo build` clean.
- [ ] `cargo test --all` clean — 9 (+1) fixture mới PASS + 14 test cũ (P001+P002) KHÔNG vỡ.
- [ ] `cargo clippy -- -D warnings` không warning (đặc biệt: regex `unwrap` trên literal — clippy OK; nếu cảnh báo `regex` recompile → cân nhắc `LazyLock`, Tầng 2).

### Manual Testing (PARITY — Quản đốc verify)
- [ ] **PARITY check thủ công:** đối chiếu `bash scripts/block-env-edit.sh` vs `claude-hooks block-env-edit` cùng input cho 9 case Task 4. Cùng exit code + (case block) cùng stderr message. Ghi kết quả vào đây.
- [ ] `.env` → bash exit 2 == binary exit 2.
- [ ] `.envrc` → bash exit 0 == binary exit 0 (case dễ sai nhất).
- [ ] `.env.example` → bash exit 0 == binary exit 0 (allowlist).

### Regression
- [ ] `architect-guard` (P002) vẫn chạy đúng (cùng `mod.rs`, không đụng).
- [ ] 3 stub còn lại (`block-unsafe-merge`/`session-banner`/`serve`) vẫn trả ALLOW/stub.

### Docs Gate (Tầng 1 — BẮT BUỘC, security-surface)
- [ ] `docs/ARCHITECTURE.md`:
  - Bảng Subcommands: `block-env-edit` Status `stub (P001)` → `real (P003)`.
  - Thêm section `### block-env-edit (P003 — real implementation)`: pipeline (basename + allowlist `.env.example` + regex `^\.env($|\.)` + block message + exit 0/2). Mirror format section `architect-guard` (L23-52).
  - Data Flow note L103: `block-env-edit (P003+)` → real; gỡ khỏi list "still stubs".
- [ ] `CHANGELOG.md` — entry P003 (port block-env-edit, regex verbatim, env-fallback note).

### Discovery Report
- [ ] `docs/discoveries/P003.md` (P038 per-phiếu pattern):
  - Anchors #10/#11/#12 — CORRECT / WRONG (với `file:line`). Đặc biệt #11 regex dep: confirmed có/không.
  - **Task 3 env-fallback decision**: ghi rõ KHÔNG port `CLAUDE_TOOL_INPUT` + lý do (xem Task 3 Lưu ý). Đây là MUST-HAVE entry.
  - basename impl: `rsplit('/')` hay `Path::file_name()` — đã chọn gì + edge nào gặp.
  - regex vs string-match: dùng cái nào (mặc định regex; fallback string nếu dep missing).
  - Docs updated: list (ARCHITECTURE.md sections). Tier escalations: "None" (Tầng 1 từ đầu).
- [ ] Append 1-line index vào `docs/DISCOVERIES.md` (link P003.md).

---

## Branch note (cho Worker)

- Phase 1 bundle trên branch `feat/P001-scaffold-cli` — **KHÔNG tạo branch mới.**
- Snapshot dùng `.backup/P003/` (Pre-phiếu snapshot ở trên).
- `git add` cả phiếu `docs/ticket/P003-block-env-edit.md` vào commit (F-002).
- Phiếu CUỐI Phase 1 → sau khi merge: BACKLOG move P003 xuống Recently shipped + tick Active sprint exit criteria (CLI parity 4-hook... thực ra 2-hook architect-guard + block-env-edit).
