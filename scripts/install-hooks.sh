#!/usr/bin/env bash
# Activate git hooks by pointing core.hooksPath at the tracked hooks/ dir.
#
# Why core.hooksPath (NOT cp into .git/hooks/): the tracked hook IS the running
# hook — an edit to hooks/pre-commit is live immediately, no stale untracked copy
# drifting from source. Under the old copy method a tracked-hook fix sat dead in
# hooks/ while .git/hooks/ ran the old copy (Két dogfood 2026-06-03). core.hooksPath
# is local git state (not in the diff) → a fresh clone re-runs this script. Idempotent.
#
# IMPORTANT: core.hooksPath redirects ALL hook lookups to hooks/, so EVERY hook must
# live there under its canonical git name — hooks/pre-commit, hooks/pre-push. (pre-push
# was relocated scripts/pre-push-hook.sh → hooks/pre-push for this; a naive switch
# without that move would silently kill pre-push.) Hooks must be tracked +x (100755) —
# git skips non-executable hooks silently.
#
# Doctrine: WORKFLOW_V2.2.md §7 (hook ship pattern). Két dogfood harvest 2026-06-03.

set -e

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

if [[ ! -f hooks/pre-commit ]]; then
    echo "ERROR: hooks/pre-commit not found (run from a sos-kit-spawned repo)" >&2
    exit 1
fi

# Ensure executable (working tree). Tracked mode is fixed once via git update-index
# --chmod=+x in the repo that ships hooks/; a clone inherits 100755.
chmod +x hooks/pre-commit 2>/dev/null || true
[[ -f hooks/pre-push ]] && chmod +x hooks/pre-push 2>/dev/null || true

git config core.hooksPath hooks
echo "✓ core.hooksPath → hooks/ (tracked hooks now live: $(ls hooks/ 2>/dev/null | tr '\n' ' '))"

# Retire any stale copies the old copy-method left in .git/hooks/ — core.hooksPath
# overrides them, but a leftover .git/hooks/pre-commit reads as 'installed' and
# confuses. Rename (don't delete) — keep an escape hatch.
for h in pre-commit pre-push; do
    if [[ -f ".git/hooks/$h" ]]; then
        mv ".git/hooks/$h" ".git/hooks/$h.pre-hookspath.bak" 2>/dev/null || true
    fi
done
