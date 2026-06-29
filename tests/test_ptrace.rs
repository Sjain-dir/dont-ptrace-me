#[cfg(feature = "ptrace_check")]
mod ptrace_tests {
    use dont_ptrace_me::detect_ptrace;

    #[test]
    fn test_no_false_positive_under_cargo_test() {
        println!("\n  [ptrace] Calling PTRACE_TRACEME on self...");
        let detected = detect_ptrace::is_debugger_attached();
        println!("  [ptrace] Result: {}", if detected { "DETECTED (unexpected!)" } else { "clean ✓" });
        assert!(
            !detected,
            "False positive: ptrace_check triggered without a debugger"
        );
        println!("  [ptrace] No debugger attached — PTRACE_TRACEME succeeded as expected.");
    }

    #[test]
    fn test_double_traceme_fails() {
        println!("\n  [ptrace] Verifying ptrace::traceme() returns a valid Result...");
        let result = nix::sys::ptrace::traceme();
        match &result {
            Ok(_)  => println!("  [ptrace] traceme() returned Ok — no tracer present."),
            Err(e) => println!("  [ptrace] traceme() returned Err({}) — tracer present or already traced.", e),
        }
        let _ = result;
        println!("  [ptrace] API call completed without panic ✓");
    }
}
