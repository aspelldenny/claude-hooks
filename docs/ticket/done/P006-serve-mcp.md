# PHIẾU P006: `serve` subcmd — MCP server (rmcp 1.7 stdio)

> **Loại:** Feature (NET-NEW, KHÔNG port — không có Bash oracle)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — Decision-core refactor đụng 4 hook security-surface; sai 1 nhịp logic làm vỡ CLI parity P002–P005 đã ship. MCP = surface mới. AUTO Tầng 1 dù phần lớn diff là mechanical.)
> **Ảnh hưởng:** `src/serve.rs`, `src/hooks/mod.rs` (refactor decision-core), `src/io.rs` (struct `Decision`), `src/main.rs` (dispatch — nếu cần), `tests/`
> **Dependency:** P001–P005 (xong). Branch base: `feat/P001-scaffold-cli` (Phase 3 stack).

---

## Context

### Vấn đề hiện tại

`serve` hiện là **stub** (`src/serve.rs`): in `"serve: not yet implemented (P006)"` ra stderr, exit 0 (ARCHITECTURE.md L184). PROJECT.md Scope IN #5 yêu cầu `serve` = MCP server stdio JSON-RPC expose 4 hook làm tool (composite `why_blocked` = P007, KHÔNG làm ở đây).

**Chặn cứng:** 4 hook hiện tại (`src/hooks/mod.rs`) là `fn xxx() -> i32` TRỘN 3 việc trong 1 hàm: (a) đọc stdin / env / git-fs, (b) quyết định block/allow, (c) print stderr + return exit code. MCP tool KHÔNG đọc stdin (nhận input qua JSON-RPC params) và KHÔNG print stderr (trả structured result) → **không gọi lại được logic hiện tại**. Phải tách *quyết định* khỏi *IO*.

**Khác bản chất P001–P005:** P006 KHÔNG có Bash oracle để parity. Verifier = **compiler** (oracle cho rmcp API correctness — Worker iterate `cargo build`) + **MCP handshake smoke** (tools/list trả đúng 4 tool) + **81 test cũ bất biến** (CLI parity verify-cò chính). Crate source đọc được tại `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rmcp-1.7.0/`.

### Giải pháp

3 phần, theo thứ tự:

1. **Decision-core refactor (Task quan trọng nhất, làm TRƯỚC):** tách mỗi hook thành `fn xxx_decide(<inputs đã parse>) -> Decision`. CLI wrapper giữ nguyên signature `-> i32`, gọi `_decide` rồi `eprintln!` + return exit. **81 test cũ PHẢI pass** (parity bất biến).
2. **MCP server (`src/serve.rs`):** struct `HooksServer` với `#[tool_router(server_handler)]`, 4 `#[tool]` method gọi `_decide`. `serve::run()` dựng tokio current-thread runtime + `serve(transport::stdio()).waiting()`.
3. **Handshake smoke + unit test:** spawn `claude-hooks serve`, JSON-RPC `initialize` + `tools/list`, assert 4 tool names. Decision-core fns test trực tiếp.

### Scope

- CHỈ sửa: `src/serve.rs` (thân chính), `src/hooks/mod.rs` (split decide/wrapper), `src/io.rs` (struct `Decision`), `src/main.rs` (CHỈ nếu dispatch `Cmd::Serve` cần đổi signature — verify trước), `tests/` (thêm handshake smoke).
- KHÔNG sửa: logic/exit-code/stderr-message của 4 hook (refactor mechanical, hành vi BẤT BIẾN). KHÔNG sửa `scripts/*.sh` (oracle). KHÔNG đụng `.mcp.json` (P006 chỉ build server; wiring vào client = P009/smoke, không thuộc phiếu này). KHÔNG thêm composite `why_blocked` (P007). KHÔNG sửa `Cargo.toml` dep TRỪ KHI compiler buộc thêm tokio feature → **escalate Quản đốc, đừng tự thêm** (Tầng 1 dep tweak).

### Skills consulted (optional)

**Quản đốc đã research rmcp 1.7 API surface + feed verbatim** (xem Task 0 anchors + Task 2/3 dưới). Nguồn: `rmcp-1.7.0/src/handler/server/router/tool.rs` top doc + `service.rs`.

---

## Task 0 — Verification Anchors

