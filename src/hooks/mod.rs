use crate::io::{self, ALLOW, BLOCK, Decision};
use regex::Regex;

// ── P010: architect_guard helpers (pure, testable) ────────────────────────────

/// Returns true if path `p` (already stripped of leading `./`) is in the forbidden
/// Read/Glob zone. Port verbatim oracle tarot L39-50.
fn is_forbidden_for_read(p: &str) -> bool {
    // Source dirs: prefix or segment
    if p.starts_with("src/") || p.starts_with("lib/") || p.starts_with("app/") || p.starts_with("pkg/") {
        return true;
    }
    if p.contains("/src/") || p.contains("/lib/") || p.contains("/app/") || p.contains("/pkg/") {
        return true;
    }
    // crates/*/src/* (segment form)
    if p.starts_with("crates/") && p.contains("/src/") {
        return true;
    }
    // prisma/ — NEW vs old port (oracle L43)
    if p.starts_with("prisma/") || p.contains("/prisma/") {
        return true;
    }
    // Test dirs
    if p.starts_with("tests/") || p.starts_with("test/") || p.starts_with("__tests__/") {
        return true;
    }
    if p.contains("/tests/") || p.contains("/test/") {
        return true;
    }
    // Build artifacts (prefix only)
    if p.starts_with("node_modules/") || p.starts_with("target/") || p.starts_with("dist/")
        || p.starts_with("build/") || p.starts_with(".next/") || p.starts_with(".nuxt/")
        || p.starts_with(".svelte-kit/")
    {
        return true;
    }
    // Code extensions
    if p.ends_with(".rs") || p.ends_with(".ts") || p.ends_with(".tsx")
        || p.ends_with(".js") || p.ends_with(".jsx") || p.ends_with(".py")
        || p.ends_with(".go") || p.ends_with(".java") || p.ends_with(".cpp")
        || p.ends_with(".c") || p.ends_with(".h") || p.ends_with(".hpp")
    {
        return true;
    }
    // .prisma / .sql — NEW vs old port (oracle L47)
    if p.ends_with(".prisma") || p.ends_with(".sql") {
        return true;
    }
    false
}

/// Returns true if path `p` (already stripped of leading `./`) is in the Architect Write
/// allowlist. Port verbatim oracle tarot L54-61. ORDER MATTERS: deny TICKET_TEMPLATE first.
fn is_allowed_for_write(p: &str) -> bool {
    // Explicit deny — template is reference, not a phiếu (defense-in-depth, oracle L57)
    if p == "docs/ticket/TICKET_TEMPLATE.md" {
        return false;
    }
    // Allow phiếu files: docs/ticket/P*-*.md or */docs/ticket/P*-*.md (oracle L58)
    // Pattern: prefix literal "docs/ticket/P", has at least one '-', ends with ".md"
    let is_phieu = |s: &str| -> bool {
        let short = s.strip_prefix("docs/ticket/P").is_some();
        let long = s.contains("/docs/ticket/P");
        (short || long)
            && s.contains('-')
            && s.ends_with(".md")
    };
    is_phieu(p)
}

/// block_read message — verbatim oracle tarot L65-76. `violator` = original candidate (pre-strip).
fn make_block_read_msg(violator: &str) -> String {
    format!(
        "🚫 Architect envelope violation (Read/Glob)\n\nArchitect cannot read source code: {violator}\n\nWhat to do instead: write a Task 0 anchor in the phiếu.\nExample:\n  | # | Assumption | Verify by | Result |\n  | 1 | <claim about {violator}> | grep ... {violator} | ⏳ TO VERIFY |\n\nWorker (separate subagent) will grep-verify it for you. The constraint IS the feature."
    )
}

/// block_write message — verbatim oracle tarot L82-92. `violator` = file_path.
fn make_block_write_msg(violator: &str) -> String {
    format!(
        "🚫 Architect envelope violation (Write/Edit)\n\nArchitect cannot Write/Edit: {violator}\n\nArchitect's Write allowlist (per architect.md line 32):\n  - docs/ticket/P*-*.md  (phiếu files only)\n\nEverything else (src/, CLAUDE.md, BACKLOG.md, CHANGELOG.md, guides) belongs to Worker.\nIf a phiếu needs to update those files, encode it as a Worker Task in the phiếu."
    )
}

