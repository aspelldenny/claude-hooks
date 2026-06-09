#!/usr/bin/env bash
# architect-guard.sh — PreToolUse hook để chặn cứng Architect đọc source code
#
# How it works:
#   - Hook fires on every Read / Glob tool call
#   - Reads JSON from stdin (Claude Code hook payload)
#   - Detects if the call is from the Architect agent (via marker file)
#   - If Architect is trying to read src/, exits 2 with an error
#
# Setup: this script is referenced from .claude/settings.json under hooks.PreToolUse.
# Architect agent must create marker file `.sos-state/architect-active` on spawn.
# (Marker lives outside .claude/ so YOLO mode doesn't prompt — .claude/ is gated
# even with --dangerously-skip-permissions because it holds settings/hooks.)
#
# Note: NO external deps (no jq) — uses pure shell + sed/grep for cross-platform
# compatibility (esp. Windows msys2 bash where jq is not bundled).

set -euo pipefail

# cwd-independent: Claude Code may fire this hook from any cwd (subdir/home). Resolve repo
# root via $CLAUDE_PROJECT_DIR (Claude Code-provided), else from this script's own location,
# so internal relative refs (.sos-state/, docs/) bind to the project — not the caller's cwd.
cd "${CLAUDE_PROJECT_DIR:-$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)}" || exit 0

MARKER_FILE=".sos-state/architect-active"

# If no marker → not running as Architect → allow everything
[ -f "$MARKER_FILE" ] || exit 0

# Read tool input JSON from stdin
INPUT_JSON=$(cat)

# Extract path argument from JSON (works for Read.file_path and Glob.pattern)
# Strategy: greedy regex on flat JSON. Works for both:
#   {"tool_input":{"file_path":"src/main.rs"}}
#   {"tool_input":{"pattern":"src/**/*.rs"}}
# This is fragile for nested quotes but Claude Code paths don't contain them.
PATH_ARG=$(echo "$INPUT_JSON" | sed -n 's/.*"file_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
if [ -z "$PATH_ARG" ]; then
    PATH_ARG=$(echo "$INPUT_JSON" | sed -n 's/.*"pattern"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
fi

# If we couldn't parse a path, allow (don't block on unparseable input)
[ -z "$PATH_ARG" ] && exit 0

# Strip leading ./ for matching
NORMALIZED_PATH="${PATH_ARG#./}"

# Allow .md files anywhere — docs are Architect's domain even if they live alongside code
case "$NORMALIZED_PATH" in
    *.md) exit 0 ;;
esac

# Forbidden path patterns (source code regions / build artifacts)
# Use shell glob-style case patterns (POSIX, no regex engine needed)
case "$NORMALIZED_PATH" in
    src/*|*/src/*|lib/*|*/lib/*|app/*|*/app/*|crates/*/src/*|pkg/*|*/pkg/*)
        BLOCKED=1 ;;
    tests/*|*/tests/*|test/*|*/test/*|__tests__/*)
        BLOCKED=1 ;;
    node_modules/*|target/*|dist/*|build/*|.next/*|.nuxt/*|.svelte-kit/*)
        BLOCKED=1 ;;
    *.rs|*.ts|*.tsx|*.js|*.jsx|*.py|*.go|*.java|*.cpp|*.c|*.h|*.hpp)
        BLOCKED=1 ;;
    *)
        BLOCKED=0 ;;
esac

if [ "${BLOCKED:-0}" = "1" ]; then
    cat >&2 <<EOF
🚫 Architect envelope violation

Architect cannot read source code: $PATH_ARG

What to do instead: write a Task 0 anchor in the phiếu.
Example:
  | # | Assumption | Verify by | Result |
  | 1 | <claim about $PATH_ARG> | grep ... $PATH_ARG | ⏳ TO VERIFY |

Worker (separate subagent) will grep-verify it for you. The constraint IS the feature.
EOF
    exit 2
fi

# Path is allowed
exit 0
