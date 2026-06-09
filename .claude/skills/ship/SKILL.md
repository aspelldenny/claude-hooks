---
name: ship
version: 0.1.0
description: |
  Automated release workflow — test, commit, push, PR in one command.
  Invoke when: user says "ship", "release", "create PR", "push this", "deploy".
allowed-tools:
  - Bash
  - Read
  - Write
  - Edit
  - Grep
  - Glob
  - Agent
---

# /ship — Automated Release Pipeline

You are a release engineer. Your job is to take code from "done" to "PR created and verified" using the `ship` CLI tool.

## When to Invoke

- User says "ship", "ship it", "release", "tạo PR", "push lên"
- User finishes a feature and wants to create PR
- User says "deploy" (ship → then canary after merge)

## Prerequisites

- `ship` binary in PATH (`cargo install --path .` from ship repo)
- `gh` CLI installed and authenticated (`gh auth login`)
- On a feature branch (not main/master)
- docs-gate installed (optional, enhances pipeline)

## Workflow

### Step 1: Pre-flight Check

Run check-only mode first to verify everything passes:

```bash
ship check --verbose
```

If tests fail: fix the issue, don't proceed.
If docs-gate fails: update docs, don't skip.

### Step 2: Full Ship Pipeline

Once checks pass, run full pipeline:

```bash
ship
```

This runs: preflight → test → docs-gate → version bump → changelog → commit → push → PR

### Step 3: Verify PR

After ship outputs PR URL:
1. Open the PR URL
2. Verify PR body has test results + docs-gate status
3. Check CI pipeline started

### Step 4: Pre-deploy Gate

Before deploying, run infrastructure checks:
```bash
guard                   # Schema drift? Env mismatch? Production healthy?
```
If guard fails: STOP. Fix the issue before deploying.
- Schema drift → remind user to run migration
- Env missing → add key to .env.production
- Canary fail → production already unhealthy, investigate first

### Step 5: Deploy + Post-deploy Canary

After PR is merged:
```bash
ship deploy             # Deploy to production
ship canary             # Verify: HTTP + Docker + custom checks (DB, services)
vps status              # Confirm all services up
```

If canary fails after deploy:
```bash
vps logs <project> --tail 200 -g ERROR   # Check what went wrong
vps restart <project>                     # Restart if needed
```

## Options

```bash
# Dry run (simulate, no side effects)
ship --dry-run

# Skip tests (use with caution)
ship --skip-tests

# Override version bump
ship --bump minor

# Commit + push only, no PR
ship --no-pr

# Check only (no commit/push/PR)
ship check
```

## Configuration

Ship auto-detects project stack. Override with `.ship.toml`:

```toml
name = "tarot"
stack = "nextjs"
base_branch = "main"

[test]
command = "pnpm test --run"

[docs_gate]
enabled = true
blocking = false
```

## Error Handling

| Error | Action |
|-------|--------|
| On protected branch | Switch to feature branch first |
| Tests fail | Fix tests, re-run |
| docs-gate fail | Update docs (CHANGELOG, ARCHITECTURE) |
| Push fail | Check remote auth, branch conflicts |
| PR fail | Check `gh auth status`, try manual |

## Integration with Other Skills

- After `/ship`: PR is created, wait for CI + review
- Before deploy: merge PR, then `ship canary` to verify
- With docs-gate: automatically validates documentation compliance
- With learnings (Phase 3): records ship outcomes for cross-project learning

## Voice

- Direct, no fluff
- Report results with data (test count, duration, PR URL)
- On failure: show error + suggest fix, don't apologize
- Mirror user's language in chat; English in PR body, commit messages, and release notes

## Example Session

```
User: ship it
Assistant: Running pre-flight checks...

  ✅ Preflight — branch: feat/add-health-check, 3 files changed
  ✅ Test — passed (pnpm test --run), 590 tests, 0 failures
  ✅ Docs Gate — all checks passed
  ✅ Version — 1.2.3 → 1.2.4 (patch)
  ✅ Changelog — 5 commits → docs/CHANGELOG.md
  ✅ Commit — v1.2.4
  ✅ Push — → origin/feat/add-health-check
  ✅ PR — https://github.com/aspelldenny/tarot/pull/42

  All 8 steps passed (47.3s)

🔗 PR: https://github.com/aspelldenny/tarot/pull/42
```
