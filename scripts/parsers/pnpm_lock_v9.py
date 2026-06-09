# scripts/parsers/pnpm_lock_v9.py
# Parse pnpm-lock.yaml v9 (YAML) lock file into a generic dep list.
# Stub introduced in P040. Implementation added in P041 (Trinh sát consumer).
"""
Generic interface contract (P040):

    parse(path: Path) -> list[dict]

Each dict in the returned list MUST have these keys:
    name: str          # package name (e.g. "react", "next")
    version: str       # resolved/pinned version (e.g. "18.2.0") — peer-suffix stripped
    ecosystem: str     # "npm" (hardcoded — this parser handles npm/pnpm ecosystem)
    source: str        # "direct" (all entries from importers: . are direct deps by definition)

Optional keys (parser may add, consumer may ignore):
    license: str | None
    integrity: str | None  # hash if available

P041 implements pnpm-lock.yaml v9 YAML 2-level layout.
Supports: lockfileVersion '9.0' (and compatible v9.x).
Unsupported: v10+, legacy v6/v8 (raise ValueError).

DESIGN NOTE (pnpm-lock v9 layout — P041 lesson):
  pnpm-lock v9 has a FLAT packages section AND an importers section.
  Do NOT try to parse direct deps from the flat packages section using
  a regex like r'^  <name>@X.Y.Z:' — that section is transitive deps.
  Direct deps are declared under importers: .: dependencies / devDependencies.
  This is the correct 2-level YAML structure navigation.

MONOREPO NOTE:
  pnpm-lock.yaml in a monorepo has multiple keys under importers:
  e.g. importers: { ".": {...}, "packages/api": {...} }
  P041 handles ONLY root "." — monorepo multi-root deferred to follow-on phiếu.

PEER-SUFFIX NOTE (pnpm v9 behavior — P041):
  pnpm v9 appends peer-dep versions to resolved versions, e.g. "18.2.0(zod@4.3.6)".
  Strip everything after the first '(' to get the clean semver.
"""
import re
import sys
from pathlib import Path


def _strip_peer_suffix(version: str) -> str:
    """Strip pnpm peer-dep suffix: '18.2.0(zod@4.3.6)' → '18.2.0'."""
    # pnpm v9 peer-suffix pattern: strip from first '(' onward
    match = re.match(r'^([^\s(]+)', version)
    return match.group(1) if match else version


def parse(path: Path) -> list[dict]:
    """Parse pnpm-lock.yaml v9 and return direct deps as list[dict].

    Returns empty list [] if:
    - File is empty or unreadable
    - No importers: . section found
    - No dependencies or devDependencies under root importer

    Raises:
        ValueError: if lockfileVersion is not v9.x compatible
        ImportError: if PyYAML not installed (caller should run Bước 0 pre-flight)
    """
    try:
        import yaml  # PyYAML — installed via Trinh sát Bước 0 pre-flight
    except ImportError as e:
        raise ImportError(
            "PyYAML required for pnpm-lock parsing. "
            "Install via: pip3 install pyyaml"
        ) from e

    try:
        with open(path, "r", encoding="utf-8") as f:
            content = f.read()
    except (OSError, IOError):
        # Missing file or unreadable — return empty gracefully
        return []

    if not content.strip():
        return []

    try:
        data = yaml.safe_load(content)
    except yaml.YAMLError:
        # Malformed YAML — return empty gracefully
        return []

    if not isinstance(data, dict):
        return []

    # Validate lockfileVersion — must be v9.x
    lock_version = str(data.get("lockfileVersion", ""))
    if not lock_version.startswith("9") and not lock_version.startswith("'9"):
        # Handle both string '9.0' and numeric 9 forms
        # pnpm writes lockfileVersion: '9.0' (string with quotes in YAML)
        # yaml.safe_load strips the outer quotes so we get string "9.0"
        cleaned = lock_version.strip("'\"")
        if not cleaned.startswith("9"):
            raise ValueError(
                f"Unsupported lockfileVersion: {lock_version!r}; "
                "this parser handles v9 only (lockfileVersion starting with '9')."
            )

    importers = data.get("importers", {})
    if not importers:
        # No importers section — possibly a simple lockfile format, return empty
        return []

    # Extract only root importer "." (monorepo multi-root deferred to follow-on phiếu)
    root_importer = importers.get(".", {})
    if not root_importer:
        return []

    deps: list[dict] = []

    for dep_group in ("dependencies", "devDependencies"):
        group = root_importer.get(dep_group, {})
        if not group or not isinstance(group, dict):
            continue

        for name, meta in group.items():
            if not isinstance(meta, dict):
                continue

            # pnpm v9 stores resolved version under "version" key
            raw_version = str(meta.get("version", "")).strip()
            if not raw_version:
                continue

            # Strip peer-dep suffix: "18.2.0(zod@4.3.6)" → "18.2.0"
            clean_version = _strip_peer_suffix(raw_version)

            deps.append({
                "name": name,
                "version": clean_version,
                "ecosystem": "npm",
                "source": "direct",
            })

    return deps


if __name__ == "__main__":
    import json
    if len(sys.argv) != 2:
        print(f"Usage: python {sys.argv[0]} <path-to-pnpm-lock.yaml>", file=sys.stderr)
        sys.exit(1)
    result = parse(Path(sys.argv[1]))
    print(json.dumps(result))
