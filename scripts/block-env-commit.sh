#!/usr/bin/env bash
# block-env-commit.sh — pre-commit gate: BLOCK committing any .env* secret file.
# Allows .env.example (the template). Agent-agnostic (git-level) — survives
# non-Claude agents, complements the Claude-only PreToolUse block-env-edit.sh (P046).
# Grounding: media audit SEC-SECRET-01 = .env.docker committed to git history (irreversible leak).
# Override: touch .sos-state/allow-env-commit  (warn + allow; high bar — leak is irreversible).
# NOTE: .envrc (direnv) is DELIBERATELY not covered — it is usually committed on purpose
#       (points at .env to load secrets). Regex stays verbatim with block-env-edit.sh (P046)
#       so the two layers share one env-file definition. See phiếu P052 Debate Log [O1.1].
set -uo pipefail

cd "${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null)}" 2>/dev/null || exit 0

# --- MERGE COMMIT ESCAPE — MUST be first (same semantics as no-code-on-default.sh, [verified] P050 #11). ---
#     A non-FF `git merge` creates a commit → pre-commit fires. PR-merge into main is the intended path.
#     MERGE_HEAD exists ONLY during an in-progress merge (absent for a plain `git commit`).
#     `$(git rev-parse --git-dir)` (not hardcoded .git/) — works inside worktrees + submodules.
[ -f "$(git rev-parse --git-dir)/MERGE_HEAD" ] && exit 0

# --- Override marker (file marker, mirrors .sos-state/allow-code-on-default) ---
if [ -f ".sos-state/allow-env-commit" ]; then
    echo "WARNING: block-env-commit: override marker .sos-state/allow-env-commit present — allowing .env* commit. (Secret leaks are IRREVERSIBLE — be sure.)" >&2
    exit 0
fi

# --- Inspect staged files; match on BASENAME (a .env may live at config/.env.docker) ---
STAGED=$(git diff --cached --name-only --diff-filter=ACM)
OFFENDERS=""
for f in $STAGED; do
    base=$(basename "$f")
    [ "$base" = ".env.example" ] && continue          # allowlist the template
    if echo "$base" | grep -qE '^\.env($|\.)'; then   # reuse block-env-edit.sh regex VERBATIM
        OFFENDERS="${OFFENDERS}${f}"$'\n'
    fi
done

if [ -n "$OFFENDERS" ]; then
    cat >&2 <<'BLOCK_MSG'
BLOCKED: block-env-commit: a .env* secret file is staged — committing it leaks secrets into git history (IRREVERSIBLE).

Offending files:
BLOCK_MSG
    printf "%s" "$OFFENDERS" >&2
    cat >&2 <<'FIX_MSG'

Fix:
  git restore --staged <file>        # unstage it
  echo '<file>' >> .gitignore        # keep it out of git
  Use .env.example (no real values) as the committed template instead.
Override (RARE, intentional, you accept the leak): touch .sos-state/allow-env-commit
FIX_MSG
    exit 1
fi
exit 0
