# Phiếu — Ticket Workflow for SOS Kit

> "Phiếu" (Vietnamese for "ticket") is the spine that connects the **Architect** and **Worker** layers. Every non-trivial change goes through one.

This folder contains:
- `TICKET_TEMPLATE.md` — the required format for every phiếu
- `phieu.sh` — shell function that creates worktree + branch + ticket file in one command
- `DISCOVERY_PROTOCOL.md` — how Worker reports back to Architect when phiếu assumptions miss reality

## Why phiếu exists

In a team, tickets live in Jira/Linear and conversations happen in Slack. A solo dev has neither. If the thing-being-built is only in your head, it gets half-built, forgotten, or re-done three weeks later.

Phiếu solves this with three constraints:

1. **A phiếu is a file** — `docs/ticket/P<NNN>-<slug>.md` — written by Architect BEFORE code starts. It's the single source of truth for "what we're building." If the phiếu isn't written, the work isn't scoped.
2. **Every phiếu has a unique ID** — `P<NNN>` auto-assigned from a per-project counter. IDs never collide, can be referenced in commits ("`P042 — fix user export`"), and give you a chronological record of every piece of work.
3. **Every phiếu starts with Task 0: Verification Anchors** — grep every assumption (file exists, function signature, constant name) against real code BEFORE writing the rest of the phiếu. This single rule prevents 80% of "the phiếu was wrong" problems.

See [`../docs/LAYERS.md`](../docs/LAYERS.md) for how phiếu fits in the 3-role model, and [`../docs/HANDOFF.md`](../docs/HANDOFF.md) for the handoff format.

## First-time checklist (new user, ~5 minutes)

Clone sos-kit, then work through this list once per machine:

- [ ] **1.** Clone sos-kit somewhere permanent: `git clone https://github.com/aspelldenny/sos-kit.git ~/tools/sos-kit`
- [ ] **2.** Add `source ~/tools/sos-kit/phieu/phieu.sh` to your `~/.zshrc` (or `~/.bashrc`), then `source ~/.zshrc`
- [ ] **3.** Verify installed: `phieu-list` (should show "No projects registered")
- [ ] **4.** Onboard your first project: `phieu-init ~/my-project`
- [ ] **5.** Copy the ticket template into the project: `cp ~/tools/sos-kit/phieu/TICKET_TEMPLATE.md ~/my-project/docs/ticket/TICKET_TEMPLATE.md`
- [ ] **6.** Create the Discoveries log: `touch ~/my-project/docs/DISCOVERIES.md` (add a header — see `phieu/DISCOVERY_PROTOCOL.md`)
- [ ] **7.** Copy vision doc skeletons: `cp ~/tools/sos-kit/phieu/VISION_TEMPLATES/*.md ~/my-project/docs/`
- [ ] **8.** Smoke-test: `phieu my-project chore test-setup` → should create worktree `~/my-project-wt/P001-test-setup/` with a pre-filled ticket file and launch Claude Code. Exit, then `phieu-done my-project P001-test-setup` to clean up.

If all 8 pass: ready for daily use. Skip to "Daily commands" below.

Full end-to-end setup including Rust tools (`ship`, `docs-gate`, `guard`, `vps`) and Claude skills: see [`../docs/SETUP.md`](../docs/SETUP.md).

## Setup

### 1. Source the shell function

Add to `~/.zshrc` (or `~/.bashrc`):

```bash
source /path/to/sos-kit/phieu/phieu.sh
```

Reload:

```bash
source ~/.zshrc
```

### 2. Onboard each project you want phiếu workflow on

```bash
phieu-init ~/my-project
```

This:
- Creates `~/my-project/.phieu-counter` (starting at 0)
- Creates `~/my-project-wt/` (worktree parent directory)
- Adds `.phieu-counter` to project `.gitignore` (counter is local per-machine, should not be committed)
- Registers `my-project` in `PHIEU_PROJECTS` map in your `~/.zshrc`