/// Core: marker gate + tool_name dispatch + forbidden/allowed path check.
/// Reads fs (marker + CLAUDE_PROJECT_DIR) but does NOT read stdin, print, or exit.
/// Returns Decision. Port verbatim oracle tarot 119-line.
///
/// Signature extended P010: tool_name dispatch (Read/Glob vs Write/Edit).
/// Divergence from tarot oracle (intentional):
///   - Marker path: `.sos-state/architect-active` (binary convention, F-005 defer).
///     Oracle tarot uses `.claude/.architect-active` (L22). KHÔNG đổi ở P010.
pub fn architect_guard_decide(
    tool_name: Option<&str>,
    file_path: Option<&str>,
    pattern: Option<&str>,
    path: Option<&str>,
) -> Decision {
    // Step 1 — resolve repo root from CLAUDE_PROJECT_DIR (fallback: cwd).
    let repo_root = std::env::var("CLAUDE_PROJECT_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());

    // Step 2 — marker gate. Oracle L25.
    let marker = repo_root.join(".sos-state/architect-active");
    if !marker.exists() {
        return Decision { exit_code: ALLOW, blocked: false, reason: None };
    }

    // Step 3 — tool_name dispatch. Oracle L96-116.
    match tool_name {
        Some("Read") | Some("Glob") => {
            // Read/Glob branch: check 3 candidates. Oracle L98-107.
            for candidate in [file_path, pattern, path].iter().flatten() {
                if candidate.is_empty() {
                    continue;
                }
                // Strip leading "./"
                let np = candidate.strip_prefix("./").unwrap_or(candidate);
                // .md early-allow (oracle L103)
                if np.ends_with(".md") {
                    continue;
                }
                if is_forbidden_for_read(np) {
                    return Decision {
                        exit_code: BLOCK,
                        blocked: true,
                        reason: Some(make_block_read_msg(candidate)),
                    };
                }
            }
            Decision { exit_code: ALLOW, blocked: false, reason: None }
        }
        Some("Write") | Some("Edit") => {
            // Write/Edit branch. Oracle L109-115.
            let fp = match file_path {
                Some(f) if !f.is_empty() => f,
                // No path → can't validate, allow (defensive — oracle L111)
                _ => return Decision { exit_code: ALLOW, blocked: false, reason: None },
            };
            let np = fp.strip_prefix("./").unwrap_or(fp);
            if !is_allowed_for_write(np) {
                return Decision {
                    exit_code: BLOCK,
                    blocked: true,
                    reason: Some(make_block_write_msg(fp)),
                };
            }
            Decision { exit_code: ALLOW, blocked: false, reason: None }
        }
        // Default: no tool_name or unknown tool → allow. Oracle L118 default.
        _ => Decision { exit_code: ALLOW, blocked: false, reason: None },
    }
}

/// CLI wrapper — reads stdin, calls _decide with 4 args (P010), prints stderr, returns exit code.
pub fn architect_guard() -> i32 {
    let p = io::read_payload();
    let d = architect_guard_decide(
        p.tool_name.as_deref(),
        p.tool_input.file_path.as_deref(),
        p.tool_input.pattern.as_deref(),
        p.tool_input.path.as_deref(),
    );
    if let Some(ref r) = d.reason { eprintln!("{r}"); }
    d.exit_code
}

/// Core: check if editing a .env* file (not .env.example) should be blocked.
/// Does NOT read stdin, print, or exit. Returns Decision.
pub fn block_env_edit_decide(file_path: Option<&str>, notebook_path: Option<&str>) -> Decision {
    // Steps 2-4 — parse path: file_path priority, fallback notebook_path (NotebookEdit).
    let path = match file_path.or(notebook_path) {
        Some(p) if !p.is_empty() => p.to_owned(),
        _ => return Decision { exit_code: ALLOW, blocked: false, reason: None },
    };

    // Step 5 — basename. Oracle L38: BASE=$(basename "$FILE_PATH").
    let base = path.rsplit('/').next().unwrap_or(&path);

    // Step 6 — allowlist: .env.example is a template, allow edit. Oracle L41.
    if base == ".env.example" {
        return Decision { exit_code: ALLOW, blocked: false, reason: None };
    }

    // Step 7 — block regex ^\.env($|\.). Oracle L44.
    let re = Regex::new(r"^\.env($|\.)").unwrap();
    if re.is_match(base) {
        let msg = format!(
            "⛔ BLOCKED: Edit/Write tới {path} bị chặn.\n\n\
             Lý do: .env* file chứa secret thật (API keys, DB credentials, webhook tokens).\n\
             KHÔNG sửa qua Claude tool — risk leak vào prompt/context/log.\n\n\
             Cách hợp lệ:\n\
             \x20 - Sửa .env.example (template, không secret thật)\n\
             \x20 - Sếp paste secret thật vào .env tay (local-only edit)\n\
             \x20 - Sửa qua SSH/console nếu là production env\n\n\
             Override (nếu thật sự cần Claude edit .env, hiếm):\n\
             \x20 - Tạm rename .env → .env.draft, edit, rename back\n\
             \x20 - Hoặc remove hook khỏi .claude/settings.json (PR review trước)"
        );
        return Decision { exit_code: BLOCK, blocked: true, reason: Some(msg) };
    }

    Decision { exit_code: ALLOW, blocked: false, reason: None }
}

/// CLI wrapper — reads stdin, calls _decide, prints stderr if blocked, returns exit code.
pub fn block_env_edit() -> i32 {
    let payload = io::read_payload();
    let d = block_env_edit_decide(
        payload.tool_input.file_path.as_deref(),
        payload.tool_input.notebook_path.as_deref(),
    );
    if let Some(ref r) = d.reason { eprintln!("{r}"); }
    d.exit_code
}

// ── P004: block-unsafe-merge helpers (pure, testable without gh) ──────────────

/// Parse `gh pr merge <N>` from a command string. Returns PR number if present.
/// Known limitation (oracle L15-16): branch-only form `gh pr merge --merge` (no
/// number) → None. This is intentional — hook only handles numbered form.
fn parse_merge_pr(command: &str) -> Option<u32> {
    // Oracle L41: match `gh pr merge[[:space:]]+[0-9]+`
    let check_re = Regex::new(r"gh pr merge\s+\d+").ok()?;
    if !check_re.is_match(command) {
        return None;
    }
    // Oracle L46: extract first numeric after `gh pr merge`
    let extract_re = Regex::new(r"gh pr merge\s+(\d+)").ok()?;
    let caps = extract_re.captures(command)?;
    caps.get(1)?.as_str().parse::<u32>().ok()
}

/// Extract `[security-review-skip:<reason>]` from a command string.
/// Returns the reason string if found. Oracle L50-51.
fn extract_skip_marker(command: &str) -> Option<String> {
    // Oracle L50: `\[security-review-skip:[^]]+\]`
    let re = Regex::new(r"\[security-review-skip:([^\]]+)\]").ok()?;
    let caps = re.captures(command)?;
    Some(caps.get(1)?.as_str().to_owned())
}

/// Outcome of checking for a security-review verdict in PR comments.
#[derive(Debug, PartialEq)]
enum VerdictResult {
    /// No `<!-- security-review-start -->` marker in comments → not required to have a review.
    NoBlock,
    /// Review present and verdict is APPROVE.
    Approve,
    /// Review present but verdict is NEEDS_REVIEW or unknown/missing.
    NeedsReview,
}

/// Check PR comments for a security-review verdict. Oracle L97-104.
/// `comments` = output of `gh pr view <N> --json comments --jq '.comments[].body'`.
fn verdict_is_approve(comments: &str) -> VerdictResult {
    // Oracle L97: check for `<!-- security-review-start -->` marker
    if !comments.contains("<!-- security-review-start -->") {
        return VerdictResult::NoBlock;
    }
    // Oracle L99: grep -A 50 marker | grep -E '^Verdict:' | head -1
    // Take up to 50 lines after the marker, find first `^Verdict:` line.
    let marker = "<!-- security-review-start -->";
    let after_marker = match comments.find(marker) {
        Some(pos) => &comments[pos + marker.len()..],
        None => return VerdictResult::NoBlock,
    };
    let verdict_line = after_marker
        .lines()
        .take(50)
        .find(|line| line.starts_with("Verdict:"));
    match verdict_line {
        Some(line) if line.contains("APPROVE") => VerdictResult::Approve,
        _ => VerdictResult::NeedsReview,
    }
}

/// Check whether the diff file list touches the security surface.
/// `files` = newline-separated output of `gh pr diff <N> --name-only`.
/// `extra` = optional extra pattern from `SECURITY_SURFACE_EXTRA` env var.
///
/// Two branches mirror oracle L86-93:
///   (a) base/extended pattern matches any line (oracle L86 `grep -qE PATTERN`)
///   (b) pattern does NOT match but a `^\.env` non-example file is present
///       (oracle L88-89: `grep -E '^\.env' | grep -v '\.env\.example'` non-empty)
///
/// Note: `\.env[^.]` in base pattern matches `.env` followed by a non-dot character
/// (e.g. `.env\n`, `.envlocal`), but NOT `.env.local` (followed by `.`). Branch (b)
/// deliberately catches `.env.local` and similar. Do NOT merge branches — this is a
/// faithful port of the oracle's two-pass check.
fn touches_security_surface(files: &str, extra: Option<&str>) -> bool {
    // Oracle L60 — VERBATIM base pattern (copy exact, do not modify any character)
    let base = r"src/|schema\.(prisma|sql)|migrations?/|nginx/|docker-compose.*\.yml|Dockerfile|\.env[^.]|middleware\.|lib/auth/|\.claude/agents/security-|docs/security/|scripts/security-gate|scripts/check-(hardcoded|runtime)-secrets|hooks/pre-commit";

    // Oracle L63-65: extend pattern per-repo via SECURITY_SURFACE_EXTRA
    let extended: String = match extra {
        Some(e) if !e.is_empty() => format!("{}|{}", base, e),
        _ => base.to_owned(),
    };

    // Oracle L86: grep -qE PATTERN (multiline, any line matches)
    // Use Regex with (?m) for line-anchored matching if needed; since base patterns
    // do not use ^ or $, a simple per-line scan is cleaner and avoids multiline quirks.
    let re = match Regex::new(&extended) {
        Ok(r) => r,
        Err(_) => return false, // bad extra pattern → fail-open (safety)
    };

    // Branch (a): base/extended pattern matches any line
    let pattern_matched = files.lines().any(|line| re.is_match(line));

    if pattern_matched {
        return true;
    }

    // Branch (b): oracle L88-89 — `grep -E '^\.env' | grep -v '\.env\.example'`
    // If any line starts with `.env` AND is NOT `.env.example`, touch surface.
    files
        .lines()
        .filter(|line| line.starts_with(".env"))
        .any(|line| !line.contains(".env.example"))
}

/// Core: check whether a `gh pr merge` command should be blocked.
/// Makes real gh shell calls (fs/gh per phiếu doctrine). Does NOT read stdin, print, or exit.
/// FAIL-CLOSED: gh unavailable → blocked=true + reason explaining gh failure.
///
/// DIVERGENCE (INTENTIONAL — fail-CLOSED): gh fail or empty diff → BLOCK (exit 2),
/// unlike other 3 hooks which fail-open. Do NOT change to fail-open.
pub fn block_unsafe_merge_decide(command: Option<&str>) -> Decision {
    // Step 1: get command. Oracle L24-31.
    let command = match command {
        Some(c) if !c.is_empty() => c.to_owned(),
        _ => return Decision { exit_code: ALLOW, blocked: false, reason: None },
    };

    // Step 2: parse PR number. Oracle L41-47.
    let pr = match parse_merge_pr(&command) {
        Some(n) => n,
        None => return Decision { exit_code: ALLOW, blocked: false, reason: None },
    };
    let pr_str = pr.to_string();

    // Step 3: override marker check. Oracle L50-55.
    if let Some(reason) = extract_skip_marker(&command) {
        let msg = format!(
            "⚠️  Security review override marker detected for PR #{}. Reason: {}\n    Allowing merge. Sếp đã review tay — em (hook) không block.",
            pr_str, reason
        );
        // Override = ALLOW but with a warning reason (printed by CLI wrapper via eprintln)
        return Decision { exit_code: ALLOW, blocked: false, reason: Some(msg) };
    }

    // Step 4: read optional extra surface pattern. Oracle L63.
    let extra = std::env::var("SECURITY_SURFACE_EXTRA").ok();

    // Step 5: gh call #1 — `gh pr diff <PR> --name-only`. Oracle L67-83.
    let diff_output = std::process::Command::new("gh")
        .args(["pr", "diff", &pr_str, "--name-only"])
        .output();

    let diff_files = match diff_output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).into_owned()
        }
        _ => String::new(),
    };

    if diff_files.trim().is_empty() {
        let msg = format!(
            "⛔ BLOCKED: gh pr diff #{pr_str} thất bại (network/auth?).\n\n\
Em (hook) KHÔNG verify được PR có touch security surface không.\n\
Fail-safe: block merge để Sếp/Quản đốc kiểm tra tay.\n\n\
Cách hợp lệ:\n\
  - Kiểm tra gh auth status\n\
  - Chạy: gh pr diff {pr_str} --name-only\n\
  - Nếu confirm KHÔNG touch security surface → re-run merge với marker:\n\
      gh pr merge {pr_str} --merge [security-review-skip:gh-cli-unavailable]"
        );
        return Decision { exit_code: BLOCK, blocked: true, reason: Some(msg) };
    }

    // Step 6: check security surface. Oracle L86-93.
    if !touches_security_surface(&diff_files, extra.as_deref()) {
        return Decision { exit_code: ALLOW, blocked: false, reason: None };
    }

    // Step 7: gh call #2 — `gh pr view <PR> --json comments --jq '.comments[].body'`.
    let view_output = std::process::Command::new("gh")
        .args(["pr", "view", &pr_str, "--json", "comments", "--jq", ".comments[].body"])
        .output();

    let comments = match view_output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).into_owned()
        }
        _ => String::new(),
    };

    // Step 8: verdict check. Oracle L97-137.
    match verdict_is_approve(&comments) {
        VerdictResult::Approve => Decision { exit_code: ALLOW, blocked: false, reason: None },
        VerdictResult::NeedsReview => {
            let verdict_line = {
                let marker = "<!-- security-review-start -->";
                comments
                    .find(marker)
                    .map(|pos| &comments[pos + marker.len()..])
                    .and_then(|after| {
                        after
                            .lines()
                            .take(50)
                            .find(|l| l.starts_with("Verdict:"))
                            .map(|s| s.to_owned())
                    })
                    .unwrap_or_default()
            };
            let msg = format!(
                "⛔ BLOCKED: PR #{pr_str} touch security surface VÀ /security-review verdict KHÔNG phải APPROVE.\n\n\
Verdict line: {verdict_line}\n\n\
Hành động:\n\
  1. Sếp đọc comment giám sát trên PR #{pr_str}\n\
  2. Nếu Sếp accept risk → re-run với marker:\n\
     gh pr merge {pr_str} --merge [security-review-skip:sep-accepted-needs-review]\n\
  3. Nếu cần fix → spawn Worker EXECUTE fix theo INV flagged, push, gate sẽ re-fire"
            );
            Decision { exit_code: BLOCK, blocked: true, reason: Some(msg) }
        }
        VerdictResult::NoBlock => {
            let msg = format!(
                "⛔ BLOCKED: PR #{pr_str} touch security surface NHƯNG chưa có /security-review.\n\n\
Em (Quản đốc) suýt MISS triệu giám sát. Hook chặn để fix structural — KHÔNG dựa LLM remember.\n\n\
Hành động:\n\
  1. Chạy slash command (em tự gõ):\n\
     /security-review {pr_str}\n\
  2. Đợi @agent-boundary-check verdict (advisory, post comment trên PR)\n\
  3. Verdict APPROVE → re-run merge bình thường (hook sẽ allow)\n\
  4. Verdict NEEDS_REVIEW → Sếp đọc comment + quyết (re-run với marker nếu accept)\n\n\
Reference:\n\
  - PR: $(gh pr view {pr_str} --json url --jq .url)\n\
  - Doctrine: WORKFLOW_V2.2.md §7 Sub-mech A (trigger gap) + §8 (rubric inject)\n\
  - Slash: .claude/commands/security-review.md"
            );
            Decision { exit_code: BLOCK, blocked: true, reason: Some(msg) }
        }
    }
}

