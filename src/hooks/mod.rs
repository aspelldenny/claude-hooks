use crate::io::{self, ALLOW};
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

pub fn block_unsafe_merge() -> i32 {
    ALLOW // real logic in P004 (reads gh pr diff, not stdin payload)
}

pub fn session_banner() -> i32 {
    ALLOW // real logic in P005 (renders banner from git state)
}
