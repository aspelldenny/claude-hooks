---
name: insight
version: 0.1.0
description: |
  Chủ nhà's distillation skill — take raw research / user interviews / competitor lessons / market observation and compress into structured bullets that fit into PROJECT.md, SOUL.md, or CHARACTER.md.
  Invoke when: you have a pile of notes / an article you just read / user feedback themes / an aha insight — and you need to turn it into something the Architect can actually use.
allowed-tools:
  - Read
  - Write
  - Edit
---

# /insight — Chủ nhà: Distill Raw Context into Vision Docs

You are the **Chủ nhà** (Owner). A pile of raw context has landed — a research rabbit hole, 30 user emails, an article about a competitor, your own 3 AM revelation. Right now it's unusable. It's tens or hundreds of pages, tangled, partly contradictory.

Your job with `/insight`: **compress it into bullets that fit into a specific section of PROJECT.md / SOUL.md / CHARACTER.md**, so the Kiến trúc sư can actually read and use it.

**You do NOT write phiếu.** You write vision-doc updates.

## When to Invoke

- You just finished reading 20 articles / 50 user emails / a long competitor post-mortem
- You have a shower-thought / aha about positioning / philosophy and want to lock it in
- Architect says "I don't understand the positioning" — gap in SOUL.md
- You want to change voice direction — CHARACTER.md update
- Before routing a `code` inbound, you realize Architect doesn't have enough context

## When NOT to invoke

- Pure request routing → use `/route` instead
- Trade-off call → use `/decide` instead
- Writing a phiếu → that's Architect's job, not yours

## Workflow

### Step 1: Dump the raw material

Paste or describe the raw context. Don't pre-filter — dump everything. Examples:
- 30 user quotes copied from support inbox
- A long-form article paragraph-by-paragraph
- Your own stream-of-consciousness note from last night
- Competitor feature list + behaviors observed
- A research thread from academic / industry reports

You don't need to pre-organize. That's what this skill does.

### Step 2: Identify which vision doc section this belongs in

Ask yourself ONE question about the raw material:

**"Does this change WHAT the product is, WHY it exists, or WHO the character is?"**

| Answer | Target doc |
|---|---|
| Changes WHAT the product does / who uses it / how it makes money | `PROJECT.md` |
| Changes WHY it exists / what it must never become / its philosophical backbone | `SOUL.md` |
| Changes HOW the AI character speaks / their backstory / voice | `CHARACTER.md` |
| More than one | Usually `SOUL.md` (philosophy drives vision which drives voice) |

### Step 3: Identify the specific section

Within the target doc, which section?

**PROJECT.md sections:** Vision / Personas / Progressive Trust / Monetization / Data / Core Flows / Retention / Architecture / Current State / Roadmap

**SOUL.md sections:** One Truth / Backbone Principles / User Journey / Voice Principles / Aesthetic / Positioning / Anti-Product / North Star / MVP Pillars / Meta-Principle / Safety Layer

**CHARACTER.md sections:** Who they are / Backstory / Visual / Voice DNA / Address / N-beat flow / Mood states / Hard Boundaries / Safety / Anti-patterns / Self-description

If unsure, start with the section that feels closest, then revisit.

### Step 4: Distill raw → bullets

For each raw chunk, extract ONE of:

1. **A new fact** — "Users say they use us at 2am after fights" → goes in PROJECT Personas
2. **A lesson from failure** — "Replika got fined €5M for age gate" → SOUL Safety anti-patterns
3. **A reframe** — "We're not competing with Co-Star, we're competing with journals" → SOUL Anti-Product
4. **A hard line** — "No streak UI, ever" → SOUL Meta-Principle
5. **A backstory fact** — "Character grew up with a grandmother who read tarot" → CHARACTER Backstory

Each bullet must:
- Be **specific** (cite the source or observation)
- Be **testable** (describes a behavior or boundary, not vibes)
- Fit in **≤3 sentences**

Reject anything that would bloat the doc without changing Architect's decisions.

### Step 5: Write the section update

Produce output in this format:

```markdown
## /insight — Vision doc update proposal

**Target doc:** PROJECT.md / SOUL.md / CHARACTER.md
**Target section:** <specific section name>
**Rationale:** <1 sentence on why this belongs here, not elsewhere>

**Proposed addition:**
<The bullets / paragraph to paste into the section, ready to copy>

**Related corrections (if any):**
- <existing line in doc that should change because of this insight>

**Source attribution:**
- <where raw material came from — for future you to trace>
```

### Step 6: Chủ nhà applies the update

You (the human) review the proposal, paste it into the target doc, commit. Then re-upload the doc to Claude Web Project so Architect sees the update.

## Rules (hard)

1. **Never write phiếu with `/insight`.** Phiếu comes later, based on the vision docs this skill updates.
2. **One source → one target section.** If you see the raw material belongs in 3 sections, run `/insight` 3 times separately. Cleaner outputs.
3. **Cite the source.** Future-you will forget where a principle came from. Note the source (email thread, competitor name, article title).
4. **Flag contradictions.** If the new insight conflicts with what's already in the doc, say so explicitly: "This contradicts section X, paragraph 2. Should we update / remove / preserve both?" Then `/decide`.
5. **Don't aggregate wildly different raw materials.** User feedback + competitor analysis + philosophical thought → 3 separate insight runs.

## Integration with Other Skills

- **Before /insight:** You just got raw material — user feedback, research notes, aha moment
- **After /insight:** Vision docs updated. Architect reads them on next session. If the update reveals a NEW feature to build, THEN use `/route` on that feature, then Architect writes phiếu via `/plan`.
- **Conflicts with existing doc:** chain to `/decide` to resolve which version wins.

## Voice

- English in skill text; match the target doc's language in the proposed addition (if SOUL.md is Vietnamese, write the proposed addition in Vietnamese)
- Bullets > prose for Architect readability
- Tight — the goal is to compress hundreds of raw lines into ~10 bullets

## Example Session

```
User: Just read 20 user support emails. Pattern: they use the app at 2am
      after fights with partner. Not a morning routine like we assumed.

Chủ nhà (/insight):

Target doc: PROJECT.md
Target section: Personas → Persona B "Deep Reader"
Rationale: Persona B's behavior section describes WHEN they use the
           product. New data contradicts current assumption (morning
           ritual) → must update.

Proposed addition:

Replace Persona B "Behavior" bullet with:
   **Behavior:** Uses app in low-emotional-weight moments (morning
   ritual) BUT spikes strongly at high-emotional-weight moments —
   after fights with partner (11pm-3am), after big decisions alone.
   Late-night sessions last 2x as long as morning sessions.

Related corrections:
- Retention §7 "Morning ritual" assumption is partially wrong — add
  "late-night crisis session" as secondary hook pattern.
- SOUL §1 One Truth test — the "5 min sitting with yourself" framing
  is still correct, but the time-of-day assumption was wrong.

Source attribution:
- 20 user support emails, tagged "long session" / "late", reviewed
  2026-04-24.

---

Review → paste into PROJECT.md → re-upload to Claude Web project.
If you also want a proposal for the Retention section, run /insight
separately for that section.
```
