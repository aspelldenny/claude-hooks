# VOICE.md — <Voice Name>

> **Version:** 1.0 (draft)
> **Status:** Draft / Locked / Archived
> **Scope:** <which surfaces use this voice — list product features explicitly>
> **Out of scope:** <which surfaces use a different voice — name the other voice file>

> **Use this template when:** your product has a *non-character voice* that needs to coexist with a character (e.g. a narrator voice for content surfaces while the character handles interactive surfaces). If your product has only one character, you don't need this file — keep voice patterns inside `CHARACTER.md` §4.

**Source material:**
- `SOUL.md` §<N> — relevant philosophical principle
- `CHARACTER.md` §4 — voice DNA the character uses (this voice is contrast or partner)
- <other docs that anchor this voice>

**Philosophy of this voice (one paragraph):**
<Why this voice exists. What it tries to NOT be. What it serves.>

---

## 1. Hard rules (max 3, non-negotiable)

1. **<Rule 1>** — <one-sentence statement>
2. **<Rule 2>** — ...
3. **<Rule 3>** — ...

> Violating any of these = voice drift. Architect / reviewer rejects.

---

## 2. Voice DNA (read before writing)

### 2.1 Sentence rhythm
<short / long / mixed; how sentences end (period, ellipsis, question); ratio of declarative to interrogative>

### 2.2 Vocabulary — DO use
<words, registers, domains the voice draws from. List 30+ concrete words if possible. Be specific.>

### 2.3 Vocabulary — DON'T use
| Category | Example forbidden words | Why |
|---|---|---|
| <e.g. mystical> | <vũ trụ, năng lượng, chakra, aura> | violates SOUL §X |
| <e.g. self-help> | <tỏa sáng, transform, manifest> | violates anti-positivity stance |
| <e.g. dictionary framing> | <"means", "represents", "symbolizes"> | reduces symbol to sign |
| <e.g. predictive> | <"will", "soon", "definitely"> | violates hard line on fortune-telling |
| <e.g. imperative> | <"should", "must", "have to"> | voice has no authority over user |
| <e.g. brand/AI self-ref> | <"AI", "the system", "we"> | breaks voice illusion |

### 2.4 Metaphor lineage
<Where metaphors come from. Specific traditions, writers, registers. The opposite: what metaphor sources are forbidden.>

### 2.5 Special-case handling

> Surfaces or contexts that tempt voice drift. Pre-specify the safe move.

| Situation | Risk of drift | Voice's safe move |
|---|---|---|
| <e.g. position labeled "Future" in spread> | predictive framing | describe seed in present, not event in future |
| <e.g. "Advice" position> | imperative framing | describe shape of a path, not command |
| <e.g. user shares trauma> | coaching positivity | hold space, no resolution offered |

---

## 3. Examples (archetype pass — N samples per category)

> Concrete examples are the contract. Prompt engineers paste these into system prompts as few-shots.

### 3.1 <Category 1> (e.g. Major arcana upright)
**<Item name>**
> <example output, 2-4 sentences in voice>

**<Item name>**
> <example>

### 3.2 <Category 2> (e.g. Major arcana reversed)
...

### 3.3 <Category 3>
...

### 3.4 <Category 4>
...

(Cover the main archetypes / item types your product surfaces. Aim for 4–6 examples per category, 16–24 total.)

---

## 4. Examples (composite pass — N samples)

> Examples that combine multiple items / positions / contexts. Show how the voice handles structure.

### 4.1 <Composite type 1, e.g. 3-card spread "general" variant>

**<Position 1 · Item A>**
> <example>

**<Position 2 · Item B>**
> <example>

**<Position 3 · Item C>**
> <example>

(Repeat for each composite type your product supports.)

---

## 5. Anti-patterns — DON'T write like this

> Pulled from real misfires (or competitor copy). Show the bad version + diagnose the violations + show the corrected version.

### 5.1 <Anti-pattern name>

**❌ Don't write:**
> <bad example>

**Violations:** <list which Hard Rule, vocab category, or framing fails>

**✅ Write this instead:**
> <good example in voice>

### 5.2 <Anti-pattern name>
...

(Aim for 3–5 anti-patterns covering the most common drift failure modes.)

---

## 6. Phrase bank (N reusable phrases by function)

> Safe phrases that obey all hard rules. Writers can mix freely. Group by function.

### 6.1 Openings (introduce the subject)
1. *<phrase>*
2. ...
5. ...

### 6.2 Bridges (connect description to impression without dictionary framing)
6. *<phrase>*
7. ...
10. ...

### 6.3 Open endings (don't close the sentence)
11. *<phrase>*
12. ...
15. ...

### 6.4 Reflection prompts (without CTA)
16. *<phrase>*
17. ...
18. ...

(Add categories that make sense for your product. Aim for 15–25 total.)

---

## 7. Pre-flight checklist (every output)

> Run before shipping any voice output. "No" to any = reject.

1. Are there any pronouns / address words? (forbidden by Rule 1)
2. Are there any value labels (good/bad/right/wrong/success/failure)?
3. Are there imperatives? (should, must, don't, need to)
4. Is there any forbidden vocabulary from §2.3?
5. Is there dictionary framing? (e.g. "means", "represents", keyword lists)
6. Is there predictive language? (will, soon, certainly)
7. Does the output describe specifics before any impression / framing?

Pass = "yes" to #7 only, "no" to #1–6.

---

## 8. Voice boundaries (relation to other voices in this product)

> If your product has multiple voices (e.g. character + narrator + UI neutral + email broadcast), draw the lines explicitly.

| Voice | Used in | Persona | Pronouns |
|---|---|---|---|
| **<This voice name>** | <list surfaces> | <none / character X> | <none / specific> |
| <Other voice 1> | <list surfaces> | <persona> | <pronoun pattern> |
| <Other voice 2> | <list surfaces> | <persona> | <pronoun pattern> |

**Boundary rule:** <how to distinguish — e.g. "This voice has no pronouns. If copy contains 'I' or 'you', it's not this voice.">

**Common drift to watch:** <list 2–3 patterns where one voice tends to leak into another's surface>

---

## 9. Open questions (unresolved, need Chủ nhà sign-off)

1. <question>
2. <question>
3. <question>

---

## 10. Changelog

- **v1.0 (<date>)** — initial draft. <N hard rules, M examples, K anti-patterns, P phrase bank entries.>
- **v1.1 (<date>, P<NNN>)** — <change + ticket reference>
- ...

---

*Implementation status:*
- [ ] **<surface 1>** — voice applied at <code path>
- [ ] **<surface 2>** — pending phiếu P<NNN>
- ...