/// CLI wrapper — reads stdin, calls _decide, prints stderr (all reasons: warn + block), returns exit code.
pub fn block_unsafe_merge() -> i32 {
    let payload = io::read_payload();
    let d = block_unsafe_merge_decide(payload.tool_input.command.as_deref());
    if let Some(ref r) = d.reason { eprintln!("{r}"); }
    d.exit_code
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_merge_pr ──────────────────────────────────────────────────────────

    #[test]
    fn parse_merge_pr_numbered_squash() {
        assert_eq!(parse_merge_pr("gh pr merge 42 --squash"), Some(42));
    }

    #[test]
    fn parse_merge_pr_numbered_merge_delete() {
        assert_eq!(
            parse_merge_pr("gh pr merge 7 --merge --delete-branch"),
            Some(7)
        );
    }

    #[test]
    fn parse_merge_pr_branch_only_bypass() {
        // Known limitation (oracle L15-16): branch-only form → None (intentional)
        assert_eq!(parse_merge_pr("gh pr merge --merge"), None);
    }

    #[test]
    fn parse_merge_pr_non_merge_command() {
        assert_eq!(parse_merge_pr("echo hi"), None);
    }

    #[test]
    fn parse_merge_pr_pr_view_not_merge() {
        // `gh pr view` is not `gh pr merge`
        assert_eq!(parse_merge_pr("gh pr view 42"), None);
    }

    // ── extract_skip_marker ─────────────────────────────────────────────────────

    #[test]
    fn extract_skip_marker_found() {
        assert_eq!(
            extract_skip_marker("gh pr merge 5 [security-review-skip:docs-only]"),
            Some("docs-only".to_owned())
        );
    }

    #[test]
    fn extract_skip_marker_not_found() {
        assert_eq!(extract_skip_marker("gh pr merge 5 --merge"), None);
    }

    #[test]
    fn extract_skip_marker_hyphenated_reason() {
        assert_eq!(
            extract_skip_marker("[security-review-skip:gh-cli-unavailable]"),
            Some("gh-cli-unavailable".to_owned())
        );
    }

    // ── touches_security_surface ────────────────────────────────────────────────

    #[test]
    fn touches_surface_src_file() {
        assert!(touches_security_surface("src/main.rs\nREADME.md", None));
    }

    #[test]
    fn touches_surface_readme_only_false() {
        assert!(!touches_security_surface("README.md\ndocs/x.md", None));
    }

    #[test]
    fn touches_surface_env_local_branch_b() {
        // .env.local does NOT match base `\.env[^.]` (followed by `.`), so must go via branch (b)
        assert!(touches_security_surface(".env.local", None));
    }

    #[test]
    fn touches_surface_env_example_false() {
        // .env.example only → grep -v removes it → branch (b) false; base pattern also false
        assert!(!touches_security_surface(".env.example", None));
    }

    #[test]
    fn touches_surface_bare_env_true() {
        // `.env\n` — base `\.env[^.]` matches `.env` followed by newline (non-dot) OR branch (b)
        assert!(touches_security_surface(".env\n", None));
    }

    #[test]
    fn touches_surface_dockerfile_true() {
        assert!(touches_security_surface("Dockerfile", None));
    }

    #[test]
    fn touches_surface_migrations_true() {
        assert!(touches_security_surface("migrations/001.sql", None));
    }

    #[test]
    fn touches_surface_pre_commit_true() {
        assert!(touches_security_surface("hooks/pre-commit", None));
    }

    #[test]
    fn touches_surface_extra_match() {
        assert!(touches_security_surface(
            "custom/secret.yml",
            Some("custom/")
        ));
    }

    #[test]
    fn touches_surface_extra_no_match_without_extra() {
        assert!(!touches_security_surface("custom/secret.yml", None));
    }

    // ── verdict_is_approve ──────────────────────────────────────────────────────

    #[test]
    fn verdict_approve() {
        let comments = "<!-- security-review-start -->\nVerdict: APPROVE\nsome text";
        assert_eq!(verdict_is_approve(comments), VerdictResult::Approve);
    }

    #[test]
    fn verdict_needs_review() {
        let comments = "<!-- security-review-start -->\nVerdict: NEEDS_REVIEW";
        assert_eq!(verdict_is_approve(comments), VerdictResult::NeedsReview);
    }

    #[test]
    fn verdict_no_marker() {
        let comments = "Some comment without the marker";
        assert_eq!(verdict_is_approve(comments), VerdictResult::NoBlock);
    }

    #[test]
    fn verdict_marker_no_verdict_line() {
        // Marker present but no `^Verdict:` line → NeedsReview
        let comments = "<!-- security-review-start -->\nSome review text\nNo verdict here";
        assert_eq!(verdict_is_approve(comments), VerdictResult::NeedsReview);
    }
}

