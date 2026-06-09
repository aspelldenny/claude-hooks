# Architecture — claude-hooks

## Overview

`claude-hooks` is a Rust binary replacing ~418 lines of Bash hot-path hooks for Claude Code.
It runs in two modes: **CLI** (invoked by Claude Code PreToolUse hooks) and **MCP** (stdio JSON-RPC server for debug tooling).
The binary name is `claude-hooks`; subcommands are kebab-case.

## CLI Surface

### Subcommands (5)

| Subcommand | Clap variant | Status | Port target |
|---|---|---|---|
| `architect-guard` | `Cmd::ArchitectGuard` | real (P002) | — |
| `block-env-edit` | `Cmd::BlockEnvEdit` | real (P003) | — |
| `block-unsafe-merge` | `Cmd::BlockUnsafeMerge` | real (P004) | — |
| `session-banner` | `Cmd::SessionBanner` | stub (P001) | P005 |
| `serve` | `Cmd::Serve` | stub (P001) | P006 |

Kebab-case names are auto-derived by clap from PascalCase variants (verified P001: no `#[command(name=...)]` needed for clap 4.6).

### `architect-guard` (P002 — real implementation)

Ports `scripts/architect-guard.sh` 1:1. Fires on every `Read`/`Glob` PreToolUse call.

**Pipeline (8 steps):**

