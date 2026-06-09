# Changelog

Format loosely follows Keep a Changelog.

## v0.1.0 — P001 scaffold CLI — 2026-06-09

- **P001**: Scaffold CLI 5-subcommand + stdin-JSON harness + exit-code convention.
  - `src/main.rs`: clap derive entry point dispatching to 5 subcmds (`architect-guard`, `block-env-edit`, `block-unsafe-merge`, `session-banner`, `serve`). Hook entries return `i32`; `main` calls `process::exit`.
  - `src/io.rs`: `HookPayload`/`ToolInput` serde model (all fields `Option`); `read_payload()` fail-open (empty stdin / invalid JSON → `Default`); `ALLOW=0` / `BLOCK=2` constants; `block(reason)` stderr helper.
  - `src/hooks/mod.rs`: 4 stub hook functions return `ALLOW`; `architect_guard` + `block_env_edit` wire `read_payload()` harness.
  - `src/serve.rs`: MCP stub, prints `"serve: not yet implemented (P006)"` to stderr, exits 0.
  - `tests/cli.rs`: 8 integration tests (verify-cò P057) — all pass.
  - Docs Gate: `docs/ARCHITECTURE.md` CLI surface section added.

## v0.0.0 — sos adopt — 2026-06-09

- Spine retrofitted via `sos adopt`.