// ── P005: session-banner helpers (pure, testable without fs/git/clock) ────────

/// Find the "Active sprint" block in a BACKLOG.md string.
///
/// Returns `Some((sprint_block, header_text, fallback_used))` or `None` if no
/// `^## ` heading exists at all. Mirrors oracle L24-51.
///
/// - `sprint_block`: from header line to the line before the next `^## ` (exclusive),
///   or to EOF if no next section. **Includes the header line itself** (oracle L51 `sed`
///   inclusive range).
/// - `header_text`: header line stripped of leading `## ` (oracle L38 `sed 's/^## *//'`).
/// - `fallback_used`: true when "Active sprint" not found and first `^## ` used instead.
fn find_sprint_block(backlog: &str) -> Option<(String, String, bool)> {
    let lines: Vec<&str> = backlog.lines().collect();

    // Oracle L25: grep -n "^## .*Active sprint" | head -1
    let active_idx = lines.iter().position(|l| {
        l.starts_with("## ") && l.contains("Active sprint")
    });

    let (header_idx, fallback_used) = if let Some(idx) = active_idx {
        (idx, false)
    } else {
        // Oracle L29-31: fallback to first "^## " line
        let idx = lines.iter().position(|l| l.starts_with("## "))?;
        (idx, true)
    };

    // Oracle L38: strip "^## *" prefix for header_text
    let header_text = lines[header_idx]
        .trim_start_matches('#')
        .trim_start()
        .to_owned();

    // Oracle L41 awk: find next "^## " AFTER header_idx
    let next_section_idx = lines
        .iter()
        .enumerate()
        .skip(header_idx + 1)
        .find(|(_, l)| l.starts_with("## "))
        .map(|(i, _)| i);

    // Oracle L44-48: end line = line before next section, or EOF
    let end_idx = match next_section_idx {
        Some(i) => i - 1, // inclusive end (last line of sprint block)
        None => lines.len().saturating_sub(1),
    };

    // Oracle L51: sed "${HEADER_LINE},${END_LINE}p" — inclusive header to end
    let block_lines = &lines[header_idx..=end_idx];
    let sprint_block = block_lines.join("\n");

    Some((sprint_block, header_text, fallback_used))
}

