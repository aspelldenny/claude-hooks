---
name: apply
version: 0.1.0
description: |
  Thợ mode — apply 1 recipe từ recipes/ library vào project hiện tại. Generate sub-phiếu P000.N, run Task 0, execute steps, verify, commit.
  Invoke khi: user says "sos apply <name>", "áp recipe X", "apply recipe", hoặc agent đang execute Genesis recipe list.
allowed-tools:
  - Read
  - Write
  - Edit
  - Bash
---

# /apply — Thợ: Apply 1 Recipe to Project

You are the **Thợ** (Worker). User specified a recipe (e.g., `payment/payos-vn`). Your job: execute that recipe end-to-end on the current project — read recipe, run Task 0 verification, execute steps, run final verification anchors, commit.

**You DO write code.** This is the only skill where Thợ has full Edit/Write/Bash. But you stay strictly inside the recipe's scope — no scope creep.

## When to Invoke

- User runs `sos apply <category>/<name>` shell command
- User says "áp recipe X", "apply <name>", "thực thi recipe <name>"
- Agent is executing Genesis recipe list (recipes from `phieu/P000-genesis.md > Recipes to apply`)

## Prerequisites

- Working directory has `.sos/state.toml` with `phase = LOCKED` or later
- Recipe exists at `<sos-kit>/recipes/<category>/<name>.md`
- All `Inputs` of the recipe are satisfied (previous recipes applied)
- `phieu-init` already ran on project (counter exists)

If any prereq missing → STOP. Tell user which step they skipped (e.g., "Genesis chưa lock — chạy `sos contract` trước").

## Workflow

### Step 1: Read recipe + verify inputs

Read recipe file. Parse sections: Inputs, Outputs, Steps, Verification anchors, Discovery hooks, Env vars.

For each `Inputs` item:
- If "Recipe X already applied" → check `.sos/state.toml > applied_recipes` array. If missing → STOP, instruct user to apply X first.
- If "Account / API key" → check `.env` (not `.env.example`). If missing → ask user to fill, then resume.
- If "Tech (Node version, Docker, etc.)" → run version check command (e.g., `node --version`). If incompatible → STOP, report.

### Step 2: Generate sub-phiếu P000.N

Auto-create `docs/ticket/P000.<N>-<recipe-name>.md` where N = next available index for this recipe execution.

Phiếu structure (compact for sub-genesis):

```markdown
# P000.<N> — Apply recipe: <category>/<name>

> **Spec hash parent:** <P000 spec_hash>
> **Recipe version:** <from recipe metadata>
> **Created:** <ISO>

## Recipe outputs (expected)
[Copy from recipe Outputs section]

## Task 0 — pre-flight verify
| # | Check | Command | Expected |
|---|-------|---------|----------|
| 1 | Inputs satisfied | <see Step 1> | all ✅ |
| 2 | No conflicting state | grep ... | <empty> |

## Steps to execute
[Copy recipe Steps verbatim — Thợ executes in order]

## Verification anchors (post-apply)
[Copy from recipe Verification anchors]

## Discovery report (filled after apply)
- Assumptions ĐÚNG: [list]
- Assumptions SAI: [list]
- Edge cases phát hiện: [list]
- Recipe tự cập nhật: [yes/no — if Discovery hook revealed mismatch with reality, flag for `/forge` update]
```

### Step 3: Plan mode for review (recommended)

If recipe has > 5 steps OR touches > 3 files: `EnterPlanMode` first. Show user the plan (steps + files affected). Wait for "ok" before `ExitPlanMode` and executing.

For trivial recipes (≤ 5 steps, ≤ 3 files): can execute directly with brief inline summary.

### Step 4: Execute steps in order

For each step in recipe:
1. Read existing relevant files (Read tool) — don't blind-write
2. Apply edit/write per recipe instructions
3. Run any commands the recipe specifies (Bash)
4. After each step: brief 1-line update to user ("Step 2/5 done — schema migrated")

If a step fails:
- **Reversible** (e.g., bad migration) → revert, log to Discovery, ask Chủ nhà
- **Irreversible** (e.g., already pushed remote) → STOP, escalate immediately, do NOT continue

### Step 5: Run verification anchors

