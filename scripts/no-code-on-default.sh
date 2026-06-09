#!/usr/bin/env bash
# no-code-on-default.sh — pre-commit gate: BLOCK committing PRODUCT CODE on the default branch.
# Forces a feature branch for code; docs-only (*.md) commits on default stay allowed.
# Agent-agnostic (git-level) — survives non-Claude agents (P049–P052 harvest thread).
# Override: touch .sos-state/allow-code-on-default  (warn + allow; style mirrors .sos-state/worker-active).
# Doctrine: gate the INVARIANT (no code on default), not the PROCEDURE (when to branch).
#           enforce_via_mechanism_not_memory. ket P020 live failure.
set -uo pipefail

cd "${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null)}" 2>/dev/null || exit 0

# --- (V2, [O1.1]) MERGE COMMIT ESCAPE — MUST be first, before any branch logic. ---
#     A non-fast-forward `git merge` creates a commit → pre-commit fires. PR-merge of a
#     feature branch into main is the INTENDED path code enters main; it must NOT be blocked.
#     MERGE_HEAD exists ONLY during an in-progress merge (absent for a plain `git commit`).
#     `$(git rev-parse --git-dir)` (not hardcoded .git/) — works inside worktrees + submodules.
[ -f "$(git rev-parse --git-dir)/MERGE_HEAD" ] && exit 0

# --- (e) sos-kit self opt-out: this repo commits maintenance to main directly. ---
#     The self-marker is committed (NOT gitignored via negation) so it travels with the kit.
#     Downstream repos do NOT get this marker → gate is live for them.
[ -f ".sos-state/sos-kit-self" ] && exit 0

# --- Resolve current branch (c: detached-HEAD fail-safe) ---
CURRENT=$(git branch --show-current 2>/dev/null || echo "")
if [ -z "$CURRENT" ]; then
    # Detached HEAD (rebase / bisect / CI checkout) — "on default branch" is undefined.
    # Fail-safe = WARN + ALLOW (blocking would break rebase/bisect mid-flight).
    echo "WARNING: no-code-on-default: detached HEAD — cannot determine branch, allowing." >&2
    exit 0
fi

# --- Resolve default branch (c: unset origin/HEAD fallback) ---
DEFAULT=$(git symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null | sed 's#^origin/##')
if [ -z "$DEFAULT" ]; then
    # origin/HEAD never set (no `remote set-head`, or no remote). Fall back to the
    # first of main/master that EXISTS as a local branch — do NOT silently assume "main"
    # and do NOT silently pass.
    if   git show-ref --verify --quiet refs/heads/main;   then DEFAULT="main"
    elif git show-ref --verify --quiet refs/heads/master; then DEFAULT="master"
    else
        echo "WARNING: no-code-on-default: cannot resolve default branch (origin/HEAD unset, no main/master) — allowing." >&2
        exit 0
    fi
fi

# Not on default → nothing to gate.
[ "$CURRENT" != "$DEFAULT" ] && exit 0

# --- Override marker (d) ---
if [ -f ".sos-state/allow-code-on-default" ]; then
    echo "WARNING: no-code-on-default: override marker .sos-state/allow-code-on-default present — allowing code on $DEFAULT." >&2
    exit 0
fi

# --- Build product-code pattern from .sos-stack.toml (b) ---
#   Schema has NO code-dir field (Task 0 #5) → derive from `type`. Extension-based approach
#   mirrors orchestrator-guard.sh:78 + hooks/pre-commit:168.
#   (V2, [O1.2]) If .sos-stack.toml is ABSENT → CODE_PATTERN = full extension-union + BLOCK
#               (NOT warn+allow). Greenfield is the harvest target (ket P020). See Task 2.

# Extension-union constant: union of ALL type→ext mappings — used by absent-stack fallback too.
EXT_UNION='\.(rs|ts|tsx|js|jsx|py|go|swift|pbxproj)$'

if [ ! -f ".sos-stack.toml" ]; then
    # (V2, [O1.2]) ABSENT stack file → full extension-union + BLOCK (not warn+allow).
    # Greenfield / early commits on main are the primary harvest target.
    # Clean escape: touch .sos-state/allow-code-on-default (NOT --no-verify).
    CODE_PATTERN="${EXT_UNION}|(^|/)src/"
else
    # Derive CODE_PATTERN from type value(s) in .sos-stack.toml.
    TYPES=$(grep -E '^[[:space:]]*type[[:space:]]*=' .sos-stack.toml 2>/dev/null | sed -E 's/.*=[[:space:]]*"([^"]+)".*/\1/')

    FRAGMENTS=""
    UNKNOWN_TYPES=""
    while IFS= read -r TTYPE; do
        case "$TTYPE" in
            node)   FRAGMENTS="${FRAGMENTS}\\.(ts|tsx|js|jsx)\$|" ;;
            python) FRAGMENTS="${FRAGMENTS}\\.py\$|" ;;
            rust)   FRAGMENTS="${FRAGMENTS}\\.rs\$|" ;;
            go)     FRAGMENTS="${FRAGMENTS}\\.go\$|" ;;
            swift)  FRAGMENTS="${FRAGMENTS}\\.(swift|pbxproj)\$|" ;;
            *)      UNKNOWN_TYPES="${UNKNOWN_TYPES}${TTYPE} " ;;
        esac
    done <<< "$TYPES"

    # Always add a generic src/ arm (mirrors orchestrator-guard.sh:78 `*/src/*`).
    FRAGMENTS="${FRAGMENTS}(^|/)src/"

    if [ -n "$UNKNOWN_TYPES" ]; then
        echo "WARNING: no-code-on-default: unmapped .sos-stack.toml type(s): ${UNKNOWN_TYPES}— falling back to src/ arm only for those." >&2
    fi

    CODE_PATTERN="${FRAGMENTS%|}"  # strip trailing | if any
fi

# --- (a) ORDER IS LOAD-BEARING: filter .md FIRST, THEN grep the FILTERED stream. ---
#   This prevents src/components/README.md from matching the src/ pattern.
#   The ket edge-hole: grepping $STAGED re-introduces .md; grep $FILTERED avoids it.
STAGED=$(git diff --cached --name-only --diff-filter=ACM)
FILTERED=$(echo "$STAGED" | grep -vE '\.md$' || true)     # drop docs first
CODE_HITS=$(echo "$FILTERED" | grep -E "$CODE_PATTERN" || true)  # grep the FILTERED var, NOT $STAGED

if [ -n "$CODE_HITS" ]; then
    cat >&2 <<EOF
BLOCKED: no-code-on-default: product code staged on default branch ($DEFAULT).

Offending files:
$CODE_HITS

Cut a feature branch first:  git switch -c feat/<slug>
(Docs-only *.md commits on $DEFAULT are allowed.)
Override (kit-maintenance / intentional): touch .sos-state/allow-code-on-default
EOF
    exit 1
fi
exit 0
