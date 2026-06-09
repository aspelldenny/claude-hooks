use crate::io::{self, ALLOW, BLOCK};
use regex::Regex;

pub fn architect_guard() -> i32 {
    // Step 1 — resolve repo root from CLAUDE_PROJECT_DIR (fallback: cwd).
    // Oracle L23: cd "${CLAUDE_PROJECT_DIR:-<script dir>/..}"
    // Rust binary has no "script dir" equivalent; cwd is the closest fallback.
    // Divergence: oracle fallback = script's parent dir; Rust fallback = cwd.
    // Accepted divergence (CLAUDE.md Port doctrine #7): Claude Code always sets
    // CLAUDE_PROJECT_DIR when firing hooks, so cwd-fallback is an edge case only.
    let repo_root = std::env::var("CLAUDE_PROJECT_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());

    // Step 2 — marker gate. Oracle L28.
    // If no marker -> not running as Architect -> allow everything.
    let marker = repo_root.join(".sos-state/architect-active");
    if !marker.exists() {
        return ALLOW;
    }

    // Step 3 — read path from stdin payload. Oracle L38-41.
    // file_path is priority; fallback to pattern.
    let payload = io::read_payload();
    let path = payload.tool_input.file_path.or(payload.tool_input.pattern);

    // Step 4 — no path -> ALLOW (fail-open). Oracle L44.
    let path = match path {
        Some(p) => p,
        None => return ALLOW,
    };

    // Step 5 — strip leading "./". Oracle L47.
    let norm = path.strip_prefix("./").unwrap_or(&path).to_owned();

    // Step 6 — .md anywhere -> ALLOW (docs are Architect's domain). Oracle L50-52.
    if norm.ends_with(".md") {
        return ALLOW;
    }

    // Step 7 — forbidden pattern check. Oracle L56-67.
    // Port semantics: X/* -> starts_with("X/"); */X/* -> contains("/X/"); *.ext -> ends_with(".ext").
    // __tests__ and build artifacts: prefix-only (no segment variant in oracle).
    let blocked =
        // Source dirs — prefix variants (oracle L57: src/*, lib/*, app/*, pkg/*)
        norm.starts_with("src/")
        || norm.starts_with("lib/")
        || norm.starts_with("app/")
        || norm.starts_with("pkg/")
        // Source dirs — segment variants (oracle L57: */src/*, */lib/*, */app/*, */pkg/*)
        || norm.contains("/src/")
        || norm.contains("/lib/")
        || norm.contains("/app/")
        || norm.contains("/pkg/")
        // Test dirs — prefix variants (oracle L59: tests/*, test/*, __tests__/*)
        || norm.starts_with("tests/")
        || norm.starts_with("test/")
        || norm.starts_with("__tests__/")
        // Test dirs — segment variants (oracle L59: */tests/*, */test/*)
        // NOTE: __tests__ has NO segment variant in oracle L59 — do NOT add contains("/__tests__/")
        || norm.contains("/tests/")
        || norm.contains("/test/")
        // Build artifacts — prefix only (oracle L61: no segment variants)
        || norm.starts_with("node_modules/")
        || norm.starts_with("target/")
        || norm.starts_with("dist/")
        || norm.starts_with("build/")
        || norm.starts_with(".next/")
        || norm.starts_with(".nuxt/")
        || norm.starts_with(".svelte-kit/")
        // Source code extensions (oracle L63)
        || norm.ends_with(".rs")
        || norm.ends_with(".ts")
        || norm.ends_with(".tsx")
        || norm.ends_with(".js")
        || norm.ends_with(".jsx")
        || norm.ends_with(".py")
        || norm.ends_with(".go")
        || norm.ends_with(".java")
        || norm.ends_with(".cpp")
        || norm.ends_with(".c")
        || norm.ends_with(".h")
        || norm.ends_with(".hpp");

    // Step 8 — blocked -> emit message to stderr + return BLOCK. Oracle L69-83.
    // PATH_ARG in message = original path (pre-strip), matching oracle L73,78.
    if blocked {
        let msg = format!(
            "🚫 Architect envelope violation\n\nArchitect cannot read source code: {path}\n\nWhat to do instead: write a Task 0 anchor in the phiếu.\nExample:\n  | # | Assumption | Verify by | Result |\n  | 1 | <claim about {path}> | grep ... {path} | ⏳ TO VERIFY |\n\nWorker (separate subagent) will grep-verify it for you. The constraint IS the feature."
        );
        return io::block(&msg);
    }

    ALLOW
}

pub fn block_env_edit() -> i32 {
    // Step 1 — read stdin payload. Oracle L16-20.
    // NOTE: env-fallback CLAUDE_TOOL_INPUT (oracle L16-20) intentionally NOT ported.
    // Hook always receives stdin from Claude Code; env-fallback requires io.rs harness
    // change (shared API, Tầng 1 scope). See docs/discoveries/P003.md for full rationale.
    let payload = io::read_payload();

    // Steps 2-4 — parse path: file_path priority, fallback notebook_path (NotebookEdit).
    // KHÔNG dùng pattern (oracle L29-32: only file_path / notebook_path for this hook).
    // Empty payload (empty stdin) -> both fields None -> falls through to return ALLOW below.
    let path = payload.tool_input.file_path
        .or(payload.tool_input.notebook_path);

    // Step 4 — no path -> ALLOW (fail-open). Oracle L35.
    let path = match path {
        Some(p) if !p.is_empty() => p,
        _ => return ALLOW,
    };

    // Step 5 — basename. Oracle L38: BASE=$(basename "$FILE_PATH").
    // rsplit('/').next() gives last segment: "/a/b/.env" -> ".env", ".env" -> ".env".
    let base = path.rsplit('/').next().unwrap_or(&path);

    // Step 6 — allowlist: .env.example is a template, allow edit. Oracle L41.
    if base == ".env.example" {
        return ALLOW;
    }

    // Step 7 — block regex ^\.env($|\.). Oracle L44.
    // Pattern is a constant literal -> unwrap() safe (never fails to compile).
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
        return io::block(&msg);
    }

    // Step 8 — else allow. Oracle L64.
    ALLOW
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

/// Port of `scripts/block-unsafe-merge.sh`.
///
/// DIVERGENCE (INTENTIONAL — fail-CLOSED): when `gh pr diff` fails or returns empty,
/// this hook returns BLOCK (exit 2), unlike architect-guard / block-env-edit / session-banner
/// which fail-open (exit 0). This is by design: unverifiable merge of unknown security
/// surface is treated as unsafe. Do NOT change to fail-open.
pub fn block_unsafe_merge() -> i32 {
    // Step 1: read stdin payload, get command. Oracle L24-31.
    // NOTE: env-fallback CLAUDE_TOOL_INPUT (oracle L24-28) intentionally NOT ported.
    // Harness always receives stdin from Claude Code; env-fallback is out-of-scope
    // (same decision as P002/P003 — see docs/discoveries/P003.md).
    let payload = io::read_payload();
    let command = match payload.tool_input.command {
        Some(ref c) if !c.is_empty() => c.clone(),
        _ => return ALLOW, // oracle L31: empty input → pass through
    };

    // Step 2: parse PR number. Oracle L41-47.
    let pr = match parse_merge_pr(&command) {
        Some(n) => n,
        None => return ALLOW,
    };
    let pr_str = pr.to_string();

    // Step 3: override marker check. Oracle L50-55.
    if let Some(reason) = extract_skip_marker(&command) {
        eprintln!(
            "⚠️  Security review override marker detected for PR #{}. Reason: {}",
            pr_str, reason
        );
        eprintln!("    Allowing merge. Sếp đã review tay — em (hook) không block.");
        return ALLOW;
    }

    // Step 4: read optional extra surface pattern. Oracle L63.
    let extra = std::env::var("SECURITY_SURFACE_EXTRA").ok();

    // Step 5: gh call #1 — `gh pr diff <PR> --name-only`. Oracle L67-83.
    // FAIL-CLOSED: fail or empty output → BLOCK (divergence from other 3 hooks).
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
        // Oracle L70-82 verbatim message (fail-CLOSED BLOCK)
        eprintln!(
            "⛔ BLOCKED: gh pr diff #{pr_str} thất bại (network/auth?).\n\n\
Em (hook) KHÔNG verify được PR có touch security surface không.\n\
Fail-safe: block merge để Sếp/Quản đốc kiểm tra tay.\n\n\
Cách hợp lệ:\n\
  - Kiểm tra gh auth status\n\
  - Chạy: gh pr diff {pr_str} --name-only\n\
  - Nếu confirm KHÔNG touch security surface → re-run merge với marker:\n\
      gh pr merge {pr_str} --merge [security-review-skip:gh-cli-unavailable]"
        );
        return BLOCK;
    }

    // Step 6: check security surface. Oracle L86-93.
    if !touches_security_surface(&diff_files, extra.as_deref()) {
        return ALLOW;
    }

    // Step 7: gh call #2 — `gh pr view <PR> --json comments --jq '.comments[].body'`.
    // Oracle L96: fail → empty string (|| echo ""), NOT fail-closed here.
    let view_output = std::process::Command::new("gh")
        .args(["pr", "view", &pr_str, "--json", "comments", "--jq", ".comments[].body"])
        .output();

    let comments = match view_output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).into_owned()
        }
        _ => String::new(), // oracle L96: fail → empty string
    };

    // Step 8: verdict check. Oracle L97-137.
    match verdict_is_approve(&comments) {
        VerdictResult::Approve => {
            // Oracle L101-102: APPROVE → allow
            ALLOW
        }
        VerdictResult::NeedsReview => {
            // Oracle L105-116 verbatim message
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
            eprintln!(
                "⛔ BLOCKED: PR #{pr_str} touch security surface VÀ /security-review verdict KHÔNG phải APPROVE.\n\n\
Verdict line: {verdict_line}\n\n\
Hành động:\n\
  1. Sếp đọc comment giám sát trên PR #{pr_str}\n\
  2. Nếu Sếp accept risk → re-run với marker:\n\
     gh pr merge {pr_str} --merge [security-review-skip:sep-accepted-needs-review]\n\
  3. Nếu cần fix → spawn Worker EXECUTE fix theo INV flagged, push, gate sẽ re-fire"
            );
            BLOCK
        }
        VerdictResult::NoBlock => {
            // Oracle L120-137 verbatim message
            // Note: oracle L133 has literal `$(gh pr view $PR --json url --jq .url)` in heredoc
            // (escaped \$ in Bash heredoc = literal text for user to run, NOT executed by hook).
            eprintln!(
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
            BLOCK
        }
    }
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

pub fn session_banner() -> i32 {
    ALLOW // real logic in P005 (renders banner from git state)
}
