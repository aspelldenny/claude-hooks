#!/usr/bin/env python3
"""
INV-010 enforcer: scan runtime state + infra files for hardcoded
secret / credential / token literal value.

Scope (extends INV-009 source-file coverage):
  1. Runtime untracked state: .git/config, .mcp.json, .claude/settings.local.json
  2. Tracked infra-extension: scripts/*.sh, .github/workflows/*.yml,
     hooks/*, Dockerfile, docker-compose*.yml

Exit 0 = clean. Exit 1 = violation(s) printed.

Detects (extends INV-009 PREFIX_PATTERNS):
- All INV-009 prefix patterns (sk-ant-, sk-, AKIA, gh[pous]_, AIza, re_, Telegram, Stripe, Slack)
- PEM private key block (RSA, OPENSSH ed25519/new, EC, DSA, PKCS#8 empty variant)
- DB conn string literal password >=8 chars (postgresql/mongodb/mysql/redis)
- Token-in-URL pattern (incident pattern — ghp_/github_pat_ embed URL)
- Fine-grained GitHub PAT (github_pat_ prefix, 82-char body)

Allowlist (skip false positive):
- Generic placeholders (your-, xxx, REPLACE, PLACEHOLDER, CHANGEME, EXAMPLE)
- Angle bracket placeholder e.g. <YOUR_TOKEN>
- Env substitution syntax (${VAR}, $VAR)
- Comments: # (shell/YAML/Python), // (JS in workflow expressions)
- .example / .md / .sample / .template extension files

Doctrine: WORKFLOW_V2.2.md §7 Sub-mech F (runtime state token leak — instance #10 P305 git/config, #11 P306 dotfile).

Per-repo customization: edit the # CUSTOMIZE blocks below.
"""
import os
import re
import sys
from pathlib import Path

# =============================================================================
# CUSTOMIZE — per-repo configuration
# =============================================================================

# Runtime untracked state files (BẮT BUỘC scan if exists, skip with note if missing)
# Sub-mech F instance #10 (.git/config) + #11 (.mcp.json + .claude/settings.local.json)
RUNTIME_FILES = [
    Path(".git/config"),
    Path(".mcp.json"),
    Path(".claude/settings.local.json"),
]

# Tracked infra-extension globs (extend per-repo for additional infra dirs)
INFRA_GLOBS = [
    ("scripts", "*.sh"),
    (".github/workflows", "*.yml"),
    (".github/workflows", "*.yaml"),
    ("hooks", "*"),
]

# Top-level infra files (exact match)
# CUSTOMIZE: add per-repo Dockerfile variants (e.g. Dockerfile.nextjs, Dockerfile.api)
INFRA_TOP_LEVEL = [
    Path("Dockerfile"),
    Path("docker-compose.yml"),
    Path("docker-compose.staging.yml"),
    Path("docker-compose.production.yml"),
    # docker-compose.dev.yml intentionally excluded (test fixture convention)
]

# Skip path substrings (extend per-repo for vendored/generated dirs)
SKIP_PATH_SUBSTR = [
    "node_modules/",
    ".next/",
    "target/",
    "dist/",
    "build/",
    ".claude/worktrees/",
]

# Allowlist substrings (skip violation if match line contains)
# CUSTOMIZE: extend with project-specific public IDs / DSN prefixes / model slugs
ALLOWLIST_SUBSTRINGS = [
    # Generic placeholders
    "process.env.",   # Node env reference (not literal)
    "os.environ",     # Python env reference (not literal)
    "your-",
    "xxx",
    "REPLACE",
    "PLACEHOLDER",
    "CHANGEME",
    "EXAMPLE",
    "${",  # env substitution syntax (Docker compose / shell)
    "<",   # angle bracket placeholder e.g. <YOUR_TOKEN>
]

# =============================================================================
# END CUSTOMIZE — fixed detection logic below
# =============================================================================

# Skip if filename ends with these (documentation/example extensions)
SKIP_EXTENSIONS = (".example", ".md", ".sample", ".template")

