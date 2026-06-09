# Architecture — claude-hooks

## Overview

`claude-hooks` is a Rust binary replacing ~418 lines of Bash hot-path hooks for Claude Code.
It runs in two modes: **CLI** (invoked by Claude Code PreToolUse hooks) and **MCP** (stdio JSON-RPC server for debug tooling).
The binary name is `claude-hooks`; subcommands are kebab-case.

## CLI Surface

### Subcommands (5)

| Subcommand | Clap variant | Status | Port target |
|---|---|---|---|
| `architect-guard` | `Cmd::ArchitectGuard` | stub (P001) | P002 |
| `block-env-edit` | `Cmd::BlockEnvEdit` | stub (P001) | P003 |
| `block-unsafe-merge` | `Cmd::BlockUnsafeMerge` | stub (P001) | P004 |
| `session-banner` | `Cmd::SessionBanner` | stub (P001) | P005 |
| `serve` | `Cmd::Serve` | stub (P001) | P006 |

Kebab-case names are auto-derived by clap from PascalCase variants (verified P001: no `#[command(name=...)]` needed for clap 4.6).

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

## Data Flow (P001 scaffold)

```
Claude Code PreToolUse trigger
  -> claude-hooks <subcmd>  (stdin = JSON payload)
     -> clap parse subcommand
     -> dispatch to hook fn
        -> read_payload() [fail-open]
        -> stub logic (return ALLOW)
     -> process::exit(0)
```

P002+ replace stub logic with real Bash-parity logic; harness wiring unchanged.

## Bash Reference (oracle)

Port doctrine: 1:1 from `scripts/` canonical copies. Do not redesign behavior.

| Rust subcmd | Bash oracle |
|---|---|
| `architect-guard` | `scripts/architect-guard.sh` |
| `block-env-edit` | `scripts/block-env-edit.sh` |
| `block-unsafe-merge` | `scripts/block-unsafe-merge.sh` |
| `session-banner` | `scripts/session-start-banner.sh` |
