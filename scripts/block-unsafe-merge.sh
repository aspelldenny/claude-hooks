#!/usr/bin/env bash
# PreToolUse hook — block `gh pr merge <N>` nếu PR touch security surface chưa có /security-review APPROVE comment.
#
# Đầu vào: Claude Code hook spec gửi JSON qua stdin với { "tool_input": { "command": "..." } }.
# Fallback: $CLAUDE_TOOL_INPUT env var nếu stdin trống.
# Exit 2 → block tool call (stderr message hiện UI). Exit 0 → allow.
#
# Doctrine: WORKFLOW_V2.2.md §7 Sub-mech A (trigger gap) + §8 (boundary-check rubric inject).
# Pattern cứng — orchestrator KHÔNG dựa LLM remember triệu giám sát.
# Tarot precedent: P297 (2026-05-25 — orchestrator MISS triệu giám sát cho 3 security PR cascade).
#
# Override marker: command chứa `[security-review-skip:<reason>]` → allow với log warning.
#   Use case: doctrine/docs-only PR mà pattern match false-positive, Sếp đã review tay.
#
# Known limitation: chỉ catch numbered form `gh pr merge <N>`.
# Branch-only form `gh pr merge --merge` (no number) BYPASS hook.

set -euo pipefail

# cwd-independent (see architect-guard.sh): bind to repo root so git ops resolve the project.
cd "${CLAUDE_PROJECT_DIR:-$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)}" || exit 0

# Đọc input
if [ ! -t 0 ]; then
  INPUT=$(cat || echo "")
else
  INPUT="${CLAUDE_TOOL_INPUT:-}"
fi

# Không có input → pass through
if [ -z "$INPUT" ]; then exit 0; fi

# Parse command từ JSON
COMMAND=$(echo "$INPUT" | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    print(data.get('tool_input', {}).get('command', ''))
except Exception:
    print('')
" 2>/dev/null || echo "")

# Không có command → tool khác → pass
if [ -z "$COMMAND" ]; then exit 0; fi

# Match `gh pr merge <N>` (allow flag variants: --squash, --merge, --delete-branch, etc.)
if ! echo "$COMMAND" | grep -qE 'gh pr merge[[:space:]]+[0-9]+'; then
  exit 0
fi

# Extract PR number (first numeric after `gh pr merge`)
PR=$(echo "$COMMAND" | sed -nE 's/.*gh pr merge[[:space:]]+([0-9]+).*/\1/p' | head -1)
if [ -z "$PR" ]; then exit 0; fi

# Override marker check
if echo "$COMMAND" | grep -qE '\[security-review-skip:[^]]+\]'; then
  REASON=$(echo "$COMMAND" | sed -nE 's/.*\[security-review-skip:([^]]+)\].*/\1/p')
  echo "⚠️  Security review override marker detected for PR #$PR. Reason: $REASON" >&2
  echo "    Allowing merge. Sếp đã review tay — em (hook) không block." >&2
  exit 0
fi

# Check security surface — generic pattern (extend per-repo bằng SECURITY_SURFACE_EXTRA env var)
# Generic surface: src/, schema/migration files, infra config, env files, auth/middleware,
#                  security agents, security docs, security scripts, pre-commit hook itself.
SECURITY_SURFACE_PATTERN='src/|schema\.(prisma|sql)|migrations?/|nginx/|docker-compose.*\.yml|Dockerfile|\.env[^.]|middleware\.|lib/auth/|\.claude/agents/security-|docs/security/|scripts/security-gate|scripts/check-(hardcoded|runtime)-secrets|hooks/pre-commit'

# Extend pattern per-repo (optional)
if [ -n "${SECURITY_SURFACE_EXTRA:-}" ]; then
  SECURITY_SURFACE_PATTERN="${SECURITY_SURFACE_PATTERN}|${SECURITY_SURFACE_EXTRA}"
fi

DIFF_FILES=$(gh pr diff "$PR" --name-only 2>/dev/null || echo "")
if [ -z "$DIFF_FILES" ]; then
  # gh CLI fail (network/auth) → fail-safe: BLOCK với fallback message (KHÔNG silent allow)
  cat >&2 <<EOF
⛔ BLOCKED: gh pr diff #$PR thất bại (network/auth?).

Em (hook) KHÔNG verify được PR có touch security surface không.
Fail-safe: block merge để Sếp/Quản đốc kiểm tra tay.

Cách hợp lệ:
  - Kiểm tra gh auth status
  - Chạy: gh pr diff $PR --name-only
  - Nếu confirm KHÔNG touch security surface → re-run merge với marker:
      gh pr merge $PR --merge [security-review-skip:gh-cli-unavailable]
EOF
  exit 2
fi

# Check pattern match
if ! echo "$DIFF_FILES" | grep -qE "$SECURITY_SURFACE_PATTERN"; then
  # Also check .env.example skip
  NON_EXAMPLE=$(echo "$DIFF_FILES" | grep -E "^\.env" | grep -v '\.env\.example' || true)
  if [ -z "$NON_EXAMPLE" ]; then
    # PR không touch security surface → allow merge
    exit 0
  fi
fi

# PR touch security surface — check security-review comment APPROVE chưa
COMMENTS=$(gh pr view "$PR" --json comments --jq '.comments[].body' 2>/dev/null || echo "")
if echo "$COMMENTS" | grep -q '<!-- security-review-start -->'; then
  # Có review block. Check verdict.
  VERDICT_LINE=$(echo "$COMMENTS" | grep -A 50 '<!-- security-review-start -->' | grep -E '^Verdict:' | head -1)
  if echo "$VERDICT_LINE" | grep -q 'APPROVE'; then
    # APPROVE → allow
    exit 0
  fi
  # NEEDS_REVIEW or unknown → block
  cat >&2 <<EOF
⛔ BLOCKED: PR #$PR touch security surface VÀ /security-review verdict KHÔNG phải APPROVE.

Verdict line: $VERDICT_LINE

Hành động:
  1. Sếp đọc comment giám sát trên PR #$PR
  2. Nếu Sếp accept risk → re-run với marker:
     gh pr merge $PR --merge [security-review-skip:sep-accepted-needs-review]
  3. Nếu cần fix → spawn Worker EXECUTE fix theo INV flagged, push, gate sẽ re-fire
EOF
  exit 2
fi

# Touch security surface NHƯNG chưa có review → BLOCK
cat >&2 <<EOF
⛔ BLOCKED: PR #$PR touch security surface NHƯNG chưa có /security-review.

Em (Quản đốc) suýt MISS triệu giám sát. Hook chặn để fix structural — KHÔNG dựa LLM remember.

Hành động:
  1. Chạy slash command (em tự gõ):
     /security-review $PR
  2. Đợi @agent-boundary-check verdict (advisory, post comment trên PR)
  3. Verdict APPROVE → re-run merge bình thường (hook sẽ allow)
  4. Verdict NEEDS_REVIEW → Sếp đọc comment + quyết (re-run với marker nếu accept)

Reference:
  - PR: \$(gh pr view $PR --json url --jq .url)
  - Doctrine: WORKFLOW_V2.2.md §7 Sub-mech A (trigger gap) + §8 (rubric inject)
  - Slash: .claude/commands/security-review.md
EOF
exit 2