> Worker verify TRƯỚC khi code. Hai nhóm anchor: (A) src hiện tại — Quản-đốc-fed line, Worker grep confirm. (B) rmcp API — compiler-verified (Worker đọc crate source + `cargo build`, KHÔNG bịa).

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `fn architect_guard()` `-> i32` tại `src/hooks/mod.rs` ~L4 `[Quản-đốc-fed, Worker verify]` | `grep -n "fn architect_guard" src/hooks/mod.rs` | ⏳ TO VERIFY |
| 2 | `fn block_env_edit()` `-> i32` ~L97 `[Quản-đốc-fed, Worker verify]` | `grep -n "fn block_env_edit" src/hooks/mod.rs` | ⏳ TO VERIFY |
| 3 | `fn block_unsafe_merge()` `-> i32` ~L261 `[Quản-đốc-fed, Worker verify]` | `grep -n "fn block_unsafe_merge" src/hooks/mod.rs` | ⏳ TO VERIFY |
| 4 | `fn session_banner()` ~L692 (render → stdout, exit 0) `[Quản-đốc-fed, Worker verify]` | `grep -n "fn session_banner" src/hooks/mod.rs` | ⏳ TO VERIFY |
| 5 | `src/serve.rs` `run()` stub trả ALLOW (in stub msg, exit 0) `[Quản-đốc-fed, Worker verify]` | `grep -n "fn run" src/serve.rs` | ⏳ TO VERIFY |
| 6 | `src/main.rs` dispatch `Cmd::Serve => serve::run()` `[Quản-đốc-fed, Worker verify]` | `grep -n "Serve" src/main.rs` | ⏳ TO VERIFY |
| 7 | `io.rs` có harness `read_payload()` + const `ALLOW`/`BLOCK`; ToolInput fields `file_path/pattern/notebook_path/command` (all `Option<String>`) `[verified — ARCHITECTURE.md L143-167, unverified ở src level → Worker confirm]` | `grep -n "read_payload\|ALLOW\|BLOCK\|struct ToolInput" src/io.rs` | ⏳ TO VERIFY |
| 8 | rmcp `#[tool_router]`/`#[tool]` macro + `Parameters`/`Json` wrapper + `schemars` re-export có sẵn với features hiện tại `[oracle: cargo build, compiler-verified]` | đọc `rmcp-1.7.0/src/handler/server/router/tool.rs` top doc + `cargo build` | ⏳ TO VERIFY |
| 9 | `ServiceExt::serve(transport)` + `RunningService::waiting()` signature đúng như feed `[oracle: cargo build, compiler-verified]` | đọc `rmcp-1.7.0/src/service.rs` (~L170 serve, ~L545 waiting) + `cargo build` | ⏳ TO VERIFY |
| 10 | `transport::stdio()` trả `(Stdin, Stdout)` dùng được làm transport `[oracle: cargo build]` | grep `pub fn stdio` trong `rmcp-1.7.0/src/transport/` + `cargo build` | ⏳ TO VERIFY |
| 11 | tokio `new_current_thread().enable_all()` build với features `["rt","macros","io-std"]` (KHÔNG cần `rt-multi-thread`) — `enable_all` có thể buộc `time` feature `[oracle: cargo build]` | `cargo build` — nếu lỗi missing feature → DISCOVERY + **escalate Quản đốc** (đừng tự thêm dep) | ⏳ TO VERIFY |
| 12 | 81 test cũ tồn tại + pass trước refactor (baseline) `[Quản-đốc-fed: "81 test cũ", Worker verify count]` | `cargo test --all 2>&1 \| tail -5` (đếm baseline) | ⏳ TO VERIFY |

**Lưu ý anchor #11 (rủi ro cao nhất):** Cargo.toml hiện CÓ `rt`, `macros`, `io-std` nhưng KHÔNG `rt-multi-thread` cũng KHÔNG `time`. Feed dùng `new_current_thread()` (đúng — né multi-thread). `enable_all()` enable mọi driver có feature; nếu rmcp stdio service nội bộ cần timer (`time`) → compiler/runtime báo. Worker: nếu `cargo build` báo thiếu feature HOẶC runtime panic "no reactor" → DISCOVERY + escalate (dep tweak = Tầng 1).

### Pre-phiếu snapshot (Worker auto first-step)

> Worker EXECUTE FIRST ACTION trước mọi edit. Snapshot `.backup/P006/` (Sếp note branch yêu cầu này).

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

