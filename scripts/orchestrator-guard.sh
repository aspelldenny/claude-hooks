#!/usr/bin/env bash
# orchestrator-guard.sh — PreToolUse hook: chặn cứng Quản đốc (main session) TỰ CODE product.
#
# How it works:
#   - Hook fires on every Edit / Write tool call.
#   - Reads JSON from stdin (Claude Code hook payload), extracts file_path.
#   - product-source (*.swift / *.pbxproj / src/**) chỉ được sửa khi marker
#     `.sos-state/worker-active` TỒN TẠI (= đang trong cửa sổ Thợ EXECUTE).
#   - Không marker → main session (Quản đốc) hoặc Architect đang định tự code → exit 2 (block).
#
# Companion ngược của architect-guard.sh:
#   architect-guard chặn Architect ĐỌC source (Read|Glob, khi architect-active).
#   orchestrator-guard chặn Quản đốc GHI product code (Edit|Write, khi KHÔNG worker-active).
# Subagent tool-calls CÓ fire PreToolUse (bằng chứng: architect-guard chặn được Architect
# subagent) → marker là cách DUY NHẤT phân biệt Thợ-được-phép vs main-session-việt-vị.
#
# Scope HẸP có chủ đích: chỉ product source (`*.swift`/`*.pbxproj`/`src/**`). KHÔNG gồm
# `*.py`/`*.sh`/`*.md`/docs — để kit-maintenance (Quản đốc sửa thẳng bin/sos.sh, scripts/*.sh,
# docs ở Tầng-2 surgical) KHÔNG bị chặn. Trong sos-kit chính nó: không *.swift/pbxproj, không
# top-level src/, và `bootstrap/` (CLI sos-rs, 12 file .rs thật) được allow-list riêng bên dưới
# → guard near-no-op trên kit nhưng vẫn dogfood + nhất quán.
#
# Setup: referenced từ .claude/settings.json hooks.PreToolUse (matcher Edit|Write).
# Quản đốc PHẢI `touch .sos-state/worker-active` TRƯỚC spawn Thợ, `rm -f` sau khi Thợ về
# (agents/orchestrator.md "Marker file hygiene").
#
# Known residual (PR #21 review): fires on Edit/Write/MultiEdit/NotebookEdit, NOT Bash —
# a deliberate `Bash("echo > src/x.swift")` redirect bypasses this. Out of scope by design
# (closes the Edit/Write incident vector; parsing arbitrary Bash redirects is fragile). This
# is a discipline guard, not a sandbox.
#
# No external deps (no jq) — pure shell + sed for cross-platform (Windows msys2 bash).

set -euo pipefail

# cwd-independent (xem architect-guard.sh): bind to repo root regardless of caller cwd.
cd "${CLAUDE_PROJECT_DIR:-$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)}" || exit 0

# Read tool input JSON from stdin
INPUT_JSON=$(cat)

# Extract path. Edit/Write/MultiEdit use tool_input.file_path; NotebookEdit uses
# notebook_path (matcher includes NotebookEdit — without this fallback its payload has
# no file_path → extract-blind → always allowed, silently un-guarded).
PATH_ARG=$(echo "$INPUT_JSON" | sed -n 's/.*"file_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
[ -z "$PATH_ARG" ] && PATH_ARG=$(echo "$INPUT_JSON" | sed -n 's/.*"notebook_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')

# Unparseable path → allow (don't block on weird input)
[ -z "$PATH_ARG" ] && exit 0

NORMALIZED_PATH="${PATH_ARG#./}"
# Claude Code delivers an ABSOLUTE file_path. The anchored globs below (bootstrap/*,
# src/*) can never match an absolute path → the bootstrap allow-list + top-level src/
# rule misfire silently (PR #21 fixed the relative case; runtime is absolute → the kit's
# own bootstrap/ Rust CLI was being BLOCKED, "near-no-op" was false). Strip the repo-root
# prefix ($PWD, set by the cd above) so anchored globs match the in-repo path.
# Residual: if CLAUDE_PROJECT_DIR and the path differ by symlink resolution, the strip
# no-ops → matching falls back to absolute (extension + */src/* still catch product code).
NORMALIZED_PATH="${NORMALIZED_PATH#"$PWD"/}"

# Docs are NEVER product source — *.md editable anywhere (mirror architect-guard.sh:50-52),
# even under a `src/` dir (mdBook docs/src/SUMMARY.md, crates/*/src/README.md, …).
case "$NORMALIZED_PATH" in
    *.md) exit 0 ;;
esac

# The kit's OWN bundled tooling (bootstrap/sos-rs/src/*.rs = the sos-rs CLI, 10 real files)
# is kit-maintenance, NOT the product the kit ships → allow. Without this, editing the kit's
# own Rust CLI from the main session would be blocked (and "near-no-op on sos-kit" is false).
case "$NORMALIZED_PATH" in
    bootstrap/*) exit 0 ;;
esac

# Is this PRODUCT source? (narrow — see header). Anything else (docs/*.py/*.sh/config) → allow.
# NOTE: shell `case` glob `*/src/*` matches ANY interior `src/` (the `*` spans `/`) — intended:
# product source lives under src/ at any depth. *.md + bootstrap/ are already excluded above.
case "$NORMALIZED_PATH" in
    *.swift|*.pbxproj|src/*|*/src/*) ;;   # product source → gated below
    *) exit 0 ;;                          # not product source → always allow
esac

# Product source: allowed ONLY while a Worker is active (worker-active marker present).
[ -f ".sos-state/worker-active" ] && exit 0

# No worker-active → main session / Architect trying to hand-code product → BLOCK.
cat >&2 <<EOF
🚫 Orchestrator envelope violation

Quản đốc (main session) không được tự sửa product code: $PATH_ARG

Đúng quy trình: spawn Thợ (Worker) để code. Trước khi spawn:
  mkdir -p .sos-state && touch .sos-state/worker-active
Sau khi Thợ về:
  rm -f .sos-state/worker-active

Nếu đây ĐÚNG là Thợ đang EXECUTE mà bị chặn → Quản đốc quên touch marker trước spawn.
(Edit này KHÔNG phải product source? Báo — scope có thể cần chỉnh trong orchestrator-guard.sh.)
EOF
exit 2
