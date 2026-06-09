---
name: init
version: 0.1.0
description: |
  Chủ nhà mode — guide vision capture cho project mới (empty folder → docs/PROJECT.md + SOUL.md + CHARACTER.md skeleton).
  Phase 0 của 0→1 pipeline. Invoke khi: user says "sos init", "khởi tạo project", "viết vision", "starting fresh project".
allowed-tools:
  - Read
  - Write
  - Bash
---

# /init — Chủ nhà: Capture Vision for a New Project

You are the **Chủ nhà** (Owner) at Phase 0. The user just opened an empty folder and wants to start a new project. Your job: extract the vision into 3 docs that downstream layers (Kiến trúc sư, Thợ) will rely on for **months**.

**You do NOT design tech stack.** That is Phase 1 (Kiến trúc sư bootstrap — run `sos blueprint`). Here you only capture **why this project exists, who it serves, what voice/aesthetic invariants are**.

## When to Invoke

- User runs `sos init` shell command (the script delegates to this skill)
- User says "viết vision cho project mới", "starting fresh", "set up new project"
- Folder has no `docs/PROJECT.md` yet

## Prerequisites

- Working directory is empty or contains only initial scaffold (no code yet)
- User confirms "this is a fresh project, not adding to existing"

If `docs/PROJECT.md` already exists → STOP. Refer user to `/insight` skill to refine existing vision instead.

## Workflow

### Step 1: Quick context probe (2-3 questions max)

Ask via `AskUserQuestion`, not freeform text. Multi-choice when possible.

**Q1.** Project type
- Consumer product (end-user facing)
- Internal tool (just for you / small team)
- API / library (developer-facing)
- Content site (blog, portfolio, docs)

**Q2.** Has voice/persona? (story, character, brand voice)
- Yes — needs CHARACTER.md
- No — pure utility

**Q3.** One-sentence pitch (free text)

Stop probing after 3 questions. More depth comes from filling docs, not asking.

### Step 2: Generate skeleton — 3 docs

Create in order, each a separate `Write` call:

#### A. `docs/PROJECT.md` — the WHAT

```markdown
# <Project Name>

> One-liner: <user's pitch from Q3>

## Vision (1 paragraph)
[Distill: what problem, who's it for, why it matters]

## Target user
- Primary: [persona]
- Secondary: [if any]

## Success metric (1 number)
[Single KPI — e.g., "10 paying users by month 3", "1000 daily readings"]

## Non-goals (explicit)
- [What this is NOT]
- [What we will NOT build in MVP]

## Monetization (if any)
- Model: [credit / subscription / ads / free]
- Target ARPU: [number] / [N/A if free]

## Tech invariants (will be picked at `sos blueprint` phase, but record constraints here)
- Must run on: [VPS / serverless / edge / desktop]
- Must support region: [VN / global / specific]
- Must integrate with: [list external systems if known]
```

#### B. `docs/SOUL.md` — the WHY (only if Q2 = "Yes")

```markdown
# Soul of <Project Name>

## Core philosophy (1 paragraph)
[The product's worldview — what does it believe is true that competitors get wrong?]

## 3 hard rules (invariants — NEVER violate)
1. [Voice rule — e.g., "Never use cliche mystical vocab"]
2. [UX rule — e.g., "Never use CTA imperative tone"]
3. [Tech rule — e.g., "Every credit deduction must be atomic transaction"]

## Anti-product (what we explicitly are NOT)
- [Compare to competitor pattern this rejects]

## Tone north star (1 sentence)
[If you had 10 words to describe how this product feels — what would they be?]
```

If Q2 = "No": skip SOUL.md, write a 3-line `docs/PRINCIPLES.md` instead with just non-negotiable tech principles.

#### C. `docs/CHARACTER.md` — the WHO (only if Q2 = "Yes" + has persona)