After onboarding, you can use `phieu` commands from anywhere (auto-detect) or explicitly (`phieu my-project <slug>`).

### 3. (Optional) Copy ticket template into the project

```bash
mkdir -p ~/my-project/docs/ticket
cp /path/to/sos-kit/phieu/TICKET_TEMPLATE.md ~/my-project/docs/ticket/TICKET_TEMPLATE.md
```

The `phieu` function auto-copies this template when creating a new ticket. Without it, the ticket file is not pre-created — you write it from scratch.

## Daily commands

```bash
# Create a new phiếu (inside or outside project dir)
phieu <slug>                      # default type=feat, auto-detect project
phieu fix <slug>                  # explicit type
phieu my-project feat user-export # explicit project + type

# List all active phiếu worktrees
phieu-list                        # if outside project: shows all registered projects
phieu-list my-project             # detailed list for one project

# Rebase worktree onto latest origin/main (run if main has moved and
# you want to stay current; halts cleanly on conflict for you to resolve)
phieu-sync P042-user-export
phieu-sync my-project P042-user-export

# Remove worktree when phiếu is done + merged
phieu-done P042-user-export
phieu-done my-project P042-user-export
```

## Naming convention

```
<type>/P<NNN>-<slug>
```

| Part | Values | Notes |
|---|---|---|
| `<type>` | `feat` / `fix` / `chore` / `docs` / `infra` | 5 fixed categories. `refactor` / `perf` / `test` / `design` → use `chore`. |
| `P<NNN>` | `P001`, `P042`, `P123`... | 3 digits, auto-assigned, never repeat. |
| `<slug>` | `user-export`, `login-redirect` | 2-4 words kebab-case (lowercase, digits, hyphens). |

**Examples of valid names:**
- `feat/P042-user-export`
- `fix/P043-login-redirect`
- `chore/P044-trim-docs`

**Filename:** `docs/ticket/P<NNN>-<slug>.md` (matches branch without the `<type>/` prefix)
**Worktree:** `~/<project>-wt/P<NNN>-<slug>/`

## Gotchas

- **Counter file is LOCAL per-machine, not committed.** If you clone fresh on a new machine, counter starts at 0 and may collide with existing IDs. Fix: `echo <N> > ~/my-project/.phieu-counter` to set manually (N = highest existing P-number + 1).
- **Branch base is `origin/main`.** The shell function auto-fetches before creating the worktree. If offline, falls back to local `main`.
- **Cannot check out the same branch in 2 worktrees.** Git blocks it. Remove one worktree first.
- **Worktrees do NOT share `node_modules`, `.env`, `.next/`.** Shell function auto-copies `.env` + `.env.local` and runs `pnpm install` / `npm install` for JS projects. Rust + Python stacks: skips install, you run `cargo build` / activate venv manually.
- **Max 2-3 phiếu in parallel per project** is practical. More = merge conflicts compound.

## Running parallel phiếu (the main reason worktrees exist)

```bash
# Terminal 1 (phiếu A on ui work)
phieu my-project feat header-redesign    # worktree P001, port 3001

# Terminal 2 (phiếu B on backend work)
phieu my-project fix rate-limit-bug      # worktree P002, port 3002

# Terminal 3 (main branch, unchanged)
cd ~/my-project && pnpm dev              # port 3000
```

Three Claude Code instances, three isolated file systems, one git repo. Work on all three simultaneously. Merge when done; git handles the diffs.

## When NOT to use phiếu

- **Same-day hotfix (< 30 min):** skip the phiếu file, just make the change + CHANGELOG entry. Still write Discovery Report if you discovered something.
- **Pure exploration ("can we even do X?"):** no phiếu. Kiến trúc sư answers Chủ nhà with findings + recommendation. Only write phiếu when Chủ nhà says "ok, build it."
- **Learning / research time:** no phiếu. Those don't ship.

Anything longer than those → use the full phiếu workflow. The overhead is small; the friction it prevents (scope creep, lost context, wrong assumptions) is large.
