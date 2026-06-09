# SOUL.md — <Project Name>

**Version:** 0.1 (draft)
**Purpose:** The philosophical backbone. Every UI, copy, and AI-prompt decision must trace back to this document.
**Related:** `PROJECT.md` (what the product does) vs `SOUL.md` (why it exists, what it must never become).

---

## 1. The One Truth

**<Project> is <what it actually is in one clear sentence>.**

Not <what it is commonly mistaken for>. Not <another shallow framing>. Not <third wrong framing>.

It is <restate the one truth, expanded into 2-3 sentences with texture>.

> **Test for the One Truth:** <a concrete behavioral test — if user does X after using the product, you succeeded. If user does Y, you failed.>

---

## 2. Backbone Principles (2-4 max)

Each principle is the **spine** — removing it kills the product's identity.

### 2.1 <Principle name, e.g. "Symbol over Sign">

<1-paragraph explanation — what the distinction is, why it matters>

**Anti-example (what cheap competitors do):** <concrete example of the wrong version>

**Our version:** <concrete example of the right version>

**Implication for design:** <1-line on how this constrains UI/UX/prompt>

### 2.2 <Principle name>

...

### 2.3 <Principle name>

...

---

## 3. User's Journey (if applicable)

> Each session is a mini-cycle of <your core psychological mechanism>. **Shape emerges from the encounter**, not imposed.

### <Movement 1 — e.g. "The call">
<1-paragraph describing this phase from user's POV>

### <Movement 2>
...

### <Movement 3>
...

### <Final movement — e.g. "Return with a clearer question">
<close the journey, not as a conclusion but as a different starting point>

*(If the project has an AI character, reference `CHARACTER.md` here for voice details.)*

---

## 4. Voice Principles (meta-level)

> Detail lives in `CHARACTER.md`. This section is the meta-rules.

1. **<Voice rule 1, e.g. "No dictionary-style interpretations">**
2. **<Voice rule 2, e.g. "No future predictions">**
3. **<Voice rule 3, e.g. "Always ask before explaining">**
4. ...

---

## 5. Aesthetic Principles

### NEVER
- <visual anti-pattern 1, e.g. "Purple→blue→pink gradient">
- <visual anti-pattern 2, e.g. "Glassmorphism neon glow">
- <visual anti-pattern 3>
- ...

### ALWAYS
- **Texture strategy:** <what you use to differentiate>
- **Color palette:** <cite sources — specific references, not "modern and clean">
- **Typography:** <with specific fonts + rationale, NOT "Inter default">
- **Layout principle:** <grid? asymmetric? mandala? hierarchy?>
- ...

### Reference anchors
<List 3-5 visual / artistic references that inspire the aesthetic. Cite eras, artists, movements. NO "Behance trending.">

---

## 6. Positioning — Narrow surface, Universal foundation

### 6.1 Phase 1 (MVP → first N months)
**Public positioning:** <target audience specifically — demographic, mindset>. Marketing, visual, voice optimize for this persona.

### 6.2 No hard gate
Product technically open to <adjacent audiences>. If they self-select in and find resonance — welcome.

### 6.3 Universal foundation
<What's the underlying truth your narrow persona is just a specific expression of? This is how you can expand later without breaking identity.>

### 6.4 Phase 2 (if data supports)
- **A:** <expansion option 1>
- **B:** <expansion option 2>
- **C:** <stay niche forever>

*(Chủ nhà's lean:)* <A/B/C — which does data need to show to warrant expansion?>

---

## 7. The Anti-Product

<Project> is NOT:

- Not <common mistaken framing 1>
- Not <common mistaken framing 2>
- Not <third wrong framing — especially one that competitors have cheapened>
- Not <framing that sounds similar but is actually wrong, e.g. "companion" — mention specific competitors that cheapened the term>
- Not <Western / foreign version translated, if relevant>
- ...

This list exists because these framings LOOK adjacent. Users + marketing + future-you will drift toward them. Lock them out now.

---

## 8. North Star Metric

**Not:** <shallow metric like DAU, time-in-app, streak length>

**Yes:** <a behavioral metric that correlates with the One Truth — e.g. "Did user do X after the session?">

**How to measure:** <concrete signal — user action, optional prompt, survey, etc.>

**Why this metric:** <1-2 sentences on why this metric, not the shallow ones>

---

## 9. MVP Focus — Core Pillars

Only N pillars (aim for 2-4). Everything else is secondary.

1. **<Pillar 1>** — <1-line on why it's load-bearing>
2. **<Pillar 2>** — <1-line>
3. **<Pillar 3>** — <1-line>

Other sections (e.g. Safety, Future Expansion, Positioning nuances) stay in the doc but don't block ship.

---

## 10. Meta-Principle: No Rule Book, Only Character Book 🔒

> If the product has an AI character: **Do not build the character via rules. Build via backstory.**

### Why this matters:
Rules stack into cages. Each rule alone is reasonable, but cumulated → output feels formulaic, repetitive, cold. Add detail to backstory instead.

- ❌ **Rule:** *"Never say X word."*
- ✅ **Backstory:** *"The character grew up in context Y, so they allergic to X-style language."*

### Max 3 hard lines (and only 3):
1. <Hard line 1 — non-negotiable, never violated>
2. <Hard line 2>
3. <Hard line 3>

Every other behavior shapes through character backstory (in `CHARACTER.md`), not rules.

### Self-audit when tempted to add a rule:
> *"Am I making this a cage instead of building character?"*

---

## 11. Safety Layer (if domain requires)

> Include only if product interfaces with sensitive domains: mental health, finance, medical, legal, minors, vulnerable populations.

### Legal framing (on every surface):
> <specific disclaimer — what the product is and isn't, especially if it might be mistaken for clinical/professional advice>

### Crisis detection (if applicable):
- Tier 1: <keyword/regex pre-LLM>
- Tier 2: <small classifier for intent>
- Tier 3: <output validation>

### Break-character triggers:
Always break character when: <list specific triggers>

### Crisis response:
1. Acknowledge + break character
2. Provide hotlines / resources (verify accuracy every 6 months)
3. In-app safety plan if relevant

### Anti-patterns from prior industry failures:
<List specific lessons learned from other products — what NOT to do, with source>

### Compliance:
<PDPL / HIPAA / GDPR / etc. as applicable to your jurisdiction>

---

## 12. Notes for future versions

- <item 1 to address in v0.2>
- <item 2>
- <open research question>

---

## Changelog

- **v0.1** — initial draft
- **v0.2** — <what changed and why>
- ...
