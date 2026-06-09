# Discovery Protocol — Worker → Architect feedback loop

> Phiếu is a one-way document. Discovery Report is the return channel.

## Why this exists

Architect writes phiếu based on docs. Docs drift from code. Worker finds the drift while implementing. Without a formal feedback loop:

- Worker silently fixes the mismatch
- Architect keeps writing phiếu with the same stale assumption
- Same bug recurs in the next ticket

Discovery Report is the gate that prevents this. Every phiếu, no exceptions, produces a Discovery Report — even if nothing was wrong ("no assumptions broke — phiếu accurate").

## When Worker writes one

- Task 0 (`/verify`) finds an anchor was wrong → report it
- During code execution, Worker hits an assumption that breaks → report it
- Worker discovers an edge case the phiếu didn't cover → report it
- Worker updates a guide doc to match reality → reference it in the report

## Discovery Report vs Handoff 3 — which channel?

Not every mismatch goes in the Discovery Report. Some require STOPPING mid-ticket and escalating to Chủ nhà (Handoff 3 in `docs/HANDOFF.md`). Classify first:

### Tầng 2 mismatch (detail) → Discovery Report only
Worker auto-adopts the correct version, ships, writes Discovery at the end.

**Examples:**
- Phiếu says `const items = ...`, Worker prefers `const entries = ...` inside a private helper → use `entries`, log to Discovery
- Phiếu says "CSS class `.card-header`", Worker picks `.header` for brevity → log
- Phiếu says internal error log `"Prisma failed"`, Worker writes `"Prisma query error"` → log
- Phiếu says helper in `lib/utils.ts`, Worker puts in `lib/helpers.ts` (same module scope) → log

**Rule:** If another Worker maintaining the code later doesn't need to know — Tầng 2. Self-decide + Discovery.

### Tầng 1 mismatch (architecture) → STOP + Handoff 3
Worker does NOT self-adapt. Writes escalation to Chủ nhà → Chủ nhà forwards to Architect → Architect updates phiếu → Worker resumes.

**Examples:**
- Task 0 finds `❌` — phiếu's file/function/constant doesn't exist
- Phiếu's approach conflicts with existing architecture (e.g. phiếu says "add to endpoint X", but X is deprecated)
- Scope balloons — phiếu implies 1 file change, reality needs migration + 5 files
- Phiếu says use `Context API`, but existing code uses `zustand` everywhere — pattern drift
- User-visible wording — phiếu says "Tải xuống", Chủ nhà hasn't approved; copy is Chủ nhà's domain
- Security / data concern — phiếu approach leaks PII or bypasses rate limit

**Rule:** If another Worker maintaining the code later needs to know — Tầng 1. Escalate, don't silently fix.

### Quick decision tree

```
Worker finds phiếu ≠ reality
  │
  ├── Is this about: variable name / CSS / internal helper / log wording?
  │   └── Tầng 2. Self-adopt correct version. Log to Discovery Report (end of ticket).
  │
  ├── Is this about: function signature / file location / API shape / schema /
  │                   dependency / user-visible text / security?
  │   └── Tầng 1. STOP. Write Handoff 3 escalation to Chủ nhà.
  │
  └── Not sure?
      └── Default to Tầng 1. Over-escalating is fixable; silent drift is not.
```

### Tier as a routing key (P036)

Tầng is no longer only a *Discovery-Report classification* — it is now the **routing key set in the phiếu header during DRAFT** (`Tầng: 1 | 2`). See `docs/ORCHESTRATION.md` "Tier routing".

The decision tree above still applies *during execute* (Worker found a mismatch — is it Tầng 1 or 2?). The new rule layered on top:

**Worker mid-execute escalation 2 → 1:**

If a phiếu was marked `Tầng: 2` by Architect but Worker discovers during EXECUTE that the change actually touches:
- A schema/migration
- An API contract (request/response shape, status codes, auth header)
- An auth/security boundary
- A new external dependency
- Cross-module data flow

→ STOP coding. Do NOT silently complete. Append a Debate Log Turn 1 with `file:line` evidence of the móng-nhà collision. Return to orchestrator. The phiếu re-routes through full CHALLENGE flow.

**Why this matters:** A "small" billing fix that touches `auth.ts` is not Tầng 2, even if the diff is 20 LOC. The tier is about **blast radius of what could break**, not lines changed.

**Tầng 2 conditions (consequence — per `docs/LAYERS.md` §2-tier; LOC/file-count is NOT a criterion):**
- Consequence is LOCAL + REVERSIBLE (no external blast)
- No schema / API contract / auth / privacy / security / `INV-LOCAL` modification
- No new dependency
- All Task 0 anchors `[verified]` or surgical-only `[needs Worker verify]`

If ANY condition fails → Tầng 1.

## Where it goes

Append to `docs/DISCOVERIES.md` in the target project. Newest on top (like CHANGELOG).

If the project doesn't have `docs/DISCOVERIES.md`, create it on the first ticket:

