#!/usr/bin/env bash
# session-start-banner.sh — SessionStart hook for sos-kit projects
#
# Behavior:
#   - When Claude Code starts in a sos-kit project, show docs/BACKLOG.md "Active sprint" section
#   - Reminds Sếp what's in flight, pre-empts "I forgot what I was doing"
#   - Architect Rule 0 reminder: phiếu only for items in Active sprint
#
# Configured in .claude/settings.json under hooks.SessionStart
#
# No external deps (no jq) — pure shell + sed/grep/awk for cross-platform.

set -uo pipefail
# Note: NOT using `set -e` because grep -c with 0 matches exits 1, which we treat as normal.

# cwd-independent (see architect-guard.sh): bind to repo root so docs/ refs resolve at session start.
cd "${CLAUDE_PROJECT_DIR:-$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)}" || exit 0

BACKLOG="docs/BACKLOG.md"

# Silent if no BACKLOG.md (project not sos-kit-equipped, OK)
[ ! -f "$BACKLOG" ] && exit 0

# Find the Active sprint header line number — strict match first
HEADER_LINE=$(grep -n "^## .*Active sprint" "$BACKLOG" 2>/dev/null | head -1 | cut -d: -f1)
FALLBACK_USED=0

# Fallback: first ^## section in the file (treats the top H2 as the active section)
if [ -z "$HEADER_LINE" ]; then
    HEADER_LINE=$(grep -n "^## " "$BACKLOG" 2>/dev/null | head -1 | cut -d: -f1)
    FALLBACK_USED=1
fi

# Silent if no ^## headings at all (malformed / empty BACKLOG)
[ -z "$HEADER_LINE" ] && exit 0

# Capture the actual header text for the fallback note (strip leading "## ")
HEADER_TEXT=$(sed -n "${HEADER_LINE}p" "$BACKLOG" | sed 's/^## *//')

# Find the next "^## " section after Active sprint (to know where to stop)
NEXT_SECTION_LINE=$(awk -v start="$HEADER_LINE" 'NR > start && /^## / {print NR; exit}' "$BACKLOG")

# Default end = end of file if no next section
if [ -z "$NEXT_SECTION_LINE" ]; then
    END_LINE=$(wc -l < "$BACKLOG")
else
    END_LINE=$((NEXT_SECTION_LINE - 1))
fi

# Extract Active sprint block
SPRINT_BLOCK=$(sed -n "${HEADER_LINE},${END_LINE}p" "$BACKLOG")

# Count open items ([ ]) and done items ([x]) in this sprint
# grep -c with 0 matches exits 1 but still prints "0" — capture that, ignore exit.
OPEN_COUNT=$(echo "$SPRINT_BLOCK" | grep -c "^- \[ \]" 2>/dev/null) || OPEN_COUNT=0
DONE_COUNT=$(echo "$SPRINT_BLOCK" | grep -c "^- \[x\]" 2>/dev/null) || DONE_COUNT=0

# Banner output
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🏠 Sếp's project — Active sprint status"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "$SPRINT_BLOCK" | head -25
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Active sprint: $OPEN_COUNT items đang treo, $DONE_COUNT đã xong"
if [ "$FALLBACK_USED" = "1" ]; then
    echo ""
    echo "📌 Treating \"$HEADER_TEXT\" as Active sprint (no \"Active sprint\" header found)."
fi

# ─────────────────────────────────────────────────────────────────────
# Doc size warn (P038) — non-blocking nudge, threshold 40k bytes
# ─────────────────────────────────────────────────────────────────────
SIZE_THRESHOLD=40960  # 40k bytes
SIZE_WARNS=""
for doc in "docs/CHANGELOG.md" "docs/DISCOVERIES.md" "CHANGELOG.md"; do
    [ ! -f "$doc" ] && continue
    # Cross-platform byte count: prefer wc -c (POSIX), fall back to stat
    bytes=$(wc -c < "$doc" 2>/dev/null | tr -d ' ')
    [ -z "$bytes" ] && continue
    if [ "$bytes" -gt "$SIZE_THRESHOLD" ]; then
        kb=$((bytes / 1024))
        SIZE_WARNS="${SIZE_WARNS}⚠️  ${doc} (${kb}k > 40k threshold) — gọi thợ trim, archive cũ ra docs/archive/\n"
    fi
done
if [ -n "$SIZE_WARNS" ]; then
    echo ""
    echo "📏 Doc size warning:"
    printf "    %b" "$SIZE_WARNS"
fi

# ─────────────────────────────────────────────────────────────────────
# Phiếu cleanup nudge (P038) — scan active phiếu for approved+merged
# ─────────────────────────────────────────────────────────────────────
NUDGES=""
# Detect phiếu directory: prefer docs/ticket/ (unified), fall back to phieu/active/ (legacy repos)
PHIEU_DIR=""
if [ -d "docs/ticket" ]; then
    PHIEU_DIR="docs/ticket"
elif [ -d "phieu/active" ]; then
    PHIEU_DIR="phieu/active"
fi