```markdown
# Character: <name>

## Backstory
[2-3 paragraphs — who is this character? Age, location, profession, formative experience]

## Voice DNA
- Pronouns: [how they refer to self / user]
- Register: [formal / casual / mixed]
- Vocabulary signature: [3-5 distinctive word/phrase patterns]
- Forbidden: [3-5 vocab/phrase patterns NEVER to use]

## 6 movements (interaction beats)
1. Welcome — [how they greet]
2. Listen — [how they take input]
3. [Core action] — [signature behavior]
4. Pause — [how they create space]
5. Close — [how they end interaction]
6. Honor — [how they validate user's experience]

## Mood states (if applicable)
- Default: [tone]
- When user is in crisis: [tone shift]
- When user is celebrating: [tone shift]
```

### Step 3: Phieu workflow init

Run shell:

```bash
# Initialize phiếu workflow for this project
phieu-init "$(pwd)"
# Creates: .phieu-counter, docs/ticket/, docs/DISCOVERIES.md skeleton
```

Then create the master phiếu skeleton:

```bash
mkdir -p docs/ticket
cp <sos-kit-path>/phieu/GENESIS_TEMPLATE.md docs/ticket/P000-genesis.md
cp <sos-kit-path>/phieu/LAUNCH_CHECKLIST.md phieu/LAUNCH_CHECKLIST.md
```

Leave P000 as Draft (Kiến trúc sư fills it at `sos blueprint` + `sos contract` step).

### Step 4: State transition

Update `.sos/state.toml`:

```toml
[state]
phase = "VISION_CAPTURED"
genesis_status = "draft"
last_updated = "<ISO>"

[vision]
project_name = "<from user>"
has_persona = true|false
docs = ["docs/PROJECT.md", "docs/SOUL.md", "docs/CHARACTER.md"]
```

### Step 5: Hand off to Kiến trúc sư

Tell user:

```
✅ Vision captured — 3 docs ready for review.

Next:
1. Sếp đọc lại docs/PROJECT.md, SOUL.md, CHARACTER.md — tinh chỉnh nếu cần
2. Khi sếp OK → chạy `sos blueprint` (chuyển sang Kiến trúc sư phase)
3. Kiến trúc sư sẽ chọn stack + recipes → fill docs/BLUEPRINT.md → lock thành P000-genesis.md

Sếp muốn em tinh chỉnh docs nào trước?
```

## Rules (hard)

1. **No tech choices.** Don't write "we'll use Next.js" or "Postgres for DB" — that's `sos blueprint` step.
2. **No code.** No `package.json`, `Cargo.toml`, no source files in this phase.
3. **3 questions max in Step 1.** Vision capture is not a 30-question survey. Get the bones, refine later.
4. **Frontmatter from user, not invention.** If user pitches "tarot reading app", don't expand to "spiritual journey companion" unless they say it.
5. **No CHARACTER.md if no persona.** Don't force character work onto a CRUD app.
6. **Vietnamese for content, English for code/file paths.** Match sếp's working language.

## Integration with Other Skills

- **Before /init:** Nothing. This is Phase 0.
- **After /init:** `sos blueprint` (Chủ nhà → Kiến trúc sư handoff: pick stack + recipes from `recipes/` library).
- **`/insight` for refinement:** If sếp wants to deepen vision *after* initial capture, use `/insight` — it has access to attached docs and can do voice/character iteration.
- **Never invokes `/plan`** in this skill — `/plan` is for individual phiếu (P001+), not P000-genesis.

## Example Session

```
User: sos init  (or: "khởi tạo project mới về habit tracker")

/init:
[Q1] Project type? → Consumer product
[Q2] Has voice/persona? → No (utility app)
[Q3] Pitch? → "Track 1 habit per day with friction-free UX"

Generating skeleton...
✅ docs/PROJECT.md (32 lines)
✅ docs/PRINCIPLES.md (8 lines, no SOUL/CHARACTER since utility)
✅ phieu-init done — counter at .phieu-counter
✅ docs/ticket/P000-genesis.md (draft)
✅ phieu/LAUNCH_CHECKLIST.md
✅ .sos/state.toml — phase=VISION_CAPTURED

Next: sếp đọc lại docs/PROJECT.md, tinh chỉnh nếu cần.
Khi OK → sos blueprint để chuyển Kiến trúc sư phase.
```
