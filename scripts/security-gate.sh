#!/usr/bin/env bash
# Security gate: run mechanical invariants.
# Minimal template — sos-kit ships only universal INV-009 + INV-010.
# Per-repo: extend with project-specific INV (port-bind, docker, CORS, UFW, etc.)
#
# Exit 0 = all pass. Exit 1 = any violation.
#
# Doctrine: WORKFLOW_V2.2.md §7 Sub-mech B (capability) + §7 Sub-mech F (runtime state).

set -u

MECHANICAL_ONLY=false
for arg in "$@"; do
    case "$arg" in
        --mechanical-only) MECHANICAL_ONLY=true ;;
        *) echo "Unknown flag: $arg"; exit 2 ;;
    esac
done

PASS=0
FAIL=0
FAILED_INVS=()

run() {
    local inv="$1"
    local desc="$2"
    shift 2
    echo "--- $inv: $desc ---"
    if "$@"; then
        echo "  PASS"
        PASS=$((PASS + 1))
    else
        echo "  FAIL"
        FAIL=$((FAIL + 1))
        FAILED_INVS+=("$inv")
    fi
    echo
}

# =============================================================================
# Universal invariants (sos-kit ships these)
# =============================================================================

# INV-009: no hardcoded secret in source files
run "INV-009" "No hardcoded secret in source files" \
    python3 scripts/check-hardcoded-secrets.py

# INV-010: no secret in runtime state + infra files
run "INV-010" "No secret in runtime state + infra files (Sub-mech F)" \
    python3 scripts/check-runtime-secrets.py

# =============================================================================
# CUSTOMIZE — extend with per-repo invariants below this line
# =============================================================================
#
# Example extensions (uncomment + adapt):
#
# # INV-001: port bindings (require scripts/check-port-bind.py)
# run "INV-001" "No host-bind 0.0.0.0 except nginx 80/443" \
#     python3 scripts/check-port-bind.py
#
# # INV-002: no :latest tag in docker-compose
# check_inv002() {
#     ! grep -E '^\s+image:.*:latest$' docker-compose*.yml 2>/dev/null
# }
# run "INV-002" "No :latest tag" check_inv002
#
# # INV-003: .env.example contains no real secrets (project-specific allowlist regex)
# # INV-004: .env.{production,staging,...} gitignored + no history leak
# # INV-005: error-tracking config has secret scrubber (Sentry beforeSend etc.)
# # INV-006: CORS not wildcard
# # INV-007: firewall config (UFW / iptables) — requires SSH, --include-ssh flag
# # INV-008: internal services use expose: not ports:

# =============================================================================
# Summary
# =============================================================================
echo "===================================="
echo "Security gate: $PASS passed, $FAIL failed"
if (( FAIL > 0 )); then
    echo "Failed invariants: ${FAILED_INVS[*]}"
    exit 1
fi
exit 0
