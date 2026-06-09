# claude-hooks

Rust binary replacing ~418 lines of Bash hot-path hooks for Claude Code. Dual mode: **CLI** (invoked by Claude Code PreToolUse/SessionStart hooks) and **MCP** (stdio JSON-RPC server for debug tooling).

## Install

```bash
# From source
cargo install --path .

# From crates.io (after publish)
cargo install claude-hooks
```

## CLI usage

All subcommands read a JSON payload from **stdin** (Claude Code PreToolUse format). No per-hook CLI flags.

**Stdin payload shape:**
```json
{ "tool_name": "Read", "tool_input": { "file_path": "...", "pattern": "...", "path": "...", "notebook_path": "...", "command": "..." } }
```

**Exit codes:** `0` = allow, `2` = block (reason on stderr).

### Subcommands

| Subcommand | Trigger | Block condition |
|---|---|---|
| `architect-guard` | `Read`, `Glob`, `Write`, `Edit` | Architect-active marker present + (Read/Glob: path is source/test/prisma/sql file) or (Write/Edit: path not in `docs/ticket/P*-*.md` allowlist) |
| `block-env-edit` | `Edit`, `Write`, `MultiEdit`, `NotebookEdit` | Target path matches `^\.env($|\.)` (except `.env.example`) |
| `block-unsafe-merge` | `Bash` | `gh pr merge <N>` on security-surface PR without `/security-review APPROVE` |
| `session-banner` | `SessionStart` | Never blocks — renders informational banner to stdout, always exits 0 |
| `serve` | — | Starts MCP server (see MCP mode below) |

**Fail-open default:** `architect-guard`, `block-env-edit`, `session-banner` all exit 0 on any parse error / missing input.

**`block-unsafe-merge` is fail-CLOSED:** if `gh pr diff` fails or returns empty, the hook blocks (exit 2). Unverifiable merges are treated as unsafe.

### Wire into `.claude/settings.json`

```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [{ "type": "command", "command": "claude-hooks session-banner" }]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Read|Glob|Write|Edit",
        "hooks": [{ "type": "command", "command": "claude-hooks architect-guard" }]
      },
      {
        "matcher": "Edit|Write|MultiEdit|NotebookEdit",
        "hooks": [{ "type": "command", "command": "claude-hooks block-env-edit" }]
      },
      {
        "matcher": "Bash",
        "hooks": [{ "type": "command", "command": "claude-hooks block-unsafe-merge" }]
      }
    ]
  }
}
```

## MCP mode

`claude-hooks serve` starts an MCP server over stdio (newline-delimited JSON-RPC, rmcp 1.7). Use for debug: call hook decision functions directly without Claude Code PreToolUse wiring.

**Server info:** name `claude-hooks`, version matches crate version.

### 5 MCP tools

| Tool | Input | Output | Description |
|---|---|---|---|
| `architect_guard` | `{ tool_name?, file_path?, pattern?, path? }` | `{ blocked, exit_code, reason? }` | Marker gate + tool_name dispatch + forbidden/allowed path check |
| `block_env_edit` | `{ file_path?, notebook_path? }` | `{ blocked, exit_code, reason? }` | `.env*` edit block (not `.env.example`) |
| `block_unsafe_merge` | `{ command? }` | `{ blocked, exit_code, reason? }` | PR merge security check (real gh calls, fail-CLOSED) |
| `session_banner` | `{}` | `{ banner: string }` | Full banner rendered from fs/git state |
| `why_blocked` | `{ tool_name, tool_input? }` | `{ hook, blocked, exit_code, reason? }` | **Debug router**: routes `tool_name` to matching hook, returns which hook fired + decision |

`why_blocked` mirrors the `.claude/settings.json` PreToolUse matchers: `Read`/`Glob` → `architect_guard`; `Edit`/`Write`/`MultiEdit`/`NotebookEdit` → `block_env_edit`; `Bash` → `block_unsafe_merge`; anything else → `hook="none"`, allowed.

### Wire into `.mcp.json`

```json
{
  "mcpServers": {
    "claude-hooks": {
      "type": "stdio",
      "command": "claude-hooks",
      "args": ["serve"]
    }
  }
}
```

## Exit convention

| Code | Meaning |
|---|---|
| `0` | Allow — action proceeds |
| `2` | Block — reason written to stderr |

Fail-open by default (all hooks except `block-unsafe-merge`). `block-unsafe-merge` is **fail-CLOSED**: `gh` unavailable or empty diff → block.

> ⚠️ **Deployment caveat — keep the binary on `PATH`.** If `claude-hooks` is **not installed** (e.g. a fresh machine that hasn't run your setup script), a hook command like `claude-hooks block-unsafe-merge` resolves to *command-not-found* → shell exit `127`, which the harness treats as non-blocking (allow). That silently turns the **fail-CLOSED** `block-unsafe-merge` gate **fail-OPEN** *before any code runs*. The binary cannot self-defend against its own absence — guard it at the wiring layer.
>
> **The deployment pattern is the adopter's choice** (for sos-kit downstream repos, it's decided in the sos-kit adoption doctrine — see `docs/handoff/SOS_KIT_HANDOFF.md`). Two reference patterns for the fail-CLOSED hook:
>
> *Option A — Bash-fallback wrapper.* Prefer the Rust binary, fall back to the always-present Bash oracle when absent, so the gate stays *functional* (not just blocking) and never fails open:
> ```bash
> #!/usr/bin/env bash
> # block-unsafe-merge: prefer Rust binary; if absent, run the Bash oracle (always present).
> if command -v claude-hooks >/dev/null 2>&1; then
>   exec claude-hooks block-unsafe-merge "$@"
> fi
> exec bash "$(dirname "$0")/block-unsafe-merge.sh" "$@"
> ```
> Point `.claude/settings.json` at this wrapper. **Caveat:** keeping a functional Bash fallback means you must keep the `.sh` oracle in **parity** with the binary (and test the fallback path periodically) — an un-exercised fallback can rot silently. The Bash files in this repo are maintained as the port oracles, so parity is already the discipline.
>
> *Option B — Fail-closed shim (if you don't want to maintain Bash):* `command -v claude-hooks || { echo BLOCKED >&2; exit 2; }` then `exec claude-hooks ...`. Safer-by-default but blocks even legit merges until the binary is installed.
>
> Lower-stakes fail-open hooks (architect-guard, block-env-edit, session-banner) don't need either — if absent they just allow, which is their default.

## Environment variables

`claude-hooks` reads no `.env` file — it is a stateless CLI binary. Two optional environment variables affect behavior; both are read at runtime via `std::env::var` and default safely when unset:

| Variable | Set by | Effect |
|---|---|---|
| `CLAUDE_PROJECT_DIR` | Claude Code (automatically, when firing a hook) | Repo root for resolving `.sos-state/` markers, `docs/BACKLOG.md`, and relative paths. Falls back to the current working directory when unset. |
| `SECURITY_SURFACE_EXTRA` | Optional, per-repo (deployer) | Extra regex alternation appended to `block-unsafe-merge`'s security-surface pattern (e.g. `mycrate/secrets/`). Unset = generic pattern only. |

No secrets are read from the environment. There is intentionally no `.env.example` — neither variable holds a credential, and `CLAUDE_PROJECT_DIR` is supplied by the Claude Code harness rather than the operator.

## See also

- `docs/ARCHITECTURE.md` — detailed pipeline docs for each hook, Decision-core refactor, MCP transport notes.
- `docs/BACKLOG.md` — current sprint.
- `CHANGELOG.md` — version history.