/// Count open (`^- [ ]`) and done (`^- [x]`) items in a sprint block.
/// Returns `(open_count, done_count)`. Oracle L55-56.
fn count_items(block: &str) -> (usize, usize) {
    let open = block.lines().filter(|l| l.starts_with("- [ ]")).count();
    let done = block.lines().filter(|l| l.starts_with("- [x]")).count();
    (open, done)
}

/// Parse ISO-8601 UTC string `"%Y-%m-%dT%H:%M:%SZ"` into an epoch second using
/// Hinnant days-from-civil algorithm (no external crate). Returns `None` on parse error.
///
/// `now_epoch` is injected so unit tests are deterministic (caller passes
/// `SystemTime::now()…as_secs() as i64` in production). Oracle L157-161.
fn staleness_days(iso: &str, now_epoch: i64) -> Option<i64> {
    // Accept both JSON-extracted and legacy-raw (same format either way after trim).
    let s = iso.trim();

    // Expected format: "2026-06-09T12:00:00Z"
    // Minimum length check: "2026-06-09T00:00:00Z" = 20 chars
    if s.len() < 20 {
        return None;
    }

    let year: i64 = s[0..4].parse().ok()?;
    let month: i64 = s[5..7].parse().ok()?;
    let day: i64 = s[8..10].parse().ok()?;
    // T at index 10 expected
    if s.as_bytes().get(10) != Some(&b'T') {
        return None;
    }
    let hour: i64 = s[11..13].parse().ok()?;
    let min: i64 = s[14..16].parse().ok()?;
    let sec: i64 = s[17..19].parse().ok()?;

    // Validate ranges
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&min)
        || !(0..=60).contains(&sec)
    {
        return None;
    }

    // Howard Hinnant days-from-civil (public domain) — UTC, no leap-second concern
    // at day granularity. Formula for (y, m, d) -> days since 1970-01-01.
    let y = year - if month <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // 0..=399
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days_since_epoch = era * 146097 + doe - 719468;

    let parsed_epoch = days_since_epoch * 86400 + hour * 3600 + min * 60 + sec;
    let days = (now_epoch - parsed_epoch) / 86400;
    Some(days)
}

