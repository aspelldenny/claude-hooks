# phieu.sh — Phiếu (ticket) workflow for SOS Kit
# Source from your shell rc:
#   bash:  echo 'source ~/sos-kit/phieu/phieu.sh' >> ~/.bashrc
#   zsh:   echo 'source ~/sos-kit/phieu/phieu.sh' >> ~/.zshrc
# Requires: bash 4.0+ (associative arrays) or zsh 5+.
#
# Commands:
#   phieu <slug>                      # auto-detect project from cwd, type=feat
#   phieu <type> <slug>               # auto-detect, explicit type
#   phieu <project> <slug>            # explicit project (from anywhere)
#   phieu <project> <type> <slug>     # explicit full
#   phieu-list [project]              # list worktrees (empty = all projects)
#   phieu-done [project] <P-slug>     # remove worktree
#   phieu-sync [project] <P-slug>     # rebase worktree on latest origin/main
#   phieu-init <project-path>         # onboard a new project (creates counter + wt dir)
#
# Ticket naming: <type>/P<NNN>-<slug>  where type ∈ feat|fix|chore|docs|infra

# --- Projects registry (populated by phieu-init) ---
# Users can also manually add: PHIEU_PROJECTS[name]="/path/to/project"
typeset -gA PHIEU_PROJECTS 2>/dev/null || declare -gA PHIEU_PROJECTS

# --- Helper: get keys of PHIEU_PROJECTS (cross-shell bash/zsh) ---
_phieu_keys() {
  if [ -n "$ZSH_VERSION" ]; then
    eval 'printf "%s\n" "${(@k)PHIEU_PROJECTS}"'
  else
    printf "%s\n" "${!PHIEU_PROJECTS[@]}"
  fi
}

