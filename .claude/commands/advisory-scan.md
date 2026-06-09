---
description: Scan project for advisories — spawn advisory-watch agent, pipe its report into the advisory-inbox CLI binary. Replaces 142-line Bash heredoc form. Doctrine WORKFLOW_V2.2.md.
---

# /advisory-scan

You are the orchestrator (Quản đốc) running the `/advisory-scan` slash command. Execute steps in order — DO NOT skip, DO NOT improvise. Parser invocation + advisory query happen INSIDE the Trinh sát subagent, NOT in this main session.

## Step 0 — Verify prerequisites

1. Verify `.sos-stack.toml` exists at project root via `Glob(".sos-stack.toml")`.
   - **If missing:** STOP. Tell user: "No `.sos-stack.toml` found. Run `sos init security` first (foundation P040 — detects stack + writes schema). Then re-run `/advisory-scan`."
2. Verify `advisory-inbox` binary installed: `which advisory-inbox`. If missing, tell user: `cargo install --path ~/advisory-inbox --locked` (not yet on crates.io as of WORKFLOW_V2.2 ship date).
3. Verify inbox file exists. Default path: `docs/security/advisory-inbox.md`. If missing, bootstrap from `templates/advisory-inbox.md`.

## Step 1 — Spawn Trinh sát subagent

Use `Task` tool with `subagent_type: "advisory-watch"`. Prompt: `$ARGUMENTS` (empty = full scan; specific dep name = focused mode).

Trinh sát returns markdown report with `<!-- INBOX_APPEND_START -->` / `<!-- INBOX_APPEND_END -->` sentinel block containing new advisory rows.

## Step 2 — Pipe report into advisory-inbox binary

Take Trinh sát's full markdown output, pipe to binary on stdin (omit `--report` flag — binary reads stdin when flag absent per `--help`):

```bash
advisory-inbox scan-and-append \
  --inbox docs/security/advisory-inbox.md \
  --state docs/security/.advisory-scan-state
```

## Step 3 — Parse JSON output

Binary stdout returns: `{ "appended": N, "skipped_dedup": M, "total_open": K }`.

## Step 4 — Report to Sếp

- `N` new advisories appended to inbox (list new row Date/ID/Severity/Package per advisory).
- `M` duplicates skipped (already in `seen_advisories[]`).
- `K` total open rows currently in inbox.
- Link: `docs/security/advisory-inbox.md` — Sếp gạt row open → processed/dismissed.

## Step 5 — Exit codes

- 0 = success
- 1 = input error (sentinel missing / inbox missing `## Rows`)
- 2 = processing error (parse/write)

Full exit codes: `advisory-inbox --help`.

---

**Binary source:** `~/advisory-inbox/` (separate repo — pilot vòng 1 doctrine source). Install: `cargo install --path ~/advisory-inbox --locked`. Version check: `advisory-inbox --version`.

**Doctrine:** WORKFLOW_V2.2.md §7 Sub-mech A (trigger gap fix — slash + binary trigger structure) + §0.1 Luật 1 (`[gate]` mechanical).