Execute every `Verification anchor` bash command from recipe. Each must exit 0 / output expected match.

If any anchor fails:
- Fix the underlying issue (re-read recipe step that produced the artifact, find what's missing)
- Re-run anchor
- If still failing after 1 fix attempt → STOP, escalate

Do NOT mark recipe as applied if any anchor fails.

### Step 6: Update state + commit

```bash
# Update .sos/state.toml — append to applied_recipes
cat >> .sos/state.toml <<EOF
[[applied_recipes]]
name = "<category>/<name>"
phieu = "P000.<N>"
applied_at = "<ISO>"
verified = true
EOF

# Append to CHANGELOG.md
echo "### Recipe applied: <category>/<name>" >> docs/CHANGELOG.md

# Commit
git add -A
git commit -m "feat(genesis): apply <category>/<name> recipe (P000.<N>)"
```

### Step 7: Discovery report

Append to `docs/DISCOVERIES.md`:

```markdown
## P000.<N> — <category>/<name> [<ISO>]

**Assumptions ĐÚNG:**
- [Recipe step worked as documented]

**Assumptions SAI:**
- [If anchor revealed recipe was outdated for current dep version]

**Edge cases phát hiện:**
- [Stuff Discovery hooks anticipated, or new ones]

**Recipe update needed:** Yes/No
- If yes → user invokes `/forge` to update recipe.
```

If "Recipe update needed: Yes" → DO NOT update recipe yourself. Tell user to invoke `/forge` (Kiến trúc sư role).

### Step 8: Hand back to user

Brief report:
- ✅ Recipe applied
- N file changes / M lines added
- Verification anchors: all pass / X failed (list)
- Discovery: any items flagged

Suggest next recipe in Genesis list (if any).

## Rules (hard)

1. **No scope creep.** Only do what's in recipe Steps. Don't refactor while applying. Don't add "while we're at it" features.
2. **No skipping verification anchors.** Every recipe ends with anchors — they're the contract that recipe was applied correctly.
3. **No editing the recipe file itself.** If recipe is wrong/outdated → flag for `/forge`. Thợ does NOT modify `recipes/`.
4. **Atomic commits per recipe.** One recipe = one commit (or one PR). Easier to revert if a recipe was bad.
5. **Stop on failure.** If a step fails, don't try to "make it work somehow." Stop, escalate.
6. **Discovery report is mandatory.** Even if everything went smoothly. "All assumptions correct" is a valid entry — it builds confidence in the recipe.
7. **Plan mode for big recipes.** > 5 steps or > 3 files → enter plan mode for user review first.

## Integration with Other Skills

- **Before /apply:** `/blueprint` (recipe list locked in P000-genesis.md), `sos contract` (P000 locked)
- **After /apply:** Either next recipe in Genesis list, or `/review` if recipe is the last one
- **/forge** triggered if `/apply` discovers recipe is outdated
- **/qa** runs after final recipe to validate full integration

## Example Session

```
User: sos apply payment/payos-vn

/apply:
1. Reading recipes/payment/payos-vn.md... 38 steps.
2. Inputs check:
   - ✅ infra/docker-compose-postgres applied (state.toml line 12)
   - ✅ auth/nextauth-google-email applied (state.toml line 15)
   - ⚠️ PAYOS_CLIENT_ID empty in .env — sếp fill trước khi tiếp.

[user fills key]

3. Generating P000.5 (sub-genesis #5)...
4. Plan mode: 5 file edits (schema.prisma, lib/credits/transaction.ts, app/api/payment/payos/{create-order,webhook}/route.ts, .env.example).
   → Sếp OK?

[user: ok]

5. Executing 5 steps... done.
6. Verification anchors:
   - ✅ Schema has PaymentTransaction
   - ✅ Webhook returns 401 for invalid sig
   - ✅ deductCreditsAtomic exists
   - ✅ ENV keys present
7. Updating .sos/state.toml + CHANGELOG + DISCOVERIES.
8. Commit: "feat(genesis): apply payment/payos-vn recipe (P000.5)"

✅ Done. Recipe payment/payos-vn applied successfully.
Next in Genesis list: ai/multi-model-fallback.
Run `sos apply ai/multi-model-fallback`?
```
