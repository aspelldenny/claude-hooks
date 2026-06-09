use crate::io::{self, ALLOW};

pub fn architect_guard() -> i32 {
    let _payload = io::read_payload(); // wire harness; real logic in P002
    ALLOW
}

pub fn block_env_edit() -> i32 {
    let _payload = io::read_payload(); // real logic in P003
    ALLOW
}

pub fn block_unsafe_merge() -> i32 {
    ALLOW // real logic in P004 (reads gh pr diff, not stdin payload)
}

pub fn session_banner() -> i32 {
    ALLOW // real logic in P005 (renders banner from git state)
}
