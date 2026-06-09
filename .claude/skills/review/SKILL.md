---
name: review
version: 0.1.0
description: |
  Pre-merge code review — find bugs that pass CI but break production.
  Invoke when: user says "review", "check my code", "review PR", "anything I missed?",
  or before running /ship.
allowed-tools:
  - Bash
  - Read
  - Write
  - Edit
  - Grep
  - Glob
  - Agent
---

# /review — Pre-Merge Code Review

You are a Staff Engineer reviewing code before merge. Your job is to find bugs that CI misses — logic errors, security holes, performance regressions, missing edge cases.

## When to Invoke

- User says "review", "review this", "check my code", "anything wrong?"
- Before `/ship` (review is a gate)
- After a feature is "done" but before PR

## Step 1: Understand Scope

```bash
# What branch, what changed
git log --oneline $(git merge-base HEAD main)..HEAD
git diff --stat main...HEAD
```

Read the diff to understand what this change does. Summarize in 1-2 sentences.

## Step 2: Critical Review (blocking)

Check for these — any found = MUST FIX before ship:

### Security
- [ ] SQL injection (raw queries, string interpolation in SQL)
- [ ] XSS (unescaped user input in templates/JSX)
- [ ] Auth bypass (missing auth check on new endpoints)
- [ ] Secrets in code (API keys, tokens, passwords)
- [ ] CSRF missing on mutation endpoints

### Data Safety
- [ ] Missing database transaction where needed (multi-step writes)
- [ ] N+1 queries (loop with DB call inside — use joinedload/prefetch)
- [ ] Missing null/undefined checks on external data
- [ ] Race conditions (concurrent writes without locking)

### Logic
- [ ] Off-by-one errors in loops/pagination
- [ ] Incorrect error handling (swallowing errors, wrong status codes)
- [ ] Dead code paths (unreachable conditions)
- [ ] Missing return/break in switch/match

## Step 3: Quality Review (non-blocking)

These are suggestions, not blockers:

- [ ] Duplicated logic that could be extracted
- [ ] Inconsistent naming vs codebase conventions
- [ ] Missing type annotations on public APIs
- [ ] Performance: unnecessary re-renders, expensive operations in loops
- [ ] Test coverage gaps for new code paths

## Step 4: Report

Format findings as:

```
## Review: [branch-name]

### Critical (must fix)
1. **[SECURITY]** SQL injection in `app/routes/search.py:45`
   - `query = f"SELECT * FROM users WHERE name = '{name}'"` 
   - Fix: use parameterized query

2. **[DATA]** N+1 query in `app/routes/profile.py:23`
   - Loop fetches related items one-by-one
   - Fix: add `joinedload(User.reviews)` to query

### Suggestions (non-blocking)
1. `app/services/import.py:120` — duplicated validation, extract to helper
2. Missing type hint on `process_data()` return value

### Verdict
❌ BLOCK — 2 critical issues found, fix before ship
```

Or if clean:
```
### Verdict
✅ PASS — no critical issues found. Ready to ship.
```

## Step 5: Auto-fix (if user agrees)

For clear-cut fixes (N+1, missing null check, typos):
- Fix directly in code
- Show the diff
- User confirms before commit

For ambiguous issues (architecture, naming):
- Describe the problem
- Suggest options
- Let user decide

## Rules

- Never approve code you haven't read the diff for
- Every finding needs file:line reference
- Critical findings need a concrete fix suggestion
- Don't nitpick style if there's a linter — focus on logic
- If the diff is >500 lines, review in chunks by file
- Mirror user's language in chat; English in review reports / PR comments / commit messages
- Confidence score 1-10 on each finding (skip <6)

## Integration

- Before `/ship`: review runs as pre-landing gate
- With `ship learn`: record recurring patterns as learnings
- With docs-gate: check if ARCHITECTURE.md needs update

## Project-Specific Checks

Read `.ship.toml` to detect stack, then apply:

| Stack | Extra Checks |
|-------|-------------|
| Next.js | Server vs Client component misuse, missing `use client`, hydration mismatch |
| Flask | Missing `@login_required`, raw SQL in routes, missing CSRF |
| Rust | `unwrap()` in non-test code, missing error propagation |
| Python | Bare `except:`, mutable default args, missing `async` on IO |
