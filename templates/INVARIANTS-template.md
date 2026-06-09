# INVARIANTS

> **Project-local invariant catalog** consumed by Giám sát (boundary-check) specialist subagent via `/security-review`.
> Copy this template to your project (typically `docs/security/INVARIANTS.md` or wherever your security docs live) and extend with project-specific INV in the "User-added INV" section.
>
> The 5 generic INV below are baked into `agents/boundary-check.md` rubric — Giám sát already checks these regardless of whether this file exists. This template documents the INV catalog so Chủ nhà has one place to read the rules and extend them.

---

## Generic INV (baked into Giám sát rubric — P042 ship)

These 5 invariants run on every `/security-review` invocation. Giám sát's full rubric per-INV lives in `agents/boundary-check.md` (the subagent file).

### INV-1 — New env var → env template update

**Statement:** PR thêm new env var read (any language pattern: `process.env.X`, `os.environ.get('X')`, `std::env::var("X")`, `os.Getenv("X")`, shell `${X}`) PHẢI update `.env.example` (or equivalent env-template doc per stack convention) với key mới.

**Why:** new env var without template update = silent failure on fresh clone + onboarding friction.

**Trigger keywords (multi-language):** `process.env.`, `os.environ`, `std::env::var`, `os.Getenv`, `env!`, shell variable expansion.

**Status:** Active. Giám sát checks per-PR.

### INV-2 — New external service call → timeout + error handling

**Statement:** PR thêm new HTTP/external-API call PHẢI có explicit timeout AND error-handling (retry optional but recommended).

**Why:** call without timeout = hung connection on outage; without error-handling = unhandled exception cascade.

**Trigger keywords:** `fetch(`, `axios.`, `requests.`, `httpx.`, `reqwest::`, `http.Get`, `http.Post`, `urllib`, etc.

**Status:** Active.

### INV-3 — Cross-user resource access → ownership binding

**Statement:** PR thêm API route/handler reading or mutating user-scoped data (DB query, cache key, session state) PHẢI có explicit ownership binding (`where userId = session.user.id`, cache key prefix with user ID, or equivalent).

**Why:** new endpoint without ownership filter = horizontal privilege escalation / data leak.

**Trigger keywords:** API route patterns per stack (Next.js `app/api/.../route.ts`, Flask `@app.route`, FastAPI `@router`, actix `route!`, axum `Router::route`, gin/echo handlers).

**Status:** Active.

### INV-4 — Webhook handler → signature verify + replay protection

**Statement:** PR thêm inbound webhook handler PHẢI verify signature/HMAC AND có replay protection (nonce or timestamp window check) trước khi đọc request body fields.

**Why:** webhook without signature verify = anyone POSTs fake events; without replay protection = attacker re-plays old signed payloads.

**Trigger keywords:** new route file matching `webhook` (case-insensitive) OR POST handler accessing `signature` / `x-signature` / `x-hub-signature` header.

**Status:** Active.

### INV-5 — Dependency major bump → changelog/migration audit

**Statement:** PR bumps any dependency's MAJOR version PHẢI cite changelog review + breaking-change scan trong PR description body.

**Why:** major bump = breaking changes by SemVer convention. Complements Trinh sát's GHSA scan (Trinh sát flags known CVEs; Giám sát flags discipline of audit-before-bump).

**Trigger keywords:** `package.json` / `requirements.txt` / `pyproject.toml` / `Cargo.toml` / `go.mod` version diff with MAJOR component change.

**Status:** Active.

---

## User-added INV (project-specific)

> Extend this section with INV-6+ as your project's domain requires. Each INV here is project-local: Giám sát's baked rubric DOES NOT check these automatically — they're documentation for human review AND a TODO list if you want to extend `agents/boundary-check.md` rubric.

Format for each user INV:

```markdown
### INV-N — <short title>

**Statement:** [the rule in 1-2 sentences]

**Why:** [risk it mitigates]

**Trigger keywords / file paths:** [where to scan]

**Status:** Active | Disabled

**Implemented in Giám sát:** Yes / No (if No → human review at PR time)
```

### Example placeholder

```
### INV-6 — [your invariant name]

**Statement:** [your rule]

**Why:** [your risk]

**Trigger keywords / file paths:** [your patterns]

**Status:** Active

**Implemented in Giám sát:** No (project-local, human-reviewed)
```

(Delete this placeholder block when you add your first real INV-6.)

---

## How INV are checked

1. Worker pushes PR.
2. Quản đốc (or user) runs `/security-review <PR>` slash command.
3. Slash command captures diff via `gh pr diff` (or `git diff` fallback) and spawns Giám sát subagent.
4. Giám sát checks 5 generic INV (rubric baked in `agents/boundary-check.md`).
5. Slash command parses sentinel-wrapped verdict (between `<!-- security-review-start -->` and `<!-- security-review-end -->` markers), posts as PR comment (silent if APPROVE + 0 FLAG; comment posted if NEEDS_REVIEW).
6. **ADVISORY mode:** verdict does NOT block merge. Chủ nhà reads comment, decides to address or accept risk.

## Why ADVISORY (not blocking)

- Generic INV at kit-level can over-flag (false positives in domain-specific code). Blocking = noisy gate that gets disabled.
- Discipline > automation: Chủ nhà reading the comment and deciding = stronger signal than CI-pass.
- Future: users can extend slash command to block on FLAGd INV in their own project — but kit ships ADVISORY default.

## Sentinel marker contract

Giám sát returns verdict wrapped in `<!-- security-review-start -->` ... `<!-- security-review-end -->`. These markers are LOAD-BEARING — slash command grep-extracts the block between them. DO NOT rename without phiếu.
