# PHIẾU P007: `why_blocked` composite MCP tool — debug router

> **Loại:** Feature (NET-NEW, KHÔNG port — không có Bash oracle)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — MCP surface mở rộng + security-debug tool: route 1 tool-call JSON tới đúng hook decision-core. Sai routing → trả lý do block/allow SAI cho Sếp debug security-surface. AUTO Tầng 1 dù diff nhỏ — security-surface + contract MCP.)
> **Ảnh hưởng:** `src/serve.rs` (thêm 1 `#[tool]` method + 2 struct I/O), `tests/` (unit routing + handshake 5-tool)
> **Dependency:** P006 (xong — `serve` MCP server + 4 tool + Decision-core `*_decide` đã ship). Branch base: `feat/P001-scaffold-cli` (Phase 3 stack). **Phiếu CUỐI Phase 3 — đóng MCP.**

---

## Context

### Vấn đề hiện tại

PROJECT.md Scope IN #5 + Success #2 yêu cầu `serve` expose **5 tools**, trong đó tool thứ 5 = composite `why_blocked`. P006 đã ship 4 hook tool riêng lẻ (`architect_guard`/`block_env_edit`/`block_unsafe_merge`/`session_banner`) — Sếp/Quản đốc muốn debug "vì sao hook chặn action X" phải BIẾT TRƯỚC tool nào fire cho `tool_name` đó rồi gọi đúng tool đó. Đó là việc routing mà Claude Code làm qua `.claude/settings.json` PreToolUse matchers — Sếp phải đọc Bash sed / settings.json để mô phỏng.

**Giá trị `why_blocked`:** Sếp đưa NGUYÊN tool-call JSON (đúng shape Claude Code gửi PreToolUse: `{"tool_name":"Read","tool_input":{"file_path":"src/x.rs"}}`) → tool TỰ route tới đúng hook theo `tool_name` (giống matcher) → trả "vì sao chặn / cho qua" có cấu trúc kèm TÊN hook đã fire. 1 lời gọi = câu trả lời, không cần Sếp biết mapping trước.

**Khác P006:** P006 = build server + 4 tool 1:1 với 4 hook. P007 = thêm 1 tool COMPOSITE đứng trên — không có logic decision MỚI, chỉ ROUTE tới `*_decide` đã có. Net-new, nhỏ, build thuần trên P006. KHÔNG có Bash oracle (debug helper không tồn tại bản Bash). Verifier = **compiler** (rmcp macro, P006 đã chứng minh pattern) + **unit routing test** + **handshake smoke trả 5 tool** + **86 test cũ bất biến**.

### Giải pháp

Thêm vào `#[tool_router(server_handler)] impl HooksServer` (serve.rs, P006) đúng 1 method mới `why_blocked`:

1. **Input struct** `WhyBlockedInput { tool_name: String, tool_input: ToolInputArg }` — mirror shape PreToolUse payload. `ToolInputArg` = struct phẳng gom field optional (`file_path?, pattern?, notebook_path?, command?`) tái dùng cho mọi tool_name.
2. **Output struct** `WhyBlockedOutput { hook: String, blocked: bool, exit_code: i32, reason: Option<String> }` — `hook` = tên hook đã route tới (`"architect_guard"|"block_env_edit"|"block_unsafe_merge"|"none"`).
3. **Routing logic:** `match tool_name.as_str()` theo đúng `.claude/settings.json` PreToolUse matchers (Quản-đốc-fed verbatim, anchor #3) → gọi `*_decide` tương ứng → wrap Decision vào output kèm `hook` name. tool_name không match matcher nào → `hook:"none", blocked:false, exit_code:0, reason:Some("no hook matches tool <name>")`.

Macro `#[tool_router]` tự thêm method vào `ToolRouter` → server giờ có **5 tool**.

### Scope

- CHỈ sửa: `src/serve.rs` (thêm 1 `#[tool]` method + 2 struct `WhyBlockedInput`/`WhyBlockedOutput` + `ToolInputArg`; reuse `DecisionOutput`/import sẵn). `tests/` (unit routing + mở rộng handshake 4→5 tool). `docs/ARCHITECTURE.md` (MCP section: 4→5 tool + routing table). `CHANGELOG.md`. `docs/discoveries/P007.md`.
- KHÔNG sửa: `src/hooks/mod.rs` (4 `*_decide` BẤT BIẾN — why_blocked chỉ GỌI, không đổi logic decision). `src/io.rs` (`Decision` reuse). 4 tool P006 cũ (giữ nguyên). `scripts/*.sh` (oracle). `.mcp.json` (wiring = P009). `Cargo.toml` (KHÔNG dep mới — rmcp/serde đã đủ; nếu compiler buộc → escalate Quản đốc Tầng 1).
- KHÔNG route tới `session_banner`/`render_banner`: banner KHÔNG phải decision (không block/allow), why_blocked CHỈ trả lý-do-chặn → SessionStart không thuộc PreToolUse block-domain. tool_name nào map banner → rơi vào nhánh `"none"`.

### Skills consulted (optional)

**Quản đốc đã feed code-state P006 verbatim** (serve.rs structure + `*_decide` signatures + settings.json routing mapping) — xem Task 0 anchors. Quản đốc có Read src/ (architect không) → src anchor = `[Quản-đốc-fed, Worker verify]`.

---

## Task 0 — Verification Anchors

> Worker verify TRƯỚC khi code. (A) src P006 — Quản-đốc-fed, Worker grep confirm. (B) routing mapping — verify từ `.claude/settings.json`. (C) rmcp macro — compiler-verified (P006 đã chứng minh).

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `struct HooksServer { tool_router: ToolRouter<Self> }` + `#[tool_router(server_handler)] impl HooksServer` (4 `#[tool]` method) tồn tại trong `src/serve.rs` `[Quản-đốc-fed, Worker verify]` | `grep -n "tool_router\|struct HooksServer" src/serve.rs` | ⏳ TO VERIFY |
| 2 | `struct DecisionOutput { blocked, exit_code, reason }` + `impl From<Decision> for DecisionOutput` có trong serve.rs (reuse được) `[Quản-đốc-fed, Worker verify]` | `grep -n "struct DecisionOutput\|impl From<Decision>" src/serve.rs` | ⏳ TO VERIFY |
| 3 | `pub fn architect_guard_decide(file_path: Option<&str>, pattern: Option<&str>) -> Decision`; `block_env_edit_decide(file_path, notebook_path)`; `block_unsafe_merge_decide(command)` trong `src/hooks/mod.rs` `[Quản-đốc-fed + ARCHITECTURE.md L160-165, Worker verify]` | `grep -n "_decide" src/hooks/mod.rs` | ⏳ TO VERIFY |
| 4 | `.claude/settings.json` PreToolUse matchers: `Read\|Glob`→architect_guard; `Edit\|Write\|MultiEdit\|NotebookEdit`→block_env_edit; `Bash`→block_unsafe_merge `[Quản-đốc-fed verbatim, Worker verify từ settings.json — đây là ROUTING TRUTH]` | `grep -n "Read\|Glob\|Edit\|Write\|MultiEdit\|NotebookEdit\|Bash\|architect-guard\|block-env-edit\|block-unsafe-merge" .claude/settings.json` | ⏳ TO VERIFY |
| 5 | rmcp `#[tool(name=..., description=...)]` + `Parameters<T>` extractor + `Json<T>` return wrapper hoạt động (P006 đã dùng cho 4 tool) `[oracle: cargo build, P006-proven]` | `cargo build` (macro pattern khớp 4 tool P006 hiện có) | ⏳ TO VERIFY |
| 6 | Input struct nested cần derive `Deserialize + rmcp::schemars::JsonSchema` để là tool param (4 Input struct P006 đã derive vậy — anchor pattern) `[oracle: cargo build, P006-proven]` | `grep -n "JsonSchema\|Deserialize" src/serve.rs` (xem pattern P006) + `cargo build` | ⏳ TO VERIFY |
| 7 | Baseline test count = 86 (P006 ship: 81 CLI + 5 mcp) `[Quản-đốc-fed "86 test cũ", Worker verify count]` | `cargo test --all 2>&1 \| tail -5` (đếm baseline TRƯỚC sửa) | ⏳ TO VERIFY |
| 8 | Handshake smoke test P006 (`tests/mcp_handshake.rs` hoặc tên Worker P006 chọn) assert 4 tool trong `tools/list` — cần đổi thành 5 `[Quản-đốc-fed, Worker verify tên file + assertion]` | `grep -rn "tools/list\|architect_guard\|block_env_edit" tests/` | ⏳ TO VERIFY |

**Lưu ý anchor #4 (ROUTING TRUTH — rủi ro cao nhất):** mapping `tool_name → hook` PHẢI khớp `.claude/settings.json` matchers verbatim — nếu lệch, why_blocked trả lý do của HOOK SAI (vd báo "block_env_edit cho qua" trong khi thực tế Claude Code fire architect_guard). Worker ĐỌC settings.json matchers thật, KHÔNG bịa từ trí nhớ. Đặc biệt: `block_env_edit` matcher bao gồm `MultiEdit` + `NotebookEdit` (KHÔNG chỉ Edit/Write) — Quản-đốc-fed có nêu, Worker confirm cả 4 tool_name route về block_env_edit.

**Lưu ý anchor #3 (KHÔNG có `session_banner` trong route):** `render_banner() -> String` KHÔNG phải `_decide` → why_blocked KHÔNG gọi nó. Chỉ 3 `*_decide` được route. Đừng thêm nhánh banner.

### Pre-phiếu snapshot (Worker auto first-step)

> Worker EXECUTE FIRST ACTION trước mọi edit. Snapshot `.backup/P007/` (Sếp note branch yêu cầu này).

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

Rollback nếu cần: `git reset --hard $(cat .backup/${PHIEU_ID}/main-head.txt)` (trong worktree phiếu thôi).

---

## Debate Log

> Schema: 1 turn = Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Quản đốc Challenge (orchestrator verify src-side trực tiếp)

**Anchor verification (✅):**
- serve.rs ✅ `HooksServer` + `#[tool_router(server_handler)]` + 4 tool + `DecisionOutput`/`From<Decision>` (P006 shipped)
- `_decide` sigs ✅ `src/hooks/mod.rs:6,91,251` — `architect_guard_decide(file_path,pattern)`, `block_env_edit_decide(file_path,notebook_path)`, `block_unsafe_merge_decide(command)` đều `Option<&str>` → Decision
- routing ✅ `.claude/settings.json:16,25,38` — `Read|Glob`→architect-guard, `Edit|Write|MultiEdit|NotebookEdit`→block-env-edit, `Bash`→block-unsafe-merge (feed verbatim, khớp)
- rmcp macro ✅ compiler-verified (P006 chứng minh; thêm tool method → 5 tool)

**Objections (Tầng 1):** None. Routing khớp settings.json verbatim (Luật #1 — lệch = báo sai lý do hook cho Sếp debug). Input nested mirror PreToolUse payload. Chỉ gọi 3 `_decide` bất biến (render_banner không route). Test ưu tiên case path-deterministic, marker-fs route fallback skip+Discovery (không over-engineer).

**Status:** ✅ ACCEPTED V1 — no challenges.

### Final consensus
- Phiếu version: V1 (no debate turn needed)
- Total turns: 0
- Approved by: Quản đốc (Sếp uỷ quyền approval gate, 2026-06-09) — EXECUTE may begin

---

## Nhiệm vụ

### Task 1: Input/Output struct cho `why_blocked`

**File:** `src/serve.rs` `[anchor #1, #2, #6 — đặt cạnh các Input struct P006 hiện có]`

**Thêm** (3 struct mới; derive theo đúng pattern 4 Input struct P006 đã chứng minh — Worker xem struct `GuardInput`/`MergeInput` P006 để copy derive set chính xác):

```rust
/// Phẳng — gom mọi field tool_input có thể có, tái dùng cho mọi tool_name.
/// Mirror shape Claude Code PreToolUse payload tool_input.
#[derive(Deserialize, schemars::JsonSchema, Default)]
struct ToolInputArg {
    file_path: Option<String>,
    pattern: Option<String>,
    notebook_path: Option<String>,
    command: Option<String>,
}

/// Mirror NGUYÊN tool-call JSON Claude Code gửi PreToolUse:
/// {"tool_name":"Read","tool_input":{"file_path":"src/x.rs"}}
#[derive(Deserialize, schemars::JsonSchema, Default)]
struct WhyBlockedInput {
    tool_name: String,
    #[serde(default)]
    tool_input: ToolInputArg,
}

/// Routed decision + TÊN hook đã fire (để Sếp biết hook nào quyết định).
#[derive(Serialize, schemars::JsonSchema)]
struct WhyBlockedOutput {
    hook: String,            // "architect_guard"|"block_env_edit"|"block_unsafe_merge"|"none"
    blocked: bool,
    exit_code: i32,
    reason: Option<String>,
}
```

**Lưu ý:**
- `[needs Worker verify: derive set]` — copy CHÍNH XÁC derive macro từ Input struct P006 (anchor #6). Nếu P006 dùng `rmcp::schemars::JsonSchema` (re-export) thay `schemars::JsonSchema` → theo P006 (compiler phán). `Serialize`/`Deserialize` import đã có trong serve.rs (P006 dùng cho DecisionOutput).
- `#[serde(default)]` trên `tool_input` để payload thiếu `tool_input` không panic (fail-open — đồng nhất triết lý harness io.rs ARCHITECTURE.md L209). `ToolInputArg` derive `Default` cho việc này.
- `WhyBlockedOutput` cố tình KHÔNG reuse `DecisionOutput` (anchor #2) vì cần thêm field `hook` đứng đầu. 3 field sau (`blocked/exit_code/reason`) shape giống `DecisionOutput` — Worker MAY build output từ `DecisionOutput` rồi prepend `hook` (Tầng 2 cách dựng, miễn output JSON đúng 4 field).

### Task 2: Method `why_blocked` — routing logic

**File:** `src/serve.rs` `[anchor #1 — thêm vào `#[tool_router(server_handler)] impl HooksServer`, cạnh 4 method P006]`

**Thêm** 1 method (theo pattern 4 `#[tool]` method P006 — Worker xem method `architect_guard` P006 để khớp signature `&self, Parameters(i): Parameters<T>) -> Json<T>` chính xác):

```rust
#[tool(
    name = "why_blocked",
    description = "Debug router: given a Claude Code PreToolUse tool-call ({tool_name, tool_input}), route to the matching hook by tool_name (per .claude/settings.json matchers) and return which hook fired + its block/allow decision + reason. tool_name with no matching hook returns hook=\"none\", allowed."
)]
fn why_blocked(&self, Parameters(i): Parameters<WhyBlockedInput>) -> Json<WhyBlockedOutput> {
    let ti = &i.tool_input;
    let (hook, d): (&str, crate::io::Decision) = match i.tool_name.as_str() {
        // Read | Glob → architect_guard (settings.json matcher, anchor #4)
        "Read" | "Glob" => (
            "architect_guard",
            hooks::architect_guard_decide(ti.file_path.as_deref(), ti.pattern.as_deref()),
        ),
        // Edit | Write | MultiEdit | NotebookEdit → block_env_edit (anchor #4 — cả 4 tool_name)
        "Edit" | "Write" | "MultiEdit" | "NotebookEdit" => (
            "block_env_edit",
            hooks::block_env_edit_decide(ti.file_path.as_deref(), ti.notebook_path.as_deref()),
        ),
        // Bash → block_unsafe_merge (anchor #4)
        "Bash" => (
            "block_unsafe_merge",
            hooks::block_unsafe_merge_decide(ti.command.as_deref()),
        ),
        // không matcher nào fire → no hook → allow
        other => {
            return Json(WhyBlockedOutput {
                hook: "none".to_string(),
                blocked: false,
                exit_code: crate::io::ALLOW,
                reason: Some(format!("no hook matches tool {other}")),
            });
        }
    };
    Json(WhyBlockedOutput {
        hook: hook.to_string(),
        blocked: d.blocked,
        exit_code: d.exit_code,
        reason: d.reason,
    })
}
```

**Lưu ý:**
- `[needs Worker verify]` đường dẫn `hooks::*_decide` — confirm `crate::hooks` đã import trong serve.rs (P006 dùng) + 3 fn `pub` (anchor #3). `crate::io::{Decision, ALLOW}` — confirm `ALLOW` const reachable (ARCHITECTURE.md L218 const trong io.rs; P006 serve.rs đã dùng `crate::io::ALLOW` trong `run()`).
- **Routing match arms PHẢI verbatim khớp settings.json matchers (anchor #4).** Đặc biệt nhánh giữa: 4 tool_name (`Edit`/`Write`/`MultiEdit`/`NotebookEdit`) cùng route block_env_edit. Worker confirm settings.json — nếu matcher thật KHÁC (vd thiếu `MultiEdit`, hoặc có tool_name khác) → SỬA match arms theo settings.json thật + DISCOVERY ghi mapping thực. Settings.json = nguồn chân lý, KHÔNG phải phiếu này.
- **Field truyền vào `_decide` phải đúng cái mỗi hook cần** (giống cách CLI wrapper P006 truyền — ARCHITECTURE.md L160-165): architect_guard cần `(file_path, pattern)`; block_env_edit cần `(file_path, notebook_path)`; block_unsafe_merge cần `(command)`. ToolInputArg gom đủ — chỉ pick field đúng cho mỗi nhánh.
- **architect_guard_decide đọc marker fs THẬT** (`.sos-state/architect-active`, ARCHITECTURE.md L54): khi gọi qua why_blocked, kết quả phản ánh marker của môi trường serve chạy. ĐÚNG ý đồ (trung thực với state) — KHÔNG mock. Sếp debug nên ý thức: nếu serve chạy ngoài context architect-active, route Read/Glob sẽ ALLOW (marker absent). Đây là behavior đúng, ghi rõ trong Discovery + ARCHITECTURE.
- **block_unsafe_merge_decide gh-shell THẬT + fail-CLOSED** (ARCHITECTURE.md L142): why_blocked route Bash → có thể trả `blocked=true, reason="gh unavailable"` nếu serve env không có gh/network. Trung thực, KHÔNG giả ALLOW.
- Method sync (`fn`) vì `_decide` sync — khớp 4 method P006. Nếu macro buộc `async` → Worker theo P006 pattern (Tầng 2).
- `description` chuỗi dài có escape `\"` — Worker giữ JSON-safe (đây là Rust string literal, `\"` hợp lệ).

### Task 3: Unit routing test + handshake 5-tool smoke

**File:** `tests/` `[Worker dùng file handshake P006 hiện có cho phần smoke; unit routing thêm cùng file hoặc file mới — Tầng 2]`

**3a. Unit routing test (KHÔNG phụ thuộc marker fs global — ưu tiên cao):**

Test routing tách khỏi fs-dependent path nếu được. Case khuyến nghị (Worker chọn, ưu tiên case deterministic không cần marker/gh global):
- `tool_name="Edit"`, `tool_input.file_path=".env.local"` → output `hook="block_env_edit", blocked=true` (block_env_edit_decide thuần path-logic, KHÔNG fs marker — deterministic).
- `tool_name="Edit"`, `tool_input.file_path=".env.example"` → `hook="block_env_edit", blocked=false`.
- `tool_name="Write"`/`"MultiEdit"`/`"NotebookEdit"` + `.env.local` → đều `hook="block_env_edit", blocked=true` (chứng minh cả 4 tool_name route đúng 1 hook).
- `tool_name="WebFetch"` (hoặc tool lạ) → `hook="none", blocked=false, exit_code=0, reason` chứa `"no hook matches tool WebFetch"`.
- `tool_name="Bash"`, `tool_input.command="echo hi"` (non-merge) → `hook="block_unsafe_merge"` + ALLOW (parse_merge_pr không match → block_unsafe_merge_decide trả ALLOW; KHÔNG gh-shell vì không phải `gh pr merge` — deterministic, không cần network). **Worker verify:** non-`gh pr merge` command có trigger gh-shell không (ARCHITECTURE.md L127: no match `gh pr merge\s+\d+` → ALLOW TRƯỚC gh call). Nếu có → đổi case command non-gh để tránh network dependency.

**3b. architect_guard route (marker fs — XỬ LÝ ISOLATION):**
- Route `tool_name="Read"` gọi `architect_guard_decide` đọc `.sos-state/architect-active` THẬT. Test cần isolation như P002 (set/unset marker trong tempdir + `CLAUDE_PROJECT_DIR` env, HOẶC skip nhánh này khỏi unit nếu inject marker khó). `[needs Worker verify: cách P002/P006 inject marker — xem test cũ; nếu `_decide` resolve root qua CLAUDE_PROJECT_DIR, set env trong test trỏ tempdir có/không có marker]`.
- **Fallback (nếu marker isolation khó trong unit):** chứng minh routing đúng qua nhánh KHÔNG-fs (3a đã đủ chứng minh match-arm dispatch + "none"). architect_guard route được cover ở handshake hoặc bỏ qua trong unit + DISCOVERY ghi "architect_guard route fs-dependent, verify qua P002 marker test riêng". KHÔNG over-engineer marker injection chỉ để test 1 match arm.

**3c. Handshake 5-tool smoke (mở rộng test P006 — anchor #8):**
- File handshake P006 hiện assert `tools/list` trả 4 tool → ĐỔI thành 5 (thêm `why_blocked`). Assert response chứa cả 5 tool name: `architect_guard`, `block_env_edit`, `block_unsafe_merge`, `session_banner`, `why_blocked`.
- **Optional (nếu test P006 đã có `tools/call` harness):** call `why_blocked` với input `{tool_name:"Edit", tool_input:{file_path:".env.local"}}` → assert response có field `hook`/`blocked`. Nếu P006 chỉ làm `tools/list` (không `tools/call`) → tối thiểu đổi 4→5 trong tools/list assertion là đủ DoD; `tools/call` why_blocked = bonus.

**Lưu ý:**
- **86 test cũ (anchor #7) PHẢI pass.** test mới chỉ THÊM; nhánh sửa duy nhất = handshake assertion 4→5 (đó là test đổi HỢP LỆ vì tool count thật đổi).
- Worker quyết file test + cách spawn — Tầng 2. Ưu tiên case deterministic (path-logic) hơn case fs/network-dependent.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/serve.rs` | Task 1: thêm `ToolInputArg`/`WhyBlockedInput`/`WhyBlockedOutput`. Task 2: thêm `#[tool] why_blocked` method vào `impl HooksServer` |
| `tests/<handshake file P006>` | Task 3c: handshake assertion 4→5 tool |
| `tests/` (unit routing — file P006 hoặc mới) | Task 3a/3b: unit routing test |
| `docs/ARCHITECTURE.md` | Docs Gate Tầng 1: MCP section 4→5 tool + routing table `tool_name→hook` |
| `CHANGELOG.md` | Entry P007 |
| `docs/discoveries/P007.md` (mới) | Discovery report |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `src/hooks/mod.rs` | 3 `*_decide` BẤT BIẾN — why_blocked chỉ GỌI, không đổi logic. `render_banner` KHÔNG route |
| `src/io.rs` | `Decision`/`ALLOW` reuse — không sửa |
| 4 tool P006 trong serve.rs | giữ nguyên — chỉ THÊM tool thứ 5 |
| `scripts/*.sh` | Oracle, KHÔNG đụng |
| `.claude/settings.json` | ROUTING TRUTH — chỉ ĐỌC (anchor #4), KHÔNG sửa |
| `.mcp.json` | wiring = P009, KHÔNG đụng |
| `Cargo.toml` | KHÔNG dep mới (rmcp/serde đủ); nếu compiler buộc → escalate Quản đốc (Tầng 1) |

---

## Luật chơi (Constraints)

1. **Routing = settings.json verbatim (cứng nhất).** match arms `tool_name → hook` PHẢI khớp `.claude/settings.json` PreToolUse matchers (anchor #4). Lệch = trả lý do hook SAI cho Sếp debug security. Worker ĐỌC settings.json thật, sửa match theo nó nếu khác feed, DISCOVERY ghi mapping thực.
2. **`*_decide` BẤT BIẾN.** why_blocked chỉ GỌI 3 decision-core P006 — KHÔNG đổi logic/exit-code/reason của chúng. 86 test cũ = chứng cứ cơ học.
3. **`session_banner`/`render_banner` KHÔNG route.** banner ≠ decision. tool_name map banner → nhánh `"none"`.
4. **Trung thực với state môi trường serve.** architect_guard route đọc marker fs thật; block_unsafe_merge route gh-shell thật + fail-CLOSED. KHÔNG mock để output "đẹp". Output phản ánh môi trường serve chạy — ghi rõ trong Discovery + ARCHITECTURE.
5. **Compiler = oracle cho rmcp.** macro `#[tool]`/`Parameters`/`Json`/derive set → theo pattern 4 tool P006 (đã pass). Worker iterate `cargo build`, KHÔNG bịa API. Import path/derive = Tầng 2, Worker tự sửa cho compiler pass.
6. **KHÔNG dep mới + KHÔNG Cargo.toml tweak.** P007 thuần trên rmcp/serde đã có (P006). Nếu compiler buộc thêm → DISCOVERY + escalate Quản đốc (Tầng 1).
7. **Fail-open input.** payload thiếu `tool_input` → `#[serde(default)]` + `ToolInputArg::default()`, không panic. Đồng nhất harness io.rs.

---

## Nghiệm thu

### Automated
- [ ] `cargo build` clean (compiler = oracle rmcp macro — 5 tool).
- [ ] `cargo test --all` pass — **86 test cũ (anchor #7) + test routing mới**. Baseline 86 không vỡ (handshake 4→5 là test đổi hợp lệ).
- [ ] `cargo clippy -- -D warnings` không warning.

### Manual Testing
- [ ] `tools/list` qua serve → liệt kê **5 tool** (thêm `why_blocked`). (Handshake smoke test tự động chứng minh — anchor #8.)
- [ ] (Optional) `tools/call why_blocked {tool_name:"Read", tool_input:{file_path:"src/x.rs"}}` → response có `hook="architect_guard"` (marker-dependent blocked/allow) + valid JSON.
- [ ] `tools/call why_blocked {tool_name:"WebFetch", tool_input:{}}` → `hook="none", blocked=false`.

### Regression
- [ ] 4 tool P006 (`architect_guard`/`block_env_edit`/`block_unsafe_merge`/`session_banner`) bất biến — vẫn fire qua MCP như cũ.
- [ ] CLI parity P002–P005 bất biến (81 CLI test trong 86 = chứng cứ).
- [ ] `block_unsafe_merge` vẫn fail-CLOSED; `session_banner` vẫn stdout exit 0.

### Docs Gate (Tầng 1 — MCP surface + security-debug)
- [ ] `docs/ARCHITECTURE.md` — MCP section: cập nhật "4 tool"→"5 tool" (L179, L236 + bảng L181-186 thêm row `why_blocked`). Thêm **routing table** `tool_name → hook` (Read/Glob→architect_guard; Edit/Write/MultiEdit/NotebookEdit→block_env_edit; Bash→block_unsafe_merge; else→none). Note: architect_guard route đọc marker fs thật + block_unsafe_merge route gh-shell thật. Cập nhật module structure note (serve.rs giờ 5 tool) + tests note (mcp_handshake giờ assert 5 tool).
- [ ] `CHANGELOG.md` — entry P007 (why_blocked composite router, 5 tool, Phase 3 DONE).

### Discovery Report
- [ ] Write `docs/discoveries/P007.md`:
  - **Routing mapping THỰC** từ `.claude/settings.json` (anchor #4): tool_name → hook verbatim. Có khác feed không? (đặc biệt nhánh block_env_edit: đúng 4 tool_name `Edit/Write/MultiEdit/NotebookEdit`?).
  - **Input shape thực:** nested `WhyBlockedInput{tool_name, tool_input:ToolInputArg}` build OK? derive set thực (`schemars::JsonSchema` vs `rmcp::schemars::JsonSchema`).
  - **Test strategy:** unit routing dùng case nào (deterministic path-logic vs fs/network)? architect_guard route — marker fs isolation cách nào HAY skip + lý do (3b fallback dùng tới không)? block_unsafe_merge non-merge command có gh-shell không?
  - **Tool count = 5:** handshake assertion đổi 4→5, file test nào.
  - **State-honesty note:** architect_guard route reads marker fs + block_unsafe_merge route gh-shell — behavior khi serve chạy ngoài architect-active context / không gh.
  - Assumptions phiếu — CORRECT/WRONG (anchor #1–8 với file:line citation).
  - Docs updated to match reality (hoặc "None").
  - Tier escalations (Cargo.toml tweak nếu xảy ra; "None" nếu không).
- [ ] Append 1-line index vào `docs/DISCOVERIES.md` (link P007.md).
- [ ] **Phase 3 DONE** — ghi trong CHANGELOG + nudge Sếp move BACKLOG P006+P007 xuống Recently shipped (Phase 3 close).