/// Categorise advisory staleness. Oracle L162-167.
#[derive(Debug, PartialEq)]
enum Staleness {
    Critical, // >= 7 days
    Warn,     // 3..=6 days
    Silent,   // 0..=2 days or negative (future timestamp / clock skew)
}

fn staleness_category(days: i64) -> Staleness {
    if days >= 7 {
        Staleness::Critical
    } else if days >= 3 {
        Staleness::Warn
    } else {
        Staleness::Silent
    }
}

/// For each `(relative_path, bytes)` pair, emit a warning string if `bytes > 40960`.
/// Caller is responsible for skipping missing files (pure fn, no fs). Oracle L76-87.
fn doc_size_warns(docs: &[(&str, u64)]) -> Vec<String> {
    docs.iter()
        .filter(|(_, bytes)| *bytes > 40960) // oracle L83: -gt (strict)
        .map(|(doc, bytes)| {
            let kb = bytes / 1024;
            // Oracle L85 verbatim (2 spaces after ⚠️):
            format!(
                "⚠️  {doc} ({kb}k > 40k threshold) — gọi thợ trim, archive cũ ra docs/archive/"
            )
        })
        .collect()
}

/// Core: build banner text from fs/git state. Does NOT print or exit.
/// Any failure (no BACKLOG, fs error, git fail) → returns empty String (fail-open).
/// F-001 verbatim bug PRESERVED (L178 missing "touch worker-active") — do NOT fix here.
pub fn render_banner() -> String {
    // Step 1 — resolve repo root.
    let root = std::env::var("CLAUDE_PROJECT_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());

    // Step 2 — BACKLOG gate: no file → empty (silent).
    let backlog_path = root.join("docs/BACKLOG.md");
    let backlog_content = match std::fs::read_to_string(&backlog_path) {
        Ok(s) => s,
        Err(_) => return String::new(),
    };

    // Step 3 — find sprint block.
    let (block, header_text, fallback_used) = match find_sprint_block(&backlog_content) {
        Some(t) => t,
        None => return String::new(),
    };

    // Step 4 — count items.
    let (open, done) = count_items(&block);

    // Build banner into a String using writeln!-style push_str.
    let mut out = String::new();

    // Step 5 — main banner.
    out.push('\n');
    out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("🏠 Sếp's project — Active sprint status\n");
    out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push('\n');
    for line in block.lines().take(25) {
        out.push_str(line);
        out.push('\n');
    }
    out.push('\n');
    out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str(&format!("📊 Active sprint: {open} items đang treo, {done} đã xong\n"));
    if fallback_used {
        out.push('\n');
        out.push_str(&format!(
            "📌 Treating \"{header_text}\" as Active sprint (no \"Active sprint\" header found).\n"
        ));
    }

    // Step 6 — doc size warnings.
    {
        let doc_list = [
            "docs/CHANGELOG.md",
            "docs/DISCOVERIES.md",
            "CHANGELOG.md",
        ];
        let sizes: Vec<(&str, u64)> = doc_list
            .iter()
            .filter_map(|&rel| {
                let path = root.join(rel);
                std::fs::metadata(&path).ok().map(|m| (rel, m.len()))
            })
            .collect();

        let warns = doc_size_warns(&sizes);
        if !warns.is_empty() {
            out.push('\n');
            out.push_str("📏 Doc size warning:\n");
            for w in &warns {
                out.push_str(&format!("    {w}\n"));
            }
        }
    }

    // Step 7 — phiếu cleanup nudge.
    {
        let phieu_dir = if root.join("docs/ticket").is_dir() {
            Some(root.join("docs/ticket"))
        } else if root.join("phieu/active").is_dir() {
            Some(root.join("phieu/active"))
        } else {
            None
        };

        if let Some(pdir) = phieu_dir {
            let merged_output = std::process::Command::new("git")
                .args(["branch", "--merged", "main"])
                .current_dir(&root)
                .output();

            let merged_branches: Vec<String> = match merged_output {
                Ok(out) if out.status.success() => {
                    String::from_utf8_lossy(&out.stdout)
                        .lines()
                        .map(|l| {
                            let stripped = if l.starts_with("* ") || l.starts_with("  ") {
                                &l[2..]
                            } else {
                                l
                            };
                            stripped.trim().to_owned()
                        })
                        .filter(|s| !s.is_empty())
                        .collect()
                }
                _ => Vec::new(),
            };

            let mut nudges: Vec<String> = Vec::new();

            if let Ok(entries) = std::fs::read_dir(&pdir) {
                let mut phieu_files: Vec<std::path::PathBuf> = entries
                    .filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| {
                        p.is_file()
                            && p.file_name()
                                .and_then(|n| n.to_str())
                                .map(|n| {
                                    n.starts_with('P')
                                        && n.ends_with(".md")
                                        && n != "TICKET_TEMPLATE.md"
                                        && n != "TEMPLATE.md"
                                })
                                .unwrap_or(false)
                    })
                    .collect();
                phieu_files.sort();

                for phieu_path in &phieu_files {
                    let content = match std::fs::read_to_string(phieu_path) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
                    let approved_line = content
                        .lines()
                        .find(|l| l.contains("Approved by Chủ nhà:"))
                        .unwrap_or("");

                    if approved_line.is_empty()
                        || approved_line.contains("[date]")
                        || approved_line.trim_end_matches(|c: char| c.is_whitespace())
                            .ends_with("Approved by Chủ nhà:")
                    {
                        continue;
                    }

                    let basename = phieu_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    let phieu_id = {
                        let mut id = String::new();
                        let mut chars = basename.chars();
                        if chars.next() == Some('P') {
                            id.push('P');
                            for c in chars {
                                if c.is_ascii_digit() {
                                    id.push(c);
                                } else {
                                    break;
                                }
                            }
                        }
                        id
                    };
                    if phieu_id.is_empty() || phieu_id == "P" {
                        continue;
                    }

                    let pattern = format!("/{phieu_id}-");
                    if merged_branches.iter().any(|b| b.contains(&pattern)) {
                        let slug = basename.to_owned();
                        nudges.push(format!(
                            "🧹 Phiếu {phieu_id} approved + merged. Run: phieu-done {slug}"
                        ));
                    }
                }
            }

            if !nudges.is_empty() {
                out.push('\n');
                out.push_str("🧹 Cleanup nudge:\n");
                for n in &nudges {
                    out.push_str(&format!("    {n}\n"));
                }
            }
        }
    }

    // Step 8 — advisory staleness.
    {
        let inbox = root.join("docs/security/advisory-inbox.md");
        if inbox.exists() {
            let state_path = root.join("docs/security/.advisory-scan-state");
            if !state_path.exists() {
                out.push('\n');
                out.push_str("🚨 Advisory-watch: chưa scan lần nào — gõ /advisory-scan (first scan)\n");
            } else {
                let state_content = std::fs::read_to_string(&state_path).unwrap_or_default();

                let adv_last = {
                    let json_key = "\"last_scan_at\"";
                    if let Some(pos) = state_content.find(json_key) {
                        let after = &state_content[pos + json_key.len()..];
                        let after = after.trim_start();
                        let after = after.strip_prefix(':').unwrap_or(after).trim_start();
                        if let Some(rest) = after.strip_prefix('"') {
                            let end = rest.find('"').unwrap_or(rest.len());
                            rest[..end].to_owned()
                        } else {
                            state_content.split_whitespace().collect::<String>()
                        }
                    } else {
                        state_content.split_whitespace().collect::<String>()
                    }
                };

                let now_epoch = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);

                if let Some(days) = staleness_days(&adv_last, now_epoch) {
                    match staleness_category(days) {
                        Staleness::Critical => {
                            out.push('\n');
                            out.push_str(&format!(
                                "🚨 Advisory-watch: scan cuối {days} ngày trước (>= 7) — orchestrator BẮT BUỘC auto-spawn advisory-watch (ORCHESTRATION Rule 11)\n"
                            ));
                        }
                        Staleness::Warn => {
                            out.push('\n');
                            out.push_str(&format!(
                                "⚠️  Advisory-watch: scan cuối {days} ngày trước — cân nhắc /advisory-scan\n"
                            ));
                        }
                        Staleness::Silent => {}
                    }
                }
            }
        }
    }

    // Step 9 — Orchestrator contract + Architect Rule 0.
    // PORT VERBATIM including bug F-001 (L178 missing "touch worker-active").
    out.push('\n');
    out.push_str("🤖 Orchestrator contract (main session — đọc kỹ, ép tuân thủ):\n");
    out.push_str("    State machine: DRAFT → CHALLENGE → [RESPOND ⇄ CHALLENGE] → APPROVAL_GATE → EXECUTE\n");
    out.push_str("    KHÔNG hỏi user giữa các phase. APPROVAL_GATE là gate DUY NHẤT (trước EXECUTE).\n");
    out.push_str("    KHÔNG đẩy đọc phiếu/code về user — Worker CHALLENGE rà & report ≤5 dòng.\n");
    out.push_str("    Marker: touch .sos-state/architect-active trước spawn architect; rm -f trước spawn worker.\n");
    out.push_str("    Deferred tools MANDATORY (load đầu session, KHÔNG fallback markdown 1/2/3):\n");
    out.push_str("        ToolSearch select:AskUserQuestion,TaskCreate,TaskUpdate\n");
    out.push_str("    Handbook: agents/orchestrator.md (~85 lines, condensed contract)\n");
    out.push_str("    Spec đầy đủ: docs/ORCHESTRATION.md\n");
    out.push('\n');
    out.push_str("📌 Architect Rule 0: chỉ viết phiếu cho item trong Active sprint (or first ^## section if absent).\n");
    out.push_str("    Idea mới → /idea skill (intake vào BACKLOG.md).\n");
    out.push_str("    Pick item hay add idea?\n");
    out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push('\n');

    out
}

