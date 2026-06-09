# scripts/parsers/package_lock_v3.py
# Parse package-lock.json v3 (npm JSON) lock file into a generic dep list.
# Stub introduced in P040. Implementation added in P041 (Trinh sát consumer).
"""
Generic interface contract (P040):

    parse(path: Path) -> list[dict]

Each dict in the returned list MUST have these keys:
    name: str          # package name (e.g. "react", "next")
    version: str       # resolved/pinned version (e.g. "18.2.0")
    ecosystem: str     # "npm" (hardcoded — this parser handles npm ecosystem)
    source: str        # "direct" (entries from packages[""]["dependencies"] are direct)

Optional keys (parser may add, consumer may ignore):
    license: str | None
    integrity: str | None  # hash if available

P041 implements package-lock.json v3 flat layout (lockfileVersion: 3).

DESIGN NOTE:
  package-lock.json v3 layout:
    packages[""] = root package entry, has "dependencies" map of direct deps.
    packages["node_modules/<name>"] = resolved package entry, has "version" field.
  Direct deps = keys in packages[""]["dependencies"].
  Resolved version = packages["node_modules/<name>"]["version"].

  lockfileVersion 2 also has a compatible "packages" field — this parser handles both.
  lockfileVersion 1 uses a different "dependencies" flat structure — not supported.
"""
import json
import sys
import warnings
from pathlib import Path


def parse(path: Path) -> list[dict]:
    """Parse package-lock.json v3 and return direct deps as list[dict].

    Returns empty list [] if:
    - File is empty or unreadable
    - No packages[""] section found
    - No dependencies under root entry

    Raises:
        ValueError: if lockfileVersion < 2 (unsupported layout)
    """
    try:
        with open(path, "r", encoding="utf-8") as f:
            content = f.read()
    except (OSError, IOError):
        return []

    if not content.strip():
        return []

    try:
        data = json.loads(content)
    except json.JSONDecodeError:
        return []

    if not isinstance(data, dict):
        return []

    lock_version = data.get("lockfileVersion", 0)
    if isinstance(lock_version, int) and lock_version < 2:
        raise ValueError(
            f"Unsupported lockfileVersion: {lock_version}; "
            "this parser handles v2/v3 only (npm install output)."
        )

    packages = data.get("packages", {})
    if not packages:
        return []

    # Root entry is the empty-string key
    root_entry = packages.get("", {})
    if not root_entry:
        return []

    direct_dep_names: set[str] = set()
    for dep_group in ("dependencies", "devDependencies", "optionalDependencies"):
        group = root_entry.get(dep_group, {})
        if isinstance(group, dict):
            direct_dep_names.update(group.keys())

    deps: list[dict] = []
    for name in direct_dep_names:
        node_key = f"node_modules/{name}"
        node_entry = packages.get(node_key, {})
        if not node_entry or not isinstance(node_entry, dict):
            # Package entry missing (incomplete install) — skip with warning
            warnings.warn(
                f"package-lock.json: resolved entry for '{name}' not found "
                f"at '{node_key}'; skipping.",
                stacklevel=2,
            )
            continue

        version = str(node_entry.get("version", "")).strip()
        if not version:
            continue

        entry: dict = {
            "name": name,
            "version": version,
            "ecosystem": "npm",
            "source": "direct",
        }

        # Optional: include integrity hash if present
        integrity = node_entry.get("integrity")
        if integrity:
            entry["integrity"] = integrity

        deps.append(entry)

    return deps


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(f"Usage: python {sys.argv[0]} <path-to-package-lock.json>", file=sys.stderr)
        sys.exit(1)
    result = parse(Path(sys.argv[1]))
    print(json.dumps(result))
