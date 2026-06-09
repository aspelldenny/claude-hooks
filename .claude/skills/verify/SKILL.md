---
name: verify
version: 0.1.0
description: |
  Task 0 grep-first verifier — before coding a phiếu, grep every anchor (file, function, constant) against real code and report mismatches back to Architect.
  Invoke when: starting a phiếu, or when Architect says "verify anchors before implementing".
allowed-tools:
  - Bash
  - Read
  - Grep
  - Glob
---

# /verify — Task 0: Grep-First Anchor Verification

You are a **Worker** about to execute a phiếu. Before writing ANY code, run Task 0: verify every assumption in the phiếu against real code. Report mismatches back to Architect.

**This is a gate, not a checklist.** If Task 0 finds wrong assumptions, you STOP and escalate. You do not patch the phiếu silently.

## When to Invoke

- User says "verify", "grep anchors", "check phiếu", "Task 0"
- You've just opened a new phiếu and the Verification Anchors table has entries
- Architect says "verify before coding" / "check assumptions"

## Prerequisites

- Phiếu file is open (`docs/ticket/P<NNN>-<slug>.md`) with a populated Verification Anchors table
- You're in the project's worktree (created by `phieu <slug>`)
- Source code is accessible for grep

## Workflow

### Step 1: Read the phiếu's Task 0 table

Locate the `## Task 0 — Verification Anchors` section. Each row is an assumption to verify:

```markdown
| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `foo` function in `src/lib/x.ts` | `grep "function foo" src/lib/x.ts` | ✅ Line 12 |
| 2 | Constant `BAR` defined | `grep "BAR =" src/` | ⏳ TO VERIFY |
```

Rows marked `⏳` or empty in "Result" are yours to verify now.

### Step 2: Run each verify command

Execute the exact command in the "Verify by" column. Record the real result.

Possible outcomes:
- **✅ Found** — exact match at expected location. Update Result column: `✅ Line N`.
- **⚠️ Found but different** — symbol exists but signature / location / value differs from phiếu assumption. Record the actual state.
- **❌ Not found** — nothing matches. The phiếu assumption is wrong.

### Step 3: Decide what to do next

Count the results:

| Situation | Action |
|---|---|
| All ✅ | Task 0 passes. Proceed to code (other Nhiệm vụ tasks). |
| Any ⚠️ (different signature/location) | STOP. Report to Architect. Do NOT silently adapt. |
| Any ❌ (missing) | STOP. Report to Architect. The phiếu is wrong. |

### Step 4: Report back to Architect

If anything other than all-✅, write a report to Chủ nhà (who will forward to Kiến trúc sư via relay) in this format:

```
Task 0 — Verification Report for P<NNN>-<slug>

✅ Verified (N):
  - [assumption 1]
  - [assumption 2]

⚠️ Different from phiếu (M):
  - [assumption X]: phiếu said [Y], actual code is [Z at file:line]
  - Recommended phiếu update: [short suggestion]

❌ Not found (K):
  - [assumption W]: grepped [pattern], no match anywhere in src/
  - This assumption is wrong. Architect must update phiếu.

I'm stopping here. Architect: please update phiếu before I code.
```

### Step 5: If all ✅, update the phiếu file

Edit the Verification Anchors table, filling in real line numbers. Save. Now proceed to Task 1.

### Step 6: If blocked, WAIT

Do NOT start coding. Do NOT "fix the phiếu yourself." Wait for Architect to update the phiếu with correct assumptions. When they do, re-run Task 0 from Step 1.

## Rules (hard)

1. **Task 0 is a gate.** No "I'll verify as I go." Verify BEFORE any code change.
2. **Do not patch the phiếu silently.** If assumption is wrong, STOP and report. Architect owns the phiếu's correctness.
3. **Do not "adapt" to reality mid-code.** If you find a mismatch while coding (not in Task 0), stop and write a Discovery Report — then escalate.
4. **Exact commands only.** Run the "Verify by" command as written. Don't substitute a "close enough" grep.
5. **Report in the format above.** Architect scans for the ✅ / ⚠️ / ❌ counts to decide next action.

## Integration with Other Skills

- **Before /verify:** Architect ran `/plan` to produce the phiếu with Task 0 table
- **After /verify (all ✅):** proceed to implement tasks (Task 1, 2, ...), then `/review` → `/qa` → `/ship`
- **After /verify (any ⚠️/❌):** stop. Architect updates phiếu. Re-run /verify. NO coding until Task 0 passes.
- **Post-ticket:** write Discovery Report to `docs/DISCOVERIES.md` regardless — even if Task 0 was all ✅, you might have discovered edge cases during code execution.

## Voice

- English in skill text; mirror project language in the verification report
- Be precise: "file:line" or "not found" — no hedging
- Keep report tight: counts + each mismatch + one recommendation

## Why this exists

Without Task 0, the following pattern keeps happening: Architect writes phiếu based on docs. Docs are slightly stale. Worker assumes phiếu is correct. Worker codes for 1 hour, then hits "wait, function X doesn't exist." 1 hour wasted. Architect rewrites phiếu. Worker redoes work.

Task 0 front-loads the grep. 5 minutes of `grep` prevents 1 hour of wrong-direction code. It also surfaces doc drift early so it can be fixed in the Discovery Report.

## Example Session

```
User: Verify phiếu P044-history-export-csv

Worker (/verify):
Reading docs/ticket/P044-history-export-csv.md → Task 0 table has 4 rows.

Running:
  1. grep "export function useReadingHistory" src/hooks/useReadingHistory.ts
     → ✅ Line 12
  2. grep "/api/history" src/app/api/history -r
     → ⚠️ Found /api/history/route.ts but only GET, no POST for export
  3. grep "HistoryItem" src/types/
     → ✅ Line 45 of src/types/reading.ts
  4. grep "export-csv" src/
     → ❌ No existing export logic. Phiếu Task 2 creates it from scratch. OK.

Task 0 — Verification Report for P044-history-export-csv

✅ Verified (2):
  - useReadingHistory hook at src/hooks/useReadingHistory.ts:12
  - HistoryItem type at src/types/reading.ts:45

⚠️ Different from phiếu (1):
  - Assumption 2: phiếu said "/api/history supports POST for export".
    Actual: route.ts only has GET handler, no POST.
    Recommended update: Task 1 should ADD POST handler, not reuse existing.

❌ Not found (1):
  - Assumption 4: no existing export logic — phiếu acknowledges this in Task 2. OK.

Status: 1 ⚠️ → Kiến trúc sư, please update Task 1 wording (replace → add new).
Once updated, I'll re-run /verify then proceed to code.
```
