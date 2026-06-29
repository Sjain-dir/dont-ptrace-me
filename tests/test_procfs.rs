#[cfg(feature = "procfs_check")]
mod procfs_tests {
    use dont_ptrace_me::detect_procfs::{
        get_tracer_pid, is_debugger_attached, parse_tracer_pid, read_status_file,
    };

    #[test]
    fn test_parse_tracer_pid_zero() {
        println!("\n  [procfs] Parsing synthetic status with TracerPid: 0 ...");
        let sample = "Name:\ttest\nState:\tR\nTracerPid:\t0\nPid:\t12345\n";
        let pid = parse_tracer_pid(sample);
        println!("  [procfs] Parsed TracerPid = {:?}", pid);
        assert_eq!(pid, Some(0));
        println!("  [procfs] Correctly parsed zero TracerPid ✓");
    }

    #[test]
    fn test_parse_tracer_pid_nonzero() {
        println!("\n  [procfs] Parsing synthetic status with TracerPid: 4242 ...");
        let sample = "Name:\ttest\nState:\tR\nTracerPid:\t4242\nPid:\t12345\n";
        let pid = parse_tracer_pid(sample);
        println!("  [procfs] Parsed TracerPid = {:?}", pid);
        assert_eq!(pid, Some(4242));
        println!("  [procfs] Correctly parsed non-zero TracerPid ✓");
    }

    #[test]
    fn test_parse_tracer_pid_missing_field() {
        println!("\n  [procfs] Parsing synthetic status with no TracerPid field ...");
        let sample = "Name:\ttest\nState:\tR\nPid:\t12345\n";
        let pid = parse_tracer_pid(sample);
        println!("  [procfs] Result = {:?} (expected None)", pid);
        assert_eq!(pid, None);
        println!("  [procfs] Missing field correctly returns None ✓");
    }

    #[test]
    fn test_parse_tracer_pid_leading_whitespace() {
        println!("\n  [procfs] Parsing status with tab-separated TracerPid: 99 ...");
        let sample = "TracerPid:\t   99\n";
        let pid = parse_tracer_pid(sample);
        println!("  [procfs] Parsed TracerPid = {:?}", pid);
        assert_eq!(pid, Some(99));
        println!("  [procfs] Whitespace trimming works correctly ✓");
    }

    #[test]
    fn test_parse_tracer_pid_malformed() {
        println!("\n  [procfs] Parsing status with non-numeric TracerPid ...");
        let sample = "TracerPid:\tnot-a-number\n";
        let pid = parse_tracer_pid(sample);
        println!("  [procfs] Result = {:?} (expected None)", pid);
        assert_eq!(pid, None);
        println!("  [procfs] Malformed value correctly returns None ✓");
    }

    #[test]
    fn test_status_file_readable() {
        println!("\n  [procfs] Reading live /proc/self/status ...");
        let result = read_status_file();
        assert!(result.is_ok(), "/proc/self/status should be readable");
        let content = result.unwrap();
        println!("  [procfs] File read OK ({} bytes)", content.len());
        assert!(content.contains("TracerPid:"), "Must contain TracerPid field");
        println!("  [procfs] TracerPid field present ✓");
    }

    #[test]
    fn test_get_tracer_pid_returns_some() {
        println!("\n  [procfs] Reading TracerPid from live /proc/self/status ...");
        let pid = get_tracer_pid();
        println!("  [procfs] TracerPid = {:?}", pid);
        assert!(pid.is_some(), "get_tracer_pid() must return Some(...)");
        println!("  [procfs] TracerPid is readable ✓");
    }

    #[test]
    fn test_no_false_positive_under_cargo_test() {
        println!("\n  [procfs] Checking is_debugger_attached() without a tracer ...");
        let detected = is_debugger_attached();
        println!("  [procfs] Result: {}", if detected { "DETECTED (unexpected!)" } else { "clean ✓" });
        assert!(!detected, "False positive: procfs_check triggered without a debugger");
        println!("  [procfs] No tracer detected as expected ✓");
    }

    #[test]
    fn test_tracer_pid_is_zero_without_debugger() {
        println!("\n  [procfs] Verifying TracerPid == 0 (no debugger attached) ...");
        let pid = get_tracer_pid().expect("/proc/self/status must be parseable");
        println!("  [procfs] TracerPid = {}", pid);
        assert_eq!(pid, 0, "TracerPid should be 0 when no debugger is attached");
        println!("  [procfs] TracerPid is 0 — process runs freely ✓");
    }
}
