#!/usr/bin/env bash
# check-case-collision.sh — pre-commit gate: chặn path mới case-đụng path/dir đã tracked.
#
# Bệnh (Két dogfood): repo đã tracked `scripts/` (thường, kit-infra) nhưng PROJECT.md +
# phiếu ghi `Scripts/` (hoa). Trên macOS case-insensitive → cùng 1 thư mục vật lý, git
# index case-sensitive → thrash; trên Linux/CI → tách thành 2 dir. grep case-sensitive
# lừa người (1 match → tưởng 1 file). → bắt bằng cơ chế, không bằng mắt.
#
# Cách bắt: gom mọi path (tracked + staged) + mọi dir-prefix của chúng, lowercase, nếu
# 1 key lowercase map sang >1 casing thật → COLLISION. Bắt cả dir-component (Scripts vs
# scripts) lẫn full-path (Scripts/x vs scripts/x).
#
# Exit 1 = block commit. No external deps beyond git + awk.

set -uo pipefail

# Chạy ở repo root (pre-commit đã ở đó, nhưng chắc cú).
cd "$(git rev-parse --show-toplevel 2>/dev/null)" || exit 0

# core.quotepath=false → non-ASCII paths come through as raw UTF-8, not C-escaped
# (`"caf\303\251.md"`) — else `dirname` mangles them + output is not git-mv-able.
# Residual known-hole: awk tolower() is ASCII-only, so a Unicode-ONLY case diff (Á vs á)
# still slips. quotepath=false + a casefold step (perl -CSDA/python) would close it — follow-up.
COLLISIONS=$(
  { git -c core.quotepath=false ls-files
    git -c core.quotepath=false diff --cached --name-only --diff-filter=ACMR; } 2>/dev/null \
  | while IFS= read -r p; do
      [ -z "$p" ] && continue
      printf '%s\n' "$p"                       # full path
      d=$(dirname -- "$p")                      # + every ancestor dir
      while [ "$d" != "." ] && [ "$d" != "/" ]; do
        printf '%s\n' "$d"
        d=$(dirname -- "$d")
      done
    done \
  | sort -u \
  | awk '
      { k = tolower($0); variants[k] = (k in variants ? variants[k] SUBSEP $0 : $0) }
      END {
        for (k in variants) {
          n = split(variants[k], a, SUBSEP)
          if (n > 1) { line = " "; for (i = 1; i <= n; i++) line = line (i > 1 ? "  <->  " : "  ") a[i]; print line }
        }
      }'
)

if [ -n "$COLLISIONS" ]; then
  printf '\033[31m❌ Case collision — path khác-case cùng tồn tại (vỡ trên Linux/CI, thrash trên macOS):\033[0m\n'
  printf '%s\n' "$COLLISIONS"
  printf 'Sửa: chọn MỘT case nhất quán (theo path đã tracked) rồi git rm/mv path lệch.\n'
  exit 1
fi

exit 0
