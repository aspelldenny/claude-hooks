#!/usr/bin/env bash
# PreToolUse hook — block Edit/Write tới .env* files (except .env.example).
# Đầu vào: Claude Code hook spec gửi JSON qua stdin với { "tool_input": { "file_path": "..." } }.
# Fallback: $CLAUDE_TOOL_INPUT env var nếu stdin trống.
# Exit 2 → block tool call. Exit 0 → allow.
#
# Doctrine: WORKFLOW_V2.2.md §7 Sub-mech F (runtime state — secret leak guard).
# Tarot precedent: P230 (2026-05-20).

set -euo pipefail

# cwd-independent (see architect-guard.sh): bind to repo root regardless of caller cwd.
cd "${CLAUDE_PROJECT_DIR:-$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)}" || exit 0

# Đọc input
if [ ! -t 0 ]; then
  INPUT=$(cat || echo "")
else
  INPUT="${CLAUDE_TOOL_INPUT:-}"
fi

# Không có input → pass through (hook không có context để check)
if [ -z "$INPUT" ]; then exit 0; fi

# Parse file_path từ JSON
FILE_PATH=$(echo "$INPUT" | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    print(data.get('tool_input', {}).get('file_path', ''))
except Exception:
    print('')
" 2>/dev/null || echo "")

# Không có file_path → tool khác (không phải Edit/Write file) → pass
if [ -z "$FILE_PATH" ]; then exit 0; fi

# Basename để check pattern
BASE=$(basename "$FILE_PATH")

# Allowlist: .env.example là template, được phép edit
if [ "$BASE" = ".env.example" ]; then exit 0; fi

# Block .env và .env.* (production, local, staging, etc.)
if echo "$BASE" | grep -qE '^\.env($|\.)'; then
  cat >&2 <<EOF
⛔ BLOCKED: Edit/Write tới $FILE_PATH bị chặn.

Lý do: .env* file chứa secret thật (API keys, DB credentials, webhook tokens).
KHÔNG sửa qua Claude tool — risk leak vào prompt/context/log.

Cách hợp lệ:
  - Sửa .env.example (template, không secret thật)
  - Sếp paste secret thật vào .env tay (local-only edit)
  - Sửa qua SSH/console nếu là production env

Override (nếu thật sự cần Claude edit .env, hiếm):
  - Tạm rename .env → .env.draft, edit, rename back
  - Hoặc remove hook khỏi .claude/settings.json (PR review trước)
EOF
  exit 2
fi

# Mọi file khác → allow
exit 0
