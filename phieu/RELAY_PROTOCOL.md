# Relay Protocol — Chủ nhà as the Courier

> Kiến trúc sư (Claude Web Project) and Thợ (Claude Code) are separate sessions. They cannot talk directly. Chủ nhà is the only bridge.
>
> Most of the time the relay is implicit (Chủ nhà commits phiếu file, Thợ reads it from disk). But when Thợ hits a Tầng 1 blocker mid-ticket, Chủ nhà has to actively route messages back and forth. This doc is the protocol for that routing.

## Why this exists

In a team, PMs don't copy-paste between engineers and architects. But a solo OS has only one human, and the two AI instances live in different tools (Claude Code CLI vs Claude Web Project with attached docs). The human is the manual routing layer.

Without a protocol:
- Context gets lost ("wait, what did the Worker say again?")
- Architect's response doesn't map back to Worker's question
- Conversations drift into "I thought you said X" between separate sessions

With a protocol: relay takes 2-3 minutes per round trip, both sides stay on the same page.

## When relay is triggered

| Scenario | Who starts | Who Chủ nhà routes to |
|---|---|---|
| Worker hits Tầng 1 blocker (Handoff 3 in HANDOFF.md) | Worker writes escalation | → Architect |
| Architect's response to a blocker | Architect writes decision | → Worker |
| Architect wants to clarify phiếu before Worker codes (rare) | Architect writes question | → Worker |
| Worker asks "is this Tầng 1 or Tầng 2?" | Worker unsure | → Architect (Chủ nhà can also decide directly if obvious) |

For the common happy-path (Architect writes phiếu → Chủ nhà approves → Worker executes), relay is just "commit the phiếu file" — not covered here.

## The Relay Checklist

### When routing Worker → Architect

**Step 1. Receive Worker's escalation.** Worker output should already be in the Handoff 3 format (`BLOCKER / PHIẾU / TẦNG / CONTEXT / OPTIONS / THỢ LEAN / NEED FROM ARCHITECT`). If not, ask Worker to reformat before you forward.

**Step 2. Add Chủ nhà context in ONE sentence before the escalation.** This is the value you add as a human — context Worker doesn't have:

```
Em Kiến trúc sư, Thợ vừa escalate phiếu P044. Context:
- Đây là phiếu deadline-sensitive (muốn ship tuần này)
- Thợ ít kinh nghiệm với area này, em xác nhận option B có risk

[paste Worker's escalation verbatim]
```

Context types Chủ nhà typically adds:
- **Timeline pressure** — "ship this week" / "no rush"
- **User impact** — "affects Persona A" / "zero users affected"
- **SOUL conflict** — "this touches hard line #2, careful"
- **Worker skill signal** — "Worker has low context on this area, double-check their lean"
- **Prior decisions** — "we already decided X in phiếu P042, mentioning for consistency"

If you can't think of context to add, just forward the escalation verbatim with "Em Kiến trúc sư, Thợ hỏi:".

**Step 3. Paste into Claude Web Project session** (or start a new session if the prior one is stale).

**Step 4. Copy Architect's response back when it arrives.**

### When routing Architect → Worker

**Step 1. Read Architect's response.** It should be one of:
- Updated phiếu (re-written sections)
- A short decision ("Thợ's option A is correct, proceed")
- Clarification question back to Worker

**Step 2. Strip Claude Web formatting** (sometimes has extra ceremony) and extract the actionable part.

**Step 3. Paste to Worker with routing label:**

```
[RELAY from Architect] P044 — response:

<Architect's decision / updated phiếu section>

Em tiếp tục được chưa.
```

If Architect updated the phiếu file significantly, save the updated file to the project's `docs/ticket/P<NNN>-<slug>.md` (overwrite), then tell Worker "phiếu đã update, em đọc lại".

### When you can SHORT-CIRCUIT (skip Architect)

Not every Worker escalation needs Architect. If:

