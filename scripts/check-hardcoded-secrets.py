#!/usr/bin/env python3
"""
INV-009 enforcer: scan source files for hardcoded API key / secret literal values.

Exit 0 = clean. Exit 1 = violation(s) printed.

Detects:
- High-precision prefix match (sk-ant-, sk-, AKIA, ghp_, gho_, ghu_, ghs_,
  AIza, re_, Telegram bot token, Stripe sk_live/sk_test, Slack xox*).
- Generic high-entropy fallback (api_key/secret/password/token = "..." 32+ char).

Allowlist (skip):
- Env reference substrings (process.env. / os.environ).
- Test fixtures (*.test.*, *.spec.*, __tests__/, __mocks__/, /tests/).
- Generated build artifacts (configurable via SKIP_PATH_SUBSTR).
- Single-line comments (// for JS/TS, # for Python).

NOT detected (accepted limitations):
- Multi-line block comment /* ... */ in JS/TS.
- Multi-line string secrets split across lines.
- Base64-encoded secrets inside JSON blobs.

Doctrine: WORKFLOW_V2.2.md §7 Sub-mech F (runtime state token leak).

Per-repo customization: edit the # CUSTOMIZE blocks below.
"""
import os
import re
import sys
from pathlib import Path

# =============================================================================
# CUSTOMIZE — per-repo configuration (override via env var OR edit defaults)
# =============================================================================

# Source directories to scan (comma-separated env var override)
SRC_DIRS_JS = os.environ.get("INV009_SRC_DIRS_JS", "src").split(",")
SRC_DIRS_PY = os.environ.get("INV009_SRC_DIRS_PY", "").split(",") if os.environ.get("INV009_SRC_DIRS_PY") else []
JS_EXTS = (".ts", ".tsx", ".js", ".jsx")
PY_EXTS = (".py",)

# Skip path substrings (extend per-repo for build artifacts containing high-entropy blobs)
SKIP_PATH_SUBSTR = [
    "node_modules/",
    ".next/",
    "__pycache__/",
    "dist/",
    "build/",
    "target/",
    ".claude/worktrees/",
    # CUSTOMIZE: add per-repo paths with coincidental high-entropy (e.g. WASM, base64 blob).
]

# Allowlist substrings (skip violation if match line contains)
# CUSTOMIZE: extend with project-specific public IDs / DSN prefixes / model slugs.
ALLOWLIST_SUBSTRINGS = [
    "process.env.",   # Node env reference (not literal)
    "os.environ",     # Python env reference (not literal)
    "your-",          # Placeholder convention (your-api-key)
    "xxx",            # Common placeholder
    "REPLACE",        # Common placeholder
    "PLACEHOLDER",    # Common placeholder
    "EXAMPLE",        # Common placeholder
    "CHANGEME",       # Common placeholder
]

# Test file patterns (allowlist — known to contain mock secrets)
TEST_FILE_PATTERNS = [
    re.compile(r"\.test\.(ts|tsx|js|jsx|py)$"),
    re.compile(r"\.spec\.(ts|tsx|js|jsx|py)$"),
    re.compile(r"/__tests__/"),
    re.compile(r"/__mocks__/"),
    re.compile(r"/tests/"),
]

# =============================================================================
# END CUSTOMIZE — fixed detection logic below
# =============================================================================

# High-precision prefix patterns (most reliable, low false positive)
PREFIX_PATTERNS = [
    # Anthropic API key (runtime sk-ant-api03-... + admin sk-ant-admin01-... both match)
    ("anthropic", re.compile(r'sk-ant-[A-Za-z0-9_\-]{40,}')),
    # OpenAI-style API key
    ("openai", re.compile(r'sk-[A-Za-z0-9]{48}')),
    # AWS access key ID
    ("aws", re.compile(r'AKIA[0-9A-Z]{16}')),
    # GitHub personal access token (ghp_/gho_/ghu_/ghs_)
    ("github-pat", re.compile(r'gh[pous]_[A-Za-z0-9]{36}')),
    # Google API key (AIza prefix)
    ("google-api", re.compile(r'AIza[0-9A-Za-z_\-]{35}')),
    # Resend API key
    ("resend", re.compile(r're_[A-Za-z0-9_\-]{30,}')),
    # Telegram bot token (bot_id:token format)
    ("telegram-bot", re.compile(r'\b\d{8,12}:[A-Za-z0-9_\-]{35}\b')),
    # Stripe live/test keys
    ("stripe", re.compile(r'sk_(live|test)_[A-Za-z0-9]{24,}')),
    # Slack tokens (bot, app, user, refresh)
    ("slack", re.compile(r'xox[baprs]-[A-Za-z0-9\-]{10,}')),
]

