# PHIẾU P008: README + ARCHITECTURE polish + ship prep

> **Filename:** `docs/ticket/P008-ship-prep.md` (active) → `docs/ticket/done/` on completion.
> **Branch:** stack tiếp trên `feat/P001-scaffold-cli` (Phase 4 stack — xem Branch note).

---

> **Loại:** Feature (release-prep) + Bugfix (serverInfo.name)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng) — Việc 1 chạm MCP server surface + `#[tool_handler]` wiring (sai → 5 tool/handshake vỡ = contract LAN cho mọi MCP client); Việc 3 chạm `cargo publish` (publish lên crates.io KHÔNG đảo được). Security-surface MCP + publish-không-đảo → AUTO Tầng 1 dù diff nhỏ. CLAUDE.md "Security-surface → AUTO Tầng 1".
> **Ảnh hưởng:** `src/serve.rs`, `README.md`, `Cargo.toml`, `docs/ARCHITECTURE.md`, `CHANGELOG.md`
> **Dependency:** P007 (Phase 3 DONE — 5 MCP tool đã ship). KHÔNG depend P009 (wire tarot — phiếu sau).

---

## Context

### Vấn đề hiện tại

P001–P007 đã ship: CLI 5 subcmd (4 hook port parity-verified + serve) + MCP server 5 tool (architect_guard / block_env_edit / block_unsafe_merge / session_banner / why_blocked). 93/93 test pass, clippy clean. Còn 4 lỗ hổng "ship ≠ buildable product" cần đánh bóng trước khi `cargo publish` (P008 = phiếu áp chót Phase 4; wire tarot = P009, KHÔNG nằm trong phiếu này):

1. **MCP `serverInfo.name` sai** — handshake thật trả `"serverInfo":{"name":"rmcp","version":"1.7.0"}`. Nguyên nhân (em Quản đốc research rmcp 1.7): `#[tool_router(server_handler)]` (serve.rs:90 `[Quản-đốc-fed, Worker verify]`) auto-gen `ServerHandler::get_info()` dùng `Implementation::from_build_env()` — expand TẠI site rmcp crate → tên "rmcp", version "1.7.0". Client thấy server tên "rmcp" thay vì "claude-hooks" → khó debug, sai branding.
2. **README.md chỉ 2 dòng stub** (`[verified]` — Read README.md: title + 1 dòng mô tả). Repo có thể `cargo publish` (Cargo.toml:11 `readme = "README.md"`) → README hiện không usable cho người `cargo install`.
3. **Cargo.toml version vẫn 0.1.0** (`[verified]` — Read Cargo.toml:3 `version = "0.1.0"`) nhưng CHANGELOG head đã v0.7.0 (`[verified]` — Read CHANGELOG.md:5). `cargo publish --dry-run` cảnh báo `claude-hooks@0.1.0 already exists on crates.io`. Package kéo cả `scripts/`, `docs/`, `phieu/` (101 files / 778KB theo recon Quản đốc) — dev/doctrine không cần cho `cargo install`.
4. **ARCHITECTURE.md** đã được P002–P007 update từng phần — cần rà nhất quán cuối + đảm bảo phản ánh code cuối (5 subcmd + 5 MCP tool + Decision-core + exit/fail-open/fail-CLOSED), thêm Status nếu thiếu. `cargo publish --dry-run` PHẢI clean sau bump + exclude.

### Giải pháp

**Việc 1 — get_info refactor (server_handler → explicit `ServerHandler` impl):**
Chuyển `#[tool_router(server_handler)]` → `#[tool_router]` (bỏ flag `server_handler`) trên impl block của `HooksServer`, rồi viết explicit:
```rust
#[tool_handler]
impl ServerHandler for HooksServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: "claude-hooks".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
```
`#[tool_handler]` macro tự wire `self.tool_router` vào — đây là điểm thay thế cho `server_handler` flag (flag cũ gen CẢ `ServerHandler` impl rỗng VỚI default get_info; explicit impl + `#[tool_handler]` tách phần tool-routing ra macro, phần get_info do ta viết). Imports cần: `rmcp::handler::server::ServerHandler`, `rmcp::model::{ServerInfo, Implementation}`. `version: env!("CARGO_PKG_VERSION")` → tự khớp Cargo.toml version sau bump 0.8.0. Optional: thêm `instructions: Some("...".into())` mô tả 5 tool. **Worker iterate `cargo build` + `cargo check` để chốt exact import path + field name — compiler là oracle SOUND ở đây.**

