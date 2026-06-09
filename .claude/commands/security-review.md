---
description: Run boundary-check security review on a PR / branch / commit range. Spawns Giám sát subagent which checks 5 generic INV (env var / external service / cross-user / webhook / dep major). Posts ADVISORY comment to PR (silent if clean). KHÔNG block merge.
---

# /security-review

You are the orchestrator (Quản đốc) running the security-review slash command. Execute these steps in order — DO NOT skip, DO NOT improvise. Boundary checks happen INSIDE the Giám sát subagent, NOT in this main session.

**ADVISORY mode reminder:** This command surfaces evidence for Chủ nhà review. It does NOT block merge, does NOT auto-fix, does NOT call `gh pr merge --block`.

## Step 0 — Determine review scope

User invokes one of:
- `/security-review <PR-number>` → review PR #<N> via `gh pr diff <N>`.
- `/security-review <branch>` → review branch vs main: `git diff main..<branch>`.
- `/security-review <commit-range>` → review explicit range: `git diff <range>`.
- `/security-review` (no arg) → review current HEAD vs `git merge-base origin/main HEAD` (default: review current branch's commits).

Resolve the diff source from user's argument. If ambiguous → ask user via 1-question multi-choice (NOT free-form).

## Step 1 — Capture diff content

Capture diff via Bash:
- PR mode: `gh pr diff <N>` (if `gh` available + authenticated)
- Branch mode: `git diff <base>..<head>`
- Range mode: `git diff <range>`

Capture file list: `gh pr diff --name-only <N>` OR `git diff --name-only <base>..<head>`.

Capture PR body (PR mode only, for INV-5 changelog check): `gh pr view <N> --json body --jq .body`.

**If `gh` is not available + PR mode requested** → fall back to branch mode using user-provided merge base, OR tell user to pass `--branch` argument.

**If diff > 100KB** → write to `/tmp/security-review-diff-<id>.txt` and pass path to subagent; otherwise inline in spawn prompt.

## Step 2 — Spawn Giám sát subagent

**Step 2a — Inject project-local invariants (INV-LOCAL slot).** Giám sát ships only the *generic* 5-INV rubric and by contract does NOT self-read INVARIANTS.md (`agents/boundary-check.md`: *"Caller's responsibility to inject INV-LOCAL-*"*). The caller MUST fill the INV-LOCAL slot below:
- If `docs/security/INVARIANTS.md` exists → read it and extract the block of entries whose headings match `^##\s*INV-LOCAL-` (dynamic read — do NOT hardcode the INV list into this command; it is an N-repo template).
- If the file is absent OR has no `INV-LOCAL-*` entries → the slot value is the literal string `N/A — no project-local invariants defined`.

**Step 2b — Spawn.** Use `Task` tool with `subagent_type: "boundary-check"`. Fill EVERY `< >` slot in the template — including the INV-LOCAL slot (do NOT drop it):

```
You are Giám sát. Run your full workflow (Bước 0 receive context → Bước 1 identify scope per INV → Bước 2 check rubric → Bước 3 compose verdict → Bước 4 emit final report).

Review scope: <PR #N | branch <name> | range <range>>
Diff content: <inline diff OR path to /tmp/security-review-diff-<id>.txt>
Files touched: <list>
PR body (for INV-5 changelog check, optional): <body OR "N/A — not a PR">
Project-local invariants (INV-LOCAL-*, check IN ADDITION to your generic 5 INV): <paste the INV-LOCAL block extracted in Step 2a verbatim — OR "N/A — no project-local invariants defined">

Return your final report with `<!-- security-review-start -->` ... `<!-- security-review-end -->` block as specified.
```

Wait for subagent return. Subagent handles 5-INV (+ any injected INV-LOCAL-*) scan + verdict composition entirely on its own (scoped Bash for cross-INV correlation if needed).

## Step 3 — Extract sentinel block from subagent output

Use `Grep` or string parsing to locate the block between `<!-- security-review-start -->` and `<!-- security-review-end -->` in the subagent's return.

- **PR mode (block-unsafe-merge-governed) — ALWAYS post, including clean APPROVE.** Vì `scripts/block-unsafe-merge.sh` ĐÒI sentinel `Verdict: APPROVE` comment trên PR để cho merge security-surface PR. Nếu silent-when-clean nuốt comment ở PR mode → merge deadlock (chỉ thoát bằng override marker = `--no-verify`-death). Trong PR mode, BỎ QUA silent-when-clean → luôn continue to Step 4 và post block (cả APPROVE lẫn NEEDS_REVIEW). Đây là Option A (ket WORKFLOW §21 precedent), scoped chặt CHỈ PR mode.
- **Branch / range mode (advisory, no PR to gate) — silent-when-clean GIỮ NGUYÊN.** Nếu verdict = `APPROVE` AND 0 FLAG → Do NOT post/write. Tell user: "Security review complete. APPROVE (0 flags). No comment posted." Nếu `NEEDS_REVIEW` OR ≥1 FLAG → continue to Step 4 (write to local file per Step 4 branch/range path).

## Step 4 — Post advisory comment (or fallback to local file)

**PR mode (preferred) — post for BOTH clean APPROVE and NEEDS_REVIEW:**
- `gh pr comment <N> --body "<sentinel-block-content>"` — post the full sentinel-wrapped block (chứa `<!-- security-review-start -->` … `Verdict: APPROVE|NEEDS_REVIEW` … `<!-- security-review-end -->`) as a PR comment.
- **Lý do post cả clean APPROVE:** `scripts/block-unsafe-merge.sh:103-109` grep comment cho `<!-- security-review-start -->` + `^Verdict:` chứa `APPROVE` để cho merge. Không có comment APPROVE = hook chặn merge (deadlock). PR mode KHÔNG áp silent-when-clean (xem Step 3).
- Verify post: `gh pr view <N> --json comments` should show the new comment with the sentinel block.

> **Known limitation — APPROVE sentinel is NOT SHA-scoped.** `scripts/block-unsafe-merge.sh:102-106` greps for ANY historical `Verdict: APPROVE` sentinel comment on the PR, with no binding to the reviewed commit's head SHA. Hệ quả: trên một multi-commit PR, một clean APPROVE trên commit A có thể satisfy gate cho commit B+C chưa review. **Mitigations:** (1) Chủ nhà đọc comment APPROVE có timestamp trước khi merge; (2) squash-merge collapse history. SHA-scoping tracked separately = **[P055]** (docs/BACKLOG.md Open backlog). Mirror pattern: documented bypass at block-unsafe-merge.sh:15-16.

**Branch/range mode (no PR context):**
- Write sentinel block to `docs/security/last-review.md` (or filename user prefers).
- Tell user the path; user reviews locally.

**If `gh pr comment` fails** (auth issue, no PR for branch yet, etc.):
- Fall back to local file (same path as branch mode).
- Surface error to user with one-line note: "PR comment failed; review at <path>".

## Step 5 — Report to user

Tell user:
- Verdict: `APPROVE` or `NEEDS_REVIEW`.
- Per-INV summary (1-line each): `INV-1 PASS/FLAG`, `INV-2 PASS/FLAG`, `INV-3 PASS/FLAG`, `INV-4 PASS/FLAG`, `INV-5 PASS/FLAG`.
- Where comment posted (PR #N) OR file written (`<path>`).
- ADVISORY reminder: merge gate is NOT affected. Chủ nhà reads the comment and decides.

## Hard rules

- Giám sát is the WORKHORSE. Diff inspection, 5-INV rubric (INV-1 through INV-5), verdict composition all happen INSIDE the subagent (scoped Bash for cross-INV correlation only). Main session ONLY captures diff + spawns + posts comment.
- ADVISORY mode is structural: this slash command does NOT call `gh pr merge --block` or set any blocking status. KHÔNG bao giờ.
- Sentinel markers `<!-- security-review-start -->` / `<!-- security-review-end -->` are LOAD-BEARING. Do not rename, do not duplicate, do not move.
- The INV-LOCAL slot (Step 2a/2b) is part of the spawn template, NOT optional. Fill it every spawn — paste the `INV-LOCAL-*` block from `docs/security/INVARIANTS.md`, or the literal `N/A — no project-local invariants defined`. Silently dropping it = the doc-rotate dormancy (project-local invariants never checked because the inject step lived only in a handbook, not here). Dynamic-read per-repo; never hardcode the INV list into this template.
- Silent-when-clean rule (generic anti-approve-fatigue principle): `APPROVE + 0 FLAG → no comment` — applies to **branch/range (advisory) mode only**. **PR mode (block-unsafe-merge-governed) ALWAYS posts sentinel comment including clean APPROVE** (P053 — needed so block-unsafe-merge.sh can allow merge; silence = deadlock). Apply this rule HERE in slash command, NOT in Giám sát (Giám sát always returns sentinel block; silent decision is caller's).
- 5 INV are the contract from P042. Adding INV-6+ requires updating BOTH `agents/boundary-check.md` rubric + `templates/INVARIANTS-template.md` user-added section in a new phiếu.
- If Giám sát reports "diff capture failed / no diff content" → relay verbatim, NOT a silent success.
