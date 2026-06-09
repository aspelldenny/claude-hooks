# PHIẾU P002: `architect-guard` subcmd (port 1:1 từ Bash oracle)

> **ID:** P002 · **Filename:** `docs/ticket/P002-architect-guard.md`
> **Branch:** `feat/P002-architect-guard`

---

> **Loại:** Feature (port)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — hook = security-surface: chặn/cho action. CLAUDE.md DOCS GATE: "Security-surface → AUTO Tầng 1". Sai exit code = Architect lọt đọc src/ HOẶC chặn nhầm docs. LOC nhỏ KHÔNG hạ Tầng.)
> **Ảnh hưởng:** `src/hooks/mod.rs` (thay stub `architect_guard()`), `tests/cli.rs` (fire-test fixtures)
> **Dependency:** P001 (đã ship — harness `src/io.rs` + stub có sẵn)

---

## Context

### Vấn đề hiện tại

`src/hooks/mod.rs::architect_guard()` hiện là stub trả `ALLOW` vô điều kiện (P001 scaffold). Hook Bash oracle `scripts/architect-guard.sh` (87 dòng) là cổng chặn Architect đọc source code khi marker `.sos-state/architect-active` tồn tại. Cần port logic này 1:1 sang Rust để đạt CLI parity (PROJECT.md Success #1 · BACKLOG Active sprint Phase 1).

### Giải pháp

Thay body stub bằng logic port TRUNG THÀNH từ oracle (CLAUDE.md §Port doctrine #1-2: cùng exit code + semantics, KHÔNG redesign). Pipeline 8 bước theo oracle:

1. Resolve repo root từ env `CLAUDE_PROJECT_DIR` (fallback cwd) — marker & path bind vào repo root, không phụ thuộc cwd của caller.
2. Marker gate: `.sos-state/architect-active` không tồn tại → `ALLOW` (cổng đầu — không marker = không phải Architect).
3. Đọc path: `read_payload().tool_input.file_path`, None → fallback `pattern`.
4. Không path → `ALLOW` (fail-open, oracle L44).
5. Strip leading `./`.
6. `.md` bất kỳ vị trí → `ALLOW` (docs là domain Architect).
7. Forbidden patterns (path-prefix/segment + test dirs + build artifacts + extensions) → `BLOCKED`. Default → `ALLOW`.
8. Blocked → `block(<message tiếng Anh nguyên văn oracle L71-81>)` → trả `BLOCK` (exit 2).

### Scope

- CHỈ sửa `src/hooks/mod.rs` (thân hàm `architect_guard`) + `tests/cli.rs` (thêm fire-test).
- KHÔNG sửa `src/io.rs` (API đã đủ — chỉ import thêm `BLOCK`/`block` nếu cần), `src/main.rs` (dispatch không đổi), Cargo.toml (KHÔNG thêm dep — xem Constraint 5).

---

## Task 0 — Verification Anchors

> Oracle (`scripts/architect-guard.sh`): Architect ĐỌC ĐƯỢC file `.sh` (guard không chặn) → `[verified]` line-thật.
> src/ (io.rs API, hooks/mod.rs stub): Architect KHÔNG đọc được `src/*.rs` (guard chặn) → `[Quản-đốc-fed, Worker verify]`. P001 Discovery (`docs/discoveries/P001.md`) corroborate các anchor io.rs.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Marker gate: `.sos-state/architect-active` không tồn tại → exit 0 | Oracle L28 `[ -f "$MARKER_FILE" ] \|\| exit 0` | ✅ `[verified]` L25,28 |
| 2 | Path parse: `file_path` ưu tiên, fallback `pattern` | Oracle L38-41 | ✅ `[verified]` L38-41 |
| 3 | No path → exit 0 (fail-open) | Oracle L44 `[ -z "$PATH_ARG" ] && exit 0` | ✅ `[verified]` L44 |
| 4 | Strip leading `./` | Oracle L47 `${PATH_ARG#./}` | ✅ `[verified]` L47 |
| 5 | `.md` bất kỳ → exit 0 | Oracle L50-52 `case ... *.md) exit 0` | ✅ `[verified]` L50-52 |
| 6 | Forbidden path-prefix/segment set | Oracle L57-58 | ✅ `[verified]` L57-58 (src/lib/app/crates*/src/pkg + */ variants) |
| 7 | Forbidden test-dir set | Oracle L59-60 | ✅ `[verified]` L59-60 (tests/test/__tests__ + */ variants) |
| 8 | Forbidden build-artifact set | Oracle L61-62 | ✅ `[verified]` L61-62 (node_modules/target/dist/build/.next/.nuxt/.svelte-kit) |
| 9 | Forbidden extension set | Oracle L63-64 | ✅ `[verified]` L63-64 (rs/ts/tsx/js/jsx/py/go/java/cpp/c/h/hpp) |
| 10 | Default (không match) → exit 0 | Oracle L65-66 `*) BLOCKED=0` + L83-86 | ✅ `[verified]` L65-66,85-86 |
| 11 | Block message tiếng Anh + exit 2 | Oracle L69-83 heredoc + `exit 2` | ✅ `[verified]` L69-83 (text verbatim trong Task 2) |
| 12 | `read_payload() -> HookPayload`, `tool_input: ToolInput{file_path,pattern,notebook_path: Option<String>}` | Worker grep `pub fn read_payload\|pub struct ToolInput` trong `src/io.rs` | ⏳ `[Quản-đốc-fed, Worker verify]` — P001 Discovery #2,6 corroborate |
| 13 | `ALLOW=0`, `BLOCK=2`, `fn block(reason)->i32` (eprintln+trả BLOCK) tồn tại trong `src/io.rs` | Worker grep `pub const ALLOW\|pub const BLOCK\|pub fn block` trong `src/io.rs` | ⏳ `[Quản-đốc-fed, Worker verify]` — P001 Discovery L40 confirm forward-declared |
| 14 | Stub hiện tại: `architect_guard()` body = `let _payload = io::read_payload(); ALLOW` với `use crate::io::{self, ALLOW};` | Worker grep `fn architect_guard` trong `src/hooks/mod.rs` | ⏳ `[Quản-đốc-fed, Worker verify]` |
| 15 | `tests/cli.rs` tồn tại (8 test P001), dùng `assert_cmd` | Worker grep `assert_cmd\|Command::cargo_bin` trong `tests/cli.rs` | ⏳ `[Quản-đốc-fed, Worker verify]` — ARCHITECTURE.md L53 + P001 Discovery confirm |

**Không có ❌.** Anchor #12-15 là `src/` mà Architect bị guard chặn → Worker verify lúc EXECUTE (đúng envelope, không bịa line number).

### Pre-phiếu snapshot (Worker auto first-step)

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

**Phiếu version:** V1 (initial draft)

### Turn 1 — Quản đốc Challenge (orchestrator có Read src/ sau khi clear marker — verify anchor #12-15 trực tiếp)

**Anchor #12-15 verification (4/4 ✅):**
- #12 ✅ `src/io.rs:6-8,23` — `read_payload()`, `ToolInput{file_path,pattern,notebook_path: Option<String>}` khớp feed
- #13 ✅ `src/io.rs:36,39,44` — `ALLOW=0`, `BLOCK=2`, `pub fn block(reason)->i32`
- #14 ✅ `src/hooks/mod.rs:1,3-6` — stub body = `let _payload = io::read_payload(); ALLOW`, import `use crate::io::{self, ALLOW};` (Worker thêm `BLOCK`/`block`)
- #15 ✅ `tests/cli.rs` — 8 test P001, dùng `Command::cargo_bin` + `.write_stdin` + `.assert().code()`

**Objections (Tầng 1):** None. Port 1:1 trung thành oracle L56-83; forbidden semantics chuẩn (`__tests__`/build-artifact prefix-only — không tự thêm contains); test isolation qua `CLAUDE_PROJECT_DIR` temp (không race marker thật); env fallback divergence (script-dir→cwd) flag trung thực, chấp nhận (CLAUDE_PROJECT_DIR luôn set khi Claude Code fire hook).

**Status:** ✅ ACCEPTED V1 — no challenges.

### Final consensus
- Phiếu version: V1 (no debate turn needed)
- Approved by: Quản đốc (Sếp uỷ quyền approval gate, 2026-06-09) — EXECUTE may begin

---

## Nhiệm vụ

### Task 1: Port logic `architect_guard()`

**File:** `src/hooks/mod.rs` `[needs Worker verify]` — Worker grep `fn architect_guard` xác nhận đúng file/fn (anchor #14).

**Tìm:** thân hàm stub:
```rust
pub fn architect_guard() -> i32 {
    let _payload = io::read_payload();
    ALLOW
}
```

**Thay bằng:** logic port 8 bước. Pseudo-spec (Worker giữ semantics, tự chọn idiom Rust — Tầng 2 nội bộ: tên local var / dùng `match` hay `if` / helper inline):

```rust
pub fn architect_guard() -> i32 {
    // Bước 1 — resolve repo root từ CLAUDE_PROJECT_DIR (fallback cwd).
    //   Oracle L23: cd "${CLAUDE_PROJECT_DIR:-<script dir>/..}". Rust: nếu env set,
    //   join marker path lên base đó; nếu không, dùng cwd (path tương đối như hiện tại).
    //   Worker quyết: std::env::var("CLAUDE_PROJECT_DIR") rồi PathBuf::join,
    //   HAY set_current_dir. Giữ semantics: marker + (gián tiếp) path bind repo root.
    //   LƯU Ý: oracle fallback = SCRIPT dir, không phải cwd. Binary Rust không có
    //   "script dir" tương đương → fallback hợp lý = cwd (Claude Code fire hook từ
    //   project root khi CLAUDE_PROJECT_DIR unset là hiếm). Worker DISCOVERY ghi rõ
    //   divergence này (oracle: script-dir fallback; Rust: cwd fallback) — đây là
    //   khác biệt KHÔNG tránh được do binary ≠ script, không phải redesign.

    // Bước 2 — marker gate. Oracle L28.
    //   if !<repo_root>/.sos-state/architect-active EXISTS { return ALLOW; }

    // Bước 3 — đọc path. Oracle L38-41.
    let payload = io::read_payload();
    //   let path = payload.tool_input.file_path
    //                  .or(payload.tool_input.pattern);  // file_path ưu tiên, fallback pattern

    // Bước 4 — no path → ALLOW. Oracle L44.
    //   let path = match path { Some(p) => p, None => return ALLOW };

    // Bước 5 — strip leading "./". Oracle L47.
    //   let norm = path.strip_prefix("./").unwrap_or(&path);

    // Bước 6 — .md anywhere → ALLOW. Oracle L50-52.
    //   if norm.ends_with(".md") { return ALLOW; }
    //   LƯU Ý: oracle case "*.md" = "kết thúc bằng .md" (glob suffix), KHÔNG phải
    //   "chứa .md". ends_with(".md") là port đúng.

    // Bước 7 — forbidden match (xem Task 1b spec bảng pattern). Default → ALLOW.

    // Bước 8 — blocked → io::block(MESSAGE) (eprintln + trả BLOCK). Oracle L69-83.
    ALLOW
}
```

**Lưu ý:**
- Import: stub hiện có `use crate::io::{self, ALLOW};`. Worker thêm `BLOCK` và/hoặc `block` nếu dùng (anchor #13). Khuyến nghị dùng `io::block(MESSAGE)` (đã eprintln + trả BLOCK) thay vì tự `eprintln!` rồi `BLOCK` — đỡ lệch convention P001 (ARCHITECTURE.md L41: "Block reason → stderr only").
- Block message giữ NGUYÊN VĂN oracle (kể cả emoji 🚫). Oracle dùng `$PATH_ARG` (path GỐC, chưa strip `./`) trong message L73,78 — Worker dùng biến path gốc, KHÔNG dùng `norm`.

### Task 1b: Port forbidden-pattern set (semantics glob → Rust string match)

**File:** `src/hooks/mod.rs` (trong Bước 7 của Task 1).

**Spec port** — oracle dùng POSIX `case` glob (L56-67). Quy ước port:
- `X/*` (oracle, vd `src/*`) = "path **bắt đầu bằng** `X/`" → Rust `norm.starts_with("X/")`.
- `*/X/*` (oracle, vd `*/src/*`) = "path **chứa** `/X/`" → Rust `norm.contains("/X/")`.
- `crates/*/src/*` = bắt đầu `crates/` VÀ chứa `/src/` → `norm.starts_with("crates/") && norm.contains("/src/")`. (Worker chọn cách biểu diễn miễn fixture pass; gợi ý đơn giản: `norm.contains("/src/")` đã bắt `crates/x/src/...`; cộng `starts_with("src/")` bắt root-level. Cặp này phủ cả `crates/*/src/*`.)
- `*.ext` = `norm.ends_with(".ext")`.

**Bộ pattern port CHÍNH XÁC (oracle L57-64) → BLOCKED:**

| Nhóm | Oracle glob | Rust check |
|---|---|---|
| Source dirs (prefix) | `src/*`, `lib/*`, `app/*`, `pkg/*` | `starts_with("src/"/"lib/"/"app/"/"pkg/")` |
| Source dirs (segment) | `*/src/*`, `*/lib/*`, `*/app/*`, `*/pkg/*` | `contains("/src/"/"/lib/"/"/app/"/"/pkg/")` |
| Crates src | `crates/*/src/*` | covered bởi `contains("/src/")` (xem trên) |
| Test dirs (prefix) | `tests/*`, `test/*`, `__tests__/*` | `starts_with("tests/"/"test/"/"__tests__/")` |
| Test dirs (segment) | `*/tests/*`, `*/test/*` | `contains("/tests/"/"/test/")` |
| Build artifacts (prefix) | `node_modules/*`, `target/*`, `dist/*`, `build/*`, `.next/*`, `.nuxt/*`, `.svelte-kit/*` | `starts_with(...)` cho mỗi |
| Extensions (suffix) | `*.rs *.ts *.tsx *.js *.jsx *.py *.go *.java *.cpp *.c *.h *.hpp` | `ends_with(".rs")` ... cho mỗi |

**Lưu ý port semantics (rủi ro #1):**
- `__tests__/` trong oracle CHỈ có biến thể prefix (`__tests__/*`), KHÔNG có `*/__tests__/*`. Port đúng: chỉ `starts_with("__tests__/")`. ĐỪNG tự thêm `contains("/__tests__/")` (đó là redesign — oracle L59 không có).
- Build artifacts trong oracle CHỈ prefix (`target/*` ...), KHÔNG segment (`*/target/*`). Port đúng: chỉ `starts_with`. ĐỪNG thêm contains.
- `test/` prefix khác `tests/` — port cả hai riêng (oracle L59).
- Quan trọng: order không đổi kết quả (BLOCKED là OR của mọi nhóm) — Worker tự chọn `if A || B || ... { BLOCK }`. Default (không nhánh nào khớp) = ALLOW (oracle L65-66).
- Worker QUYẾT dùng string match (`starts_with`/`contains`/`ends_with`) — đủ và rõ. Dep `regex` CÓ trong Cargo.toml nhưng KHÔNG cần ở đây; ưu tiên std string match (đơn giản = ít sai). Đây là Tầng 2 (cách biểu diễn nội bộ), miễn fixture pass + semantics khớp oracle.

### Task 2: Block message (nguyên văn oracle L71-81)

**File:** `src/hooks/mod.rs` — const string truyền vào `io::block(...)`.

**Thêm** const (Worker chọn tên, gợi ý `BLOCK_MSG`), nội dung NGUYÊN VĂN heredoc oracle L71-81 (path gốc nội suy):

```
🚫 Architect envelope violation

Architect cannot read source code: {PATH_ARG}

What to do instead: write a Task 0 anchor in the phiếu.
Example:
  | # | Assumption | Verify by | Result |
  | 1 | <claim about {PATH_ARG}> | grep ... {PATH_ARG} | ⏳ TO VERIFY |

Worker (separate subagent) will grep-verify it for you. The constraint IS the feature.
```

**Lưu ý:**
- `{PATH_ARG}` = path GỐC (chưa strip `./`) — oracle dùng `$PATH_ARG` không phải `$NORMALIZED_PATH` ở L73,78. Worker format-string với biến path gốc.
- `io::block()` đã `eprintln!` → message ra **stderr**, không stdout (ARCHITECTURE.md L41). Worker KHÔNG `println!`.
- Giữ emoji 🚫 và xuống dòng y hệt. Đây là CLI parity surface (test fixture có thể assert substring "Architect envelope violation").

### Task 3: Fire-test fixtures (P057 — verify-cò BẮT BUỘC cùng phiếu)

**File:** `tests/cli.rs` `[needs Worker verify]` — Worker grep cấu trúc test P001 (`Command::cargo_bin`, `.write_stdin`, `.assert().code(...)`) để mirror style (anchor #15).

**Test isolation (rủi ro #2 — ĐỌC KỸ, ĐỪNG TỰ CHẾ):**

Marker `.sos-state/architect-active` là **state toàn cục của repo**. Test KHÔNG được:
- Tạo/xoá marker thật của session đang chạy (race với Quản đốc/Worker đang active).
- Để 2 test đua nhau trên cùng marker (cargo test chạy song song theo default).

**Chiến lược isolation BẮT BUỘC — dùng `CLAUDE_PROJECT_DIR` env trỏ temp dir per-test (KHÔNG dùng marker thật):**

Oracle L23 + Bước 1 của port: hook resolve repo root từ env `CLAUDE_PROJECT_DIR`. Test set env này trỏ tới thư mục tạm RIÊNG mỗi case, rồi tạo/không-tạo `.sos-state/architect-active` TRONG temp đó. Như vậy:
- Mỗi test có repo-root ảo riêng → không đụng `.sos-state/` thật của repo.
- `assert_cmd::Command::cargo_bin(...).env("CLAUDE_PROJECT_DIR", <temp>)` set env per-process → không race (mỗi test process độc lập).

Tạo temp dir: dùng `std::env::temp_dir()` + unique subdir (vd tên test + `std::process::id()`), `std::fs::create_dir_all`, cleanup cuối test (best-effort `fs::remove_dir_all`). **KHÔNG thêm crate `tempfile`** — chưa có trong Cargo.toml; nếu Worker thấy std không đủ tiện → DISCOVERY + escalate Quản đốc, ĐỪNG tự thêm dep (Constraint 5).

> **Phụ thuộc port Bước 1:** isolation này CHỈ hoạt động nếu Task 1 Bước 1 thực sự đọc `CLAUDE_PROJECT_DIR` để resolve marker path. Worker phải implement Bước 1 theo env (không hardcode cwd-relative marker), nếu không test không isolate được. Nếu Worker chọn `set_current_dir` thay join: vẫn set `CLAUDE_PROJECT_DIR` + đảm bảo marker đọc dưới đó. Ghi rõ cách chọn trong Discovery.

**Fixture set (6 case — mirror BACKLOG P002 spec):**

| # | Setup (trong temp CLAUDE_PROJECT_DIR) | stdin JSON | Expect |
|---|---|---|---|
| 1 | marker present | `{"tool_input":{"file_path":"src/main.rs"}}` | exit 2 |
| 2 | marker present | `{"tool_input":{"file_path":"README.md"}}` | exit 0 |
| 3 | marker present | `{"tool_input":{"pattern":"src/**/*.rs"}}` | exit 2 |
| 4 | marker present | `{"tool_input":{"file_path":"docs/x.txt"}}` | exit 0 |
| 5 | **no marker** | `{"tool_input":{"file_path":"src/main.rs"}}` | exit 0 (cổng marker) |
| 6 | marker present | *(empty stdin)* | exit 0 (fail-open) |

**Lưu ý:**
- Case 3 (`src/**/*.rs`): path qua `pattern` (file_path None) → strip `./` không đổi → `starts_with("src/")` → BLOCK. Verify path-fallback (anchor #2) + glob string không phá match (chỉ cần prefix `src/`).
- Case 4 `docs/x.txt`: không phải `.md`, không match forbidden (docs/ không trong set) → ALLOW. Verify default-allow.
- Case 2 README.md: `.md` → ALLOW dù không phải src. Verify Bước 6.
- Optional (Worker thêm nếu muốn coverage chắc): `{"tool_input":{"file_path":"crates/foo/src/lib.rs"}}` → exit 2 (verify crates/*/src semantics); `lib/x.ts` → 2; `node_modules/x.js` → 2 (đã đôi-bắt bởi ext nhưng verify build-artifact). Không bắt buộc — 6 case trên đủ DoD.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/hooks/mod.rs` | Task 1+1b+2: thay stub `architect_guard()` bằng logic port (marker gate, path parse, .md allow, forbidden set, block message) |
| `tests/cli.rs` | Task 3: thêm 6 fire-test fixture (isolation qua `CLAUDE_PROJECT_DIR` temp) |
| `docs/ARCHITECTURE.md` | Docs Gate: update `architect-guard` từ "stub (P001)" → real (forbidden set + marker gate + exit semantics) |
| `CHANGELOG.md` | Entry P002 |
| `docs/discoveries/P002.md` + `docs/DISCOVERIES.md` | Discovery report + index |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `src/io.rs` | `read_payload()`, `ToolInput`, `ALLOW`/`BLOCK`/`block()` tiếp tục dùng được — Worker chỉ IMPORT, không sửa. Nếu `block()` chưa `pub` / signature lệch feed → DISCOVERY + escalate (đừng sửa io.rs trong phiếu này) |
| `src/main.rs` | dispatch `ArchitectGuard => architect_guard()` không đổi |
| `Cargo.toml` | KHÔNG thêm dep (Constraint 5) |

---

## Luật chơi (Constraints)

1. **Port 1:1, KHÔNG redesign** (CLAUDE.md §Port doctrine #1). Mỗi nhánh forbidden khớp đúng oracle L56-67 — không thêm/bớt pattern. Đặc biệt: `__tests__` chỉ prefix, build-artifacts chỉ prefix (xem Task 1b Lưu ý).
2. **Exit code parity cứng:** ALLOW=0, BLOCK=2. Cùng exit như Bash trên 100% fixture (PROJECT.md Success #1).
3. **Fail-open giữ nguyên:** no marker → 0; no path → 0; empty stdin → 0 (oracle L28,44 + P001 harness fail-open).
4. **Block message → stderr nguyên văn** (kể cả 🚫). Dùng `io::block()`, không `println!`.
5. **KHÔNG thêm dep mới vào Cargo.toml.** `tempfile` chưa có → nếu cần thì DISCOVERY + escalate Quản đốc, không tự thêm. String match dùng std (regex đã có nhưng không cần).
6. **Tầng 2 nội bộ là quyền Worker:** tên local var, `match` vs `if/else`, helper inline vs tách, cách biểu diễn pattern (miễn semantics khớp oracle + fixture pass). ĐỪNG hỏi lại Architect mấy cái này.
7. **Divergence env fallback** (oracle: script-dir; Rust: cwd) — Worker implement cwd-fallback, ghi DISCOVERY rõ lý do (binary ≠ script). Không phải redesign, là khác biệt bản chất.

---

## Nghiệm thu

### Automated
- [ ] `cargo build` clean
- [ ] `cargo test --all` pass (gồm 6 fixture mới + 8 test P001 cũ không vỡ)
- [ ] `cargo clippy -- -D warnings` không warning (gỡ `#[allow(dead_code)]` khỏi `BLOCK`/`block` nếu giờ đã dùng — verify không còn dead-code lẫn không thừa allow)

### Manual Testing (fire-test parity — đối chiếu Bash oracle)
- [ ] marker + `src/main.rs` → exit 2 (Rust == `bash scripts/architect-guard.sh` cùng input)
- [ ] marker + `README.md` → exit 0
- [ ] marker + pattern `src/**/*.rs` → exit 2
- [ ] marker + `docs/x.txt` → exit 0
- [ ] no marker + `src/main.rs` → exit 0
- [ ] empty stdin (marker present) → exit 0
- [ ] stderr message chứa "Architect envelope violation" + path gốc khi block

### Regression
- [ ] 8 test P001 (`tests/cli.rs`) vẫn pass — harness/dispatch không đổi
- [ ] `block-env-edit`/`block-unsafe-merge`/`session-banner`/`serve` stub vẫn ALLOW/no-op (P002 không đụng)

### Docs Gate (Tầng 1 — hook = security-surface, CLAUDE.md DOCS GATE BẮT BUỘC)
- [ ] `docs/ARCHITECTURE.md` — bảng Subcommands: `architect-guard` Status "stub (P001)" → "real (P002)"; thêm mô tả forbidden path set + marker gate + exit semantics
- [ ] `CHANGELOG.md` — entry P002 (port architect-guard, forbidden set, marker gate, 6 fire-test)

### Discovery Report
- [ ] `docs/discoveries/P002.md`:
  - Anchor #12-15 (src/) — CORRECT / WRONG (file:line thật của `read_payload`/`block`/stub/`tests`)
  - Env fallback divergence (oracle script-dir vs Rust cwd) — ghi rõ
  - Isolation strategy thực dùng (`CLAUDE_PROJECT_DIR` temp) + có phải escalate `tempfile` không
  - Pattern representation chọn (string match vs regex) + tại sao
  - Docs updated (ARCHITECTURE + CHANGELOG)
  - Tier escalations (None dự kiến)
- [ ] Append 1-line index vào `docs/DISCOVERIES.md` (link P002.md)
