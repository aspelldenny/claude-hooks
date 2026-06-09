# PROJECT.md — <Project Name>

> <One-line product description — who it's for, what it does, in the metaphor or frame you want users to hold>
> This document is the **PRD** — Chủ nhà maintains, Kiến trúc sư references, Thợ reads for context.

---

## 1. Vision

**Một câu** (one-liner): <what the product actually is — NOT "app that does X", but the experience / truth behind it>

**Tại sao khác biệt** (why different):
- <differentiator 1 — specific, not "AI-powered">
- <differentiator 2>
- <differentiator 3>
- ...

---

## 2. Personas (2-3 max — don't list every user segment)

### Persona A — "<short label>" (<motive in 1-2 words>)
- **Ai:** <demographic + mindset>
- **Muốn:** <what they want from your product>
- **Hành vi:** <how they use it — 1-2 sentences>
- **Trả tiền:** <willing to pay? how much?>
- **Giá trị cho app:** <why you care about them>

### Persona B — "<short label>"
...

---

## 3. Progressive Trust Model (if auth matters)

Skip this section if your product is internal / b2b / does not have anonymous tier.

### Level 0 — <anonymous label>
- <what they can do without signing up>

### Level 1 — <lite label>
- <trigger to ask for minimum info — name, email>
- <what unlocks>

### Level 2 — <registered label>
- <trigger for full account>
- <what unlocks>

### Level 3 — <premium label, if applicable>
- <what defines Pro user>

---

## 4. Monetization

**Model:** <subscription / credit / one-time / free / freemium>

**Reasoning:** <why this model fits your audience and product>

### If credit-based:

| Package | Price | Credits | Unit price |
|---|---|---|---|
| ... | ... | ... | ... |

### Cost per action:

| Action | Credits | Reasoning |
|---|---|---|
| ... | ... | ... |

### Payment integration:
<payment provider, e.g. Stripe / PayOS / PayPal>

---

## 5. Data Sources

### Content / dataset origin:
- <source 1 — e.g. public domain book, licensed API, generated>
- <source 2>

### Legal / license:
- <public domain / commercial license / Creative Commons / etc.>

### Storage format:
- <JSON / SQL / external API>

---

## 6. Core Flows

### Flow 1: <primary user flow, e.g. "Quick Reading">
```
User opens X → picks Y → system Z → result R
```
- <constraint or unique behavior>
- <edge case handled>

### Flow 2: ...

---

## 7. Retention / Hooks

> Optional section. Add only if retention is a real concern (consumer product). Skip for b2b / one-time tools.

### Primary hook:
<e.g. daily reminder, streak, community, etc.>

### Secondary:
<badges, rewards, notifications>

### What you deliberately avoid:
<e.g. "no gamification because it breaks the metaphor" — lock this reasoning in>

---

## 8. Technical Architecture

### Stack:
- **Frontend:** <framework + major libs>
- **Backend:** <approach>
- **Database:** <DB + ORM>
- **Auth:** <method>
- **AI / ML** (if applicable): <model(s), provider, fallback chain>
- **Hosting:** <where it runs>
- **Payment:** <integration>
- **Monitoring:** <Sentry / Grafana / none>

### Architecture diagram (ASCII is fine):

```
[User browser]
    │
    ▼
[Frontend framework]
    │
    ▼
[API layer]
    ├── [DB]
    ├── [External API]
    └── [AI provider]
```

### Database models (if applicable):
- **<Model 1>** — <what it tracks>
- **<Model 2>** — <what it tracks>

---

## 9. Current State

> Update this section after major milestones. Don't let it rot.

### ✅ Working
- <shipped feature 1>
- <shipped feature 2>

### ⚠️ Hidden / pending decision
- <feature code exists but not exposed in UI>

### ❌ Deliberately NOT doing (intentional scope out)
- <thing 1> — <reason>
- <thing 2> — <reason>

This "not doing" list is as important as the "doing" list. It locks past decisions so future-you doesn't re-debate them.

---

## 10. Roadmap

> Phases, not deadlines. No "Q3 2026" — use "after X lands" or "when Y metric hits Z."

### Phase 1 — <name> ✅ / 🚧 / 📅
> <what gets done in this phase>

### Phase 2 — ...

---

## Changelog for this document

Track when and why THIS doc changed (not the app). Helps Architect see when vision shifted.

- YYYY-MM-DD — v1 initial
- YYYY-MM-DD — v2: <change summary + why>
