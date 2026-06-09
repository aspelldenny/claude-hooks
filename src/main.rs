mod io;
mod hooks;
mod serve;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "claude-hooks", version, about = "Rust hooks for Claude Code")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Block Architect Read/Glob outside envelope
    ArchitectGuard,
    /// Block edit of .env* files
    BlockEnvEdit,
    /// Block gh pr merge without security APPROVE
    BlockUnsafeMerge,
    /// Render SessionStart banner
    SessionBanner,
    /// MCP server (stdio JSON-RPC)
    Serve,
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.cmd {
        Cmd::ArchitectGuard => hooks::architect_guard(),
        Cmd::BlockEnvEdit => hooks::block_env_edit(),
        Cmd::BlockUnsafeMerge => hooks::block_unsafe_merge(),
        Cmd::SessionBanner => hooks::session_banner(),
        Cmd::Serve => serve::run(),
    };
    std::process::exit(code);
}
