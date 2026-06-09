# PHIẾU P005: `session-banner` subcmd (port oracle 188 dòng — RENDER hook, stdout, ALWAYS exit 0)

> **ID format:** `P005`. Branch: `feat/P001-scaffold-cli` (Quản đốc stack Phase 2 tiếp trên branch PR #1, giống P004 — KHÔNG tách branch mới).
> **Filename:** `docs/ticket/P005-session-banner.md` (active) → `docs/ticket/done/` khi xong.
> **Phiếu CUỐI Phase 2** (sau P004 block-unsafe-merge).

---

> **Loại:** Feature (port 1:1 từ Bash oracle)
> **Ưu tiên:** P1
> **Tầng:** 1 (security-surface hook — banner surface orchestrator contract + advisory staleness = part of security/orchestration pipeline; CLAUDE.md "hook = chặn/cho action → AUTO Tầng 1". Render hook KHÔNG block nhưng là contract-surface mà Quản đốc load mỗi session → sai text = lệch protocol như F-001. CHALLENGE bắt buộc.)
> **Ảnh hưởng:** `src/hooks/mod.rs` (stub→real + pure helpers), `tests/cli.rs` (integration), `docs/ARCHITECTURE.md`, `CHANGELOG.md`, `docs/discoveries/P005.md`.
> **Dependency:** P001-P004 shipped (CLI scaffold + io harness + 3 hook). None blocking.

---

## Context

### Vấn đề hiện tại

`session-banner` hiện là **stub** (P001) trả `ALLOW`. Stub comment (anchor):
```rust
pub fn session_banner() -> i32 {
    ALLOW // real logic in P005 (renders banner from git state)
}
```

Oracle `scripts/session-start-banner.sh` (188 dòng) là hook **DUY NHẤT loại RENDER** trong bộ — khác 3 hook block (architect-guard/block-env-edit/block-unsafe-merge):
- **Đọc state** (file/git), KHÔNG đọc stdin payload (oracle KHÔNG `cat` stdin → **KHÔNG gọi `read_payload()`**).
- **IN banner ra `stdout`** (KHÔNG `stderr` — đây là banner hiển thị SessionStart, khác block message ra stderr).
- **LUÔN exit 0** (informational — KHÔNG bao giờ block; mọi nhánh fail → exit 0 silent).

### Giải pháp

Port 1:1 oracle (CLAUDE.md §Port doctrine — TRUNG THÀNH, KHÔNG redesign). Tách **PURE functions** (test được không cần fs/git/clock) khỏi **IO orchestration** (`session_banner()` ráp fs read + git shell + render). Date computation: **manual ISO→epoch KHÔNG thêm dep** (xem ĐIỂM KHÓ #1). Banner text port **VERBATIM** kể cả bug F-001 (xem ĐIỂM KHÓ #3).

### Scope

- **CÓ sửa** `src/hooks/mod.rs` — `session_banner()` stub→real + pure helpers + `#[cfg(test)] mod tests`.
- **CÓ sửa** `tests/cli.rs` — integration test (CLAUDE_PROJECT_DIR temp fixture, KHÔNG thêm `tempfile` dep).
- **KHÔNG sửa** `src/io.rs` — session_banner KHÔNG đọc stdin, KHÔNG cần field mới (khác P004). Dùng `ALLOW` (=0) từ io. (Worker verify: nếu cần helper từ io thì chỉ import `ALLOW`.)
- **KHÔNG sửa** `scripts/session-start-banner.sh` (oracle, read-only).
- **KHÔNG sửa** logic P002/P003/P004 (3 hook frozen — 50 test cũ phải PASS).
- **KHÔNG thêm dependency** — KHÔNG `chrono`/`time` (date manual-epoch), KHÔNG `tempfile` (test dùng CLAUDE_PROJECT_DIR env như P002). Cần dep → DISCOVERY + escalate Quản đốc (Tầng 1 dep-add), đừng thêm im lặng.

---

## Task 0 — Verification Anchors

> Anchor oracle (.sh): Architect ĐÃ Read `scripts/session-start-banner.sh` 188 dòng → `[verified]`.
> Anchor src/*.rs: Architect KHÔNG Read được src (envelope) → `[Quản-đốc-fed, Worker verify]`.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Oracle cd repo root từ `CLAUDE_PROJECT_DIR` fallback script-dir (L17); KHÔNG `cat` stdin (render từ state, KHÔNG đọc payload) | Read oracle L13-17 | ✅ `[verified]` L17 — KHÔNG stdin |
| 2 | BACKLOG gate: `docs/BACKLOG.md` không tồn tại → **exit 0 silent** (L19-22) | Read oracle L19-22 | ✅ `[verified]` L22 |
| 3 | Active sprint header: grep `^## .*Active sprint` head -1 (L25); fallback first `^## ` + FALLBACK_USED=1 (L29-32); không có `^## ` nào → exit 0 silent (L35); HEADER_TEXT strip `^## *` (L38) | Read oracle L24-38 | ✅ `[verified]` L24-38 |
| 4 | Sprint block: từ HEADER_LINE tới dòng TRƯỚC `^## ` kế (L41 awk `NR>start && /^## /`); fallback END = wc -l (EOF) nếu no next (L44-48); SPRINT_BLOCK = sed range (L51) | Read oracle L40-51 | ✅ `[verified]` L40-51 |
| 5 | Count: OPEN = `grep -c "^- \[ \]"`, DONE = `grep -c "^- \[x\]"` trên SPRINT_BLOCK (L55-56) | Read oracle L53-56 | ✅ `[verified]` L55-56 |
| 6 | Banner chính: `━`×58 lines, "🏠 Sếp's project — Active sprint status", SPRINT_BLOCK `head -25`, "📊 Active sprint: $OPEN items đang treo, $DONE đã xong", fallback note "📌 Treating ..." nếu FALLBACK_USED (L58-71) | Read oracle L58-71 | ✅ `[verified]` L58-71 — port VERBATIM |
| 7 | Doc size warn: threshold 40960 bytes, loop `docs/CHANGELOG.md` `docs/DISCOVERIES.md` `CHANGELOG.md`; byte > threshold → "⚠️  $doc (${kb}k > 40k threshold) — gọi thợ trim..." (kb = bytes/1024); header "📏 Doc size warning:" (L73-92) | Read oracle L73-92 | ✅ `[verified]` L73-92 |
| 8 | Phiếu cleanup nudge: PHIEU_DIR ưu tiên `docs/ticket` fallback `phieu/active` (L100-104); MERGED = `git branch --merged main` strip `^[* ] ` + trim space (L108); loop `P*.md` skip TEMPLATE (L113); "Approved by Chủ nhà:" non-placeholder (skip `[date]`/empty L118-120); id = `^P[0-9]+` (L123); branch merged match `/${id}-` (L127) → "🧹 Phiếu $id approved + merged. Run: phieu-done $slug" (L129); header "🧹 Cleanup nudge:" (L134-138) | Read oracle L94-138 | ✅ `[verified]` L94-138 |
| 9 | Advisory staleness: CHỈ nếu `docs/security/advisory-inbox.md` tồn tại (L148); state `docs/security/.advisory-scan-state` missing → "🚨 ... chưa scan lần nào" (L149-151); else parse `"last_scan_at":"<ISO>"` JSON (L153-154) hoặc legacy raw trimmed (L155); epoch>0 → days = (now - epoch)/86400 (L161); >=7 → 🚨 (L162-164), >=3 → ⚠️ (L165-167), 0-2 silent | Read oracle L140-171 | ✅ `[verified]` L140-171 |
| 10 | Orchestrator contract + Architect Rule 0: VERBATIM block L173-187 (state machine, marker line **mang bug F-001**, deferred tools, handbook refs, Rule 0); cuối `━`×58 + blank (L187-188) | Read oracle L173-188 | ✅ `[verified]` L173-188 — F-001 bug, port verbatim |
| 11 | exit 0 LUÔN LUÔN (mọi nhánh) — render hook informational | Read oracle (no `exit 2` anywhere; L22/35 exit 0, fall-through exit 0) | ✅ `[verified]` — fail-OPEN toàn bộ |
| 12 | `src/io.rs`: `ALLOW=0` (session_banner return cuối) | `grep -nE "ALLOW" src/io.rs` | ⏳ `[Quản-đốc-fed, Worker verify]` |
| 13 | `src/hooks/mod.rs`: stub `session_banner() -> i32` trả `ALLOW`, comment "renders banner from git state" | `grep -n "session_banner" -A3 src/hooks/mod.rs` | ⏳ `[Quản-đốc-fed, Worker verify]` |
| 14 | **KHÔNG có dep `chrono`/`time`** trong Cargo.toml (→ date manual-epoch BẮT BUỘC); `regex` có sẵn | `grep -nE "^(chrono\|time\|regex)" Cargo.toml` | ⏳ `[Quản-đốc-fed, Worker verify]` — nếu `chrono` THỰC SỰ có → DISCOVERY, đơn giản hóa date |

**Anchor #11 = trái tim phiếu.** session-banner LUÔN exit 0 — ĐỐI LẬP block-unsafe-merge (fail-closed) và 2 hook fail-open kia (chúng có thể exit 2 khi match). session-banner KHÔNG BAO GIỜ exit ≠ 0. Worker thấy oracle có `exit 2` bất kỳ đâu → CHALLENGE (nhưng Architect verify: không có).

**Anchor #14 = gốc ĐIỂM KHÓ #1.** Nếu Worker grep thấy `chrono`/`time` ĐÃ có sẵn (Quản đốc feed sai) → DISCOVERY + dùng lib (đơn giản hơn manual epoch). Default (feed đúng): manual epoch, no dep.

### Pre-phiếu snapshot (Worker auto first-step)

```bash
# Branch là feat/P001-scaffold-cli (Phase 2 stack) → worktree basename có thể là P001. Override:
PHIEU_ID=P005
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

### Turn 1 — Quản đốc Challenge (orchestrator có Read src/ — verify #12-14)

**Anchor verification (3/3 ✅):**
- #12 ✅ `src/io.rs:8,17` — `ToolInput{file_path,...,command}` (command từ P004); session_banner KHÔNG dùng payload nên field nào cũng OK
- #13 ✅ `src/hooks/mod.rs:539-541` — `session_banner()` stub trả ALLOW, comment "renders from git state" (đúng — không đọc stdin)
- #14 ✅ `Cargo.toml` — KHÔNG có `chrono`/`time` → manual ISO→epoch (days-from-civil) là path đúng, no dep-add

**Objections (Tầng 1):** None. Port trung thành oracle L17-188. Render hook: stdout + LUÔN exit 0 (fail-open informational — ĐỐI LẬP block-unsafe-merge fail-CLOSED, không nhầm). Banner text VERBATIM kể cả bug F-001 (port doctrine — fix upstream sos-kit, ghi Discovery text sống 2 nơi). Date: `staleness_days(iso, now_epoch)` inject now_epoch → unit test deterministic. Risk epoch-math sẽ verify ở nghiệm thu qua parity vs `bash` + `date -u`.

**Status:** ✅ ACCEPTED V1 — no challenges.

### Final consensus
- Phiếu version: V1 (no debate turn needed)
- Approved by: Quản đốc (Sếp uỷ quyền approval gate, 2026-06-09) — EXECUTE may begin

---

## Nhiệm vụ

> **Kiến trúc đề xuất (pure fn cho testability — cách impl chi tiết là Tầng 2, Worker quyết miễn semantics khớp test cases + parity oracle).** Tách render-text builders khỏi IO để unit test không cần fs/git/clock.

### Task 1: Pure helper `find_sprint_block` (`src/hooks/mod.rs`)

**File:** `src/hooks/mod.rs`

**Thêm:** pure function (input = nội dung BACKLOG.md, KHÔNG đọc file):
```rust
/// Returns (sprint_block, header_text, fallback_used) or None if no ^## heading.
fn find_sprint_block(backlog: &str) -> Option<(String, String, bool)> { /* ... */ }
```

**Lưu ý (port oracle L24-51):**
- Tìm dòng đầu tiên khớp `^## .*Active sprint` → đó là header (fallback_used = false). (oracle L25 `grep -n "^## .*Active sprint" | head -1`.)
- Nếu KHÔNG có → fallback dòng `^## ` ĐẦU TIÊN (fallback_used = true). (oracle L29-31.)
- Nếu KHÔNG có `^## ` nào → `None` (caller exit 0 silent). (oracle L35.)
- `header_text` = nội dung header strip prefix `^## *` (oracle L38 `sed 's/^## *//'` — strip `## ` + space thừa).
- `sprint_block` = từ dòng header TỚI dòng TRƯỚC `^## ` kế tiếp (exclusive); nếu không có `^## ` kế → tới HẾT file (EOF). (oracle L41 awk `NR>start && /^## /`, L44-48 fallback EOF.) **Sprint block BAO GỒM dòng header.** (oracle L51 sed `${HEADER_LINE},${END_LINE}p` — inclusive header.)
- ⚠️ `^## ` match là **đầu dòng + đúng 2 dấu `#` + space** (oracle dùng literal `^## `). Dòng `### ` (H3) KHÔNG phải boundary (bắt đầu `###` vẫn match `^## `? — KHÔNG: oracle grep `^## ` cần space sau 2 `#`, mà `###` có `#` thứ 3 ở vị trí space → KHÔNG khớp `^## `). Worker port chính xác: regex `^## ` (2 hash + space), KHÔNG `^##` lỏng. Test case H3 dưới verify.

### Task 2: Pure helper `count_items` (`src/hooks/mod.rs`)

**File:** `src/hooks/mod.rs`

**Thêm:**
```rust
/// (open_count, done_count) — counts "^- [ ]" and "^- [x]" lines.
fn count_items(block: &str) -> (usize, usize) { /* ... */ }
```

**Lưu ý (port oracle L55-56):**
- OPEN = số dòng khớp `^- \[ \]` (literal: dash, space, `[`, space, `]`).
- DONE = số dòng khớp `^- \[x\]` (literal `[x]`, lowercase x — oracle dùng `[x]` không `[X]`).
- Per-line match (oracle `grep -c` per line). Worker: split `\n` rồi đếm, hoặc regex `(?m)`. Đếm `^- [ ]` chính xác (1 space giữa brackets) — `^- [  ]` (2 space) KHÔNG khớp.

### Task 3: Pure helper `staleness_days` + `staleness_category` (`src/hooks/mod.rs`) — ĐIỂM KHÓ #1

**File:** `src/hooks/mod.rs`

**Thêm 2 pure fn (date manual-epoch, KHÔNG thêm dep):**
```rust
/// Parse ISO-8601 UTC "%Y-%m-%dT%H:%M:%SZ" (or legacy raw ISO) → epoch seconds,
/// then days = (now_epoch - parsed_epoch) / 86400. None if unparseable.
/// `now_epoch` injected for deterministic unit test (KHÔNG gọi clock thật trong fn).
fn staleness_days(iso: &str, now_epoch: i64) -> Option<i64> { /* ... */ }

enum Staleness { Critical, Warn, Silent }  // >=7 | 3..=6 | 0..=2 (and negative → Silent)
fn staleness_category(days: i64) -> Staleness { /* ... */ }
```

**Lưu ý (port oracle L153-168 — ĐIỂM KHÓ #1, Architect SPEC RÕ, KHÔNG để Worker mơ hồ):**
- **Date strategy = (a) manual ISO→epoch, KHÔNG thêm dep** (KHUYẾN NGHỊ, default). Lý do: KHÔNG có `chrono`/`time` (anchor #14), MSRV 1.85 an toàn, không vi phạm no-dep-add / no-network.
- **Parse:** ISO `"%Y-%m-%dT%H:%M:%SZ"` (vd `2026-06-09T12:00:00Z`). Split string lấy year/month/day/hour/min/sec. Legacy raw ISO (oracle L155 fallback `tr -d '[:space:]'`) cũng cùng format → cùng parser. Parse fail (format sai / non-numeric) → `None`.
- **Days-from-civil algorithm (Howard Hinnant, public-domain, ~10 dòng well-known):** convert (y,m,d) → days since epoch 1970-01-01. Công thức:
  ```
  y -= m <= 2 ? 1 : 0
  era = (y >= 0 ? y : y-399) / 400
  yoe = y - era*400
  doy = (153*(m + (m > 2 ? -3 : 9)) + 2)/5 + d-1
  doe = yoe*365 + yoe/4 - yoe/100 + doy
  days = era*146097 + doe - 719468
  ```
  → epoch_secs = days*86400 + hour*3600 + min*60 + sec. (UTC, `Z` suffix → no tz offset.)
- **`now_epoch`:** caller truyền `std::time::SystemTime::now().duration_since(UNIX_EPOCH).as_secs() as i64`. fn KHÔNG tự gọi clock → unit test truyền now_epoch CỐ ĐỊNH → deterministic.
- **days** = `(now_epoch - parsed_epoch) / 86400` (integer div, oracle L161). Có thể âm (last_scan ở tương lai / clock skew) → caller treat như Silent (oracle: chỉ check `>= 7` rồi `>= 3`, âm rơi xuống silent).
- **`staleness_category`:** `>= 7` → Critical; `3..=6` → Warn; còn lại (0,1,2 và âm) → Silent. (oracle L162 `-ge 7`, L165 `-ge 3`.)
- ⚠️ Architect KHÔNG thể chạy Rust → **Worker verify days-from-civil cho ≥1 test case tay** (vd `2026-06-09T00:00:00Z` → epoch 1780012800; verify với `date -u -d 2026-06-09 +%s`). Nếu manual epoch THỰC SỰ không khả thi (vd cần leap-second / tz phức tạp) → DISCOVERY + escalate Quản đốc lựa chọn (b) thêm dep. **Default: (a) khả thi — format cố định UTC `Z`, no tz, no leap-second concern cho day-granularity.**

### Task 4: Pure helper `doc_size_warns` (`src/hooks/mod.rs`)

**File:** `src/hooks/mod.rs`

**Thêm:**
```rust
/// For each (path, bytes), if bytes > 40960 → warn string. Returns Vec of warn lines.
fn doc_size_warns(docs: &[(&str, u64)]) -> Vec<String> { /* ... */ }
```

**Lưu ý (port oracle L76-87):**
- Threshold = 40960 bytes (40k). `bytes > 40960` (strict `>`, oracle L83 `-gt`).
- kb = `bytes / 1024` (integer div, oracle L84).
- Warn line VERBATIM (oracle L85): `⚠️  ${doc} (${kb}k > 40k threshold) — gọi thợ trim, archive cũ ra docs/archive/`
  ⚠️ **2 space sau `⚠️`** (oracle L85 `"⚠️  ${doc}"` — emoji + 2 space). Port chính xác.
- Caller cung cấp `(path, bytes)` chỉ cho file TỒN TẠI (oracle L79 skip nếu không tồn tại). Pure fn chỉ nhận list đã filter → KHÔNG đụng fs. (Worker: trong `session_banner()` đọc byte size 3 file, skip missing, truyền list vào fn.)

### Task 5: Ráp `session_banner()` real (`src/hooks/mod.rs`)

**File:** `src/hooks/mod.rs`

**Tìm:** stub hiện tại:
```rust
pub fn session_banner() -> i32 {
    ALLOW // real logic in P005 (renders banner from git state)
}
```

**Thay bằng:** orchestration ráp pure helpers + fs/git IO + render ra **stdout** (theo đúng thứ tự oracle, in ra `println!`/`print!` = stdout). Pipeline:

1. **cd repo root** (oracle L17): xác định root = `std::env::var("CLAUDE_PROJECT_DIR")` nếu set, else cwd (`std::env::current_dir()`). KHÔNG thực sự `cd` process — thay vào đó **join mọi path tương đối (`docs/BACKLOG.md`...) với root** (Rust idiom: `let root = ...; root.join("docs/BACKLOG.md")`). (Worker Tầng 2: dùng `PathBuf::join` cho mọi path dưới.)
2. **BACKLOG gate** (oracle L19-22): đọc `<root>/docs/BACKLOG.md`. Không tồn tại / không đọc được → `return ALLOW` ngay (KHÔNG in gì — silent).
3. `find_sprint_block(&backlog_content)` → `None` → `return ALLOW` silent (oracle L35). `Some((block, header, fallback))` → tiếp.
4. `count_items(&block)` → `(open, done)`.
5. **In banner chính** (oracle L58-71, stdout, VERBATIM):
   - blank line, `━`×58, `🏠 Sếp's project — Active sprint status`, `━`×58, blank.
   - block `head -25`: in tối đa 25 dòng đầu của `block` (oracle L64 `head -25`).
   - blank, `━`×58, `📊 Active sprint: {open} items đang treo, {done} đã xong`.
   - nếu `fallback` → blank + `📌 Treating "{header}" as Active sprint (no "Active sprint" header found).` (oracle L70 — chú ý escaped quotes quanh header_text).
6. **Doc size warn** (oracle L73-92): đọc byte size 3 file `<root>/docs/CHANGELOG.md`, `<root>/docs/DISCOVERIES.md`, `<root>/CHANGELOG.md` (skip missing); `doc_size_warns(...)` → nếu Vec non-empty: blank + `📏 Doc size warning:` + mỗi warn in `    {warn}` (oracle L91 `printf "    %b"` — 4-space indent). ⚠️ **path trong warn = path tương đối** (`docs/CHANGELOG.md`, KHÔNG phải absolute root-joined) — oracle L85 dùng `$doc` = literal `"docs/CHANGELOG.md"`. Worker: truyền relative path string vào `doc_size_warns`, đọc byte qua root-joined path.
7. **Phiếu cleanup nudge** (oracle L94-138):
   - PHIEU_DIR = `<root>/docs/ticket` nếu tồn tại, else `<root>/phieu/active` nếu tồn tại, else skip (oracle L100-104). (Repo này có `docs/ticket` → dùng nó.)
   - MERGED branches: shell `git branch --merged main` (xem ĐIỂM KHÓ #2), strip `^[* ] ` prefix + trim space mỗi dòng (oracle L108). Fail → empty (silent, skip nudge).
   - Loop `P*.md` trong PHIEU_DIR (Worker: `std::fs::read_dir` filter `P*.md`, hoặc Glob-style; skip `TICKET_TEMPLATE.md`/`TEMPLATE.md` oracle L113).
   - Mỗi file: tìm dòng chứa `Approved by Chủ nhà:` (oracle L116 `head -1`); skip nếu chứa `[date]` HOẶC value rỗng (oracle L118-120 placeholder check).
   - phieu_id = `^P[0-9]+` từ basename (oracle L123); skip nếu rỗng.
   - nếu MERGED có dòng khớp `/{phieu_id}-` (oracle L127 `grep -qE "/${phieu_id}-"`) → nudge `🧹 Phiếu {phieu_id} approved + merged. Run: phieu-done {slug}` (slug = basename không `.md`, oracle L128-129).
   - nếu có nudge: blank + `🧹 Cleanup nudge:` + mỗi nudge `    {nudge}` (oracle L135-137).
8. **Advisory staleness** (oracle L140-171) — ĐIỂM KHÓ #1 dùng Task 3:
   - CHỈ nếu `<root>/docs/security/advisory-inbox.md` tồn tại (oracle L148). Không tồn tại → skip cả block.
   - state file `<root>/docs/security/.advisory-scan-state`. Không tồn tại → blank + `🚨 Advisory-watch: chưa scan lần nào — gõ /advisory-scan (first scan)` (oracle L150-151).
   - Tồn tại → đọc nội dung, extract `last_scan_at`: regex JSON `"last_scan_at"\s*:\s*"([^"]+)"` (oracle L153-154); nếu rỗng → legacy raw = toàn bộ content trimmed whitespace (oracle L155).
   - `staleness_days(&iso, now_epoch)`:
     - `None` (parse fail, ~oracle epoch 0 → block bị skip L160 `-gt 0`) → KHÔNG in gì.
     - `Some(days)` → `staleness_category(days)`:
       - `Critical` → blank + `🚨 Advisory-watch: scan cuối {days} ngày trước (>= 7) — orchestrator BẮT BUỘC auto-spawn advisory-watch (ORCHESTRATION Rule 11)` (oracle L163-164).
       - `Warn` → blank + `⚠️  Advisory-watch: scan cuối {days} ngày trước — cân nhắc /advisory-scan` (oracle L166-167, **2 space sau ⚠️**).
       - `Silent` → KHÔNG in gì.
9. **Orchestrator contract + Architect Rule 0** (oracle L173-187) — **VERBATIM block, port NGUYÊN VĂN** (xem ĐIỂM KHÓ #3 — text MANG bug F-001, port y nguyên):
   ```
   (blank)
   🤖 Orchestrator contract (main session — đọc kỹ, ép tuân thủ):
       State machine: DRAFT → CHALLENGE → [RESPOND ⇄ CHALLENGE] → APPROVAL_GATE → EXECUTE
       KHÔNG hỏi user giữa các phase. APPROVAL_GATE là gate DUY NHẤT (trước EXECUTE).
       KHÔNG đẩy đọc phiếu/code về user — Worker CHALLENGE rà & report ≤5 dòng.
       Marker: touch .sos-state/architect-active trước spawn architect; rm -f trước spawn worker.
       Deferred tools MANDATORY (load đầu session, KHÔNG fallback markdown 1/2/3):
           ToolSearch select:AskUserQuestion,TaskCreate,TaskUpdate
       Handbook: agents/orchestrator.md (~85 lines, condensed contract)
       Spec đầy đủ: docs/ORCHESTRATION.md
   (blank)
   📌 Architect Rule 0: chỉ viết phiếu cho item trong Active sprint (or first ^## section if absent).
       Idea mới → /idea skill (intake vào BACKLOG.md).
       Pick item hay add idea?
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   (blank)
   ```
   ⚠️ **F-001 BUG CỐ TÌNH GIỮ:** dòng `Marker: ...` THIẾU `touch .sos-state/worker-active` (xem `docs/SOS_KIT_FEEDBACK.md` F-001). **KHÔNG tự sửa** — port doctrine = trung thành oracle. Fix phải đi upstream sos-kit (update CẢ canonical .sh + banner Rust + orchestrator.md + ORCHESTRATION.md đồng bộ). Sửa lẻ ở port này = lệch oracle = mất parity. Ghi DISCOVERY.
10. **return `ALLOW`** (oracle L188 exit 0 — LUÔN, mọi nhánh trên cũng return ALLOW).

**Lưu ý chung Task 5:**
- **In stdout** (`println!`/`print!`), KHÔNG `eprintln!` (stderr) — banner hiển thị. (Khác 3 hook block in reason ra stderr.)
- `━` = U+2501 (BOX DRAWINGS HEAVY HORIZONTAL), **58 ký tự** mỗi đường (đếm oracle L60). Worker copy chuỗi từ oracle, KHÔNG tự gõ lại (tránh sai số ký tự).
- **head -25** (block): in min(25, số dòng block) dòng ĐẦU.
- **No `unwrap()`/`expect()` trên IO** — mọi fs read fail / git fail → fail-OPEN (skip section đó hoặc return ALLOW), KHÔNG panic. Render hook KHÔNG được crash session start.
- Emoji/text/space VERBATIM oracle (`🏠`/`📊`/`📏`/`🧹`/`🚨`/`⚠️`/`🤖`/`📌` + interpolation + 4-space/2-space indent). Sai chữ = sai parity (Quản đốc đối chiếu stdout).

### Task 6: Tests (`#[cfg(test)] mod tests` + `tests/cli.rs`) — Verify-cò P057

Xem section **Verify-cò** dưới. BẮT BUỘC cùng phiếu (CLAUDE.md §Port doctrine #3 + DoD #5).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/hooks/mod.rs` | Task 1-4: pure helpers (`find_sprint_block`, `count_items`, `staleness_days`+`staleness_category`, `doc_size_warns`); Task 5: ráp `session_banner()` real (fs+git+render stdout); Task 6: `#[cfg(test)] mod tests` |
| `tests/cli.rs` | Task 6: integration (CLAUDE_PROJECT_DIR temp fixture, no `tempfile` dep) |
| `docs/ARCHITECTURE.md` | Docs Gate: session-banner "stub"→"real" (render pipeline) |
| `CHANGELOG.md` | Entry P005 |
| `docs/discoveries/P005.md` | Discovery report (per-phiếu, P038) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `scripts/session-start-banner.sh` | Oracle read-only — đối chiếu parity stdout, KHÔNG sửa |
| `src/io.rs` | KHÔNG đổi (session_banner KHÔNG đọc stdin) — chỉ import `ALLOW`. P002/P003/P004 path bất biến |
| `src/hooks/mod.rs` (P002/P003/P004 fns) | 3 hook block KHÔNG đổi behavior — 50 test cũ PASS |

---

## Luật chơi (Constraints)

1. **Port 1:1 — KHÔNG redesign** (CLAUDE.md §Port doctrine #1). stdout + exit 0 + text verbatim như oracle.
2. **LUÔN exit 0** — render hook informational, KHÔNG BAO GIỜ block (anchor #11). Mọi fail (no BACKLOG, no `^##`, fs error, git fail, parse fail) → fail-OPEN return ALLOW. ĐỐI LẬP block-unsafe-merge (fail-closed).
3. **In ra stdout** (`println!`/`print!`), KHÔNG stderr. Banner hiển thị, khác block message.
4. **KHÔNG đọc stdin** — KHÔNG gọi `read_payload()`. Render từ file/git state (oracle KHÔNG `cat` stdin).
5. **Date = manual ISO→epoch, KHÔNG thêm dep** (ĐIỂM KHÓ #1, option a). KHÔNG `chrono`/`time`. `staleness_days(iso, now_epoch)` nhận now_epoch injected → testable không phụ thuộc clock. Nếu (a) bất khả thi → DISCOVERY + escalate Quản đốc (Tầng 1 dep-add).
6. **git shelling không qua shell** — `Command::new("git")` + args vec `["branch","--merged","main"]`, KHÔNG `sh -c` (ĐIỂM KHÓ #2). Fail → empty (silent skip nudge).
7. **Banner text VERBATIM kể cả bug F-001** (ĐIỂM KHÓ #3) — KHÔNG tự "sửa cho đúng" (thiếu touch worker-active). Fix đi upstream. Ghi DISCOVERY: text giờ sống 2 nơi (.sh + Rust).
8. **KHÔNG thêm dependency** — `regex` (có sẵn) + `std` only. KHÔNG `chrono`/`time`/`tempfile`. Test isolation qua `CLAUDE_PROJECT_DIR` env (như P002), KHÔNG tempfile.
9. **No `unwrap()`/`expect()` trên IO/parse path** — render hook KHÔNG được panic crash SessionStart. fail-OPEN toàn bộ.
10. **50 test cũ (P001-P004) PASS** — regression hard gate.

---

## Nghiệm thu

### Automated
- [ ] `cargo build` clean
- [ ] `cargo test --all` — unit (pure helpers) + integration (CLAUDE_PROJECT_DIR fixture) mới PASS + **50 test cũ KHÔNG vỡ**
- [ ] `cargo clippy -- -D warnings` — zero warning

### Manual Testing (Quản đốc đối chiếu `bash scripts/session-start-banner.sh` vs binary `session-banner` trên REPO THẬT — so stdout VERBATIM)
- [ ] Repo có `docs/BACKLOG.md` (Active sprint Phase 1) → stdout chứa `🏠 Sếp's project`, sprint block, `📊 Active sprint: N items đang treo, M đã xong`, `🤖 Orchestrator contract`, `📌 Architect Rule 0` — KHỚP Bash từng dòng. exit 0.
- [ ] Doc size: nếu CHANGELOG/DISCOVERIES > 40k → `📏 Doc size warning` khớp Bash (hoặc cả 2 im nếu < 40k).
- [ ] Advisory: `docs/security/advisory-inbox.md` không tồn tại → KHÔNG có dòng advisory (cả 2). Nếu tồn tại → so 🚨/⚠️/silent theo days.
- [ ] **F-001 verify:** dòng `Marker: ...` trong stdout Rust KHỚP Bash (đều THIẾU touch worker-active) — đó là parity ĐÚNG (bug giữ nguyên).

### Regression
- [ ] `architect-guard` + `block-env-edit` + `block-unsafe-merge` exit codes KHÔNG đổi (50 test cũ green)
- [ ] `src/io.rs` KHÔNG đổi → `read_payload()` 3 hook kia bất biến

### Docs Gate (Tầng 1 — security/orchestration-surface, BẮT BUỘC)
- [ ] `CHANGELOG.md` — entry P005 (session-banner port, render hook, date manual-epoch, F-001 verbatim note, **→ Phase 2 DONE**)
- [ ] `docs/ARCHITECTURE.md`:
  - session-banner "stub (P001)"→"real (P005)" trong bảng Subcommands
  - thêm section `### session-banner (P005 — real implementation)`: render pipeline (BACKLOG parse → sprint block + count → doc-size warn → cleanup nudge → advisory staleness → orchestrator contract); **stdout / always-exit-0** (note: ĐỐI LẬP 3 hook block — render hook fail-OPEN toàn bộ); date manual-epoch (no chrono/time dep) note; git shelling `branch --merged`.
  - Data Flow: session-banner "stub"→"real (render from fs/git state, no stdin)".

### Discovery Report
- [ ] Write `docs/discoveries/P005.md`:
  - Anchor #12-14 (io ALLOW / stub / no chrono dep): CORRECT / WRONG (file:line).
  - **Date strategy:** manual ISO→epoch (days-from-civil Hinnant) — CONFIRM khả thi? (Worker verify ≥1 epoch case tay vs `date -u`.) Nếu bất khả thi → escalation note.
  - **git shelling** `branch --merged main` qua `Command` arg-vec (no `sh -c`); fail → empty.
  - **Banner text MANG bug F-001** (thiếu touch worker-active, oracle L178) — port VERBATIM. Text giờ sống 2 NƠI (canonical `scripts/session-start-banner.sh` + Rust port). **Fix F-001 upstream phải update CẢ canonical .sh + Rust port + orchestrator.md + ORCHESTRATION.md đồng bộ** — nếu chỉ sửa 1 nơi → drift. Document để future không "sửa nhầm" lẻ.
  - **Render hook fail-OPEN divergence** — session-banner LUÔN exit 0 (informational), khác block-unsafe-merge fail-CLOSED. Document để future hook không nhầm.
  - **stdin NOT read** — render từ state, khác 3 hook đọc payload.
  - Docs updated (write "None" nếu không).
  - Tier escalations (write "None" nếu không).
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`

---

## Verify-cò (P057 — fixture cùng phiếu)

**Chiến lược 2 tầng:** pure fn unit-test (không cần fs/git/clock — date qua injected now_epoch) + integration CLI (CLAUDE_PROJECT_DIR temp fixture, no gh/git-dependent paths trong CI).

### Unit test PURE functions (`#[cfg(test)] mod tests` — deterministic)

**`find_sprint_block`:**
- [ ] BACKLOG có `## 🔥 Active sprint: Phase 1` + items → `Some((block chứa items, "🔥 Active sprint: Phase 1", false))`. block dừng TRƯỚC `## 🎯 Next sprint`.
- [ ] BACKLOG KHÔNG có "Active sprint" header, có `## Intro` đầu → fallback `Some((block, "Intro", true))`.
- [ ] BACKLOG KHÔNG có `^## ` nào (chỉ prose / `#` H1) → `None`.
- [ ] Active sprint là section CUỐI (no next `^## `) → block tới EOF.
- [ ] H3 `### Sub` giữa sprint KHÔNG cắt block (chỉ `^## ` đúng 2-hash mới cắt).

**`count_items`:**
- [ ] block `"- [ ] a\n- [x] b\n- [ ] c"` → `(2, 1)`.
- [ ] block không có item → `(0, 0)`.
- [ ] `- [X] big-x` (uppercase) → KHÔNG đếm done (oracle `[x]` lowercase) → done=0. (Verify oracle behavior — nếu Worker thấy oracle dùng `[xX]` thì sửa; Architect đọc L56 = `[x]` lowercase.)

**`staleness_days` (now_epoch CỐ ĐỊNH → deterministic):**
- [ ] `staleness_days("2026-06-09T00:00:00Z", now=<2026-06-16T00:00:00Z epoch>)` → `Some(7)`.
- [ ] cùng iso, now = +3 ngày → `Some(3)`.
- [ ] cùng iso, now = +1 ngày → `Some(1)`.
- [ ] legacy raw `"2026-06-09T00:00:00Z"` (no JSON wrap) → cùng kết quả.
- [ ] `"garbage"` / `"2026-13-99..."` invalid → `None`.
- [ ] now < iso (tương lai) → `Some(âm)` (caller → Silent).
- [ ] ⚠️ Worker verify ≥1 epoch tay: `staleness_days("2026-06-09T00:00:00Z", 0)` parsed_epoch khớp `date -u -d "2026-06-09" +%s`.

**`staleness_category`:**
- [ ] `7` → Critical; `10` → Critical.
- [ ] `3` → Warn; `6` → Warn.
- [ ] `0`/`1`/`2` → Silent; `-5` → Silent.

**`doc_size_warns`:**
- [ ] `[("docs/CHANGELOG.md", 50000)]` → 1 warn chứa `docs/CHANGELOG.md (48k > 40k threshold)` (50000/1024=48).
- [ ] `[("CHANGELOG.md", 40960)]` → empty (strict `>`, 40960 KHÔNG > 40960).
- [ ] `[("CHANGELOG.md", 40961)]` → 1 warn (`40k`, 40961/1024=40).
- [ ] empty input → empty Vec.

### Integration test CLI (assert_cmd, `tests/cli.rs` — CLAUDE_PROJECT_DIR temp, no tempfile dep)

> Isolation qua `CLAUDE_PROJECT_DIR` env trỏ vào temp dir tạo thủ công (như P002 pattern). Worker: dùng `std::env::temp_dir()` + unique subdir (vd PID/counter), tạo `docs/BACKLOG.md` fixture, set env, chạy binary, cleanup. KHÔNG thêm `tempfile`.

- [ ] CLAUDE_PROJECT_DIR = temp có `docs/BACKLOG.md` (vài `- [ ]` + `- [x]` dưới `## 🔥 Active sprint: Test`) → stdout chứa `🏠 Sếp's project`, `📊 Active sprint: N items`, `🤖 Orchestrator contract`, `📌 Architect Rule 0`; exit 0.
- [ ] CLAUDE_PROJECT_DIR = temp KHÔNG có `docs/BACKLOG.md` → stdout RỖNG (silent) + exit 0.
- [ ] CLAUDE_PROJECT_DIR = temp có BACKLOG KHÔNG có `^## ` → stdout rỗng + exit 0.
- [ ] Fallback header: BACKLOG có `## Foo` (no "Active sprint") → stdout chứa `📌 Treating "Foo" as Active sprint`.

### git/advisory-dependent paths — MANUAL / careful
- ⚠️ **Cleanup nudge gọi `git branch --merged main`** → trong CI temp dir có thể không phải git repo → git fail → empty (silent). Test integration KHÔNG assert nudge content (git-state-dependent). Nudge path = MANUAL (Quản đốc đối chiếu trên repo thật).
- ⚠️ **Advisory staleness** = clock + fs dependent → unit test qua pure fn (injected now_epoch). Integration KHÔNG cần advisory-inbox.md trong fixture (→ block skip, im lặng — OK).

---

## Branch note

Phase 2 — Quản đốc stack tiếp trên branch `feat/P001-scaffold-cli` (giống P004, KHÔNG tách branch mới). Snapshot `.backup/P005/`. **`git add` cả phiếu này** (F-002 — adopt repo không có `phieu`/`phieu-done` auto, phiếu phải vào commit thủ công, đừng để untracked). Khi xong → move `docs/ticket/done/` + commit. **Phiếu CUỐI Phase 2 → cập nhật BACKLOG: P005 xuống Recently shipped, đánh dấu Phase 2 DONE.**
