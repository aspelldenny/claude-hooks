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
{ "tool_input": { "file_path": "...", "pattern": "...", "notebook_path": "...", "command": "..." } }
```

**Exit codes:** `0` = allow, `2` = block (reason on stderr).

### Subcommands

| Subcommand | Trigger | Block condition |
|---|---|---|
| `architect-guard` | `Read`, `Glob` | Architect-active marker present + path is source/test file |
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
        "matcher": "Read|Glob",
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
| `architect_guard` | `{ file_path?, pattern? }` | `{ blocked, exit_code, reason? }` | Marker gate + forbidden path check |
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

## See also

- `docs/ARCHITECTURE.md` — detailed pipeline docs for each hook, Decision-core refactor, MCP transport notes.
- `docs/BACKLOG.md` — current sprint.
- `CHANGELOG.md` — version history.
