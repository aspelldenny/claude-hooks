# DESIGN_SPEC.md — <Product / Surface Name>

> **Version:** 1.0 (draft)
> **Purpose:** Map visual design (palette, typography, chrome, ornaments) to voice / character. Every visual decision should be traceable back to a SOUL principle, a CHARACTER fact, or a VOICE rule.
> **Related:** `SOUL.md`, `CHARACTER.md`, `VOICE.md` (if present), `tailwind.config.ts` or equivalent design tokens.

> **Use this template when:** your product has a strong character/voice identity that the visual layer must reinforce. If your product is purely functional (B2B SaaS dashboard), this template may be overkill — use a simpler design system doc.

---

## 1. The One Truth (non-negotiable)

> One sentence that the entire visual layer must serve. If a design choice contradicts this, the design is wrong.

> *<e.g. "The product enforces slowness. Every visual choice — palette, typography, motion — must make rushing feel awkward.">*

This sentence comes from SOUL.md §<N>. Do not weaken it.

---

## 2. Metaphor: <product metaphor>

> What does the product *feel like* as a place / object / setting? The metaphor anchors visual choices.

<2–4 paragraphs describing the metaphor. Specific, sensory, observable. "A chapbook from 1920s Vietnam, ink on cream paper, hand-bound" beats "elegant and timeless".>

**Why this metaphor:** ties to <CHARACTER.md fact / SOUL.md principle / cultural context>.

---

## 3. Palette

> Colors with intentional names. Each color has a role and a forbidden use.

### 3.1 Tokens (paste-ready for `tailwind.config.ts` or design system)

```ts
const palette = {
  // Primary
  '<token-name-1>': '#<hex>',  // <role: e.g. ink-on-paper body text>
  '<token-name-2>': '#<hex>',  // <role: e.g. cream paper background>
  // Accents
  '<token-name-3>': '#<hex>',  // <role: e.g. wax-seal red, only for closing CTAs>
  // ...
};
```

### 3.2 Hierarchy of use
- **Primary surfaces** (90% of pixels): <token, token>
- **Body text:** <token>
- **Accents** (used sparingly): <token, token>
- **System / chrome:** <token>
- **Error / safety break:** <token>

### 3.3 Never
- Don't use <color X> for <use case Y> — reason: <ties to SOUL or CHARACTER>
- Don't combine <X + Y> — reason: ...
- Don't use pure white / black / saturated primaries unless <reason>

---

## 4. Typography

> Type choices = voice in pixels. Font choice, weight, leading, kerning all encode character.

### 4.1 Type stack
- **Display / heading:** <font name>, <fallbacks>
- **Body:** <font name>, <fallbacks>
- **Monospace / accent:** <font name>, <fallbacks>

**Why these fonts:** <tie to metaphor — e.g. "Display font references hand-set lead type from 1920s print shops">

### 4.2 Scale & usage

| Token | Size | Weight | Used for |
|---|---|---|---|
| `<text-xs>` | <px / rem> | <weight> | <use case> |
| `<text-sm>` | ... | ... | ... |
| `<text-base>` | ... | ... | ... |
| `<text-lg>` | ... | ... | ... |
| `<text-xl>` | ... | ... | ... |
| `<heading-1>` | ... | ... | ... |
| ... | ... | ... | ... |

### 4.3 Weight philosophy
<Why few weights or many? Hand-set type = restricted weight palette. Modern variable fonts = wider range. Tied to metaphor.>

---

## 5. Texture & surface

> Surfaces are voice. Pure flat = digital. Subtle texture = analog. Choose intentionally.

### 5.1 Background treatment
- **Component name:** <e.g. PaperBackground (`src/components/.../PaperBackground.tsx`)>
- **Treatment:** <e.g. cream base + subtle noise + occasional ink-spot ornament>
- **When applied:** <which surfaces — listed explicitly>

### 5.2 Surface choices
| Surface type | Treatment | When to use |
|---|---|---|
| <e.g. card> | <ornament + shadow + border treatment> | <use case> |
| <e.g. modal> | ... | ... |
| <e.g. error banner> | ... | ... |

### 5.3 Dividers / ornaments
- <how dividers look — line weight, ornament>
- <ornament library — describe or list components>

---

## 6. Layout principles

### 6.1 The page metaphor
> What does a "page" feel like? Centered chapbook column? Asymmetric editorial spread? Monospace dashboard?

<description tied to product metaphor>

### 6.2 Grid / asymmetry
<grid system used — e.g. 12-col, asymmetric narrative columns, single-column chapbook>

### 6.3 Spacing
- **Default rhythm:** <e.g. 8px base, generous vertical breathing>
- **Tight contexts:** <where compactness is allowed>
- **Why generous spacing matters:** tied to "slowness" hard line / SOUL principle