if [ -n "$PHIEU_DIR" ]; then
    # Get list of branches merged into main (no gh CLI required)
    MERGED_BRANCHES=$(git branch --merged main 2>/dev/null | sed 's/^[* ] //' | tr -d ' ')

    for phieu_file in "$PHIEU_DIR"/P*.md; do
        [ ! -f "$phieu_file" ] && continue
        # Skip TICKET_TEMPLATE.md if it lives here
        case "$(basename "$phieu_file")" in TICKET_TEMPLATE.md|TEMPLATE.md) continue;; esac

        # Check if phiếu has "Approved by Chủ nhà:" line with non-empty value
        approved_line=$(grep "Approved by Chủ nhà:" "$phieu_file" 2>/dev/null | head -1)
        # Skip if no approval line OR approval value is placeholder (e.g., "[date]" or empty)
        case "$approved_line" in
            *"[date]"*|*"Approved by Chủ nhà: $"*|"") continue;;
        esac

        # Extract phiếu ID
        phieu_id=$(basename "$phieu_file" .md | grep -oE '^P[0-9]+')
        [ -z "$phieu_id" ] && continue

        # Check if a branch matching this phiếu is merged
        if echo "$MERGED_BRANCHES" | grep -qE "/${phieu_id}-"; then
            slug=$(basename "$phieu_file" .md)
            NUDGES="${NUDGES}🧹 Phiếu ${phieu_id} approved + merged. Run: phieu-done ${slug}\n"
        fi
    done
fi

if [ -n "$NUDGES" ]; then
    echo ""
    echo "🧹 Cleanup nudge:"
    printf "    %b" "$NUDGES"
fi

# ─────────────────────────────────────────────────────────────────────
# Advisory-watch staleness (gap-audit sync 2026-05-29; from tarot P281/P301)
# Reads docs/security/.advisory-scan-state (JSON {"last_scan_at":"<ISO-8601 UTC>"} or legacy raw ISO).
#   >=7 days OR missing → 🚨 orchestrator MUST auto-spawn advisory-watch (ORCHESTRATION Rule 11)
#   3-6 days → ⚠️ narrate + offer · 0-2 days → silent. Cross-platform date (GNU + BSD).
# Only nags if the advisory pipeline is installed (inbox present) — else silent.
# ─────────────────────────────────────────────────────────────────────
ADV_STATE="docs/security/.advisory-scan-state"
if [ -f "docs/security/advisory-inbox.md" ]; then
    if [ ! -f "$ADV_STATE" ]; then
        echo ""
        echo "🚨 Advisory-watch: chưa scan lần nào — gõ /advisory-scan (first scan)"
    else
        ADV_LAST=$(grep -oE '"last_scan_at"[[:space:]]*:[[:space:]]*"[^"]+"' "$ADV_STATE" 2>/dev/null \
                   | sed -E 's/.*"last_scan_at"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/' | head -1)
        [ -z "$ADV_LAST" ] && ADV_LAST=$(tr -d '[:space:]' < "$ADV_STATE" 2>/dev/null)
        # cross-platform ISO→epoch: GNU `date -d` first, BSD `date -j -f` fallback, else 0
        ADV_EPOCH=$(date -d "$ADV_LAST" +%s 2>/dev/null \
                    || date -j -f "%Y-%m-%dT%H:%M:%SZ" "$ADV_LAST" +%s 2>/dev/null \
                    || echo 0)
        if [ "$ADV_EPOCH" -gt 0 ]; then
            ADV_DAYS=$(( ( $(date +%s) - ADV_EPOCH ) / 86400 ))
            if [ "$ADV_DAYS" -ge 7 ]; then
                echo ""
                echo "🚨 Advisory-watch: scan cuối $ADV_DAYS ngày trước (>= 7) — orchestrator BẮT BUỘC auto-spawn advisory-watch (ORCHESTRATION Rule 11)"
            elif [ "$ADV_DAYS" -ge 3 ]; then
                echo ""
                echo "⚠️  Advisory-watch: scan cuối $ADV_DAYS ngày trước — cân nhắc /advisory-scan"
            fi
        fi
    fi
fi

echo ""
echo "🤖 Orchestrator contract (main session — đọc kỹ, ép tuân thủ):"
echo "    State machine: DRAFT → CHALLENGE → [RESPOND ⇄ CHALLENGE] → APPROVAL_GATE → EXECUTE"
echo "    KHÔNG hỏi user giữa các phase. APPROVAL_GATE là gate DUY NHẤT (trước EXECUTE)."
echo "    KHÔNG đẩy đọc phiếu/code về user — Worker CHALLENGE rà & report ≤5 dòng."
echo "    Marker (2 chiều — ĐỦ CẢ TOUCH+RM): spawn architect → touch architect-active + rm worker-active;"
echo "            spawn worker → touch worker-active + rm architect-active; worker XONG → rm worker-active."
echo "            (quên touch worker-active → orchestrator-guard CHẶN Write đầu của Worker. F-001 claude-hooks.)"
echo "    Deferred tools MANDATORY (load đầu session, KHÔNG fallback markdown 1/2/3):"
echo "        ToolSearch select:AskUserQuestion,TaskCreate,TaskUpdate"
echo "    Handbook: agents/orchestrator.md (~85 lines, condensed contract)"
echo "    Spec đầy đủ: docs/ORCHESTRATION.md"
echo ""
echo "📌 Architect Rule 0: chỉ viết phiếu cho item trong Active sprint (or first ^## section if absent)."
echo "    Idea mới → /idea skill (intake vào BACKLOG.md)."
echo "    Pick item hay add idea?"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