```markdown
# Discoveries Log

> Worker → Architect feedback loop. Each entry records what the phiếu assumed vs. what the code actually was, plus edge cases found during implementation. Architect reads this BEFORE writing the next phiếu.
>
> Newest entries on top. See SOS Kit phieu/DISCOVERY_PROTOCOL.md for format.

---

```

## Entry format

```markdown
## [P<NNN>-<slug>] — YYYY-MM-DD — <one-line title>

### Assumptions in phiếu — CORRECT
- [Assumption X: phiếu said Y, code at file:line matches]
- (if nothing was right, write "None" — rare, but possible)

### Assumptions in phiếu — WRONG
- [Assumption W: phiếu said "function foo in lib/a.ts", actually in lib/b.ts line 42]
- [Assumption Z: phiếu said "constant BAR defined", actually inline string everywhere]
- (if all assumptions were correct, write "None")

### Edge cases / limitations discovered
- [Phiếu didn't mention Safari compat — found iOS 15 broke with X, added workaround]
- [Discovered existing rate limiter kicks in at 30 req/min, not 60 as docs said — updated BACKEND_GUIDE.md]
- (write "None" if nothing unexpected)

### Docs updated to match reality
- `BACKEND_GUIDE.md` section 3: corrected function signature for `foo`
- `PROMPTS.md` section 7: marker name was stale, updated
- (write "None" if no docs changed)
```

## Architect's responsibility

Before writing the NEXT phiếu on the same project, Architect MUST:
1. Read the top N entries of `docs/DISCOVERIES.md` (at least since last phiếu)
2. Fold corrections into the new phiếu's assumptions
3. If a discovery invalidates a common doc pattern, update the affected guide doc

This is not optional. A phiếu that repeats a previously-discovered mistake is a worse bug than the original miss.

## Chủ nhà's visibility

Chủ nhà does NOT need to read every Discovery Report. But if the same assumption breaks across 2+ phiếu, that's a signal something structural is wrong — surface it to Chủ nhà via `/decide` for a fix (maybe the guide doc needs rewrite, or the docs-gate rule needs strengthening).

## Rules (hard)

1. **Discovery Report is a commit gate.** Phiếu isn't "done" until the entry is in `docs/DISCOVERIES.md`. docs-gate can enforce this (check for `## [P<NNN>` header added in the same PR).
2. **Write "None" when nothing was wrong.** Don't skip the section. Skipping looks like forgetting; explicit "None" proves you checked.
3. **Newest on top.** Keep chronological order. Do NOT rewrite history — older entries stay verbatim, even if later discoveries contradicted them.
4. **Cross-reference.** If phiếu P044 discovers something that invalidates P038's assumptions, note it in both places (the P044 entry mentions "supersedes P038 finding").
5. **Don't editorialize.** "Docs were wrong" is fact. "Docs are terrible, we need to rewrite everything" is opinion — take that to `/decide`.

## Lifecycle

```
Kiến trúc sư writes phiếu → Chủ nhà approves → Thợ runs /verify (Task 0)
                                             │
                                             ├── All ✅ → Worker codes
                                             │             │
                                             │             ↓
                                             │       Implementation reveals edge case / mismatch
                                             │             │
                                             │             ↓
                                             │       Worker appends to DISCOVERIES.md
                                             │             │
                                             │             ↓
                                             │       Worker ships ticket
                                             │             │
                                             │             ↓
                                             │       Architect reads DISCOVERIES.md before NEXT phiếu
                                             │
                                             └── Any ⚠️/❌ → STOP, report to Architect (not a Discovery Report — this is a phiếu correction request)
```

Discovery Report is for things found DURING or AFTER implementation. Pre-implementation mismatches (caught by `/verify`) are a different channel — they go back to Architect as a phiếu correction, not as a Discovery Report.

## Example Entry

```markdown
## [P044-history-export-csv] — 2026-04-25 — CSV export for reading history

### Assumptions in phiếu — CORRECT
- `useReadingHistory` hook exists at `src/hooks/useReadingHistory.ts:12`
- `HistoryItem` type at `src/types/reading.ts:45`

### Assumptions in phiếu — WRONG
- Phiếu Task 1 said "add POST to existing /api/history route.ts"
- Actual: existing route.ts only exports GET handler and is `export async function GET`
- Adding POST required `export async function POST` in same file; confirmed Next.js App Router supports multiple HTTP methods per route.ts file

### Edge cases / limitations discovered
- CSV needs BOM for Excel to render Vietnamese diacritics correctly (discovered when Sếp opened export in Excel and saw "Tình yêu" as "Tu00ecnh you00eau")
- Added `﻿` BOM to response Content-Type charset. Tested on Excel + Google Sheets + macOS Numbers.
- PostgreSQL `JSONB` cast to text required for CSV serialization of reading interpretation — noted in query

### Docs updated to match reality
- `BACKEND_GUIDE.md` Section "API routes": added Vietnamese diacritic + BOM note under CSV endpoints
- `CONVENTION.md` Known Constraints: added "Excel exports need UTF-8 BOM" gotcha
```
