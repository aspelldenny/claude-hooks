# PHIẾU P001: Scaffold CLI — khung 5-subcmd + stdin-JSON harness

---

> **Loại:** Feature (scaffold — chưa port logic hook thật)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — đây là khung CLI dispatch + exit-code convention + fail-open semantics mà P002/P003/P004/P005 đều mọc lên trên. Sai exit-code convention hoặc fail-open default = LAN sang mọi subcmd sau. Security-surface contract → AUTO Tầng 1 dù P001 chỉ là stub.)
> **Ảnh hưởng:** `src/main.rs`, `src/` (module mới), `tests/` (integration), `CHANGELOG.md`, `docs/ARCHITECTURE.md`
> **Dependency:** None (phiếu đầu tiên của repo)

---

## Context

### Vấn đề hiện tại

`src/main.rs` hiện chỉ là stub `fn main(){ println!("Hello, world!"); }` `[needs Worker verify]` — src/ trống ngoài file này. Repo cần một **khung CLI** để các phiếu sau (P002 `architect-guard`, P003 `block-env-edit`, P004 `block-unsafe-merge`, P005 `session-banner`, P006 `serve`) port logic Bash vào mà không phải dựng lại scaffold.

P001 KHÔNG port logic hook thật. P001 chỉ dựng:
1. Khung `clap` derive với 5 subcommand registered.
2. Harness parse stdin JSON (Claude Code hook payload shape) bằng `serde_json`.
3. Exit-code convention (0 = allow, 2 = block, reason → stderr).
4. Mỗi subcmd = stub trả exit hợp lệ.

### Giải pháp

Tách scaffold thành 3 lớp module tối thiểu (đừng over-engineer — P002+ sẽ mọc thịt):

- **`src/main.rs`** — entry point + `clap` parse + dispatch sang subcmd. Mỏng.
- **`src/io.rs`** (shared harness) — model struct cho hook payload + helper đọc stdin + parse JSON fail-open + exit-code helper. Đây là phần MỌI subcmd dùng chung; đặt riêng để P002+ import.
- **`src/hooks/mod.rs`** + 1 hàm stub / subcmd — chứa entry function của từng hook (`architect_guard`, `block_env_edit`, `block_unsafe_merge`, `session_banner`) + `src/serve.rs` (hoặc `src/hooks/serve.rs`) cho MCP stub.

Worker tự quyết chi tiết tên file nội bộ (Tầng 2) miễn giữ đúng 3 ràng buộc cứng: 5 subcmd resolve, fail-open semantics, exit-code convention.

**Stdin JSON payload model** (shape 2 Bash script dùng — `architect-guard.sh` + `block-env-edit.sh`):

```json
{ "tool_input": { "file_path": "...", "pattern": "...", "notebook_path": "..." } }
```

Tất cả field optional. Harness PHẢI fail-open: stdin trống / JSON invalid / không có path field → **exit 0** (allow). Xem Luật chơi #1.

### Scope

