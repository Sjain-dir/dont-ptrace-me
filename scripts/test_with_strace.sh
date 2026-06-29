#!/usr/bin/env bash
# =============================================================================
# test_with_strace.sh — Run dont-ptrace-me under strace
#
# strace uses ptrace internally to intercept syscalls. This triggers:
#   - ptrace_check  : PTRACE_TRACEME fails (strace is the tracer)
#   - procfs_check  : TracerPid == strace's PID
#
# breakpoint_scan is less relevant under strace alone (strace doesn't inject
# breakpoint opcodes into the target's memory).
#
# Exit codes:
#   0 — program detected strace (expected)
#   1 — program did NOT detect strace (test failure)
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
if ! command -v strace &>/dev/null; then
    fail "strace is not installed. Install with: sudo dnf install strace"
    exit 1
fi

# ── Build (ptrace and procfs — most relevant for strace) ──────────────────
info "Building with ptrace_check and procfs_check..."
cd "$REPO_ROOT"
cargo build --features "ptrace_check,procfs_check" 2>&1
info "Build complete: $BINARY"

# ── Run under strace ───────────────────────────────────────────────────────
info "Launching under strace..."
echo ""

# -o /dev/null : discard strace syscall output (we care about program output)
# -qq          : suppress "attached" / "detached" messages
PROG_OUTPUT=$(strace -o /dev/null -qq "$BINARY" 2>&1 || true)
STRACE_EXIT=$?

echo "$PROG_OUTPUT"
echo ""
info "Program exited with code: $STRACE_EXIT"
echo ""

# ── Evaluate result ────────────────────────────────────────────────────────
if echo "$PROG_OUTPUT" | grep -q "DEBUGGER DETECTED"; then
    pass "Program detected strace as a tracer."
    exit 0
elif [[ $STRACE_EXIT -eq 1 ]]; then
    pass "Program exited with code 1 (detection triggered)."
    exit 0
else
    fail "Program did NOT detect strace (exit code: $STRACE_EXIT)."
    fail "This may happen if ptrace_scope restricts self-tracing."
    info "Try: echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope"
    exit 1
fi