Rollback nếu refactor làm vỡ parity giữa chừng: `git reset --hard $(cat .backup/${PHIEU_ID}/main-head.txt)` (trong worktree phiếu thôi).

---

## Debate Log

> Schema: 1 turn = Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Quản đốc Challenge (orchestrator có Read src/ — verify src anchors)

**Anchor verification (src ✅):**
- Hook sigs ✅ `src/hooks/mod.rs:4,97,261,692` — `architect_guard/block_env_edit/block_unsafe_merge/session_banner` đều `-> i32` (refactor target)
- serve stub ✅ `src/serve.rs` — `run()->i32` trả ALLOW + eprintln "not yet implemented"
- main dispatch ✅ `src/main.rs:35` — `Cmd::Serve => serve::run()`
- io.rs ✅ — Decision struct sẽ thêm; rmcp API anchors = compiler-verified (Worker iterate cargo)

**Objections (Tầng 1):** None. Decision-core refactor sound (tách `_decide` khỏi IO, CLI wrapper giữ `-> i32` → 81 test cũ bất biến). rmcp macro pattern khớp crate source. tokio runtime `new_current_thread().enable_all()`.

**Pre-authorization (Quản đốc, để né escalate roundtrip):** anchor #11 — nếu `enable_all()` buộc thêm tokio feature (`time`, hoặc `net`/`rt-multi-thread` cho IO/timer driver mà rmcp đòi), Worker ĐƯỢC PHÉP thêm feature TỐI THIỂU cần thiết vào `tokio` dep trong Cargo.toml. Đây là dep tweak BẮT BUỘC + đã lường trước cho MCP runtime, KHÔNG phải scope creep. Điều kiện: (a) chỉ thêm feature tối thiểu compiler đòi, KHÔNG thêm crate mới; (b) GHI Discovery rõ feature nào + tại sao; (c) nếu cần crate MỚI (ngoài tokio feature) → DỪNG + escalate thật.

**Status:** ✅ ACCEPTED V1 — no challenges.

### Final consensus
- Phiếu version: V1 (no debate turn needed)
- Total turns: 0
- Approved by: Quản đốc (Sếp uỷ quyền approval gate, 2026-06-09) — EXECUTE may begin. Tokio-feature pre-auth ghi trên.

---

## Nhiệm vụ

### Task 1: Struct `Decision` (shared contract)

**File:** `src/io.rs` `[anchor #7 — Worker confirm io.rs là nơi đúng; nếu hooks/mod.rs hợp lý hơn, Worker tự quyết — Tầng 2 vị trí, miễn pub reachable từ cả hooks + serve]`

**Thêm:**
```rust
/// Decision tách khỏi IO: hook core trả cái này, KHÔNG print, KHÔNG exit, KHÔNG đọc stdin.
/// CLI wrapper map -> stderr + exit code. MCP tool map -> structured JSON output.
#[derive(Debug, Clone, PartialEq)]
pub struct Decision {
    pub exit_code: i32,        // ALLOW (0) hoặc BLOCK (2)
    pub blocked: bool,
    pub reason: Option<String>, // stderr message khi blocked; None khi allow
}
```