/// CLI wrapper — prints banner to stdout, always returns ALLOW (exit 0).
pub fn session_banner() -> i32 {
    print!("{}", render_banner());
    ALLOW
}

#[cfg(test)]
mod session_banner_tests {
    use super::*;

    // ── find_sprint_block ─────────────────────────────────────────────────────

    #[test]
    fn sprint_block_active_header_found() {
        let backlog = "## 🔥 Active sprint: Phase 1\n- [ ] task a\n- [x] task b\n## 🎯 Next sprint\n- [ ] future";
        let result = find_sprint_block(backlog);
        assert!(result.is_some());
        let (block, header, fallback) = result.unwrap();
        assert!(!fallback);
        assert_eq!(header, "🔥 Active sprint: Phase 1");
        assert!(block.contains("task a"));
        assert!(block.contains("task b"));
        // Must NOT contain the next section
        assert!(!block.contains("future"));
    }

    #[test]
    fn sprint_block_fallback_to_first_h2() {
        let backlog = "## Intro\n- [ ] do thing\n## Other\n- [ ] skip";
        let result = find_sprint_block(backlog);
        assert!(result.is_some());
        let (block, header, fallback) = result.unwrap();
        assert!(fallback);
        assert_eq!(header, "Intro");
        assert!(block.contains("do thing"));
        assert!(!block.contains("skip"));
    }

    #[test]
    fn sprint_block_no_h2_returns_none() {
        let backlog = "# Title\nSome prose\n### H3 only";
        assert!(find_sprint_block(backlog).is_none());
    }

