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
| `block-unsafe-merge` | `Cmd::BlockUnsafeMerge` | stub (P001) | P004 |
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

### stdin-JSON Harness (`src/io.rs`)

Claude Code PreToolUse hooks pass a JSON payload on stdin:

```json
{ "tool_input": { "file_path": "...", "pattern": "...", "notebook_path": "..." } }
```

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

`architect-guard` (P002): real logic. `block-env-edit` (P003): real logic. `block-unsafe-merge` (P004+), `session-banner` (P005+): still stubs returning ALLOW. Harness wiring unchanged.

## Bash Reference (oracle)

Port doctrine: 1:1 from `scripts/` canonical copies. Do not redesign behavior.

| Rust subcmd | Bash oracle |
|---|---|
| `architect-guard` | `scripts/architect-guard.sh` |
| `block-env-edit` | `scripts/block-env-edit.sh` |
| `block-unsafe-merge` | `scripts/block-unsafe-merge.sh` |
| `session-banner` | `scripts/session-start-banner.sh` |
