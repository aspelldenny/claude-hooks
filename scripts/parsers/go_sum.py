# scripts/parsers/go_sum.py
# Parse go.sum (Go modules) lock/manifest file into a generic dep list.
# Skeleton stub introduced in P040. Implementation deferred to P041 (advisory-watch consumer).
"""
Generic interface contract (P040):

    parse(path: Path) -> list[dict]

Each dict in the returned list MUST have these keys:
    name: str          # package name (e.g. "react", "django", "serde")
    version: str       # resolved/pinned version (e.g. "18.2.0", "4.2.1")
    ecosystem: str     # one of: "npm" | "pypi" | "crates" | "go"
    source: str        # "direct" | "transitive" (best-effort; "transitive" OK if unsure)

Optional keys (parser may add, consumer may ignore):
    license: str | None
    integrity: str | None  # hash if available

P040 ships an empty-list stub. P041 fills implementation.
"""
from pathlib import Path


def parse(path: Path) -> list[dict]:
    # TODO(P041): implement go.sum (Go modules) parsing.
    # Reference: tarot's `.claude/agents/advisory-watch.md` documents the parser
    # contract; port logic without tarot-specific path assumptions.
    _ = path  # silence unused-arg lint until P041 implements
    return []


if __name__ == "__main__":
    import sys
    if len(sys.argv) != 2:
        print(f"Usage: python {sys.argv[0]} <path-to-lock-file>", file=sys.stderr)
        sys.exit(1)
    deps = parse(Path(sys.argv[1]))
    print(f"Parsed {len(deps)} deps from {sys.argv[1]} (stub returns empty list — P040 skeleton)")
