#!/usr/bin/env bash
# =============================================================================
# test_with_gdb.sh — Run dont-ptrace-me under GDB to trigger debugger detection
#
# Tests:
#   - ptrace_check   : PTRACE_TRACEME fails under GDB (GDB is the tracer)
#   - procfs_check   : TracerPid is non-zero (GDB's PID)
#   - breakpoint_scan: GDB inserts BRK #0 (AArch64) or INT3 (x86_64) breakpoints
#
# Exit codes:
#   0 — program detected the debugger (expected)
#   1 — program did NOT detect the debugger (test failure)
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BINARY="$REPO_ROOT/target/debug/dont-ptrace-me"

# ── Colour helpers ─────────────────────────────────────────────────────────
GREEN='\033[0;32m'; RED='\033[0;31m'; YELLOW='\033[1;33m'; NC='\033[0m'
pass() { echo -e "${GREEN}[PASS]${NC} $*"; }
fail() { echo -e "${RED}[FAIL]${NC} $*"; }
info() { echo -e "${YELLOW}[INFO]${NC} $*"; }

# ── Prerequisite checks ────────────────────────────────────────────────────
if ! command -v gdb &>/dev/null; then
    fail "GDB is not installed. Install with: sudo dnf install gdb"
    exit 1
fi

# ── Check ptrace_scope ─────────────────────────────────────────────────────
PTRACE_SCOPE=$(cat /proc/sys/kernel/yama/ptrace_scope 2>/dev/null || echo "0")
if [[ "$PTRACE_SCOPE" -gt 1 ]]; then
    info "ptrace_scope=$PTRACE_SCOPE may restrict GDB. If tests fail, run:"
    info "  echo 1 | sudo tee /proc/sys/kernel/yama/ptrace_scope"
fi

# ── Build ──────────────────────────────────────────────────────────────────
info "Building with all_checks features..."
cd "$REPO_ROOT"
cargo build --features all_checks 2>&1
info "Build complete: $BINARY"

# ── Run under GDB ─────────────────────────────────────────────────────────
info "Launching under GDB..."
echo ""

# -batch mode: GDB runs non-interactively.
# 'run' starts the process; GDB is the tracer so PTRACE_TRACEME will fail.
GDB_OUTPUT=$(gdb \
    --batch \
    -ex "set confirm off" \
    -ex "run" \
    -ex "quit" \
    "$BINARY" 2>&1 || true)

echo "$GDB_OUTPUT"
echo ""

# ── Evaluate result ────────────────────────────────────────────────────────
if echo "$GDB_OUTPUT" | grep -q "DEBUGGER DETECTED"; then
    pass "Program detected GDB and exited with the expected message."
    exit 0
else
    fail "Program did NOT print 'DEBUGGER DETECTED' under GDB."
    fail "Check the output above for clues."
    exit 1
fi