**Việc 2 — README.md usable** (English-friendly, repo publishable). Cấu trúc:
- Title + 1-câu vision (Rust binary thay 4 Bash hot-path hook, dual CLI + MCP).
- **Install:** `cargo install --path .` (local) / `cargo install claude-hooks` (crates.io sau publish).
- **CLI usage:** 5 subcmd. Mỗi cái: stdin JSON shape + exit convention (0 allow / 2 block). Ví dụ wire trong `.claude/settings.json` (PreToolUse cho 4 hook, SessionStart cho session-banner).
- **MCP mode:** `claude-hooks serve` (stdio JSON-RPC), 5 tool gồm `why_blocked` debug. Ví dụ `.mcp.json` entry.
- **Exit convention:** 0 allow / 2 block. Fail-open MẶC ĐỊNH, TRỪ `block-unsafe-merge` fail-CLOSED.
- Giữ gọn, chính xác. KHÔNG bịa flag không có — claude-hooks CHỈ nhận input qua stdin JSON, KHÔNG có CLI flag riêng cho từng hook (`[verified]` ARCHITECTURE.md "stdin-JSON Harness": payload qua stdin, không flag). Worker đối chiếu CLI thật trước khi viết ví dụ.

**Việc 3 — version bump + package slim:**
- Bump `Cargo.toml:3` `version` → `0.8.0` (CHANGELOG head 0.7.0 từ P007, P008 = bump kế).
- Thêm `exclude = [...]` vào `[package]` để slim crate: loại `scripts/`, `docs/`, `phieu/`, `.sos-state/`, `.backup/`, `hooks/`, `.claude/`, `tests/` không cần thiết KHÔNG loại (test cần cho `cargo publish` verify). Binary tự đủ với `src/` + `Cargo.toml` + `README.md` + `LICENSE` (`[verified]` LICENSE tồn tại ở root). **Worker quyết exclude list cuối — Tầng 2 self-decide — miễn `cargo publish --dry-run` build clean + binary chạy.** (Lưu ý: `tests/` cần giữ nếu publish verify chạy `cargo test`; nếu chỉ build thì có thể exclude — Worker iterate dry-run.)

**Việc 4 — ARCHITECTURE.md polish + verify publish:**
- Rà `docs/ARCHITECTURE.md` khớp code cuối (5 subcmd + 5 MCP tool + Decision-core + exit/fail-open/fail-CLOSED). Update serve.rs section: `#[tool_router(server_handler)]` → `#[tool_router]` + explicit `ServerHandler::get_info()` (Việc 1 đổi đây — ARCHITECTURE.md:174,177 hiện ghi `#[tool_router(server_handler)]` `[unverified]`, Worker grep verify + sửa khớp).
- Thêm "Status: Phase 1-3 complete" nếu thiếu.
- `cargo publish --dry-run` PHẢI clean sau bump + exclude.