- CHỈ tạo/sửa: `src/main.rs` (thay stub), module mới trong `src/`, integration tests trong `tests/`, `CHANGELOG.md`, `docs/ARCHITECTURE.md`.
- KHÔNG sửa: `Cargo.toml` (deps đã đủ — xem Task 0 #2), `scripts/*.sh` (Bash reference, read-only oracle), `docs/PROJECT.md`, `docs/BACKLOG.md`.
- KHÔNG port logic hook thật (marker file check, regex `.env`, gh pr diff, banner render) — đó là P002–P005.

---

## Task 0 — Verification Anchors

> Repo gần như trống src/ → phần lớn anchor là "❌ NOT FOUND — sẽ tạo mới". Đó là trung thực, không phải lỗi.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `src/main.rs` hiện là Hello-world stub (sẽ thay toàn bộ) | `cat src/main.rs` → expect `println!("Hello, world!")` | ⏳ `[needs Worker verify]` — Quản đốc feed là stub; nếu đã có nội dung khác, DISCOVERY_REPORT trước khi ghi đè |
| 2 | Cargo.toml đã có clap 4 (derive), serde+serde_json, anyhow, thiserror, regex, tokio, rmcp 1.7 — KHÔNG cần thêm dep | `grep -E '^(clap\|serde\|serde_json\|anyhow\|thiserror\|regex\|tokio\|rmcp)' Cargo.toml` | ⏳ `[needs Worker verify]` — Quản đốc xác nhận đủ; nếu thiếu `derive` feature trên clap → DISCOVERY (KHÔNG tự thêm dep mới, escalate) |
| 3 | Cargo.toml dev-deps có assert_cmd 2 + predicates 3 (cho integration test) | `grep -E 'assert_cmd\|predicates' Cargo.toml` | ⏳ `[needs Worker verify]` — Quản đốc xác nhận có |
| 4 | `docs/ticket/` tồn tại (nơi ghi phiếu này) | `ls docs/ticket/` | ✅ `[verified]` — Architect đã Glob, thư mục tồn tại (phiếu này nằm trong đó) |
| 5 | `tests/` directory cho integration test | `ls tests/` | ⏳ `[needs Worker verify]` — nếu chưa có, `cargo` chấp nhận tạo mới `tests/*.rs`; tạo mới |
| 6 | `scripts/architect-guard.sh` fail-open khi không parse được path (dòng ~44 `[ -z "$PATH_ARG" ] && exit 0`) | `grep -n 'exit 0' scripts/architect-guard.sh` | ⏳ `[oracle: grep]` `[needs Worker verify]` — đây là spec cho fail-open. Worker grep xác nhận semantics trước khi code harness |
| 7 | `scripts/block-env-edit.sh` input trống → exit 0; no path → exit 0 (dòng ~23, ~35) | `grep -n 'exit 0' scripts/block-env-edit.sh` | ⏳ `[oracle: grep]` `[needs Worker verify]` — spec fail-open thứ 2 |
| 8 | Binary name = `claude-hooks` (cho `assert_cmd::Command::cargo_bin`) | `grep -E '^name' Cargo.toml` (`[package]` hoặc `[[bin]]`) | ⏳ `[needs Worker verify]` — assert_cmd cần đúng tên binary; nếu khác `claude-hooks` → dùng tên thật |

**Lưu ý anchor #6, #7:** đây là 2 oracle spec cho fail-open. Worker grep được dòng `exit 0` tương ứng = đóng claim "Bash mirror fail-open". Đây là điều CỨNG nhất của phiếu — sai = mất parity (xem Luật chơi #1).

### Pre-phiếu snapshot (Worker auto first-step)

> **Worker EXECUTE FIRST ACTION** (trước mọi edit): rollback point.

```bash
# Run from project root (worktree root):
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
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

### Turn 1 — Quản đốc Challenge (CHALLENGE done by orchestrator — src/ trống, không có code thật để Worker grep mâu thuẫn; orchestrator có Bash/grep nên đóng anchor trực tiếp)

**Anchor verification (8/8 ✅):**
- #1 ✅ `src/main.rs` = `Hello, world!` stub (đã đọc trực tiếp)
- #2 ✅ Cargo.toml: clap `features=["derive"]` (L14), serde/serde_json/anyhow/thiserror/regex/tokio/rmcp 1.7 đủ
- #3 ✅ dev-deps assert_cmd 2 + predicates 3 có
- #4 ✅ `docs/ticket/` tồn tại
- #5 ✅ `tests/` chưa có → cargo tạo mới `tests/*.rs` OK (không phải lỗi)
- #6 ✅ `scripts/architect-guard.sh:44` `[ -z "$PATH_ARG" ] && exit 0` — fail-open oracle xác nhận
- #7 ✅ `scripts/block-env-edit.sh:23` `if [ -z "$INPUT" ]; then exit 0` — fail-open oracle xác nhận
- #8 ✅ binary name `claude-hooks` (`[package] name`, Cargo.toml:2) → `cargo_bin("claude-hooks")` đúng

**Objections (Tầng 1):** None. Phiếu trung thành Bash reference, scope bounded (chỉ scaffold), fail-open bake bằng type system (`serde(default)` + `Default` + `unwrap_or_default`, cấm panic). Verify-cò 8 integration test đủ.

**Status:** ✅ ACCEPTED V1 — no challenges.

### Final consensus
- Phiếu version: V1 (no debate turn needed)
- Approved by: Quản đốc (Sếp uỷ quyền approval gate, 2026-06-09) — code execution may begin

---

## Nhiệm vụ

### Task 1: Thay `src/main.rs` stub bằng clap entry + dispatch

**File:** `src/main.rs`

**Tìm:** nội dung stub hiện tại (`fn main()` in `Hello, world!`) `[needs Worker verify]`.

**Thay bằng:** entry point dùng `clap` derive với 1 enum 5 variant. Cấu trúc gợi ý (Worker quyết tên module nội bộ — Tầng 2):

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "claude-hooks", version, about = "Rust hooks for Claude Code")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Block Architect Read/Glob outside envelope
    ArchitectGuard,
    /// Block edit of .env* files
    BlockEnvEdit,
    /// Block gh pr merge without security APPROVE
    BlockUnsafeMerge,
    /// Render SessionStart banner
    SessionBanner,
    /// MCP server (stdio JSON-RPC)
    Serve,
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.cmd {
        Cmd::ArchitectGuard   => hooks::architect_guard(),
        Cmd::BlockEnvEdit     => hooks::block_env_edit(),
        Cmd::BlockUnsafeMerge => hooks::block_unsafe_merge(),
        Cmd::SessionBanner    => hooks::session_banner(),
        Cmd::Serve            => serve::run(),
    };
    std::process::exit(code);
}
```

**Lưu ý:**
- Subcmd name kebab-case: clap derive mặc định chuyển `ArchitectGuard` → `architect-guard`. Worker verify CLI resolve đúng `claude-hooks architect-guard` (không phải `architect_guard`). Nếu clap version sinh tên khác → thêm `#[command(name = "architect-guard")]` explicit cho từng variant.
- `serve` ở P001 là MCP stub — KHÔNG dựng tokio runtime thật. Worker quyết: in `"serve: not yet implemented"` ra stderr + **exit 0** (giữ fail-open + không làm CI đỏ khi smoke test toàn bộ subcmd). KHÔNG exit 1 — sẽ làm `assert_cmd` test "subcmd resolve" phải special-case. Mọi stub exit 0.
- Mỗi hook entry function trả `i32` (exit code) để `main` `process::exit`. KHÔNG để hook tự gọi `process::exit` (khó test) — return code lên main.

### Task 2: Module shared harness `src/io.rs` (stdin JSON + exit helper)

**File:** `src/io.rs` (tạo mới) `[needs Worker verify]` — Worker quyết tên (`io.rs` / `payload.rs`); giữ 1 module shared.

**Thêm:**
- Struct model cho payload (serde `Deserialize`), tất cả field `Option`:

```rust
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct ToolInput {
    pub file_path: Option<String>,
    pub pattern: Option<String>,
    pub notebook_path: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct HookPayload {
    #[serde(default)]
    pub tool_input: ToolInput,
}
```

- Helper đọc stdin + parse fail-open:

```rust
/// Đọc stdin, parse JSON. FAIL-OPEN: stdin trống / JSON invalid → Default (rỗng).
/// Mirror scripts/architect-guard.sh:44 + scripts/block-env-edit.sh:23,35 (anchor #6,#7).
pub fn read_payload() -> HookPayload {
    use std::io::Read;
    let mut buf = String::new();
    if std::io::stdin().read_to_string(&mut buf).is_err() {
        return HookPayload::default();
    }
    let buf = buf.trim();
    if buf.is_empty() {
        return HookPayload::default();
    }
    serde_json::from_str(buf).unwrap_or_default()
}
```

- Exit-code constants + (tuỳ chọn) helper in reason ra stderr:

```rust
pub const ALLOW: i32 = 0;
pub const BLOCK: i32 = 2;

/// Block với reason ra stderr (P002+ dùng). Trả BLOCK để caller return.
pub fn block(reason: &str) -> i32 {
    eprintln!("{reason}");
    BLOCK
}
```

**Lưu ý:**
- `serde(default)` + `Default` derive là chìa khoá fail-open: JSON thiếu `tool_input` hoặc field nào → `None`, không error. Đây là semantics CỨNG (Luật chơi #1).
- `unwrap_or_default()` trên parse = fail-open khi JSON invalid. KHÔNG `expect()`/`unwrap()` panic — panic = non-zero exit khác 2 = mất parity (fail-closed de facto).
- KHÔNG đọc stdin trong test bằng đường này (khó); integration test feed stdin qua `assert_cmd .write_stdin()`. Unit-test logic parse có thể tách hàm `parse_payload(&str) -> HookPayload` để test trực tiếp string — Worker quyết.

### Task 3: Module `src/hooks/` — 4 hook stub

**File:** `src/hooks/mod.rs` (tạo mới) `[needs Worker verify]` — Worker quyết tách 1 file/hook hay gộp; giữ minimal cho scaffold.

**Thêm:** 4 entry function stub, mỗi cái đọc payload (để verify harness wire đúng) rồi **return ALLOW (0)**:

```rust
use crate::io::{self, ALLOW};

pub fn architect_guard() -> i32 {
    let _payload = io::read_payload(); // wire harness; logic thật ở P002
    ALLOW
}

pub fn block_env_edit() -> i32 {
    let _payload = io::read_payload(); // logic thật ở P003
    ALLOW
}

pub fn block_unsafe_merge() -> i32 {
    ALLOW // logic thật ở P004 (đọc gh pr diff, không qua stdin payload)
}

pub fn session_banner() -> i32 {
    ALLOW // logic thật ở P005 (render banner)
}
```

**Lưu ý:**
- `architect_guard` + `block_env_edit` GỌI `read_payload()` (dù chưa dùng) để chứng minh harness wire đúng — đây là 2 hook đọc stdin JSON ở P002/P003. `_payload` prefix `_` tránh clippy `unused`.
- `block_unsafe_merge` + `session_banner` KHÔNG đọc stdin payload (Bash gốc đọc `gh pr diff` / render từ git state, không từ tool_input) → stub không gọi `read_payload`. Đừng wire sai.
- TẤT CẢ return ALLOW (0). Stub không bao giờ block — block logic mọc ở phiếu sau.

### Task 4: Module `src/serve.rs` — MCP stub

**File:** `src/serve.rs` (tạo mới) `[needs Worker verify]`.

**Thêm:**

```rust
/// MCP server stub. Logic thật (rmcp stdio + tools) ở P006.
pub fn run() -> i32 {
    eprintln!("serve: not yet implemented (P006)");
    crate::io::ALLOW // exit 0 — đừng làm CI/smoke đỏ
}
```

**Lưu ý:**
- KHÔNG import/khởi tokio runtime ở P001 — chưa cần, thêm phức tạp vô ích. rmcp/tokio wire ở P006.
- Stderr message + exit 0 (đã quyết ở Task 1 Lưu ý): nhất quán "stub exit 0".

### Task 5: Declare modules trong entry

**File:** `src/main.rs` (cùng file Task 1, phần đầu).

**Thêm:** module declarations:

```rust
mod io;
mod hooks;
mod serve;
```

**Lưu ý:** Worker điều chỉnh theo tên module thực đã chọn ở Task 2-4. Nếu dùng `src/lib.rs` thay vì khai trong `main.rs` → Worker quyết (Tầng 2), miễn `cargo build` clean + binary resolve subcmd.

### Task 6: Integration tests (verify-cò P057)

**File:** `tests/cli.rs` (tạo mới) `[needs Worker verify]`.

**Thêm:** integration test dùng `assert_cmd` + `predicates`. Tối thiểu các case (xem Nghiệm thu để khớp):

```rust
use assert_cmd::Command;

fn bin() -> Command {
    Command::cargo_bin("claude-hooks").unwrap() // verify binary name = anchor #8
}

#[test]
fn architect_guard_empty_stdin_allows() {
    bin().arg("architect-guard").write_stdin("").assert().code(0);
}

#[test]
fn block_env_edit_empty_stdin_allows() {
    bin().arg("block-env-edit").write_stdin("").assert().code(0);
}

#[test]
fn block_unsafe_merge_resolves() {
    bin().arg("block-unsafe-merge").assert().code(0);
}

#[test]
fn session_banner_resolves() {
    bin().arg("session-banner").assert().code(0);
}

#[test]
fn serve_resolves() {
    bin().arg("serve").assert().code(0);
}

#[test]
fn harness_parses_valid_json_no_panic() {
    bin()
        .arg("architect-guard")
        .write_stdin(r#"{"tool_input":{"file_path":"x"}}"#)
        .assert()
        .code(0);
}

#[test]
fn harness_invalid_json_fails_open() {
    bin()
        .arg("architect-guard")
        .write_stdin("{not valid json")
        .assert()
        .code(0); // fail-open — KHÔNG panic, KHÔNG exit 2
}

#[test]
fn unknown_subcommand_errors() {
    bin().arg("nonexistent-cmd").assert().failure(); // clap reject, exit != 0
}
```

**Lưu ý:**
- `unknown_subcommand_errors` xác nhận clap reject subcmd lạ (clap exit code 2 cho usage error — đây là clap's convention, KHÁC với hook BLOCK=2. Test chỉ assert `.failure()`, đừng assert `.code(2)` để tránh nhầm 2 nghĩa).
- `harness_invalid_json_fails_open` là test PARITY quan trọng nhất — verify fail-open thật sự xảy ra ở binary level.
- Worker verify binary name từ anchor #8 trước khi viết `cargo_bin("claude-hooks")`.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/main.rs` | Task 1+5: thay stub bằng clap entry + dispatch + mod declarations |
| `src/io.rs` (mới) | Task 2: HookPayload model + read_payload fail-open + exit constants |
| `src/hooks/mod.rs` (mới) | Task 3: 4 hook stub return ALLOW |
| `src/serve.rs` (mới) | Task 4: MCP stub return ALLOW + stderr message |
| `tests/cli.rs` (mới) | Task 6: integration tests (verify-cò) |
| `CHANGELOG.md` | Entry cho P001 |
| `docs/ARCHITECTURE.md` | Section CLI surface: 5 subcmd + harness + exit convention |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `Cargo.toml` | Deps đã đủ (anchor #2,#3,#8); KHÔNG thêm dep. Nếu thiếu → escalate, đừng tự thêm |
| `scripts/architect-guard.sh` | Read-only oracle — verify fail-open dòng `exit 0` (anchor #6), KHÔNG sửa |
| `scripts/block-env-edit.sh` | Read-only oracle — verify fail-open (anchor #7), KHÔNG sửa |
| `docs/PROJECT.md`, `docs/BACKLOG.md` | KHÔNG sửa (BACKLOG update khi phiếu DONE, không phải lúc EXECUTE) |

---

## Luật chơi (Constraints)

1. **FAIL-OPEN là CỨNG (parity).** stdin trống / JSON parse fail / không tìm thấy path field → **exit 0 (allow)**. KHÔNG fail-closed. Mirror 2 Bash oracle: `scripts/architect-guard.sh` (`[ -z "$PATH_ARG" ] && exit 0`, anchor #6) + `scripts/block-env-edit.sh` (input trống → exit 0, no path → exit 0, anchor #7). Cấm `unwrap()`/`expect()` panic trên parse stdin — panic = non-zero exit ≠ 2 = de-facto fail-closed = chặn oan = mất parity.
2. **Exit-code convention:** 0 = allow, 2 = block (reason → **stderr**, không stdout). P001 stub luôn return 0; convention (`ALLOW`/`BLOCK` constants) dựng sẵn cho P002+.
3. **TRUNG THÀNH Bash reference, KHÔNG redesign** (CLAUDE.md §Port doctrine). P001 chưa port logic nên ràng buộc này chủ yếu áp vào exit-code + fail-open semantics. KHÔNG "tiện tay" thêm flag/behavior ngoài scope.
4. **KHÔNG thêm dependency.** Cargo.toml đã đủ (anchor #2). Nếu Worker thấy thiếu → DISCOVERY_REPORT + escalate, KHÔNG tự `cargo add`.
5. **Stub luôn exit 0** (kể cả `serve`). Không stub nào exit 1/2. Lý do: smoke "mọi subcmd resolve" phải pass đồng nhất; block logic mọc ở phiếu sau.
6. **Hook entry trả `i32`, main gọi `process::exit`.** KHÔNG để hook tự `process::exit` (khó unit-test).
7. **MINIMAL scaffold.** Đừng over-engineer (trait abstraction, plugin registry, config loader). 3 module: entry / shared harness / hook stubs. P002+ mọc thịt.
8. **Edition 2024, MSRV 1.85** — code phải build trên 1.85 (đừng dùng feature stable sau 1.85).

---

## Nghiệm thu

### Automated
- [ ] `cargo build` clean (no warning).
- [ ] `cargo clippy -- -D warnings` không warning (chú ý `unused` trên `_payload`).
- [ ] `cargo test --all` pass — tối thiểu 8 integration test (Task 6):
  - [ ] `architect-guard` + stdin rỗng → exit 0
  - [ ] `block-env-edit` + stdin rỗng → exit 0
  - [ ] `block-unsafe-merge` resolve → exit 0
  - [ ] `session-banner` resolve → exit 0
  - [ ] `serve` resolve → exit 0
  - [ ] harness parse `{"tool_input":{"file_path":"x"}}` không panic → exit 0
  - [ ] harness JSON invalid `{not valid json` → fail-open exit 0 (KHÔNG panic, KHÔNG 2)
  - [ ] unknown subcmd → clap reject (`.failure()`)

### Manual Testing
- [ ] `echo '' | cargo run -- architect-guard` → exit 0 (`echo $?`).
- [ ] `echo '{"tool_input":{"file_path":"src/x.rs"}}' | cargo run -- block-env-edit` → exit 0 (stub chưa block).
- [ ] `cargo run -- serve` → in "serve: not yet implemented (P006)" ra stderr + exit 0.
- [ ] `cargo run -- --help` → liệt kê đủ 5 subcmd kebab-case.

### Regression
- [ ] N/A — repo chưa có code production (P001 là phiếu đầu). Chỉ cần `cargo build` từ stub → khung mới clean.

### Docs Gate
- [ ] `CHANGELOG.md` — entry P001 (scaffold CLI 5-subcmd + harness + exit convention).
- [ ] `docs/ARCHITECTURE.md` — section CLI surface: liệt kê 5 subcmd, stdin-JSON harness shape, exit-code convention (0/2), fail-open semantics. (Per CLAUDE.md DOCS GATE: subcmd add → ARCHITECTURE.md.)

### Discovery Report
- [ ] Write to `docs/discoveries/P001.md` (per-phiếu file):
  - Anchor #1–#8 — CORRECT / WRONG (file:line citations). Đặc biệt: `src/main.rs` stub đúng dạng? Cargo deps đủ? Binary name = `claude-hooks`?
  - Tên module thực Worker chọn (io.rs? payload.rs? hooks/ tách mấy file?) — ghi lại cho P002+ reference.
  - Clap kebab-case auto đúng không, hay phải `#[command(name=...)]` explicit?
  - Scope expansions (nếu có — original vs shipped + lý do).
  - Docs updated: ARCHITECTURE.md section nào.
  - Tier escalations (write "None" nếu không có).
- [ ] Append 1-line index entry vào `docs/DISCOVERIES.md` (link tới `docs/discoveries/P001.md`).
