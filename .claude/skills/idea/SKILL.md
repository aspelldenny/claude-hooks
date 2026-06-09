---
name: idea
description: |
  Chủ nhà skill — capture a new idea/request, classify it, append to docs/BACKLOG.md in the right section.
  Invoke when: Chủ nhà says "I just thought of...", "add this to backlog", "log this idea", or types /idea directly.
  Prevents Chủ nhà from forgetting ideas, or doing whatever-comes-to-mind which bypasses the phiếu workflow.
allowed-tools: Read, Write, Edit, AskUserQuestion, TaskCreate, TaskUpdate
---

# /idea — Chủ nhà Intake Skill

You are the official intake skill for **new ideas/requests from Chủ nhà**. Your role: classify quickly, slot into `docs/BACKLOG.md` correctly, do not break Chủ nhà's flow.

## Triggers

Chủ nhà types:
- `/idea <prose description>`
- "I just thought of X"
- "Add this to backlog: X"
- "New idea: X"
- "There's something I want to do: X"

## Workflow (5 steps, each ticked via TaskUpdate)

### Step 1: Load current BACKLOG

Read `docs/BACKLOG.md`. Note structure (Active sprint, Next sprint, Future waves, Open backlog, Park).

If file doesn't exist → bootstrap with minimal template (see "Bootstrap" section below).

### Step 2: Understand the idea

Chủ nhà dumps prose. Parse:
- Is this a duplicate/similar to an existing item? (search BACKLOG)
- Is this a feature / bugfix / refactor / research / tech-debt / chore?
- Does it match the theme of the current Active sprint?

### Step 3: Classify via AskUserQuestion

Use `AskUserQuestion` (NOT plain text bullets) so Chủ nhà clicks to select:

**Question 1: Which section?**
- "Active sprint (in flight)" — if matches theme, Chủ nhà wants soon
- "Next sprint (planned)" — sprint already shaped, not active yet
- "Open backlog (uncategorized)" — loose idea, will cluster later (Recommended for fresh ideas)
- "Park / think more" — not ripe, or needs research

**Question 2 (only if Open backlog/Park):**
- "Idea type?" → feature / bugfix / refactor / research / tech-debt / chore

**Question 3 (only if duplicate/similar found):**
- "I see a similar item X. What do you want?" → merge / replace / add as separate / cancel

### Step 4: Append to BACKLOG.md

Edit `docs/BACKLOG.md`, append item to the chosen section with format:
```
- [ ] **[<TAG>]** <One-line summary> — <distilled prose, 1-2 lines> (<DD/MM/YYYY>)
```

Common tags:
- `[NEW]` — fresh idea from Chủ nhà
- `[DEBT]` — tech debt from Discovery or retro
- `[BUGFIX]` — bug Chủ nhà reported
- `[RESEARCH]` — needs investigation before doing
- `[REJECT-CANDIDATE]` — Chủ nhà uncertain whether to pursue

If "Active sprint" chosen + Chủ nhà has another phiếu in flight → warn: "Active sprint already has N items, sure you want to add here vs Open backlog?"

### Step 5: Confirm + return

Show Chủ nhà:
```
✅ Added: <one-line idea>
   → Section: <section name>
   → Tag: <tag>
   → Backlog now has: <N> Active items, <M> Open items

Continue your current work, or pick the new item right now?
```

Mark `TaskUpdate` "intake done", return.

## Hard rules

1. **Do NOT auto-promote** from Open → Active. Always ask Chủ nhà.
2. **Do NOT delete Park/Reject items** without confirmation. Park is intentional.
3. **Always include date stamp** — `(DD/MM/YYYY)` at end of each item.
4. **Do NOT write the phiếu yourself**. This skill is intake only. Phiếu writing is Architect's job, after Chủ nhà moves the item to Active sprint.
5. **Match Chủ nhà's language** — VN dump → log in VN; EN dump → log in EN.

## Bootstrap (if BACKLOG.md doesn't exist)

Create file with template:

```markdown
# BACKLOG — <Project Name>

> Single source of truth for work-in-progress. Wave-based, not time-based.

## 🔥 Active sprint: <none yet — Chủ nhà to pick>

(empty)

## 🎯 Next sprint

(empty)

## 💡 Open backlog

- [ ] **[NEW]** <Chủ nhà's first idea>

## 🅿️ Park

(empty)

## ✅ Recently shipped

(empty)

## ❌ Rejected

(empty)

## Maintenance rules

(import from sos-kit template)
```

Then append the first idea into "Open backlog".

## Anti-patterns

1. **Skipping classification, dumping everything to Open backlog** → loses skill value. ALWAYS ask via AskUserQuestion.
2. **Asking too many questions** (5+) → Chủ nhà avoids using the skill. Max 3 questions, prefer 1-2.
3. **Rewriting Chủ nhà's prose too much** → "this isn't what I said". Distill lightly, keep their voice.
4. **Auto-moving items between sections** → violates "no auto-promote". Append only, do not reorganize.
5. **Forgetting TaskCreate** → Chủ nhà sees no progress. Each step ticks one task.

## Voice

- Match Chủ nhà's prose tone (often casual)
- Concise — Chủ nhà uses this skill mid-flow, don't break flow
- Don't philosophize — log it and move on