1. **It's obviously Tầng 2** — Worker misclassified; tell Worker "đây là Tầng 2, em tự quyết + log Discovery"
2. **You can decide it yourself** — if Worker's options are all reasonable and choice depends on Chủ nhà judgment (wording, scope trade-off), use `/decide` directly, don't loop Architect
3. **Identical to a previous decision** — Architect already decided this in a prior phiếu; cite the prior decision, don't re-ask

Short-circuit saves round-trip time. Rule of thumb: loop Architect only when the question is genuinely architectural and you can't answer it.

## Format template — Chủ nhà relay message

For Worker → Architect:

```
Em Kiến trúc sư,

Context từ em: <Chủ nhà's context, 1 sentence>

Thợ escalate:

────────────────────────────────
BLOCKER:                 <from Worker>
PHIẾU:                   P<NNN>-<slug>
TẦNG:                    Tầng 1
CONTEXT:                 <from Worker>
OPTIONS:                 <from Worker>
THỢ LEAN:                <from Worker>
NEED FROM ARCHITECT:     <from Worker>
────────────────────────────────

Em quyết đi. Nếu cần thêm context gì từ em, hỏi.
```

For Architect → Worker:

```
[RELAY từ Kiến trúc sư] P<NNN>-<slug>

<Architect's decision/response>

Em (Thợ) tiếp tục được.
```

## Anti-patterns

### 1. Chủ nhà paraphrases instead of forwarding
Symptom: Worker's 5-line escalation becomes a 1-line paraphrase "Worker says option B won't work." Context lost.
Fix: paste verbatim. Add Chủ nhà context as a prefix, don't replace Worker's detail.

### 2. Chủ nhà decides for Architect when the question is genuinely architectural
Symptom: "I told Worker to go with option A because it's faster." But option A violates an assumption Architect had about future extensibility.
Fix: if you can't confidently say "this is in Architect's domain" or "this is in Chủ nhà's domain," err toward Architect.

### 3. Chủ nhà forgets to relay back
Symptom: Worker is blocked for 2 hours waiting. Chủ nhà got Architect's response, didn't paste back.
Fix: when you paste Worker's escalation to Architect, set a mental reminder: "I now have a pending relay back to Worker." Close the loop.

### 4. Architect responds to stale context
Symptom: Chủ nhà didn't include "Worker already tried option A and it failed" — Architect recommends option A. Waste.
Fix: Step 2 context should include "what's been tried" if relevant.

### 5. Long-running parallel threads
Symptom: Phiếu P042 blocker happening while Phiếu P044 escalation unresolved — Chủ nhà context-switches, forgets which is which.
Fix: label every relay with phiếu ID `P<NNN>-<slug>` so both sides never confuse tickets.

## What Chủ nhà does NOT do in relay

- Does NOT solve the architectural question themselves (that's Architect's job — route to them)
- Does NOT edit Worker's escalation (just add context as prefix)
- Does NOT edit Architect's response beyond stripping ceremony (Worker needs the actual decision)
- Does NOT hide relay history from future-self — keep a log if tickets run long

## Lightweight logging (optional)

For long-running tickets (> 1 day), keep a mini relay log in the phiếu file itself, appended at the bottom:

```markdown
---
## Relay log (Chủ nhà tracking)

- 2026-04-25 10:00 — Worker escalated Task 0 anchor 3 ❌
- 2026-04-25 10:15 — Architect responded: updated Task 0 anchor 3 to `lib/helpers.ts` instead of `lib/utils.ts`
- 2026-04-25 10:30 — Worker re-verified, all ✅, proceeding
```

This log is only for Chủ nhà's memory. Doesn't block anything.

## Summary

| Step | Who | What |
|---|---|---|
| 1 | Worker | Writes Handoff 3 escalation in formatted format |
| 2 | Chủ nhà | Adds 1-sentence context, pastes verbatim to Architect |
| 3 | Architect | Decides / updates phiếu |
| 4 | Chủ nhà | Strips ceremony, labels `[RELAY from Architect]`, pastes back |
| 5 | Worker | Continues with new direction |

2-3 minutes of human routing time saves Worker from silent architectural drift.