---

## 7. Voice ↔ design system

> Where voice meets pixels. Some surfaces are character-bearing; some are tool-layer chrome. Pre-define which is which to prevent voice drift in copy.

### 7.1 Two layers
- **Tool layer** — buttons, errors, system toasts, navigation labels. Voice: <neutral / functional / brief>.
- **Character layer** — interactive content, response surfaces, narrative text. Voice: <character voice from CHARACTER.md>.

### 7.2 Hybrid surfaces
> Surfaces that need both layers (e.g. SEO content with both keyword H2 and character body).

| Surface | Tool-layer element | Character-layer element |
|---|---|---|
| <e.g. card detail page> | SEO H2 ("Meaning") | Character body impression |

### 7.3 Do
- <copy patterns that work>
- ...

### 7.4 Don't
- <copy anti-patterns>
- ...

### 7.5 Phrase library (copy freely)
- **CTAs:** <list of safe CTA wording>
- **Section markers:** <list>
- **Empty / error states:** <list>

---

## 8. Chrome (navigation & shell)

### 8.1 Desktop navigation
- **Component:** <name + path>
- **Pattern:** <described>

### 8.2 Mobile navigation
- **Component:** <name + path>
- **Pattern:** ...

### 8.3 Footer
<treatment, what's in it, what's not>

### 8.4 Conditional chrome
> When does chrome appear / disappear? Which surfaces are full-bleed vs. framed?

---

## 9. Core illustrations & ornaments

> If product has illustrations or recurring decorative elements.

- <ornament 1: when used, how it relates to character>
- <ornament 2: ...>
- <illustration style: tied to metaphor>

---

## 10. Component patterns

### 10.1 Button hierarchy
- **Primary** (rare, only for <use cases>): <visual treatment>
- **Secondary** (default action): <visual treatment>
- **Tertiary / ghost** (alternative paths): <visual treatment>

### 10.2 Input / textarea
<treatment + tied to metaphor>

### 10.3 Section markers
<treatment>

### 10.4 Error banner
<treatment + tone — e.g. quiet vs alarming>

### 10.5 Motion
- **Default duration:** <e.g. 200ms>
- **Easing:** <function — e.g. ease-out for entrances, ease-in for dismissals>
- **Forbidden motions:** <e.g. no bouncy spring physics — violates slowness>

---

## 11. Page-by-page status

> Track which pages match the spec and which are still legacy.

| Page | Status | Notes |
|---|---|---|
| <e.g. Landing> | ✅ matches spec v1.0 | — |
| <e.g. Reading flow> | ⚠️ partial | <what's missing> |
| <e.g. Profile> | ❌ legacy | <ticket P<NNN> to migrate> |

---

## 12. Migration heuristic — reskin vs redesign

> When updating a legacy page, decide reskin (cosmetic only) vs redesign (structural).

### Q1. Is this surface tool-layer or character-layer?
- Tool → reskin is usually enough
- Character → redesign likely needed (legacy structure encodes wrong voice)

### Q2. Does the surface present <product domain> as <sign or symbol>?
- Sign (correct framing for this product) → reskin
- Symbol (wrong framing) → redesign

### Q3. Does the layout force <unwanted user behavior>?
- Yes → redesign (layout shapes behavior)
- No → reskin

### Reskin checklist (cosmetic update)
- [ ] Palette tokens swapped
- [ ] Typography tokens swapped
- [ ] Texture / background swapped
- [ ] Copy reviewed against voice
- [ ] No structural change to component tree

### Redesign checklist (structural update)
- [ ] Layout reconsidered against §6 layout principles
- [ ] Component hierarchy reviewed
- [ ] Voice layer (tool vs character) re-mapped
- [ ] User flow re-tested against `TEST_CASES.md`

---

## 13. Known constraints & gotchas

> Things that broke or surprised previous designers / engineers.

- <e.g. "Tailwind JIT doesn't pick up dynamic class names — use safelist for character-conditional styles">
- <e.g. "PaperBackground component re-renders on every nav — memoize or use CSS-only approach">
- ...

---

## 14. Open questions

1. <unresolved design question>
2. ...

---

## 15. Doing the work

### 15.1 File locations (reference)
- Tokens: <path>
- Background components: <path>
- Chrome components: <path>
- Page templates: <path>

### 15.2 Verify a migration is done
- [ ] Visual diff against spec
- [ ] Voice review (run `TEST_CASES.md` cases for affected surfaces)
- [ ] Pre-commit / docs gate clean
- [ ] Page added / updated in §11 page-by-page status table

---

## 16. Changelog

- **v1.0 (<date>)** — initial spec lock
- **v1.1 (<date>, P<NNN>)** — <change + ticket reference>
- ...