**Lưu ý:**
- `exit_code` dùng const `ALLOW`/`BLOCK` đã có (anchor #7). `blocked == (exit_code == BLOCK)` — giữ cả 2 field cho MCP output rõ ràng (tool trả `blocked` + `exit_code` + `reason`, P006 mô tả MCP tools dưới).
- KHÔNG `#[derive(Serialize)]` ở đây nếu io.rs không có serde import cho struct này — output serialize làm ở Task 3 (`DecisionOutput` riêng trong serve.rs). Tránh kéo schemars vào io.rs (CLI path không cần). `[needs Worker verify: io.rs đã import serde chưa]`

### Task 2: Decision-core refactor — tách `_decide` khỏi 4 hook

**File:** `src/hooks/mod.rs` `[anchors #1–4]`

**Mục tiêu:** mỗi hook tách 2 phần. **Hành vi CLI BẤT BIẾN** — 81 test cũ pass không sửa.

**2a. `architect_guard` (anchor #1):**

**Tìm:** thân `fn architect_guard() -> i32` hiện tại (marker gate + path forbidden check + block, theo ARCHITECTURE.md L23-52 pipeline 8 bước).

**Thay bằng cấu trúc:**
```rust
/// Core: marker gate + forbidden path. fs check THẬT (đọc .sos-state/architect-active).
/// Nhận path đã parse (Option), KHÔNG đọc stdin. Trả Decision.
pub fn architect_guard_decide(file_path: Option<&str>, pattern: Option<&str>) -> Decision { /* logic cũ, return Decision thay vì print+exit */ }

/// CLI wrapper — hành vi cũ y nguyên.
pub fn architect_guard() -> i32 {
    let p = read_payload();
    let d = architect_guard_decide(p.tool_input.file_path.as_deref(), p.tool_input.pattern.as_deref());
    if let Some(r) = &d.reason { eprintln!("{r}"); }
    d.exit_code
}
```

**Lưu ý:** marker gate (`.sos-state/architect-active` exists) là fs read THẬT bên trong `_decide` — MCP tool gọi sẽ check marker thật của môi trường serve chạy. Đây ĐÚNG ý đồ (Sếp feed: "call _decide với marker fs check thật"). Repo-root resolve (`CLAUDE_PROJECT_DIR`) giữ trong `_decide`. `[needs Worker verify: signature `architect_guard()` không nhận arg — confirm anchor #1]`

**2b. `block_env_edit` (anchor #2):** tách `block_env_edit_decide(file_path: Option<&str>, notebook_path: Option<&str>) -> Decision`. Wrapper `block_env_edit()` gọi nó + eprintln + exit. Logic basename + allowlist `.env.example` + regex `^\.env($|\.)` (ARCHITECTURE.md L54-73) BẤT BIẾN.

**2c. `block_unsafe_merge` (anchor #3):** tách `block_unsafe_merge_decide(command: Option<&str>) -> Decision`. **Fail-CLOSED giữ nguyên** (ARCHITECTURE.md L116: gh fail/empty → BLOCK). gh-shelling NẰM TRONG `_decide` (core làm fs/gh per Sếp feed). Wrapper gọi + eprintln + exit. **Lưu ý MCP context:** khi gọi qua MCP, `gh` có thể fail (env khác) → `_decide` fail-CLOSED trả `blocked=true` + reason phản ánh "gh unavailable" — tool output trung thực, KHÔNG giả ALLOW.

**2d. `session_banner` (anchor #4):** tách `pub fn render_banner() -> String` (core: build banner text từ fs/git state, ARCHITECTURE.md L84-95 render pipeline 10 bước — KHÔNG print). Wrapper `session_banner() -> i32` gọi `print!("{}", render_banner())` + return ALLOW (luôn exit 0, fail-open). **F-001 verbatim bug GIỮ NGUYÊN** trong `render_banner()` (ARCHITECTURE.md L101 — KHÔNG fix ở đây).

**Lưu ý chung Task 2:**
- Đây là refactor MECHANICAL: di chuyển logic vào `_decide`, wrapper mỏng. KHÔNG đổi 1 ký tự nào của block message / exit code / regex / pipeline order.
- Test cũ có thể test qua wrapper (`-> i32` + stderr) HOẶC test nội bộ. Nếu test cũ gọi hàm internal đã đổi tên → đó là test sửa hợp lệ NHƯNG phải assert CÙNG kết quả. Worker: ưu tiên giữ wrapper signature để test integration `assert_cmd` (CLI level) không đụng. `[needs Worker verify: 81 test cũ gọi gì — internal fn hay CLI binary]`
- Nếu split lộ ra logic không tách sạch được (vd hàm trộn stdin-read sâu) → DISCOVERY, KHÔNG đổi hành vi để cho dễ tách.

### Task 3: MCP server — `src/serve.rs`

**File:** `src/serve.rs` `[anchor #5]`

**Tìm:** stub `run()` hiện tại (in `"serve: not yet implemented (P006)"`, exit 0).

**Thay bằng** (pattern Quản-đốc-fed từ `rmcp-1.7.0/src/handler/server/router/tool.rs` top doc — `[oracle: cargo build, Worker verify API verbatim từ crate source]`):

```rust
use rmcp::{tool_router, tool, ServiceExt, transport, schemars};
use rmcp::handler::server::wrapper::{Parameters, Json};
use rmcp::handler::server::tool::ToolRouter;
use serde::{Serialize, Deserialize};
use crate::hooks; // architect_guard_decide, block_env_edit_decide, block_unsafe_merge_decide, render_banner
use crate::io::Decision;

struct HooksServer { tool_router: ToolRouter<Self> }

#[derive(Deserialize, schemars::JsonSchema, Default)]
struct GuardInput { file_path: Option<String>, pattern: Option<String> }
#[derive(Deserialize, schemars::JsonSchema, Default)]
struct EnvEditInput { file_path: Option<String>, notebook_path: Option<String> }
#[derive(Deserialize, schemars::JsonSchema, Default)]
struct MergeInput { command: Option<String> }
#[derive(Deserialize, schemars::JsonSchema, Default)]
struct EmptyInput {}

#[derive(Serialize, schemars::JsonSchema)]
struct DecisionOutput { blocked: bool, exit_code: i32, reason: Option<String> }
impl From<Decision> for DecisionOutput {
    fn from(d: Decision) -> Self { Self { blocked: d.blocked, exit_code: d.exit_code, reason: d.reason } }
}
#[derive(Serialize, schemars::JsonSchema)]
struct BannerOutput { banner: String }

#[tool_router(server_handler)]
impl HooksServer {
    #[tool(name = "architect_guard", description = "Check Architect envelope: block Read/Glob to source paths when architect-active marker present. Returns block decision + reason.")]
    fn architect_guard(&self, Parameters(i): Parameters<GuardInput>) -> Json<DecisionOutput> {
        Json(hooks::architect_guard_decide(i.file_path.as_deref(), i.pattern.as_deref()).into())
    }
    #[tool(name = "block_env_edit", description = "Check if Edit/Write to a .env* file (not .env.example) should be blocked.")]
    fn block_env_edit(&self, Parameters(i): Parameters<EnvEditInput>) -> Json<DecisionOutput> {
        Json(hooks::block_env_edit_decide(i.file_path.as_deref(), i.notebook_path.as_deref()).into())
    }
    #[tool(name = "block_unsafe_merge", description = "Check if a `gh pr merge` command targets a security surface without an APPROVE review. May report gh-unavailable (fail-closed).")]
    fn block_unsafe_merge(&self, Parameters(i): Parameters<MergeInput>) -> Json<DecisionOutput> {
        Json(hooks::block_unsafe_merge_decide(i.command.as_deref()).into())
    }
    #[tool(name = "session_banner", description = "Render the SessionStart banner text (sprint + advisory staleness + orchestrator contract).")]
    fn session_banner(&self, Parameters(_): Parameters<EmptyInput>) -> Json<BannerOutput> {
        Json(BannerOutput { banner: hooks::render_banner() })
    }
}

impl HooksServer {
    fn new() -> Self { Self { tool_router: Self::tool_router() } }
}

/// Dispatch entry: dựng tokio current-thread runtime, serve stdio tới client close.
pub fn run() -> i32 {
    let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(e) => { eprintln!("serve: runtime build failed: {e}"); return crate::io::ALLOW; }
    };
    rt.block_on(async {
        match HooksServer::new().serve(transport::stdio()).await {
            Ok(service) => { let _ = service.waiting().await; }
            Err(e) => eprintln!("serve: handshake failed: {e}"),
        }
    });
    crate::io::ALLOW
}
```

**Lưu ý Task 3 (Worker verify từng điểm — đây là vùng compiler-oracle, KHÔNG bịa):**
- `[oracle: cargo build]` Tên đường dẫn import (`rmcp::handler::server::wrapper::{Parameters, Json}`, `ToolRouter`, `ServiceExt`, `transport::stdio`) — Worker ĐỌC `rmcp-1.7.0/src/` confirm path thật. Nếu re-export ở `rmcp::` top level → dùng đường ngắn. Đây là Tầng 2 (import path local), Worker tự sửa cho compiler pass, KHÔNG cần escalate.
- `[oracle: cargo build]` `#[tool_router(server_handler)]` flag: feed nói flag này auto-gen `#[tool_handler]` + `ServerHandler` (tools-only). Nếu rmcp 1.7 đặt tên flag khác (vd `server` thay `server_handler`) → Worker đọc macro source `tool.rs` confirm, sửa cho đúng. DISCOVERY ghi flag thật.
- `[oracle: cargo build]` `Json<T>` return wrapper + `Parameters<T>` extractor — confirm tồn tại & là cách trả structured output. Nếu rmcp 1.7 dùng `CallToolResult` thay `Json<T>` → Worker đọc top-doc example, theo example crate (example = oracle vì compiler pass).
- Method sync (`fn`, không `async`) vì `_decide` sync. Nếu macro buộc `async fn` → Worker wrap `async { ... }`. Tầng 2.
- `serve()` cần `ServiceExt` trait in scope. `.waiting()` chạy tới client close. Anchor #9.
- `run()` trả `i32` (giữ dispatch contract `Cmd::Serve => serve::run()` anchor #6). Nếu main dispatch expect khác → verify anchor #6, đổi tối thiểu.
- **session_banner qua MCP:** `render_banner()` đọc fs/git của môi trường serve. Banner có thể rỗng/khác nếu serve chạy ngoài repo — ĐÚNG (trung thực với state). KHÔNG mock.

### Task 4: Handshake smoke + unit test

**File:** `tests/` (file mới, vd `tests/mcp_handshake.rs` — Worker quyết tên `[Tầng 2]`)

**Thêm 2 nhóm test:**

**4a. Decision-core unit (test trực tiếp, không qua MCP):**
- `architect_guard_decide(Some("src/main.rs"), None)` với marker present → `blocked=true, exit_code=BLOCK`. Marker absent → ALLOW. (Marker = fs; test set/unset `.sos-state/architect-active` trong tempdir HOẶC test path-logic riêng nếu marker khó inject — Worker quyết.)
- `block_env_edit_decide(Some(".env.local"), None)` → blocked; `Some(".env.example")` → allow.
- `[needs Worker verify: cách inject marker/repo-root trong test — nếu `_decide` đọc env CLAUDE_PROJECT_DIR, set env trong test]`

**4b. MCP handshake smoke (Sếp: verify-cò chính cho MCP):**
- Spawn `claude-hooks serve`, gửi qua stdin chuỗi JSON-RPC: `initialize` request → rồi `tools/list` request. Assert response chứa 4 tool names (`architect_guard`, `block_env_edit`, `block_unsafe_merge`, `session_banner`) + valid JSON-RPC envelope.
- **Cách làm (Worker quyết `[Tầng 2]`):** `assert_cmd` `write_stdin` với handshake sequence rồi assert stdout; HOẶC script test riêng (`std::process::Command` spawn + pipe stdin/stdout + đọc response). Lưu ý JSON-RPC stdio cần đúng framing (rmcp dùng line-delimited hay Content-Length? → Worker đọc `transport::stdio` source confirm framing trước khi viết bytes).
- **Fallback nếu full handshake khó trong test sync** (đọc response async, framing phức tạp): tối thiểu assert `tools/list` trả đúng 4 tool. Nếu cả thế vẫn cần seam (vd phải refactor server để testable) → **DISCOVERY ghi seam cần**, KHÔNG over-engineer. Một smoke chứng minh server handshake + liệt kê tool là đủ DoD.

**Lưu ý:** MCP server cần client gửi `initialize` TRƯỚC `tools/list` (JSON-RPC MCP spec). Nếu gửi `tools/list` trần không initialize → server có thể reject. Worker: gửi đúng sequence. Đọc 1 example handshake trong `rmcp-1.7.0/` nếu có (`tests/` hoặc `examples/`).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/io.rs` | Task 1: thêm `pub struct Decision` (hoặc nơi Worker chọn nếu io.rs không hợp) |
| `src/hooks/mod.rs` | Task 2: tách 4× `_decide` core + giữ wrapper `-> i32`; `render_banner()` core |
| `src/serve.rs` | Task 3: thay stub `run()` bằng `HooksServer` + 4 `#[tool]` + tokio runtime + serve stdio |
| `src/main.rs` | CHỈ NẾU dispatch `Cmd::Serve` cần đổi (anchor #6 verify trước) |
| `tests/mcp_handshake.rs` (mới) | Task 4: unit decide + MCP handshake smoke |
| `docs/ARCHITECTURE.md` | Docs Gate: MCP section thật (serve, 4 tool, Decision-core refactor note) |
| `CHANGELOG.md` | Entry P006 |
| `docs/discoveries/P006.md` (mới) | Discovery report |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `scripts/*.sh` | Oracle, KHÔNG đụng |
| `.mcp.json` | KHÔNG wire `claude_hooks` ở P006 (chỉ build server; wiring = P009/smoke) |
| `Cargo.toml` | KHÔNG thêm dep TRỪ KHI compiler buộc tokio feature → **escalate Quản đốc** (Tầng 1) |
| 4 hook block message / exit code / regex | BẤT BIẾN sau refactor — 81 test cũ là chứng cứ |

---

## Luật chơi (Constraints)

1. **CLI parity bất biến (cứng nhất):** sau Decision-core refactor, 81 test cũ PHẢI pass KHÔNG sửa hành vi. Exit code + stderr message + regex + pipeline order của 4 hook giữ verbatim. Refactor = di chuyển code, KHÔNG đổi logic.
2. **Compiler là oracle cho rmcp API.** Worker iterate `cargo build` tới clean. Mọi import path / macro flag / wrapper type → ĐỌC `rmcp-1.7.0/src/` + theo example crate, KHÔNG bịa. Pattern Quản-đốc-fed là điểm khởi đầu, không phải chân lý — compiler phán cuối.
3. **Tokio dep tweak = Tầng 1 → escalate.** Nếu `cargo build`/runtime buộc thêm feature (`time`, `rt-multi-thread`, …) → DISCOVERY + escalate Quản đốc. KHÔNG tự sửa Cargo.toml.
4. **Fail-CLOSED giữ cho block_unsafe_merge** kể cả qua MCP (gh fail → blocked + reason, KHÔNG giả ALLOW).
5. **F-001 verbatim bug giữ nguyên** trong `render_banner()` (port doctrine — fix ở upstream sos-kit, không ở đây).
6. **KHÔNG composite `why_blocked`** (P007). KHÔNG mock fs/git/gh trong MCP tool (trung thực với state môi trường serve).
7. **Import path / test file name / internal var = Tầng 2** — Worker tự quyết cho compiler pass, không cần escalate.

---

## Nghiệm thu

### Automated
- [ ] `cargo build` clean (compiler = oracle rmcp API).
- [ ] `cargo test --all` pass — **81 test cũ + test mới** (baseline 81 không vỡ — anchor #12).
- [ ] `cargo clippy -- -D warnings` không warning.

### Manual Testing
- [ ] `echo '<initialize>\n<tools/list>' | claude-hooks serve` → response liệt kê 4 tool, valid JSON-RPC (HOẶC handshake smoke test tự động chứng minh điều này).
- [ ] `claude-hooks architect-guard < payload.json` (CLI path) cho cùng exit code + stderr như trước refactor (sanity 1 hook).

### Regression
- [ ] 4 hook CLI parity P002–P005 bất biến (81 test cũ = chứng cứ cơ học).
- [ ] `session-banner` vẫn stdout + exit 0; `block-unsafe-merge` vẫn fail-CLOSED.

### Docs Gate (Tầng 1 — security-surface + MCP surface mới)
- [ ] `docs/ARCHITECTURE.md` — MCP section thật: serve subcmd (rmcp stdio current-thread runtime), 4 tool + input/output schema, **Decision-core refactor note** (`_decide` tách khỏi IO, wrapper mỏng). Cập nhật L182-184 (đang ghi stub) + L177 serve.rs comment.
- [ ] `CHANGELOG.md` — entry P006.

### Discovery Report
- [ ] Write `docs/discoveries/P006.md`:
  - **rmcp API thực dùng:** macro flag thật (`server_handler` vs khác), wrapper type (`Json<T>` vs `CallToolResult`), import path thật, method sync/async.
  - **Tokio runtime setup:** `new_current_thread().enable_all()` build OK với feature nào? Có phải thêm `time`/`rt-multi-thread`? (nếu có → ghi escalation Tầng 1).
  - **Decision-core refactor impact:** test nào đổi (internal vs CLI), có hook nào không tách sạch được.
  - **Handshake smoke cách làm:** assert_cmd vs script riêng, JSON-RPC framing (line-delimited vs Content-Length), fallback dùng tới không, seam cần (nếu có).
  - Assumptions phiếu — CORRECT/WRONG (anchor #1–12 với file:line citation).
  - Docs updated to match reality (hoặc "None").
  - Tier escalations (Cargo.toml tweak nếu xảy ra; "None" nếu không).
- [ ] Append 1-line index vào `docs/DISCOVERIES.md` (link P006.md).
