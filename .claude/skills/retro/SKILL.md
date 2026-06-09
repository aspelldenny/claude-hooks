---
name: retro
version: 0.1.0
description: |
  Weekly engineering retrospective — shipping velocity, code quality, patterns.
  Invoke when: user says "retro", "weekly summary", "what did I ship?",
  end of week/sprint, or "how's the project going?"
allowed-tools:
  - Bash
  - Read
  - Grep
  - Glob
  - Agent
---

# /retro — Engineering Retrospective

You are a Team Lead running a weekly retro. Your job is to analyze what was shipped, identify patterns, celebrate wins, and flag risks.

## When to Invoke

- User says "retro", "weekly summary", "what did I ship this week?"
- End of week (Friday/Sunday)
- End of sprint
- User asks "how's the project going?"

## Step 1: Gather Data

```bash
# Commits this week (or custom range)
git log --oneline --since="1 week ago" --format="%h %s (%an, %ar)"

# Diff stats
git diff --stat $(git log --since="1 week ago" --format="%H" | tail -1)..HEAD 2>/dev/null

# Files most changed
git log --since="1 week ago" --name-only --format="" | sort | uniq -c | sort -rn | head -15

# Commit frequency by day
git log --since="1 week ago" --format="%ad" --date=format:"%A" | sort | uniq -c | sort -rn
```

If multi-project, detect from current directory or ask user which project.

## Step 2: Shipping Velocity

Calculate and report:

```
## Shipping Velocity

| Metric | This Week | Trend |
|--------|-----------|-------|
| Commits | N | ↑/↓/→ |
| Lines added | N | |
| Lines removed | N | |
| Files changed | N | |
| PRs merged | N | |
| Bugs fixed | N (from commit msgs) | |
| Features shipped | N (from commit msgs) | |
```

Detect from conventional commits:
- `feat:` → feature shipped
- `fix:` → bug fixed
- `refactor:` → tech debt addressed
- `docs:` → documentation improved

## Step 3: Code Quality Signals

```bash
# Check test results if possible
# Rust:
cargo test 2>&1 | tail -3
# Next.js:
pnpm test --run 2>&1 | tail -5
# Flask:
python -m pytest tests/ -x --tb=line 2>&1 | tail -5
# Or read last CI result
```

Report:
```
## Code Quality

- Tests: X passing / Y failing / Z skipped
- Lint: clean / N warnings
- docs-gate: passing / failing
```

## Step 4: Hotspot Analysis

Identify files that changed most — these are complexity hotspots:

```
## Hotspots (most changed files)

1. `src/lib/ai/prompts.ts` — 12 changes (⚠️ high churn)
2. `app/routes/reading.py` — 8 changes
3. `src/components/TarotCard.tsx` — 5 changes
```

Flag files with >5 changes as potential refactor candidates.

## Step 5: Learnings Review

```bash
# Check recent learnings
ship learn list --recent 5 2>/dev/null
```

If learnings exist, summarize patterns:
- What mistakes were repeated?
- What worked well?
- What should change next week?

## Step 6: Report

```markdown
## Retro: [project] — Week of [date]

### 🚀 Shipped
- [Feature 1] — [1-line description]
- [Feature 2]
- [Bug fix 1]

### 📊 Velocity
| Metric | Value |
|--------|-------|
| Commits | N |
| Lines +/- | +X / -Y |
| Features | N |
| Fixes | N |

### 🔥 Hotspots
- `file1` — N changes (consider refactoring)
- `file2` — N changes

### ✅ What Went Well
- [observation from data]

### ⚠️ Watch Out
- [risk or pattern to address]

### 📝 Action Items
- [ ] [concrete action for next week]
```

## Step 7: Cross-Project Summary (if asked)

If user manages multiple projects, run retro on each:

```bash
for proj in tarot jarvis media-rating-app docs-gate ship; do
  echo "=== $proj ==="
  cd ~/$proj 2>/dev/null && \
  git log --oneline --since="1 week ago" 2>/dev/null | wc -l
  cd - > /dev/null
done
```

```
## Cross-Project Summary

| Project | Commits | Features | Fixes |
|---------|---------|----------|-------|
| tarot | 12 | 2 | 3 |
| jarvis | 5 | 1 | 1 |
| ship | 8 | 3 | 0 |
| Total | 25 | 6 | 4 |
```

## Rules

- Data-driven: every statement backed by git log or metrics
- No fluff: skip "great job!" — show numbers
- Actionable: every "watch out" has a concrete action item
- Honest: if nothing shipped, say so — don't pad
- Mirror user's language in chat; English in any committed retro document
- Compare to last retro if available (stored in ship learn)

## Integration

- With `ship learn`: record retro insights as learnings
- With `ship canary`: include production health in report
- After retro: `ship learn add "retro: [key insight]" -t retro,weekly`
