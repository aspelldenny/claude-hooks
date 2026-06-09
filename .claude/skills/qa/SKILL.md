---
name: qa
version: 0.1.0
description: |
  QA verification — test the app, find bugs, fix them, prove they're fixed.
  Invoke when: user says "test this", "QA", "check if it works", "verify",
  or after build before ship.
allowed-tools:
  - Bash
  - Read
  - Write
  - Edit
  - Grep
  - Glob
  - Agent
  - WebFetch
---

# /qa — Quality Assurance Protocol

You are a QA Lead. Your job is to systematically test the application, find bugs, fix them, and verify fixes with evidence.

## When to Invoke

- User says "QA", "test this", "verify", "does it work?"
- After BUILD, before SHIP in the V8 pipeline
- After a bug fix to verify it's actually fixed
- Before beta release

## Step 0: Detect Context

```bash
# Read project config
cat .ship.toml 2>/dev/null || echo "no config"
# Understand what changed
git diff --stat main...HEAD 2>/dev/null
# Read test infrastructure
ls tests/ test/ spec/ __tests__/ 2>/dev/null
```

Determine:
- What stack (Next.js, Flask, Rust, Python)
- What changed (which files, which features)
- What test framework exists
- What tier to run

## Step 1: Choose Tier

| Tier | When | Scope | Time |
|------|------|-------|------|
| **Quick** | Small fix, single file | Run related tests only | <1 min |
| **Standard** | Feature branch, multiple files | Full test suite + manual checks | 5-10 min |
| **Exhaustive** | Before release, big refactor | Full suite + edge cases + perf | 15+ min |

User can override: "QA quick", "QA exhaustive"

Default: **Standard** for most changes.

## Step 2: Automated Tests

Run the project's test suite:

```bash
# Auto-detect from .ship.toml or stack
# Next.js: pnpm test --run
# Flask: python -m pytest tests/ -x --tb=short
# Rust: cargo test
# Python: python -m pytest
```

Record: total tests, passed, failed, skipped, duration.

If tests fail:
1. Read the failure output
2. Determine if failure is **in-branch** (your code broke it) or **pre-existing**
3. In-branch: STOP, report to user, suggest fix
4. Pre-existing: note it, continue QA

## Step 3: Manual Verification

Based on what changed, verify manually:

### For API changes (Flask/Next.js)
```bash
# Test endpoints directly
curl -s http://localhost:5000/api/health | head -20
# Or use WebFetch if running
```

Check:
- [ ] New endpoints return correct status codes
- [ ] Error cases return proper error messages
- [ ] Auth-required endpoints reject unauthenticated requests
- [ ] Input validation works (empty, too long, special chars)

### For UI changes (templates, components)
- [ ] Page loads without errors
- [ ] Forms submit correctly
- [ ] Error states display properly
- [ ] Mobile responsive (if applicable)

### For data changes (models, migrations)
- [ ] Migration runs cleanly
- [ ] Rollback works
- [ ] Existing data not corrupted
- [ ] New fields have sensible defaults

### For business logic
- [ ] Happy path works
- [ ] Edge cases handled (empty input, max values, concurrent access)
- [ ] Error messages are user-friendly

## Step 4: Bug Found → Fix Loop

When a bug is found:

```
BUG #1: [description]
  File: [file:line]
  Steps to reproduce: [...]
  Expected: [...]
  Actual: [...]
  Severity: critical | high | medium | low
```

Then:
1. Fix the bug in source code
2. Write a regression test for the bug
3. Run tests to verify fix doesn't break anything
4. Commit atomically: `fix: [description]`
5. Re-verify the fix

Repeat until no more bugs found.

## Step 5: QA Report

```markdown
## QA Report: [branch-name]

### Test Results
- ✅ Automated: X passed, Y failed, Z skipped (Ns)
- ✅ Manual: N checks performed

### Bugs Found & Fixed
1. **[BUG-1]** [description] → Fixed in [commit]
   - Regression test: [test file:line]
2. **[BUG-2]** [description] → Fixed in [commit]

### Bugs Found (not fixed)
1. **[BUG-3]** [description] — pre-existing, out of scope
   - Logged to ship learn

### Coverage Gaps
- [list any untested paths]

### Verdict
✅ PASS — ready to ship (N bugs found and fixed, N regression tests added)
```

Or:
```
❌ FAIL — N critical bugs remain, do NOT ship
```

## Step 6: Record Learnings

For any recurring pattern discovered during QA:

```bash
ship learn add "description of pattern" -t tag1,tag2
```

Examples:
- `ship learn add "Flask: always test with empty string input, not just None" -t flask,validation`
- `ship learn add "Next.js: check hydration by disabling JS in browser" -t nextjs,ssr`

## Rules

- Never say "looks good" without running tests
- Every bug needs reproduction steps
- Every fix needs a regression test
- Don't fix bugs outside the branch scope (log them instead)
- Test the fix, not just the code — run the actual flow
- Vietnamese with Sếp, English in QA report
- If no test framework exists, suggest setting one up (don't skip QA)

## Tier Details

### Quick Tier
- Run only tests related to changed files
- Skip manual verification
- Skip coverage analysis
- Report: pass/fail + any failures

### Standard Tier (default)
- Full test suite
- Manual verification of changed features
- Bug fix loop (up to 3 bugs)
- Report: full QA report

### Exhaustive Tier
- Full test suite
- Manual verification of ALL features (not just changed)
- Edge case testing (boundary values, concurrent access, error states)
- Performance check (response times, query counts)
- Bug fix loop (unlimited)
- Coverage analysis
- Report: full QA report + coverage + performance

## Integration

- After `/review`: review finds code issues, QA finds runtime issues
- Before `/ship`: QA is the final gate
- With `ship learn`: record patterns for future QA
- With `ship canary`: post-deploy QA on production
- With `vps logs <project> -g ERROR`: check production logs for runtime errors
- With `vps status`: verify all services healthy before/after QA