# --- Helper: detect project from cwd ---
_phieu_detect_project() {
  local cwd="$PWD"
  local key
  while IFS= read -r key; do
    [ -z "$key" ] && continue
    local root="${PHIEU_PROJECTS[$key]}"
    local wt="${root}-wt"
    if [[ "$cwd" == "$root" || "$cwd" == "$root"/* || "$cwd" == "$wt"/* ]]; then
      echo "$key"
      return 0
    fi
  done < <(_phieu_keys)
  return 1
}

# --- Core: create phiếu + worktree ---
_phieu_impl() {
  local cmd="$1"
  local root="$2"
  local wt_parent="$3"
  shift 3

  local type slug
  if [ $# -eq 0 ]; then
    echo "Usage:"
    echo "  ${cmd} <slug>              # default type=feat"
    echo "  ${cmd} <type> <slug>       # type ∈ feat|fix|chore|docs|infra"
    return 1
  fi
  if [ $# -eq 1 ]; then
    type="feat"
    slug="$1"
  else
    type="$1"
    slug="$2"
  fi
  case "$type" in
    feat|fix|chore|docs|infra) ;;
    *)
      echo "❌ Invalid type: $type. Must be one of: feat, fix, chore, docs, infra"
      return 1
      ;;
  esac
  if ! [[ "$slug" =~ ^[a-z0-9][a-z0-9-]*$ ]]; then
    echo "❌ Slug invalid: '$slug'. Must be kebab-case (lowercase letters, digits, hyphens)"
    return 1
  fi

  local counter_file="${root}/.phieu-counter"
  [ ! -f "$counter_file" ] && echo "0" > "$counter_file"
  local n=$(cat "$counter_file")
  n=$((n + 1))
  local id=$(printf "P%03d" "$n")
  local branch="${type}/${id}-${slug}"
  local wt_name="${id}-${slug}"
  local wt_dir="${wt_parent}/${wt_name}"
  local ticket_file="docs/ticket/${id}-${slug}.md"

  echo "📝 Creating phiếu ${id} (project: $(basename "$root"))"
  echo "   Branch:   ${branch}"
  echo "   Worktree: ${wt_dir}"
  echo "   Ticket:   ${ticket_file}"
  echo ""

  cd "$root" || return 1
  echo "🔄 Fetching origin/main (base for new phiếu)..."
  git fetch origin main --quiet 2>/dev/null
  local base_ref="origin/main"
  if ! git rev-parse --verify "$base_ref" >/dev/null 2>&1; then
    echo "⚠️  origin/main not found, falling back to local main"
    base_ref="main"
  fi
  if ! git worktree add "$wt_dir" -b "$branch" "$base_ref"; then
    echo "❌ git worktree add failed — counter not incremented"
    return 1
  fi
  echo "$n" > "$counter_file"

  # Copy .env files if they exist (common for JS projects); silently skip if absent
  cp "${root}/.env" "${root}/.env.local" "$wt_dir/" 2>/dev/null

  cd "$wt_dir" || return 1

  # Create ticket from template — search in order: TICKET_TEMPLATE.md, TEMPLATE.md
  local template=""
  [ -f "docs/ticket/TICKET_TEMPLATE.md" ] && template="docs/ticket/TICKET_TEMPLATE.md"
  [ -z "$template" ] && [ -f "docs/ticket/TEMPLATE.md" ] && template="docs/ticket/TEMPLATE.md"
  if [ -n "$template" ]; then
    cp "$template" "${ticket_file}"
    sed -i.bak "1s|.*|# PHIẾU ${id}: ${slug}|" "${ticket_file}"
    rm -f "${ticket_file}.bak"
    echo "✅ Ticket created: ${ticket_file}"
  else
    echo "⚠️  No template found (docs/ticket/TICKET_TEMPLATE.md or TEMPLATE.md)"
    echo "    → ticket file NOT pre-created. Write it manually at ${ticket_file}"
  fi

  # Install dependencies based on detected stack
  if [ -f "pnpm-lock.yaml" ]; then
    echo "📦 pnpm install..."
    pnpm install || {
      echo "⚠️  pnpm install failed — fix it and run 'claude' in $wt_dir"
      return 1
    }
  elif [ -f "package-lock.json" ]; then
    echo "📦 npm install..."
    npm install || {
      echo "⚠️  npm install failed — fix it and run 'claude' in $wt_dir"
      return 1
    }
  elif [ -f "Cargo.toml" ]; then
    echo "📦 Rust project — skipping install (run cargo build/test when needed)"
  elif [ -f "requirements.txt" ] || [ -f "pyproject.toml" ]; then
    echo "📦 Python project — skipping install (activate venv when needed)"
  else
    echo "📦 Package manager not detected — skipping install"
  fi

  echo ""
  echo "🚀 Launching Claude Code in ${wt_dir}"
  if command -v claude >/dev/null 2>&1; then
    claude
  else
    echo "   (claude CLI not found — open your editor in ${wt_dir} manually)"
  fi
}

_phieu_done_impl() {
  local cmd="$1"
  local root="$2"
  local wt_parent="$3"
  shift 3
  if [ -z "$1" ]; then
    echo "Usage: ${cmd} <P042-slug>"
    echo "Example: ${cmd} P042-user-export"
    return 1
  fi
  local name="$1"
  cd "$root" || return 1

  # Extract phiếu ID (e.g., P038-foo → P038)
  local phieu_id
  phieu_id=$(echo "$name" | grep -oE '^P[0-9]+')
  if [ -z "$phieu_id" ]; then
    echo "❌ Could not extract phiếu ID from '$name' (expected P<NNN>-<slug>)"
    return 1
  fi

  # 1. Detect phiếu file location (unified: docs/ticket/; phieu/active/ legacy fallback)
  local active_path="" done_path=""
  if [ -f "docs/ticket/${name}.md" ]; then
    active_path="docs/ticket/${name}.md"
    done_path="docs/ticket/done/${name}.md"
    mkdir -p "docs/ticket/done"
  elif [ -f "phieu/active/${name}.md" ]; then
    active_path="phieu/active/${name}.md"
    done_path="phieu/done/${name}.md"
    mkdir -p "phieu/done"
  else
    echo "⚠️  Phiếu file not found at docs/ticket/${name}.md or phieu/active/${name}.md"
    echo "    Skipping strip + move; will still remove worktree + branch."
  fi

  # 2. Strip Debate Log + move active → done
  if [ -n "$active_path" ]; then
    # awk: keep all lines EXCEPT "### Turn N — Worker Challenge" / "### Turn N — Architect Response" subsections.
    # Preserve: header, Context, Task 0, "## Debate Log" header itself, "**Phiếu version:**" line,
    #          "### Final consensus" subsection, Nhiệm vụ, Files, Constraints, Nghiệm thu.
    awk '
      BEGIN { skip = 0 }
      /^### Turn [0-9]+ — Worker Challenge/ { skip = 1; next }
      /^### Turn [0-9]+ — Architect Response/ { skip = 1; next }
      /^### Final consensus/ { skip = 0 }
      /^---$/ { skip = 0 }
      /^## / { skip = 0 }
      skip == 0 { print }
    ' "$active_path" > "${active_path}.stripped"

    if [ -s "${active_path}.stripped" ]; then
      mv "${active_path}.stripped" "$done_path"
      rm -f "$active_path"
      echo "✅ Phiếu moved + Debate Log stripped: $active_path → $done_path"
    else
      rm -f "${active_path}.stripped"
      echo "⚠️  Strip produced empty file — leaving original at $active_path untouched"
    fi
  fi

  # 3. Remove worktree
  if git worktree remove "${wt_parent}/$name" 2>/dev/null; then
    echo "✅ Worktree removed: ${wt_parent}/$name"
  else
    echo "⚠️  Worktree remove failed (already gone? uncommitted changes?) — continuing"
  fi

  # 4. Delete local branch (safe -d only, NOT -D force)
  # Detect branch by listing branches matching the phiếu ID
  local branch
  branch=$(git branch --list "*/${name}" "*/${phieu_id}-*" 2>/dev/null | head -1 | sed 's/^[* ] //' | tr -d ' ')
  if [ -n "$branch" ]; then
    if git branch -d "$branch" 2>/dev/null; then
      echo "✅ Branch deleted (safe): $branch"
    else
      echo "⚠️  Branch '$branch' not fully merged — keeping it (use 'git branch -D $branch' manually if intentional)"
    fi
  else
    echo "ℹ️  No matching local branch found for $phieu_id — skipping branch delete"
  fi

  # 5. Cleanup .backup/<phieu-id>/
  if [ -d ".backup/${phieu_id}" ]; then
    rm -rf ".backup/${phieu_id}"
    echo "✅ Backup cleaned: .backup/${phieu_id}/"
  fi

  echo ""
  echo "🎉 phieu-done complete for $phieu_id"
}

_phieu_list_impl() {
  local root="$1"
  local counter="${root}/.phieu-counter"
  echo "Active worktrees ($(basename "$root")):"
  cd "$root" && git worktree list
  local next=$(($(cat "$counter" 2>/dev/null || echo 0) + 1))
  printf "\nNext phiếu ID: P%03d\n" "$next"
}

# --- Main commands ---
phieu() {
  local project
  if [ $# -ge 1 ] && [[ -n "${PHIEU_PROJECTS[$1]}" ]]; then
    project="$1"
    shift
  fi
  [ -z "$project" ] && project=$(_phieu_detect_project)

  if [ -z "$project" ]; then
    echo "❌ Could not detect project from cwd '$PWD'."
    echo "Usage:"
    echo "  phieu <project> <slug>              # explicit (from anywhere)"
    echo "  cd ~/<project> && phieu <slug>      # auto-detect"
    if [ "${#PHIEU_PROJECTS[@]}" -gt 0 ]; then
      echo "Registered projects: $(_phieu_keys | tr '\n' ' ')"
    else
      echo "No projects registered. Run: phieu-init <project-path>"
    fi
    return 1
  fi

  if [ $# -eq 0 ]; then
    echo "Usage (current project: $project):"
    echo "  phieu <slug>              # default type=feat"
    echo "  phieu <type> <slug>       # type ∈ feat|fix|chore|docs|infra"
    echo "  phieu $project <slug>     # explicit from outside project"
    return 1
  fi

  local root="${PHIEU_PROJECTS[$project]}"
  _phieu_impl "phieu $project" "$root" "${root}-wt" "$@"
}

phieu-list() {
  local project
  if [ $# -ge 1 ] && [[ -n "${PHIEU_PROJECTS[$1]}" ]]; then
    project="$1"
  else
    project=$(_phieu_detect_project)
  fi

  if [ -z "$project" ]; then
    if [ "${#PHIEU_PROJECTS[@]}" -eq 0 ]; then
      echo "No projects registered. Run: phieu-init <project-path>"
      return 0
    fi
    echo "📋 Registered projects:"
    local p
    while IFS= read -r p; do
      [ -z "$p" ] && continue
      local root="${PHIEU_PROJECTS[$p]}"
      local next=$(($(cat "$root/.phieu-counter" 2>/dev/null || echo 0) + 1))
      local wt_count=$(cd "$root" 2>/dev/null && git worktree list 2>/dev/null | wc -l | tr -d ' ')
      printf "  %-20s  %d worktree(s), next ID: P%03d\n" "$p" "$wt_count" "$next"
    done < <(_phieu_keys)
    echo ""
    echo "Run 'phieu-list <project>' for project detail."
    return 0
  fi

  _phieu_list_impl "${PHIEU_PROJECTS[$project]}"
}

phieu-done() {
  local project
  if [ $# -ge 1 ] && [[ -n "${PHIEU_PROJECTS[$1]}" ]]; then
    project="$1"
    shift
  fi
  [ -z "$project" ] && project=$(_phieu_detect_project)

  if [ -z "$project" ] || [ $# -eq 0 ]; then
    echo "Usage:"
    echo "  phieu-done <P042-slug>                # auto-detect project"
    echo "  phieu-done <project> <P042-slug>      # explicit"
    return 1
  fi

  local root="${PHIEU_PROJECTS[$project]}"
  _phieu_done_impl "phieu-done $project" "$root" "${root}-wt" "$@"
}

phieu-sync() {
  local project
  if [ $# -ge 1 ] && [[ -n "${PHIEU_PROJECTS[$1]}" ]]; then
    project="$1"
    shift
  fi
  [ -z "$project" ] && project=$(_phieu_detect_project)

  if [ -z "$project" ] || [ $# -eq 0 ]; then
    echo "Usage:"
    echo "  phieu-sync <P042-slug>                # auto-detect project"
    echo "  phieu-sync <project> <P042-slug>      # explicit"
    echo ""
    echo "What it does: rebase the worktree's branch onto latest origin/main"
    echo "              so your phiếu stays current with main's changes."
    return 1
  fi

  local name="$1"
  local root="${PHIEU_PROJECTS[$project]}"
  local wt_dir="${root}-wt/${name}"

  if [ ! -d "$wt_dir" ]; then
    echo "❌ Worktree not found: $wt_dir"
    return 1
  fi

  echo "🔄 Syncing worktree $wt_dir with origin/main..."
  cd "$wt_dir" || return 1

  # Fetch latest origin/main
  if ! git fetch origin main --quiet 2>/dev/null; then
    echo "⚠️  git fetch failed — check network"
    return 1
  fi

  # Check clean working tree (rebase needs clean)
  if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "⚠️  Working tree has uncommitted changes. Commit or stash first."
    git status --short
    return 1
  fi

  # Attempt rebase
  if git rebase origin/main; then
    echo "✅ Rebased onto origin/main successfully."
    echo "   Run tests to confirm nothing broke, then continue."
  else
    echo ""
    echo "⚠️  Rebase hit conflict(s). NOT auto-resolving."
    echo ""
    echo "Options:"
    echo "  1. Resolve manually:"
    echo "     - Edit conflict files (git status shows which)"
    echo "     - git add <files>"
    echo "     - git rebase --continue"
    echo "  2. Abort and stay on old base:"
    echo "     - git rebase --abort"
    echo ""
    echo "You're now in ${wt_dir} — fix or abort from here."
    return 1
  fi
}

# --- Onboard a new project ---
phieu-init() {
  if [ -z "$1" ]; then
    echo "Usage: phieu-init <project-path>"
    echo "Example: phieu-init ~/my-project"
    return 1
  fi
  local path="${1%/}"
  path="${path/#\~/$HOME}"
  if [ ! -d "$path/.git" ]; then
    echo "❌ '$path' is not a git repo (.git/ not found)"
    return 1
  fi
  local name=$(basename "$path")
  if [[ -n "${PHIEU_PROJECTS[$name]}" ]]; then
    echo "⚠️  Project '$name' already registered at: ${PHIEU_PROJECTS[$name]}"
    return 1
  fi

  # Counter
  if [ ! -f "$path/.phieu-counter" ]; then
    echo "0" > "$path/.phieu-counter"
    echo "✓ Created $path/.phieu-counter = 0"
  fi
  # Worktree parent
  if [ ! -d "${path}-wt" ]; then
    mkdir -p "${path}-wt"
    echo "✓ Created ${path}-wt/"
  fi
  # Gitignore
  if [ -f "$path/.gitignore" ] && ! grep -q "^\.phieu-counter" "$path/.gitignore"; then
    printf "\n# Phiếu ID counter (local, per-machine)\n.phieu-counter\n" >> "$path/.gitignore"
    echo "✓ Added .phieu-counter to $path/.gitignore"
  fi
  # Register in current session + persist to detected shell rc
  PHIEU_PROJECTS[$name]="$path"
  local rc_file
  if [ -n "$ZSH_VERSION" ]; then
    rc_file="$HOME/.zshrc"
  elif [ -n "$BASH_VERSION" ]; then
    rc_file="$HOME/.bashrc"
  else
    rc_file="$HOME/.profile"
  fi
  printf "\n# Added by phieu-init\nPHIEU_PROJECTS[%s]=\"%s\"\n" "$name" "$path" >> "$rc_file"
  echo "✓ Appended to $rc_file: PHIEU_PROJECTS[$name]=\"$path\""
  echo ""
  echo "🎉 Project '$name' is ready. Try:"
  echo "   phieu $name <slug>"
  echo "   phieu-list $name"
}