1. Resolve repo root from `CLAUDE_PROJECT_DIR` env (fallback: cwd). All internal paths bind to this root.
2. **Marker gate:** if `.sos-state/architect-active` does not exist → `ALLOW` (not running as Architect).
3. Parse path from stdin JSON: `tool_input.file_path` (priority), fallback `tool_input.pattern`.
4. No path parsed → `ALLOW` (fail-open).
5. Strip leading `./`.
6. Path ends with `.md` → `ALLOW` (docs are Architect's domain).
7. **Forbidden pattern check** — `BLOCK` if any match:

   | Group | Pattern | Rust check |
   |---|---|---|
   | Source dirs (prefix) | `src/*`, `lib/*`, `app/*`, `pkg/*` | `starts_with` |
   | Source dirs (segment) | `*/src/*`, `*/lib/*`, `*/app/*`, `*/pkg/*` | `contains` |
   | Test dirs (prefix) | `tests/*`, `test/*`, `__tests__/*` | `starts_with` |
   | Test dirs (segment) | `*/tests/*`, `*/test/*` | `contains` |
   | Build artifacts (prefix only) | `node_modules/*`, `target/*`, `dist/*`, `build/*`, `.next/*`, `.nuxt/*`, `.svelte-kit/*` | `starts_with` |
   | Extensions | `*.rs *.ts *.tsx *.js *.jsx *.py *.go *.java *.cpp *.c *.h *.hpp` | `ends_with` |

   Default (no match) → `ALLOW`.

8. Blocked → `io::block(msg)` (writes message to stderr, returns exit 2).

**Exit codes:** `0` (ALLOW), `2` (BLOCK — see exit-code table above).

**Block message** (stderr, verbatim oracle): `🚫 Architect envelope violation` + path (original, pre-strip) + instructions for Task 0 anchor workflow.

### `block-env-edit` (P003 — real implementation)

Ports `scripts/block-env-edit.sh` 1:1. Fires on every `Edit`/`Write` PreToolUse call. Guards against secret leak (`.env*` files contain API keys, DB credentials, webhook tokens).

**Pipeline (8 steps):**

1. Read stdin payload via `read_payload()` (fail-open). **Note:** env-fallback `CLAUDE_TOOL_INPUT` (oracle L16-20) intentionally not ported — Claude Code always pipes stdin; see `docs/discoveries/P003.md` for rationale.
2. Empty payload (empty stdin) → `ALLOW` (fail-open via steps 3-4).
3. Parse path: `tool_input.file_path` (priority), fallback `tool_input.notebook_path` (NotebookEdit). **No `pattern` field** — this hook does not handle Glob.
4. No path parsed → `ALLOW` (fail-open).
5. **Basename:** take last `/`-delimited segment of path. (`/a/b/.env` → `.env`).
6. **Allowlist:** basename `== ".env.example"` → `ALLOW` (template, no real secrets).
7. **Block regex:** basename matches `^\.env($|\.)` → `io::block(msg)` (stderr) → `BLOCK` (exit 2).
   - Matches: `.env`, `.env.local`, `.env.production`, `.env.staging`, …
   - Does NOT match: `.envrc`, `.environment`, `config.yaml`, …
8. Else → `ALLOW`.

**Exit codes:** `0` (ALLOW), `2` (BLOCK).

**Block message** (stderr, verbatim oracle L46-59): `⛔ BLOCKED: Edit/Write tới <full-path> bị chặn.` + secret-leak rationale + valid alternatives + override instructions.

### `block-unsafe-merge` (P004 — real implementation)

Ports `scripts/block-unsafe-merge.sh` 1:1. Fires on every `Bash` PreToolUse call. Guards against merging PRs that touch security surface without a `/security-review APPROVE` comment.

**DIVERGENCE — FAIL-CLOSED (intentional, do not change):** When `gh pr diff` fails or returns empty, this hook returns `BLOCK` (exit 2). This is the **opposite** of `architect-guard`, `block-env-edit`, and `session-banner`, which all fail-open (exit 0). Rationale: an unverifiable merge of an unknown security surface must be treated as unsafe. Any future hook inheriting this pattern must explicitly document fail-closed intent.

**Pipeline (8 steps):**

1. Read stdin payload → `tool_input.command`. Empty/missing → `ALLOW` (fail-open). **Note:** env-fallback `CLAUDE_TOOL_INPUT` intentionally not ported (same decision as P002/P003).
2. `parse_merge_pr(command)` — matches `gh pr merge\s+\d+` (Rust regex ≈ oracle L41 `[[:space:]]+[0-9]+`). No match → `ALLOW`. Known limitation: branch-only form `gh pr merge --merge` (no number) bypasses hook (oracle L15-16, intentional).
3. `extract_skip_marker(command)` — matches `[security-review-skip:<reason>]`. Match → print override warning to stderr (verbatim oracle L52-53) → `ALLOW`.
4. Read `SECURITY_SURFACE_EXTRA` env var (optional, per-repo pattern extension).
5. **gh call #1:** `gh pr diff <PR> --name-only`. Fail OR empty stdout → **FAIL-CLOSED BLOCK** (exit 2, verbatim oracle L71-81 message). DO NOT change to fail-open.
6. `touches_security_surface(diff_files, extra)` — two-branch check:
   - **(a)** base pattern (oracle L60 verbatim) or extended pattern matches any filename.
   - **(b)** pattern does NOT match but a `^\.env` non-example file is present (catches `.env.local`, `.env.staging`, etc. that `\.env[^.]` misses). No match in either branch → `ALLOW`.
7. **gh call #2:** `gh pr view <PR> --json comments --jq '.comments[].body'`. Fail → empty string (fail-open, oracle L96).
8. `verdict_is_approve(comments)`:
   - `NoBlock` (no `<!-- security-review-start -->` marker) → `BLOCK` (oracle L120-137, verbatim message including literal `$(gh pr view …)` instruction for user, NOT executed by hook).
   - `Approve` (`^Verdict: APPROVE` within 50 lines of marker) → `ALLOW`.
   - `NeedsReview` (marker present but verdict is not APPROVE) → `BLOCK` (oracle L105-116, includes `$VERDICT_LINE` interpolation).

**Exit codes:** `0` (ALLOW), `2` (BLOCK).

**Security-surface base pattern (oracle L60 — verbatim, do not modify):**
```
src/|schema\.(prisma|sql)|migrations?/|nginx/|docker-compose.*\.yml|Dockerfile|\.env[^.]|middleware\.|lib/auth/|\.claude/agents/security-|docs/security/|scripts/security-gate|scripts/check-(hardcoded|runtime)-secrets|hooks/pre-commit
```

**Override marker:** `[security-review-skip:<reason>]` in command → hook logs warning to stderr and allows merge. Intended for docs-only PRs where pattern false-positives.

### stdin-JSON Harness (`src/io.rs`)

Claude Code PreToolUse hooks pass a JSON payload on stdin:

```json
{ "tool_input": { "file_path": "...", "pattern": "...", "notebook_path": "...", "command": "..." } }
```

`ToolInput` fields (all `Option<String>`, `#[serde(default)]`):
- `file_path` — used by `architect-guard` (priority) and `block-env-edit`.
- `pattern` — used by `architect-guard` (fallback when `file_path` absent).
- `notebook_path` — used by `block-env-edit` (NotebookEdit fallback).
- `command` — added P004. Used by `block-unsafe-merge`. Bash-tool payload field. P006/P007 may reuse.

The harness (`read_payload()`) reads all stdin and parses via serde_json.
**Fail-open semantics (HARD):** empty stdin / invalid JSON / missing fields → `HookPayload::default()` (all `Option` fields are `None`). No `unwrap()`/`expect()` panics on parse path. Mirrors `scripts/architect-guard.sh:44` and `scripts/block-env-edit.sh:23,35`.

### Exit-Code Convention

| Code | Meaning | Usage |
|---|---|---|
| `0` (`ALLOW`) | Allow action to proceed | All stubs + fail-open fallback |
| `2` (`BLOCK`) | Block action, reason on stderr | P002+ real hook logic |

Block reason is written to **stderr only** (not stdout). Constants `ALLOW` and `BLOCK` are defined in `src/io.rs`. All P001 stubs return `ALLOW`. `process::exit(code)` is called in `main`; hook functions return `i32` (not self-exit, enabling unit tests).

## Module Structure

```
src/
  main.rs        -- clap entry + dispatch (thin)
  io.rs          -- shared stdin harness + exit constants
  hooks/
    mod.rs       -- 4 hook stub functions
  serve.rs       -- MCP stub (P006 wires rmcp/tokio)
tests/
  cli.rs         -- 8 integration tests (assert_cmd)
```

## MCP Surface

`serve` subcommand: stdio JSON-RPC server (rmcp 1.7). P001 = stub only (prints `"serve: not yet implemented (P006)"` to stderr, exits 0). Full implementation in P006: `why_blocked` debug tool for Quản đốc/Sếp sessions.

## Data Flow

```
Claude Code PreToolUse trigger
  -> claude-hooks <subcmd>  (stdin = JSON payload)
     -> clap parse subcommand
     -> dispatch to hook fn
        -> read_payload() [fail-open]
        -> hook logic (marker gate + path match, or stub ALLOW)
     -> process::exit(code)
```

`architect-guard` (P002): real logic. `block-env-edit` (P003): real logic. `block-unsafe-merge` (P004): real logic (gh-shelling, fail-CLOSED). `session-banner` (P005+): still stub returning ALLOW. Harness wiring unchanged.

## Bash Reference (oracle)

Port doctrine: 1:1 from `scripts/` canonical copies. Do not redesign behavior.

| Rust subcmd | Bash oracle |
|---|---|
| `architect-guard` | `scripts/architect-guard.sh` |
| `block-env-edit` | `scripts/block-env-edit.sh` |
| `block-unsafe-merge` | `scripts/block-unsafe-merge.sh` |
| `session-banner` | `scripts/session-start-banner.sh` |
