# PHIẾU P<NNN>: <short title>

> **ID format:** `P` + 3 digits (P001, P042, P123). ID is auto-assigned by the `phieu` shell function from `<project>/.phieu-counter` — do not set manually.
> **Filename:** `docs/ticket/P<NNN>-<slug>.md` (active) → moved to `docs/ticket/done/` on completion. Unified path for sos-kit + downstream (canonical value = `ticket_dir` in `.docs-gate.toml`). `phieu-done` handles the active → done/ move.
> **Branch:** `<type>/P<NNN>-<slug>` where `<type>` ∈ {feat, fix, chore, docs, infra}.
> **Usually created via `phieu <slug>`** (shell function auto-fills ID + creates branch + worktree + this file).

---

> **Loại:** Feature / Bugfix / Prompt-only / Hotfix
> **Ưu tiên:** P0 / P1 / P2
> **Tầng:** 1 (móng — sai thì LAN hoặc KHÔNG-đảo: kiến trúc/API/schema/auth/privacy/security/payment/INV-LOCAL → AUTO Tầng 1 dù nhỏ) | 2 (lặt vặt — sai thì cục bộ + sửa được, no móng). Định nghĩa: `docs/LAYERS.md` §2-tier. **LOC KHÔNG quyết Tầng.**
> **Ảnh hưởng:** [main files affected]
> **Dependency:** [which phiếu must finish first, or "None"]

---

## Context

### Vấn đề hiện tại
[Describe the problem or feature]

### Giải pháp
[Describe the approach]

### Scope
- CHỈ sửa [list files]
- KHÔNG sửa [list files that must not be touched]

### Skills consulted (optional)

<!-- Orchestrator: nếu bạn đã chạy skill (e.g., /frontend-design, /security-review) trước khi spawn Architect, paste output VERBATIM here. Frozen artifact for reproducibility. Most phiếu won't need this section — leave blank or delete it if no skill consulted. -->

<!-- Format example:
**Skill:** `/frontend-design` — invoked by Orchestrator 2026-MM-DD
**Output:**
[verbatim skill output here — design tokens, color palette, component spec, etc.]
-->

---

## Task 0 — Verification Anchors

> **REQUIRED** — Architect must grep/verify real code before writing assumptions.
> Worker reads this table to know which assumptions were verified vs. unverified.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | [Function X exists in file Y] | `grep "function X" src/...` | ✅ Line 123 |
| 2 | [Constant Z = "abc"] | `grep "Z" src/...` | ✅ Line 456 |
| 3 | [Spread list is constant SPREADS_REBUILD] | `grep "SPREADS_REBUILD" src/...` | ❌ NOT FOUND — inline string instead |

**If "Result" column has ❌ → architect acknowledged the wrong assumption and specified how to handle it in the Nhiệm vụ section below.**

### Pre-phiếu snapshot (Worker auto first-step)

> **Worker EXECUTE FIRST ACTION** (before any code edit, before Task 0 grep verification): take a rollback point so failed mid-execute can revert.

```bash
# Run from project root (worktree root for phiếu workflow):
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/ — auto-cleaned on phieu-done"
```

If the phiếu hits ❌ mid-execute and you need to roll back: `cp .backup/${PHIEU_ID}/settings.local.json .claude/` and `git reset --hard $(cat .backup/${PHIEU_ID}/main-head.txt)` (within phiếu worktree only — NEVER on main per safety rails).

`.backup/` is gitignored. `phieu-done` cleans up automatically.

---

## Debate Log

> Auto-populated by Worker (CHALLENGE mode) and Architect (RESPOND mode) when v2.1 debate flow is active. Chủ nhà chỉ đọc khi nghiệm thu phiếu — không cần can thiệp mid-debate trừ khi orchestrator triệu (max-turn cap reached or DEFER TO Chủ nhà).
> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Phiếu version bump V1 → V2 → ... mỗi turn Architect refine.
> Cap = 3 turns. Sau Turn 3 chưa consensus → force-escalate Chủ nhà.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge
*(Worker fills this when invoked in CHALLENGE mode. If no objections, write "Worker accepted V1 — no challenges. Ready for Chủ nhà approval." and skip to Final consensus.)*

**Anchor verification (recap from Task 0):**
- Anchor #N: ✅/⚠️/❌ + 1-line summary if ⚠️/❌

**Objections (Tầng 1 only — phiếu cần sửa):**
- [O1.1] Phiếu giả định X tại file Y, code thật là Z (cite `file:line`). Tác động: …
- [O1.2] …

**Proposed alternatives** (Worker recommends 1):
- A. … (Worker lean — vì …)
- B. …

**Status:** ⏳ AWAITING ARCHITECT RESPONSE

### Turn 1 — Architect Response
*(Architect fills this when invoked in RESPOND mode. Cannot read source code — relies on Worker's `file:line` citations.)*

- [O1.1] → ACCEPT / DEFEND / REFRAME (Tầng 2) / DEFER TO CHỦ NHÀ → action taken
- [O1.2] → …

**Status:** ✅ RESPONDED — phiếu bumped to V2

*(Repeat Turn 2, Turn 3 if needed. Cap = 3.)*

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Chủ nhà: [date] — code execution may begin

---

## Nhiệm vụ

### Task 1: [task name]

**File:** `src/path/to/file.ts`

**Tìm:** [exact text to locate — use content, NOT constant/variable names unless verified]

**Thay bằng / Thêm:**
```
[new content]
```

**Lưu ý:** [edge cases, conditions, cross-module interactions]

### Task 2: [...]

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/path/file.ts` | Task 1: short description |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `src/path/other.ts` | [function X should continue to work with the change] |

---

## Luật chơi (Constraints)

1. [Constraint 1]
2. [Constraint 2]

---

## Nghiệm thu

### Automated
- [ ] Type-check clean (e.g., `pnpm type-check` / `cargo check`)
- [ ] Tests pass (e.g., `pnpm test --run` / `cargo test`)

### Manual Testing
- [ ] [Test case 1]
- [ ] [Test case 2]

### Regression
- [ ] [Feature X still works]

### Docs Gate
- [ ] `CHANGELOG.md` — entry for this phiếu
- [ ] `[GUIDE].md` — [section updated]

### Discovery Report
- [ ] Write to `docs/discoveries/P<NNN>.md` (per-phiếu file, P038 pattern)
  - Assumptions in phiếu — CORRECT / WRONG (with file:line citations)
  - Scope expansions (if any — note original vs shipped, with reason)
  - Edge cases / limitations found
  - Docs updated to match reality (write "None" if nothing — explicit)
  - Tier escalations (write "None" if no 2→1 escalation)
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md` (link to per-phiếu file)