# Generic high-entropy fallback (catch unprefixed secrets when hardcoded inline).
GENERIC_PATTERN = re.compile(
    r'(?i)(api[_-]?key|apikey|secret|password|token)\s*[:=]\s*[\'"`]([A-Za-z0-9_\-+/=]{32,})[\'"`]'
)

# Comment line prefixes (minimal — single-line only)
JS_COMMENT_PREFIX = ("//",)
PY_COMMENT_PREFIX = ("#",)


def is_test_file(path_str):
    return any(p.search(path_str) for p in TEST_FILE_PATTERNS)


def should_skip_path(path_str):
    return any(s in path_str for s in SKIP_PATH_SUBSTR)


def is_comment_line(line, ext):
    stripped = line.lstrip()
    if ext == ".py":
        return stripped.startswith(PY_COMMENT_PREFIX)
    return stripped.startswith(JS_COMMENT_PREFIX)


def is_allowlisted(text):
    return any(s in text for s in ALLOWLIST_SUBSTRINGS)


def mask(secret):
    """Mask the matched secret: keep first 4 + last 4 chars only."""
    if len(secret) <= 12:
        return "***" + secret[-4:] if len(secret) >= 4 else "***"
    return f"{secret[:4]}...{secret[-4:]}"


def scan_file(path):
    """Return list of violation tuples (line_no, masked_match, pattern_name)."""
    violations = []
    try:
        content = path.read_text(encoding="utf-8")
    except (UnicodeDecodeError, OSError):
        return violations
    ext = path.suffix
    for lineno, line in enumerate(content.splitlines(), start=1):
        if is_comment_line(line, ext):
            continue
        # Prefix-based detection (high precision)
        for name, pat in PREFIX_PATTERNS:
            for m in pat.finditer(line):
                hit = m.group(0)
                if is_allowlisted(line) or is_allowlisted(hit):
                    continue
                violations.append((lineno, mask(hit), name))
        # Generic high-entropy fallback
        for m in GENERIC_PATTERN.finditer(line):
            hit = m.group(2)
            full = m.group(0)
            if is_allowlisted(line) or is_allowlisted(full) or is_allowlisted(hit):
                continue
            violations.append((lineno, mask(hit), "generic-entropy"))
    return violations


def collect_files():
    files = []
    for d in SRC_DIRS_JS:
        d = d.strip()
        if not d:
            continue
        root = Path(d)
        if not root.exists():
            continue
        for ext in JS_EXTS:
            files.extend(root.rglob(f"*{ext}"))
    for d in SRC_DIRS_PY:
        d = d.strip()
        if not d:
            continue
        root = Path(d)
        if not root.exists():
            continue
        for ext in PY_EXTS:
            files.extend(root.rglob(f"*{ext}"))
    return files


def main():
    all_violations = []
    file_count = 0
    for path in collect_files():
        path_str = str(path)
        if should_skip_path(path_str):
            continue
        if is_test_file(path_str):
            continue
        file_count += 1
        for lineno, masked, pat_name in scan_file(path):
            all_violations.append(
                f"{path_str}:{lineno}: INV-009 violated -- {masked} ({pat_name})"
            )
    if all_violations:
        print("\n".join(all_violations))
        sys.exit(1)
    print(f"INV-009: PASS (0 hardcoded secrets, {file_count} files scanned)")


if __name__ == "__main__":
    main()
