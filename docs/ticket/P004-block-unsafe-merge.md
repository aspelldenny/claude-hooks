# PHIẾU P004: `block-unsafe-merge` subcmd (port oracle 137 dòng — gh shelling, fail-CLOSED)

> **ID format:** `P004`. Branch: `feat/P001-scaffold-cli` (Quản đốc quyết stack tiếp trên branch PR #1 — Phase 2, KHÔNG tách branch mới).
> **Filename:** `docs/ticket/P004-block-unsafe-merge.md` (active) → `docs/ticket/done/` khi xong.

---

> **Loại:** Feature (port 1:1 từ Bash oracle)
> **Ưu tiên:** P1
> **Tầng:** 1 (security-surface hook — chặn/cho `gh pr merge`. AUTO Tầng 1: security. Thêm cả shared-harness change `io.rs ToolInput.command`. CHALLENGE bắt buộc.)
> **Ảnh hưởng:** `src/hooks/mod.rs` (stub→real), `src/io.rs` (ToolInput thêm `command`), `tests/cli.rs` (integration), `docs/ARCHITECTURE.md`, `CHANGELOG.md`, `docs/discoveries/P004.md`.
> **Dependency:** P001-P003 shipped (CLI scaffold + io harness + 2 hook). None blocking.

---

## Context

### Vấn đề hiện tại

`block-unsafe-merge` hiện là **stub** (P001) trả `ALLOW`. Stub comment hiện tại:
```rust
pub fn block_unsafe_merge() -> i32 {
    ALLOW // real logic in P004 (reads gh pr diff, not stdin payload)
}
```
Comment **SAI** (P001 mis-assume): oracle THỰC SỰ đọc `tool_input.command` từ **stdin** (oracle L24-31), rồi mới shell ra `gh pr diff`/`gh pr view`. P004 sửa: thêm `read_payload()` + đọc field `command`.

Oracle `scripts/block-unsafe-merge.sh` (137 dòng) là hook PHỨC TẠP NHẤT trong bộ: shell ra `gh` CLI 2 lần, security-surface regex, parse review verdict, và — **divergence quan trọng nhất** — **fail-CLOSED** (block exit 2) khi không verify được, ĐỐI LẬP 3 hook kia fail-OPEN (exit 0).

### Giải pháp

Port 1:1 oracle (CLAUDE.md §Port doctrine — TRUNG THÀNH, cùng exit + semantics, KHÔNG redesign). Tách **PURE functions** (test được không cần gh) khỏi **gh-shelling** (integration/manual). Thêm field `command` vào `ToolInput` (`src/io.rs`) — shared-harness change, additive thuần (P006/P007 dùng lại được).

### Scope

- **CÓ sửa** `src/io.rs` — thêm `pub command: Option<String>` vào `ToolInput` (khác P002/P003 KHÔNG đụng io.rs). Đây là Tầng 1 shared-harness nhưng additive: chỉ thêm field `#[serde(default)]`, không đổi field cũ → P002/P003 path không vỡ.
- **CÓ sửa** `src/hooks/mod.rs` — `block_unsafe_merge()` stub→real + pure helper functions + `#[cfg(test)] mod tests`.
- **CÓ sửa** `tests/cli.rs` — thêm integration test (gh-free paths).
- **KHÔNG sửa** `scripts/block-unsafe-merge.sh` (oracle, read-only).
- **KHÔNG sửa** `architect-guard`/`block-env-edit` logic (P002/P003 frozen — 24 test cũ phải PASS).
- **KHÔNG** thêm mocking dependency. Nếu Worker thấy cần gh-mock seam → DISCOVERY, đừng thêm dep im lặng.
- **KHÔNG** port env-fallback `CLAUDE_TOOL_INPUT` (oracle L24-28 stdin-vs-env branch) — harness chỉ stdin, nhất quán P003 (ghi Discovery).

---

## Task 0 — Verification Anchors

> Anchor oracle (.sh): Architect ĐÃ Read `scripts/block-unsafe-merge.sh` 137 dòng → `[verified]`.
> Anchor src/*.rs: Architect KHÔNG Read được src (envelope) → `[Quản-đốc-fed, Worker verify]`.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Oracle đọc stdin → `tool_input.command`, env-fallback `CLAUDE_TOOL_INPUT` (L24-31); input rỗng → exit 0 (L31) | Read oracle L24-31 | ✅ `[verified]` L24-31 |
| 2 | Match regex `gh pr merge[[:space:]]+[0-9]+` (L41); không match → exit 0 (L42) | Read oracle L41-43 | ✅ `[verified]` L41 |
| 3 | Extract PR: `sed -nE 's/.*gh pr merge[[:space:]]+([0-9]+).*/\1/p' \| head -1` (L46); rỗng → exit 0 (L47) | Read oracle L46-47 | ✅ `[verified]` L46 |
| 4 | Override marker regex `\[security-review-skip:[^]]+\]` (L50); match → warning stderr (L52-53) + exit 0 (L54) | Read oracle L50-55 | ✅ `[verified]` L50-55 |
| 5 | Security-surface pattern verbatim L60 (xem Task 4 Lưu ý cho chuỗi đầy đủ); extend qua `SECURITY_SURFACE_EXTRA` append `\|$EXTRA` (L63-65) | Read oracle L60,L63-65 | ✅ `[verified]` L60,63-65 |
| 6 | `gh pr diff <N> --name-only` (L67); rỗng/fail → **fail-CLOSED BLOCK exit 2** (L68-83), message L71-81 | Read oracle L67-83 | ✅ `[verified]` L68-83 — DIVERGENCE |
| 7 | Pattern KHÔNG match → check `.env` non-example: `grep -E '^\.env' \| grep -v '\.env\.example'`; nếu cũng rỗng → exit 0 (L86-92) | Read oracle L86-93 | ✅ `[verified]` L86-93 |
| 8 | Touch surface → `gh pr view <N> --json comments --jq '.comments[].body'` (L96); có `<!-- security-review-start -->` → grep -A 50 → first `^Verdict:` line; chứa `APPROVE`→exit 0 (L100-103), else→BLOCK exit 2 (L104-116) | Read oracle L96-117 | ✅ `[verified]` L96-117 |
| 9 | Touch surface NHƯNG chưa có review block (`<!-- security-review-start -->` không có trong comments) → BLOCK exit 2, message L120-137 | Read oracle L119-137 | ✅ `[verified]` L119-137 |
| 10 | `src/io.rs`: `read_payload() -> HookPayload`; `ToolInput{ file_path, pattern, notebook_path }` serde `#[serde(default)]`; **THIẾU `command`** | `grep -n "struct ToolInput" -A8 src/io.rs` | ⏳ `[Quản-đốc-fed, Worker verify]` |
| 11 | `src/io.rs`: `ALLOW=0`, `BLOCK=2`, `block(reason)->i32` | `grep -nE "ALLOW\|BLOCK\|fn block" src/io.rs` | ⏳ `[Quản-đốc-fed, Worker verify]` |
| 12 | `src/hooks/mod.rs`: stub `block_unsafe_merge() -> i32` trả `ALLOW`, comment "not stdin payload" (SAI — sửa) | `grep -n "block_unsafe_merge" -A3 src/hooks/mod.rs` | ⏳ `[Quản-đốc-fed, Worker verify]` |
| 13 | Dep `regex` có sẵn (Cargo.toml:20), dùng được như P003 | `grep -n "^regex" Cargo.toml` | ⏳ `[Quản-đốc-fed, Worker verify]` |

**Anchor #6 = trái tim phiếu.** Nếu Worker thấy oracle KHÔNG fail-closed ở L68-83 → CHALLENGE ngay (nhưng Architect đã verify: L82 là `exit 2`).

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
# Branch là feat/P001-scaffold-cli (Phase 2 stack) → worktree basename có thể là P001.
# Override thủ công nếu cần: PHIEU_ID=P004
PHIEU_ID=P004
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Quản đốc Challenge (orchestrator có Read src/ — verify anchor src-side)

**Anchor verification (4/4 src ✅):**
- #10 ✅ `src/io.rs:6-9` — `ToolInput{file_path,pattern,notebook_path}`, KHÔNG có `command` (P004 thêm additive `#[serde(default)] command`)
- stub ✅ `src/hooks/mod.rs:148-150` — `block_unsafe_merge()` trả ALLOW (comment "not stdin payload" SAI, P004 sửa)
- regex dep ✅ `Cargo.toml:20` `regex = "1"`
- import ✅ `use crate::io::{self, ALLOW};` — Worker thêm BLOCK/block qualified như P002/P003

**Objections (Tầng 1):** None. Port trung thành oracle L24-137. Fail-CLOSED divergence (gh fail→BLOCK, ngược 3 hook kia) khóa 3 lớp (anchor #6 + Constraint #2 + Discovery) — verify đúng oracle L68-83. Logic lồng `.env` (Task 4): phân tích chuẩn — `\.env[^.]` KHÔNG bắt `.env.local` (theo sau dấu `.`) nên nhánh (b) `^\.env`+`grep -v example` bắt riêng. Pure-fn decomposition test được không cần gh; gh-paths manual/seam (không thêm mocking dep). gh shelling `std::process::Command` arg-vec (né injection, né shell).

**Status:** ✅ ACCEPTED V1 — no challenges.

### Final consensus
- Phiếu version: V1 (no debate turn needed)
- Approved by: Quản đốc (Sếp uỷ quyền approval gate, 2026-06-09) — EXECUTE may begin

---

## Nhiệm vụ

### Task 1: Thêm field `command` vào `ToolInput` (`src/io.rs`)

**File:** `src/io.rs`

**Tìm:** struct `ToolInput` (Worker grep `struct ToolInput`). Hiện có `file_path`, `pattern`, `notebook_path: Option<String>`, mỗi field `#[serde(default)]`.

**Thay bằng / Thêm:** thêm 1 field (additive, không đổi field cũ):
```rust
#[serde(default)]
pub command: Option<String>,
```

**Lưu ý:**
- Additive thuần — `#[serde(default)]` đảm bảo payload P002/P003 (không có `command`) vẫn parse OK → 24 test cũ KHÔNG vỡ.
- `command` là Bash-tool payload field (`{"tool_input":{"command":"gh pr merge 42"}}`).
- Worker verify exact struct layout (anchor #10) trước khi thêm — match đúng style `#[serde(default)]` field cũ.

### Task 2: Pure helper `parse_merge_pr` (`src/hooks/mod.rs`)

**File:** `src/hooks/mod.rs`

**Thêm:** pure function, KHÔNG cần gh:
```rust
fn parse_merge_pr(command: &str) -> Option<u32> { /* ... */ }
```

**Lưu ý (port oracle L41 + L46):**
- Trước tiên match `gh pr merge[[:space:]]+[0-9]+` (oracle L41). Không match → `None`.
- Extract first numeric sau `gh pr merge` (oracle L46 sed first-numeric + `head -1`).
- Dùng `regex::Regex` (P003 precedent). Pattern Rust: `gh pr merge\s+(\d+)` (`\s` ≈ `[[:space:]]`, `\d+` ≈ `[0-9]+`). Worker verify regex parity với oracle.
- **Known limitation (oracle L15-16):** branch-only form `gh pr merge --merge` (no number) BYPASS → `None`. Đây là CỐ Ý (oracle documents nó), KHÔNG phải bug. Test phải assert `None` cho case này.
- Trả `u32` (PR number). Lưu ý overflow: PR number > u32::MAX cực hiếm; nếu `parse::<u32>()` fail → `None` (an toàn, fail-through như rỗng → exit 0). Worker dùng `.ok()` không `unwrap`.

### Task 3: Pure helper `extract_skip_marker` (`src/hooks/mod.rs`)

**File:** `src/hooks/mod.rs`

**Thêm:**
```rust
fn extract_skip_marker(command: &str) -> Option<String> { /* reason string */ }
```

**Lưu ý (port oracle L50-51):**
- Regex `\[security-review-skip:([^]]+)\]` — capture reason giữa `:` và `]`.
- Rust regex: char class `[^\]]` (escape `]` trong class). Worker verify escape đúng.
- Trả `Some(reason)` nếu match, `None` nếu không.

### Task 4: Pure helper `touches_security_surface` (`src/hooks/mod.rs`)

**File:** `src/hooks/mod.rs`

**Thêm:**
```rust
fn touches_security_surface(files: &str, extra: Option<&str>) -> bool { /* ... */ }
```

**Lưu ý — ĐÂY LÀ LOGIC LỒNG (oracle L60 + L86-93), port cẩn thận:**
- `files` = output `gh pr diff --name-only` (newline-separated paths).
- **Base pattern (oracle L60 VERBATIM — copy nguyên, KHÔNG sửa ký tự nào):**
  ```
  src/|schema\.(prisma|sql)|migrations?/|nginx/|docker-compose.*\.yml|Dockerfile|\.env[^.]|middleware\.|lib/auth/|\.claude/agents/security-|docs/security/|scripts/security-gate|scripts/check-(hardcoded|runtime)-secrets|hooks/pre-commit
  ```
- **Extend (oracle L63-65):** nếu `extra` = `Some(s)` → append `|{s}` vào cuối pattern (như Bash `${PATTERN}|${EXTRA}`).
- **return `true` nếu:**
  - (a) base/extended pattern match bất kỳ dòng nào của `files` (oracle L86 `grep -qE PATTERN`), HOẶC
  - (b) pattern KHÔNG match NHƯNG có `.env` non-example file (oracle L88-89): dòng match `^\.env` mà KHÔNG match `\.env\.example`. Logic Bash: `grep -E '^\.env' | grep -v '\.env\.example'` → nếu non-empty → vẫn touch surface.
- **return `false`** chỉ khi cả (a) và (b) đều fail (oracle L89-91 `exit 0`).
- ⚠️ **Cẩn thận edge:** `.env[^.]` trong base pattern (oracle L60) match `.env` theo sau bởi non-dot (vd `.env\n`, `.envlocal`). `.env.example` KHÔNG match `\.env[^.]` (theo sau là `.`). Nhưng `.env.local` cũng KHÔNG match `\.env[^.]` (theo sau `.`)! → đó là LÝ DO có nhánh (b) bắt riêng `.env.local` qua `^\.env` + `grep -v example`. Port đúng cả 2 nhánh, đừng gộp ẩu. Worker verify từng test case Task 4 dưới.
- Regex multiline: oracle dùng `grep` per-line. Rust: hoặc dùng `(?m)` multiline + `^`, hoặc split `files` theo `\n` rồi match từng dòng. Worker quyết Tầng 2 (cách impl), miễn semantics khớp test cases.

### Task 5: Pure helper `verdict_is_approve` (`src/hooks/mod.rs`)

**File:** `src/hooks/mod.rs`

**Thêm:**
```rust
enum VerdictResult { NoBlock, Approve, NeedsReview }
fn verdict_is_approve(comments: &str) -> VerdictResult { /* ... */ }
```

**Lưu ý (port oracle L97-104):**
- `comments` = output `gh pr view --json comments --jq '.comments[].body'`.
- KHÔNG có `<!-- security-review-start -->` trong `comments` → `NoBlock` (oracle L97 if-false → rơi xuống L119 block path; xem Task 6 ráp).
- CÓ marker → lấy đoạn từ marker, grep first `^Verdict:` line (oracle L99 `grep -A 50 marker | grep -E '^Verdict:' | head -1`):
  - Verdict line chứa `APPROVE` (oracle L100) → `Approve`.
  - Else (NEEDS_REVIEW / unknown / không có Verdict line) → `NeedsReview`.
- **Lưu ý `grep -A 50`:** oracle giới hạn 50 dòng sau marker. Worker port: lấy slice từ marker tới +50 dòng (hoặc tới hết nếu < 50), tìm `^Verdict:` trong slice. Đơn giản hóa được nếu semantics khớp test — nhưng đừng bỏ giới hạn nếu gây sai (2 review block trong cùng comments → phải lấy verdict của block ĐẦU). Worker verify edge với test, DISCOVERY nếu giới hạn 50 ảnh hưởng.

### Task 6: Ráp `block_unsafe_merge()` real (`src/hooks/mod.rs`)

**File:** `src/hooks/mod.rs`

**Tìm:** stub hiện tại:
```rust
pub fn block_unsafe_merge() -> i32 {
    ALLOW // real logic in P004 (reads gh pr diff, not stdin payload)
}
```
(comment "not stdin payload" SAI — bỏ.)

**Thay bằng:** pipeline ráp các pure helper + 2 gh call (theo đúng thứ tự oracle):
1. `read_payload()` → lấy `tool_input.command`. `None`/empty → `return ALLOW` (oracle L31).
2. `parse_merge_pr(&command)` → `None` → `return ALLOW` (oracle L42,L47).
3. `extract_skip_marker(&command)` → `Some(reason)` → in warning stderr (oracle L52-53 verbatim, gồm `#$PR` + `Reason: $REASON`) → `return ALLOW` (oracle L54).
4. Đọc env `SECURITY_SURFACE_EXTRA` (std::env::var, `.ok()`).
5. **gh call #1:** `gh pr diff <PR> --name-only` (std::process::Command). Capture stdout. Nếu command fail HOẶC stdout rỗng → **fail-CLOSED:** in message oracle L71-81 verbatim (gồm `#$PR`, `$PR` interpolations) → `return BLOCK` (exit 2). ⚠️ **KHÔNG fail-open ở đây** — đây là DIVERGENCE so 3 hook kia.
6. `touches_security_surface(&diff_files, extra.as_deref())` → `false` → `return ALLOW` (oracle L91).
7. **gh call #2:** `gh pr view <PR> --json comments --jq '.comments[].body'`. Capture stdout (fail → empty string, oracle L96 `|| echo ""`).
8. `verdict_is_approve(&comments)`:
   - `Approve` → `return ALLOW` (oracle L102).
   - `NeedsReview` → in message oracle L106-115 verbatim (gồm `#$PR`, `$VERDICT_LINE`) → `return BLOCK` (oracle L116).
   - `NoBlock` → in message oracle L121-136 verbatim (gồm `#$PR`, `$PR`) → `return BLOCK` (oracle L137).

**Lưu ý:**
- **gh shelling:** dùng `std::process::Command::new("gh")`. KHÔNG dùng shell `sh -c` (tránh injection + đảm bảo cross-platform — Rust binary né Bash). Args truyền vec: `["pr","diff",&pr_str,"--name-only"]`.
- **Mã hóa PR cho gh:** truyền `PR` dạng string (`pr.to_string()`) cho gh args.
- **Tách seam testable:** Worker NÊN tách 1 thin layer cho 2 gh call (vd `fn gh_diff(pr) -> Option<String>` + `fn gh_view(pr) -> String`) để `block_unsafe_merge` core logic có thể test với injected closures NẾU Worker muốn. Nhưng **KHÔNG thêm mocking dep** — nếu cần seam phức tạp → DISCOVERY, default là pure helpers test riêng + gh-paths manual.
- **Message verbatim:** mọi block/warning message port NGUYÊN VĂN oracle (gồm emoji `⛔`/`⚠️`, xuống dòng, interpolation `$PR`/`$VERDICT_LINE`/`$REASON`). Đây là security-surface UI — Sếp đọc message này để quyết. Sai chữ = sai parity.
- **Lưu ý L133 oracle:** message NoBlock có literal `\$(gh pr view $PR --json url --jq .url)` (escaped `\$` trong heredoc) — in RA literal text `$(gh pr view <PR> --json url --jq .url)` chứ KHÔNG execute. Worker port: in literal string, KHÔNG shell-out. (Đây là gợi ý cho Sếp tự chạy, không phải hook chạy.)

### Task 7: Tests (`tests/cli.rs` + `#[cfg(test)] mod tests` trong mod.rs) — Verify-cò P057

**File:** `src/hooks/mod.rs` (unit, pure functions) + `tests/cli.rs` (integration, gh-free).

Xem section **Verify-cò** dưới cho danh sách test cases đầy đủ. BẮT BUỘC cùng phiếu (CLAUDE.md §Port doctrine #3 — build cò ≠ cò sống).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/io.rs` | Task 1: thêm `pub command: Option<String>` (`#[serde(default)]`) vào `ToolInput` |
| `src/hooks/mod.rs` | Task 2-6: 4 pure helpers + enum `VerdictResult` + ráp `block_unsafe_merge()` real; Task 7: `#[cfg(test)] mod tests` |
| `tests/cli.rs` | Task 7: integration test gh-free paths (echo/override/empty stdin → exit 0) |
| `docs/ARCHITECTURE.md` | Docs Gate: block-unsafe-merge "stub"→"real" + io.rs ToolInput thêm `command` |
| `CHANGELOG.md` | Entry P004 |
| `docs/discoveries/P004.md` | Discovery report (per-phiếu, P038) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `scripts/block-unsafe-merge.sh` | Oracle read-only — đối chiếu parity, KHÔNG sửa |
| `src/hooks/mod.rs` (P002/P003 fns) | `architect_guard`/`block_env_edit` KHÔNG đổi behavior — 24 test cũ PASS |
| `src/io.rs` (field cũ) | `file_path`/`pattern`/`notebook_path` parse KHÔNG vỡ sau khi thêm `command` |

---

## Luật chơi (Constraints)

1. **Port 1:1 — KHÔNG redesign** (CLAUDE.md §Port doctrine #1). Cùng exit code (0/2) + stderr message verbatim như oracle.
2. **Fail-CLOSED là CỐ Ý** — gh diff fail/rỗng → BLOCK exit 2 (oracle L82). KHÔNG đổi sang fail-open dù "an toàn hơn cho UX". Đây là điểm khác biệt cốt lõi của hook này.
3. **KHÔNG thêm dependency** — chỉ dùng `regex` (có sẵn) + `std::process::Command` + `std::env`. Mocking dep cần → DISCOVERY trước.
4. **Message verbatim** — emoji, xuống dòng, interpolation port nguyên văn oracle. Security-surface UI.
5. **io.rs change additive-only** — chỉ THÊM field, KHÔNG đổi/xóa field cũ. P002/P003 path bất biến.
6. **gh shelling không qua shell** — `Command::new("gh")` + args vec, KHÔNG `sh -c` (injection + cross-platform).
7. **No `unwrap()`/`expect()` trên parse/IO path** — fail-open cho parse (như io.rs harness), fail-CLOSED cho gh-verify (oracle semantics). PR parse fail → fail-through (exit 0).
8. **24 test cũ (P001-P003) PASS** — regression hard gate.

---

## Nghiệm thu

### Automated
- [ ] `cargo build` clean
- [ ] `cargo test --all` — unit (pure helpers) + integration (gh-free) mới PASS + **24 test cũ KHÔNG vỡ**
- [ ] `cargo clippy -- -D warnings` — zero warning

### Manual Testing (Quản đốc đối chiếu `bash scripts/block-unsafe-merge.sh` vs binary, gh-free cases)
- [ ] stdin `{"tool_input":{"command":"echo hi"}}` → exit 0 (cả Bash + Rust)
- [ ] stdin `{"tool_input":{"command":"gh pr merge 42 [security-review-skip:docs-only]"}}` → exit 0 + warning stderr (cả 2)
- [ ] empty stdin → exit 0 (cả 2)
- [ ] **(gh-dependent — manual only, đừng để CI gọi gh thật):** PR thật touch security surface chưa review → exit 2; gh auth tắt → fail-CLOSED exit 2. Đối chiếu message verbatim.

### Regression
- [ ] `architect-guard` + `block-env-edit` exit codes KHÔNG đổi (24 test cũ green)
- [ ] io.rs `read_payload()` parse payload KHÔNG có `command` field → OK (P002/P003 payload)

### Docs Gate (Tầng 1 — security-surface, BẮT BUỘC)
- [ ] `CHANGELOG.md` — entry P004 (block-unsafe-merge port, fail-CLOSED divergence note)
- [ ] `docs/ARCHITECTURE.md`:
  - block-unsafe-merge "stub (P001)"→"real (P004)" trong bảng Subcommands
  - thêm section `### block-unsafe-merge (P004 — real implementation)`: gh shelling 2-call + security-surface regex + verdict parse + override marker + **fail-CLOSED divergence note** (đối lập 3 hook fail-open)
  - stdin-JSON Harness: `ToolInput` thêm field `command`
  - Data Flow: block-unsafe-merge "still stub"→"real (gh-shelling)"

### Discovery Report
- [ ] Write `docs/discoveries/P004.md`:
  - Anchor #10-13 (io.rs/stub/regex): CORRECT / WRONG (file:line)
  - **Fail-CLOSED divergence (CỐ Ý)** — block-unsafe-merge fail-closed (exit 2 khi không verify), ĐỐI LẬP architect-guard/block-env-edit/session-banner fail-open. Document để hook tương lai không "sửa nhầm" thành fail-open.
  - **io.rs `command` field add** — shared-harness change, additive, P006/P007 reuse note.
  - **gh test strategy** — pure helpers unit-tested (no gh); gh-paths manual/seam. Worker ghi rõ: có tách seam không? cần mock dep không? (nếu DISCOVERY mock seam — note quyết định.)
  - env-fallback `CLAUDE_TOOL_INPUT` KHÔNG port (consistency P003 — anchor #1).
  - Stub comment P001 "not stdin payload" SAI → đã sửa.
  - `grep -A 50` verdict limit edge (nếu ảnh hưởng).
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`

---

## Verify-cò (P057 — fixture cùng phiếu)

**Thách thức:** gh-dependent paths khó test (cần network/auth/PR thật). Chiến lược 2 tầng:

### Unit test PURE functions (`#[cfg(test)] mod tests` — KHÔNG cần gh)

**`parse_merge_pr`:**
- [ ] `"gh pr merge 42 --squash"` → `Some(42)`
- [ ] `"gh pr merge 7 --merge --delete-branch"` → `Some(7)`
- [ ] `"gh pr merge --merge"` → `None` (branch-only bypass, oracle known-limitation L15-16, CỐ Ý)
- [ ] `"echo hi"` → `None`
- [ ] `"gh pr view 42"` → `None` (không phải merge)

**`extract_skip_marker`:**
- [ ] `"gh pr merge 5 [security-review-skip:docs-only]"` → `Some("docs-only")`
- [ ] `"gh pr merge 5 --merge"` → `None`
- [ ] `"[security-review-skip:gh-cli-unavailable]"` → `Some("gh-cli-unavailable")`

**`touches_security_surface`:**
- [ ] `"src/main.rs\nREADME.md"`, `None` → `true` (src/ match)
- [ ] `"README.md\ndocs/x.md"`, `None` → `false`
- [ ] `".env.local"`, `None` → `true` (nhánh b: ^\.env non-example — CHÚ Ý: `.env.local` KHÔNG match base `\.env[^.]` vì theo sau `.`, phải qua nhánh b)
- [ ] `".env.example"`, `None` → `false` (chỉ example → grep -v loại → không touch)
- [ ] `".env\n"`, `None` → `true` (base `\.env[^.]` HOẶC nhánh b)
- [ ] `"Dockerfile"`, `None` → `true`
- [ ] `"migrations/001.sql"`, `None` → `true` (`migrations?/`)
- [ ] `"hooks/pre-commit"`, `None` → `true`
- [ ] `"custom/secret.yml"`, `Some("custom/")` → `true` (extra extend)
- [ ] `"custom/secret.yml"`, `None` → `false` (không có extra → không match)

**`verdict_is_approve`:**
- [ ] marker + `"Verdict: APPROVE"` → `Approve`
- [ ] marker + `"Verdict: NEEDS_REVIEW"` → `NeedsReview`
- [ ] không marker → `NoBlock`
- [ ] marker nhưng không có Verdict line → `NeedsReview`

### Integration test CLI (assert_cmd, gh-free paths — `tests/cli.rs`)
- [ ] `{"tool_input":{"command":"echo hi"}}` → exit 0 (không phải merge)
- [ ] `{"tool_input":{"command":"gh pr merge 9 [security-review-skip:test]"}}` → exit 0 (override)
- [ ] empty stdin → exit 0
- [ ] `{"tool_input":{"command":"gh pr merge --merge"}}` → exit 0 (no number, bypass)

### gh-dependent paths (fail-CLOSED BLOCK, verdict) — MANUAL / seam
- ⚠️ **ĐỪNG để integration test gọi `gh` thật** → CI flaky/fail (no auth trong CI). Test gh-paths = MANUAL (Quản đốc đối chiếu) HOẶC Worker tách seam (injected gh-runner closure) NẾU muốn — nhưng **KHÔNG thêm mocking dep**. Cần seam phức tạp → DISCOVERY, Quản đốc quyết.

---

## Branch note

Phase 2 — Quản đốc quyết stack tiếp trên branch `feat/P001-scaffold-cli` (PR #1 đang chạy build). KHÔNG tách branch mới. Snapshot `.backup/P004/`. `git add` cả phiếu này (F-002). Khi xong → `phieu-done` move sang `docs/ticket/done/`.