### Scope
- CHỈ sửa: `src/serve.rs` (get_info), `README.md` (viết mới), `Cargo.toml` (version + exclude), `docs/ARCHITECTURE.md` (polish serve section + Status), `CHANGELOG.md` (entry P008), `docs/discoveries/P008.md` (mới).
- KHÔNG sửa: `src/hooks/mod.rs`, `src/io.rs`, `src/main.rs` (logic 4 hook + Decision-core đã ship, KHÔNG đụng); `scripts/*.sh` (Bash oracle — read-only reference); `tests/cli.rs` (CLI parity test giữ nguyên); `.claude/settings.json` (routing table không đổi).
- KHÔNG wire tarot (P009). KHÔNG `cargo publish` thật (chỉ `--dry-run`).

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `serve.rs:90` impl block của `HooksServer` mang `#[tool_router(server_handler)]` | `grep -n "tool_router(server_handler)" src/serve.rs` | ⏳ TO VERIFY `[Quản-đốc-fed, Worker verify]` |
| 2 | `Cargo.toml:3` `version = "0.1.0"` | `grep -n 'version = ' Cargo.toml` | ✅ `[verified]` Read Cargo.toml:3 = `version = "0.1.0"` |
| 3 | README.md = 2-dòng stub | (Architect Read) | ✅ `[verified]` Read README.md — title + 1 mô tả |
| 4 | CHANGELOG head = v0.7.0 (P007) | `head -6 CHANGELOG.md` | ✅ `[verified]` Read CHANGELOG.md:5 = `## v0.7.0 — P007` |
| 5 | LICENSE tồn tại ở root (cần cho publish) | `ls LICENSE` | ✅ `[verified]` Glob → `LICENSE` ở root |
| 6 | rmcp 1.7: `#[tool_router]` (no flag) + `#[tool_handler] impl ServerHandler` + `get_info() -> ServerInfo` với `server_info: Implementation { name, version, .. }` là API đúng để override serverInfo | `cargo build` + `cargo check` (compiler SOUND oracle) | ⏳ TO VERIFY `[oracle: cargo, SOUND]` — crate source `~/.cargo/registry/src/.../rmcp-1.7.0/src/model.rs` (Implementation), `handler/server.rs` (get_info default) `[Quản-đốc-fed, needs Worker verify]` |
| 7 | Imports `rmcp::handler::server::ServerHandler`, `rmcp::model::{ServerInfo, Implementation}` exist | `cargo build` (compiler) | ⏳ TO VERIFY `[oracle: cargo, SOUND]` `[needs Worker verify]` exact path |
| 8 | `cargo publish --dry-run` hiện cảnh báo `0.1.0 already exists` + kéo dev files | `cargo publish --dry-run` (before fix) | ⏳ TO VERIFY — `target/package/claude-hooks-0.1.0/` tồn tại (`[verified]` Glob) xác nhận package từng chạy ở 0.1.0 |
| 9 | Handshake thật trả `serverInfo.name == "rmcp"` (before fix) | spawn `serve` + `initialize` → parse `serverInfo.name` | ⏳ TO VERIFY `[Quản-đốc-fed]` |
| 10 | 5 MCP tool list đủ sau refactor (`#[tool_handler]` wire tool_router đúng) | `tests/mcp_handshake.rs` `*_5_tools` test | ⏳ TO VERIFY — đây là RỦI RO CHÍNH (xem Lưu ý Task 1) |

**Anchor #6, #7, #10 là điểm rủi ro chính** — compiler (SOUND oracle) phán được #6/#7 (claim "API tồn tại + biên dịch"); #10 cần test thật phán (claim "tool_router vẫn wired sau khi bỏ `server_handler` flag"). Worker iterate cargo cho tới khi cả 3 xanh.

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

Rollback nếu refactor get_info làm vỡ 5-tool wiring: `git reset --hard $(cat .backup/P008/main-head.txt)` (trong worktree only).

---

## Debate Log

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Quản đốc Challenge (orchestrator recon đã verify src-side)

**Anchor verification (✅):**
- serve.rs ✅ `:90` `#[tool_router(server_handler)]` (refactor target → bỏ flag + explicit ServerHandler)
- Cargo.toml ✅ `:3` version `0.1.0` (bump 0.8.0) — `cargo publish --dry-run` xác nhận "0.1.0 already exists on crates.io" → bump bắt buộc
- README ✅ 2-line stub → viết usable
- rmcp get_info API ✅ crate source `model.rs:994` Implementation, `handler/server.rs:158` get_info default `Implementation::from_build_env()` (expand tại rmcp → "rmcp") — compiler-verified path

**Objections (Tầng 1):** None. get_info refactor approach (explicit `#[tool_handler] impl ServerHandler`) đúng cách rmcp 1.7. Version 0.8.0 hợp. exclude slim = Tầng 2. Rủi ro #10 (bỏ server_handler làm tool list rỗng) → bắt buộc verify real handshake `serverInfo.name=="claude-hooks"` + 5 tool NGAY sau build (Quản đốc sẽ chạy lại độc lập).