    #[test]
    fn sprint_block_last_section_goes_to_eof() {
        let backlog = "## Active sprint\n- [ ] task\n- [x] done\n";
        let result = find_sprint_block(backlog);
        assert!(result.is_some());
        let (block, _, _) = result.unwrap();
        assert!(block.contains("task"));
        assert!(block.contains("done"));
    }

    #[test]
    fn sprint_block_h3_does_not_cut_block() {
        // H3 (###) must NOT be a boundary — only "^## " (2 hashes + space) cuts
        let backlog = "## Active sprint\n- [ ] item\n### Sub heading\n- [ ] sub\n## Next\n- [ ] skip";
        let (block, _, _) = find_sprint_block(backlog).unwrap();
        assert!(block.contains("sub"));       // H3 sub stays inside block
        assert!(!block.contains("skip"));     // "## Next" cuts
    }

    // ── count_items ───────────────────────────────────────────────────────────

    #[test]
    fn count_items_mixed() {
        let block = "- [ ] a\n- [x] b\n- [ ] c";
        assert_eq!(count_items(block), (2, 1));
    }

    #[test]
    fn count_items_empty() {
        assert_eq!(count_items(""), (0, 0));
    }

    #[test]
    fn count_items_uppercase_x_not_counted() {
        // Oracle uses [x] lowercase only (L56)
        let block = "- [X] big-x\n- [x] small-x";
        let (_, done) = count_items(block);
        assert_eq!(done, 1); // only lowercase x counts
    }

    // ── staleness_days ────────────────────────────────────────────────────────

    // Epoch for 2026-06-09T00:00:00Z verified via `date -j -f` = 1780963200
    // Epoch for 2026-06-16T00:00:00Z = 1781568000 (7 days later)
    const EPOCH_2026_06_09: i64 = 1780963200;
    const EPOCH_2026_06_16: i64 = 1781568000; // +7 days

    #[test]
    fn staleness_days_7_days() {
        let result = staleness_days("2026-06-09T00:00:00Z", EPOCH_2026_06_16);
        assert_eq!(result, Some(7));
    }

    #[test]
    fn staleness_days_3_days() {
        let now = EPOCH_2026_06_09 + 3 * 86400;
        let result = staleness_days("2026-06-09T00:00:00Z", now);
        assert_eq!(result, Some(3));
    }

    #[test]
    fn staleness_days_1_day() {
        let now = EPOCH_2026_06_09 + 86400;
        let result = staleness_days("2026-06-09T00:00:00Z", now);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn staleness_days_legacy_raw_same_result() {
        // Legacy raw = no JSON wrapping, just the ISO string trimmed
        let result = staleness_days("2026-06-09T00:00:00Z", EPOCH_2026_06_16);
        assert_eq!(result, Some(7));
    }

    #[test]
    fn staleness_days_garbage_returns_none() {
        assert_eq!(staleness_days("garbage", 1000000), None);
    }

    #[test]
    fn staleness_days_invalid_date_returns_none() {
        // Month 13 is invalid
        assert_eq!(staleness_days("2026-13-99T00:00:00Z", 1000000), None);
    }

    #[test]
    fn staleness_days_future_returns_negative() {
        // now < parsed → days negative
        let past_epoch = EPOCH_2026_06_09 - 86400; // 1 day before
        let result = staleness_days("2026-06-09T00:00:00Z", past_epoch);
        assert!(result.is_some());
        assert!(result.unwrap() < 0);
    }

    #[test]
    fn staleness_days_epoch_math_spot_check() {
        // Verify Hinnant formula: 2026-06-09T00:00:00Z → 1780963200
        // (verified against `date -j -f "%Y-%m-%dT%H:%M:%SZ" "2026-06-09T00:00:00Z" +%s`)
        let result = staleness_days("2026-06-09T00:00:00Z", EPOCH_2026_06_09);
        assert_eq!(result, Some(0)); // same day = 0 days
    }

    // ── staleness_category ────────────────────────────────────────────────────

    #[test]
    fn staleness_category_critical_7() {
        assert_eq!(staleness_category(7), Staleness::Critical);
    }

    #[test]
    fn staleness_category_critical_10() {
        assert_eq!(staleness_category(10), Staleness::Critical);
    }

    #[test]
    fn staleness_category_warn_3() {
        assert_eq!(staleness_category(3), Staleness::Warn);
    }

    #[test]
    fn staleness_category_warn_6() {
        assert_eq!(staleness_category(6), Staleness::Warn);
    }

    #[test]
    fn staleness_category_silent_0() {
        assert_eq!(staleness_category(0), Staleness::Silent);
    }

    #[test]
    fn staleness_category_silent_2() {
        assert_eq!(staleness_category(2), Staleness::Silent);
    }

    #[test]
    fn staleness_category_silent_negative() {
        assert_eq!(staleness_category(-5), Staleness::Silent);
    }

    // ── doc_size_warns ────────────────────────────────────────────────────────

    #[test]
    fn doc_size_warns_over_threshold() {
        let warns = doc_size_warns(&[("docs/CHANGELOG.md", 50000)]);
        assert_eq!(warns.len(), 1);
        // 50000 / 1024 = 48
        assert!(warns[0].contains("docs/CHANGELOG.md (48k > 40k threshold)"));
    }

    #[test]
    fn doc_size_warns_exact_threshold_not_warned() {
        // strict ">", 40960 is NOT > 40960
        let warns = doc_size_warns(&[("CHANGELOG.md", 40960)]);
        assert!(warns.is_empty());
    }

    #[test]
    fn doc_size_warns_one_above_threshold() {
        // 40961 / 1024 = 40
        let warns = doc_size_warns(&[("CHANGELOG.md", 40961)]);
        assert_eq!(warns.len(), 1);
        assert!(warns[0].contains("CHANGELOG.md (40k > 40k threshold)"));
    }

    #[test]
    fn doc_size_warns_empty_input() {
        let warns = doc_size_warns(&[]);
        assert!(warns.is_empty());
    }
}
