#!/usr/bin/env bash
# Pre-push hook: warn nếu push touch security surface mà chưa /security-review.
# Advisory only — KHÔNG block, remind only.
# Generic version (extend SECURITY_PATHS via env var per-repo).
#
# Doctrine: WORKFLOW_V2.2.md §7 Sub-mech A — pre-push remind layer (catch pre-PR push).

# Generic security paths (extend với SECURITY_PATHS_EXTRA env var per-repo)
SECURITY_PATHS='src/|schema\.(prisma|sql)|migrations?/|nginx/|docker-compose.*\.yml|Dockerfile|middleware\.|lib/auth/|docs/security/|scripts/security-gate'

if [[ -n "${SECURITY_PATHS_EXTRA:-}" ]]; then
  SECURITY_PATHS="${SECURITY_PATHS}|${SECURITY_PATHS_EXTRA}"
fi

# Diff staged commits vs origin/main
CHANGED=$(git diff --name-only origin/main...HEAD 2>/dev/null || git diff --name-only HEAD~1...HEAD 2>/dev/null || echo "")

if [[ -z "$CHANGED" ]]; then
  exit 0
fi

TOUCHED_SECURITY=0

# Check security paths (excluding .env separately)
if echo "$CHANGED" | grep -qE "$SECURITY_PATHS"; then
  TOUCHED_SECURITY=1
fi

# Check .env files (allow .env.example)
ENV_TOUCHED=$(echo "$CHANGED" | grep -E "^\.env" | grep -v '\.env\.example' || true)
if [[ -n "$ENV_TOUCHED" ]]; then
  TOUCHED_SECURITY=1
fi

if [[ "$TOUCHED_SECURITY" -eq 1 ]]; then
  echo ""
  echo "  ⚠️  Push touches security surface:"
  echo "$CHANGED" | grep -E "$SECURITY_PATHS|^\.env" | grep -v '\.env\.example' | sed 's/^/    - /'
  echo ""
  echo "  Recommend: create PR first, then run /security-review <PR-number>"
  echo "  to trigger advisory judgment invariant check."
  echo ""

  # Only prompt in interactive terminal (skip in CI/CD)
  if [[ -t 1 ]]; then
    printf "  Continue push? [y/N] "
    read -r REPLY < /dev/tty
    if [[ ! "$REPLY" =~ ^[Yy]$ ]]; then
      echo "  Push aborted by user."
      exit 1
    fi
  fi
fi

exit 0