**Status:** ✅ ACCEPTED V1 — no challenges.

### Final consensus
- Phiếu version: V1 (no debate turn needed)
- Approved by: Quản đốc (Sếp uỷ quyền approval gate, 2026-06-09) — EXECUTE may begin

---

## Nhiệm vụ

### Task 1: get_info refactor — serverInfo.name "rmcp" → "claude-hooks"

**File:** `src/serve.rs`

**Tìm:** macro attribute `#[tool_router(server_handler)]` trên impl block của `HooksServer` (gần serve.rs:90 `[Quản-đốc-fed, Worker verify]` — grep `tool_router(server_handler)` để định vị exact line).

**Thay bằng / Thêm:**
1. Đổi `#[tool_router(server_handler)]` → `#[tool_router]` (bỏ flag `server_handler`).
2. Thêm explicit `ServerHandler` impl với `#[tool_handler]` macro:
```rust
#[tool_handler]
impl ServerHandler for HooksServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: "claude-hooks".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
            // optional: instructions: Some("5 hook tools: architect_guard, block_env_edit, block_unsafe_merge, session_banner, why_blocked".into()),
            ..Default::default()
        }
    }
}
```
3. Thêm imports cần: `rmcp::handler::server::ServerHandler`, `rmcp::model::{ServerInfo, Implementation}` `[needs Worker verify]` exact path — compiler chỉ ra nếu sai.

