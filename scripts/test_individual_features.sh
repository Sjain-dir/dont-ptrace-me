#!/usr/bin/env bash
# =============================================================================
# test_individual_features.sh — Test each detection feature in isolation
#
# For each feature:
#   1. Build the binary with ONLY that feature enabled.
#   2. Run normally (no debugger) — must exit 0.
#   3. Run under GDB (if available) — must exit 1 (detected).
#
# This script is designed to be run on the target machine (Fedora ARM or x86_64).
# GDB tests are skipped if GDB is not installed.
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

# ── Colour helpers ─────────────────────────────────────────────────────────
GREEN='\033[0;32m'; RED='\033[0;31m'; YELLOW='\033[1;33m'; BOLD='\033[1m'; NC='\033[0m'
pass()   { echo -e "  ${GREEN}✓ PASS${NC}  $*"; }
fail()   { echo -e "  ${RED}✗ FAIL${NC}  $*"; FAILURES=$((FAILURES+1)); }
info()   { echo -e "  ${YELLOW}→${NC} $*"; }
header() { echo -e "\n${BOLD}══ $* ══${NC}"; }

HAS_GDB=0; command -v gdb &>/dev/null && HAS_GDB=1
FAILURES=0
BINARY="$REPO_ROOT/target/debug/dont-ptrace-me"

run_normal() {
    local feature="$1"
    header "Feature: $feature — normal run (no debugger)"
    info "Building..."
    cargo build --features "$feature" -q 2>&1
    info "Running (expect exit 0 or timeout into sleep loop)..."
    if timeout 5 "$BINARY"; then
        pass "Exited 0 — no false positive."
    else
        local code=$?
        if [[ $code -eq 124 ]]; then
            # timeout: the 30-second sleep started — detection was clean
            pass "Detection clean (program entered normal sleep loop — timed out as expected)."
        else
            fail "Exited $code — false positive! Detection triggered without a debugger."
        fi
    fi
}

run_under_gdb() {
    local feature="$1"
    header "Feature: $feature — GDB run (expect detection)"
    if [[ $HAS_GDB -eq 0 ]]; then
        info "GDB not installed — skipping GDB test."
        return
    fi
    info "Running under GDB (expect DETECTED)..."
    GDB_OUT=$(gdb --batch \
        -ex "set confirm off" \
        -ex "run" \
        -ex "quit" \
        "$BINARY" 2>&1 || true)
    echo "$GDB_OUT" | sed 's/^/    /'
    if echo "$GDB_OUT" | grep -q "DETECTED\|detected\|exit code 1"; then
        pass "Debugger detected under GDB."
    else
        fail "Program did NOT detect GDB with feature='$feature'."
    fi
}

# =============================================================================
# 1. ptrace_check
# =============================================================================
run_normal "ptrace_check"
run_under_gdb "ptrace_check"

# =============================================================================
# 2. procfs_check
# =============================================================================
run_normal "procfs_check"
run_under_gdb "procfs_check"

# =============================================================================
# 3. breakpoint_scan
# =============================================================================
run_normal "breakpoint_scan"
header "Feature: breakpoint_scan — GDB breakpoint injection"
if [[ $HAS_GDB -eq 0 ]]; then
    info "GDB not installed — skipping."
else
    info "To test breakpoint_scan with GDB:"
    info "  1. gdb ./target/debug/dont-ptrace-me"
    info "  2. (gdb) break dont_ptrace_me::detect_breakpoint::scan_for_breakpoints"
    info "  3. (gdb) run"
    info "  GDB inserts a BRK #0 (AArch64) or INT3 (x86_64) at the breakpoint."
    info "  The scanner will detect the injected opcode vs the on-disk binary."
fi

# =============================================================================
# 4. all_checks combined
# =============================================================================
run_normal "all_checks"
run_under_gdb "all_checks"

# =============================================================================
# Summary
# =============================================================================
echo ""
echo "═══════════════════════════════════════════"
if [[ $FAILURES -eq 0 ]]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}$FAILURES test(s) FAILED.${NC}"
    exit 1
fi