# High-precision prefix patterns (extends INV-009 set with 4 new families)
PREFIX_PATTERNS = [
    # === INV-009 patterns (mirror) ===
    ("anthropic", re.compile(r'sk-ant-[A-Za-z0-9_\-]{40,}')),
    ("openai", re.compile(r'sk-[A-Za-z0-9]{48}')),
    ("aws", re.compile(r'AKIA[0-9A-Z]{16}')),
    ("github-pat-classic", re.compile(r'gh[pous]_[A-Za-z0-9]{36}')),
    ("github-pat-fine", re.compile(r'github_pat_[A-Za-z0-9_]{82}')),
    ("google-api", re.compile(r'AIza[0-9A-Za-z_\-]{35}')),
    ("resend", re.compile(r're_[A-Za-z0-9_\-]{30,}')),
    ("telegram-bot", re.compile(r'\b\d{8,12}:[A-Za-z0-9_\-]{35}\b')),
    # === INV-010 new patterns ===
    # PEM private key block (covers RSA, OPENSSH ed25519/new, EC, DSA, PKCS#8 empty)
    ("pem-private-key", re.compile(r'-----BEGIN (RSA |OPENSSH |EC |DSA |)PRIVATE KEY-----')),
    # Token-in-URL (incident pattern: ghp_/github_pat_ embed plaintext URL)
    ("token-in-url", re.compile(
        r'https://[a-zA-Z0-9._\-]+:(gh[pous]_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{82})@'
    )),
    # DB connection string with literal password >=8 chars
    # Negative lookahead `(?!\$)` excludes ${VAR} / $VAR
    ("db-conn-postgresql", re.compile(r'postgresql://[^:/\s\$]+:(?!\$)[^@/\s\$]{8,}@')),
    ("db-conn-mongodb", re.compile(r'mongodb(\+srv)?://[^:/\s\$]+:(?!\$)[^@/\s\$]{8,}@')),
    ("db-conn-mysql", re.compile(r'mysql://[^:/\s\$]+:(?!\$)[^@/\s\$]{8,}@')),
    ("db-conn-redis", re.compile(r'redis://[^:/\s\$]*:(?!\$)[^@/\s\$]{8,}@')),
    # Stripe keys
    ("stripe-live-secret", re.compile(r'sk_live_[A-Za-z0-9]{24,}')),
    ("stripe-test-secret", re.compile(r'sk_test_[A-Za-z0-9]{24,}')),
    ("stripe-live-restricted", re.compile(r'rk_live_[A-Za-z0-9]{24,}')),
    ("stripe-test-restricted", re.compile(r'rk_test_[A-Za-z0-9]{24,}')),
    # Slack bot/app tokens
    ("slack-token", re.compile(r'xox[baprs]-[A-Za-z0-9-]{10,}')),
]

# Generic high-entropy fallback (expanded identifier list)
GENERIC_PATTERN = re.compile(
    r'(?i)(api[_-]?key|apikey|secret|password|token|jwt[_-]?secret|signing[_-]?key|private[_-]?key)\s*[:=]\s*[\'"`]([A-Za-z0-9_\-+/=]{32,})[\'"`]'
)

# Comment line prefixes per file type
SHELL_COMMENT_PREFIX = ("#",)
JS_COMMENT_PREFIX = ("//",)


def should_skip_path(path_str):
    return any(s in path_str for s in SKIP_PATH_SUBSTR)


def should_skip_extension(path):
    return path.name.endswith(SKIP_EXTENSIONS)


def is_comment_line(line, path):
    stripped = line.lstrip()
    ext = path.suffix.lower()
    name = path.name.lower()
    # Shell / YAML / Python (.git/config also # comments)
    if ext in (".sh", ".yml", ".yaml", ".py") or name in (".gitconfig", "config"):
        return stripped.startswith(SHELL_COMMENT_PREFIX)
    # Docker compose YAML / Dockerfile
    if name.startswith("docker-compose") or name == "dockerfile" or name.startswith("dockerfile."):
        return stripped.startswith(("#",))
    # JS in workflow expressions (rare)
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
        content = path.read_text(encoding="utf-8", errors="ignore")
    except (UnicodeDecodeError, OSError, IsADirectoryError):
        return violations
    for lineno, line in enumerate(content.splitlines(), start=1):
        if is_comment_line(line, path):
            continue
        # Prefix-based detection
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


def collect_runtime_files():
    """Hard-coded runtime untracked state paths. Skip silently if not exists."""
    files = []
    for p in RUNTIME_FILES:
        if p.exists() and p.is_file():
            files.append(p)
    return files


def collect_infra_files():
    """Glob tracked infra-extension files."""
    files = []
    for d, pattern in INFRA_GLOBS:
        root = Path(d)
        if not root.exists():
            continue
        for f in root.glob(pattern):
            if f.is_file():
                files.append(f)
    for p in INFRA_TOP_LEVEL:
        if p.exists() and p.is_file():
            files.append(p)
    return files


def main():
    all_violations = []
    scanned = 0
    for path in collect_runtime_files() + collect_infra_files():
        path_str = str(path)
        if should_skip_path(path_str):
            continue
        if should_skip_extension(path):
            continue
        scanned += 1
        for lineno, masked, pat_name in scan_file(path):
            all_violations.append(
                f"{path_str}:{lineno}: INV-010 violated -- {masked} ({pat_name})"
            )
    if all_violations:
        print("\n".join(all_violations))
        sys.exit(1)
    print(f"INV-010: PASS (0 runtime/infra secrets, {scanned} files scanned)")


if __name__ == "__main__":
    main()