**Lưu ý (RỦI RO CHÍNH):**
- `#[tool_handler]` PHẢI wire `self.tool_router` vào ServerHandler — nếu macro không tìm thấy field `tool_router` hoặc đặt sai chỗ, **5 tool list sẽ rỗng** → handshake vỡ. Verify bằng `tests/mcp_handshake.rs` `*_5_tools` test (Anchor #10) NGAY sau khi build pass.
- `#[tool_handler]` macro mặc định đọc field tên `tool_router` trên struct (`HooksServer { tool_router: ToolRouter<Self> }` — ARCHITECTURE.md:174 `[unverified]`). Nếu field tên khác hoặc macro cần arg → compiler/test báo. Worker iterate.
- KHÔNG đổi `run()` / 5 `#[tool]` method / struct field — chỉ đổi macro attr + thêm impl block.
- `version: env!("CARGO_PKG_VERSION")` compile-time đọc Cargo.toml → tự thành "0.8.0" sau Task 3. KHÔNG hardcode version string.
- `[oracle: cargo, SOUND]` cho "API biên dịch"; nhưng claim "serverInfo.name THẬT == claude-hooks" cần handshake test phán (compiler câm với giá trị runtime) — xem Nghiệm thu Manual.

### Task 2: README.md — viết usable (English-friendly)

**File:** `README.md` (overwrite toàn bộ 2-dòng stub)

**Thay bằng:** README với các section (Worker đối chiếu CLI thật trước khi viết ví dụ — KHÔNG bịa flag):
- `# claude-hooks` + 1-câu vision.
- `## Install`: `cargo install --path .` / `cargo install claude-hooks`.
- `## CLI usage`: 5 subcmd. Mỗi cái — stdin JSON shape (`{ "tool_input": { "file_path": "...", ... } }`) + exit 0/2. Ví dụ wire `.claude/settings.json`: PreToolUse matcher `Read|Glob`→architect-guard, `Edit|Write|MultiEdit|NotebookEdit`→block-env-edit, `Bash`→block-unsafe-merge; SessionStart→session-banner.
- `## MCP mode`: `claude-hooks serve` (stdio JSON-RPC), liệt kê 5 tool gồm `why_blocked` (debug: tool_name+tool_input → which hook fired + blocked + reason). Ví dụ `.mcp.json` entry.
- `## Exit convention`: 0 allow / 2 block. Fail-open mặc định; `block-unsafe-merge` fail-CLOSED (gh fail/diff empty → block).

**Lưu ý:**
- Input QUA STDIN JSON, KHÔNG có CLI flag per-hook (`[verified]` ARCHITECTURE.md stdin-JSON Harness). Worker grep `Cmd::` / clap derive nếu nghi ngờ có flag.
- Số liệu/tên tool phải khớp ARCHITECTURE.md MCP tool table (5 tool). KHÔNG viết "4 tool".
- English-friendly (repo `cargo publish`-able). Gọn — README không phải doc đầy đủ, ARCHITECTURE.md mới là chi tiết; có thể link sang.

### Task 3: Cargo.toml — version bump 0.8.0 + exclude slim

**File:** `Cargo.toml`

**Tìm:** `version = "0.1.0"` (line 3 `[verified]`).

**Thay bằng:** `version = "0.8.0"`.

**Thêm vào `[package]` block:** key `exclude`:
```toml
exclude = ["scripts/", "docs/", "phieu/", ".sos-state/", ".backup/", "hooks/", ".claude/", ".github/"]
```

**Lưu ý:**
- Exclude list là Tầng 2 — Worker self-decide list cuối miễn `cargo publish --dry-run` build clean + binary đủ. Binary cần CHỈ `src/` + `Cargo.toml` + `README.md` + `LICENSE`.
- `tests/`: cân nhắc — `cargo publish` verify build chạy `cargo build` (không nhất thiết `cargo test`). Nếu dry-run clean khi exclude `tests/` → exclude được (giảm package size); nếu verify fail → giữ `tests/`. Worker iterate dry-run, log quyết định vào Discovery.
- KHÔNG đổi `description` / `keywords` / `license` / `repository` — đã đúng (`[verified]` Cargo.toml).
- `env!("CARGO_PKG_VERSION")` ở Task 1 sẽ đọc "0.8.0" sau bump này — thứ tự không quan trọng (compile-time).

### Task 4: ARCHITECTURE.md polish + Status

**File:** `docs/ARCHITECTURE.md`

**Tìm + sửa:**
1. Mọi chỗ ghi `#[tool_router(server_handler)]` (ARCHITECTURE.md:174 "with `#[tool_router(server_handler)]` macro", :177 "Macro `#[tool_router(server_handler)]` emits `ServerHandler` impl automatically") `[unverified]` → cập nhật phản ánh Việc 1: `#[tool_router]` + explicit `#[tool_handler] impl ServerHandler::get_info()` trả `server_info.name = "claude-hooks"`. Worker grep `server_handler` trong ARCHITECTURE.md, sửa mọi match.
2. Thêm dòng Status (Overview hoặc cuối): "Status: Phase 1-3 complete (P001–P007). Phase 4 ship-prep (P008): README + publish-ready. Phase 4 wire-tarot = P009." nếu chưa có.

**Lưu ý:**
- Đây là Docs Gate Tầng 1 cho MCP-surface change (get_info đổi behavior handshake) — CLAUDE.md bảng Docs Gate: "MCP tool add/đổi → ARCHITECTURE.md MCP section".
- KHÔNG viết lại toàn bộ — P002–P007 đã update; chỉ sửa serve section cho khớp + Status. Rà nhất quán 5 subcmd + 5 tool + exit/fail-open/fail-CLOSED (đã đúng — verify only).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/serve.rs` | Task 1: `#[tool_router(server_handler)]` → `#[tool_router]` + explicit `ServerHandler::get_info()` (name="claude-hooks") |
| `README.md` | Task 2: viết mới — Install + CLI + MCP + exit convention |
| `Cargo.toml` | Task 3: version 0.1.0→0.8.0 + `exclude` slim |
| `docs/ARCHITECTURE.md` | Task 4: serve section khớp get_info refactor + Status |
| `CHANGELOG.md` | Entry P008 (v0.8.0) |
| `docs/discoveries/P008.md` | Discovery report (mới) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `src/hooks/mod.rs` | 4 hook `_decide` + wrapper + render_banner — KHÔNG đụng, 93 test giữ pass |
| `src/io.rs` | `Decision` struct + exit const — không đổi |
| `src/main.rs` | clap dispatch — không đổi |
| `tests/mcp_handshake.rs` | `*_5_tools` test PHẢI pass sau refactor (có thể THÊM assertion `serverInfo.name == "claude-hooks"` nếu parse được — không bắt buộc) |
| `tests/cli.rs` | 32 CLI parity test giữ pass |
| `scripts/*.sh` | Bash oracle read-only — KHÔNG đụng |
| `.claude/settings.json` | routing matcher giữ nguyên (README ví dụ phải khớp file thật) |

---

## Luật chơi (Constraints)

1. **Port doctrine giữ:** Task 1 KHÔNG đổi behavior 4 hook / 5 tool routing — chỉ đổi serverInfo metadata. Tool routing table BẤT BIẾN.
2. **93 test cũ KHÔNG vỡ** — refactor get_info là metadata-only; nếu bất kỳ test cũ đỏ → có nghĩa wiring vỡ, escalate qua Discovery + iterate, KHÔNG nới test cho qua.
3. **`#[tool_handler]` PHẢI wire tool_router** — verify 5 tool list ngay sau build (Anchor #10). Nếu tool list rỗng/thiếu → rollback, tìm cách wire đúng (có thể `#[tool_handler]` cần ở vị trí khác hoặc cần `tool_router` arg).
4. **KHÔNG hardcode version** trong serve.rs — dùng `env!("CARGO_PKG_VERSION")`.
5. **KHÔNG `cargo publish` thật** — chỉ `--dry-run`. Publish lên crates.io không đảo được.
6. **KHÔNG bịa flag/tool trong README** — Worker đối chiếu CLI/MCP thật; số tool = 5.
7. **F-002:** `git add` cả file phiếu này.

---

## Nghiệm thu

### Automated
- [ ] `cargo build` clean
- [ ] `cargo test --all` — 93 test cũ pass (+ serverInfo.name assertion nếu thêm)
- [ ] `cargo clippy -- -D warnings` không warning

### Manual Testing
- [ ] **Real handshake (RỦI RO CHÍNH):** spawn `claude-hooks serve`, gửi `initialize` → parse response → `serverInfo.name == "claude-hooks"` (KHÔNG còn "rmcp"). 5 tool vẫn list đủ qua `tools/list`. (Worker chạy thủ công hoặc thêm assertion vào `tests/mcp_handshake.rs` nếu parse được serverInfo.)
- [ ] `cargo publish --dry-run` clean: build pass + version 0.8.0 (không conflict 0.1.0) + package slim (exclude áp dụng).

### Regression
- [ ] 5 MCP tool (`architect_guard / block_env_edit / block_unsafe_merge / session_banner / why_blocked`) list đủ + routing đúng (mcp_handshake routing tests pass).
- [ ] 4 CLI hook + serve behavior không đổi (cli.rs 32 test pass).

### Docs Gate
- [ ] `CHANGELOG.md` — entry P008 (v0.8.0): get_info refactor + serverInfo.name fix + README + version bump + exclude + ARCHITECTURE polish.
- [ ] `docs/ARCHITECTURE.md` — serve section khớp `#[tool_router]` + explicit get_info + Status (Tầng 1 MCP-surface).
- [ ] `README.md` — usable (Install + CLI + MCP + exit convention).

### Discovery Report
- [ ] Write `docs/discoveries/P008.md` (P038 per-phiếu pattern):
  - get_info refactor: `server_handler` flag → explicit impl — exact import path rmcp dùng (verified), `#[tool_handler]` wiring cách wire tool_router, có cần arg không.
  - version bump 0.8.0 + exclude list CUỐI Worker chốt (vs phiếu đề xuất) + lý do (`tests/` giữ hay loại).
  - `cargo publish --dry-run` result (before/after: 0.1.0-conflict → 0.8.0-clean, package size before/after).
  - serverInfo.name before "rmcp" → after "claude-hooks" (handshake bằng chứng).
  - Assumptions phiếu CORRECT/WRONG (Anchor #1, #6, #7, #10 với file:line citations).
  - Tier escalations (write "None" nếu không).
- [ ] Append 1-dòng index vào `docs/DISCOVERIES.md` (link P008.md).
