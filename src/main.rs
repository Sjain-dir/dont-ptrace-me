#[cfg(feature = "ptrace_check")]
use dont_ptrace_me::detect_ptrace;

#[cfg(feature = "procfs_check")]
use dont_ptrace_me::detect_procfs;

#[cfg(feature = "breakpoint_scan")]
use dont_ptrace_me::detect_breakpoint;

fn main() {
    println!("dont-ptrace-me: starting anti-debug checks...");
    println!("Enabled features: {}", enabled_features());

    #[allow(unused_mut)]
    let mut detected = false;

    #[cfg(feature = "ptrace_check")]
    {
        print!("[1/3] ptrace_check   ... ");
        let hit = detect_ptrace::is_debugger_attached();
        if hit { println!("DETECTED"); } else { println!("clean"); }
        detected |= hit;
    }
    #[cfg(not(feature = "ptrace_check"))]
    println!("[1/3] ptrace_check   ... skipped (feature not enabled)");

    #[cfg(feature = "procfs_check")]
    {
        print!("[2/3] procfs_check   ... ");
        let hit = detect_procfs::is_debugger_attached();
        if hit {
            let pid = detect_procfs::get_tracer_pid().unwrap_or(0);
            println!("DETECTED (TracerPid={})", pid);
        } else {
            println!("clean");
        }
        detected |= hit;
    }
    #[cfg(not(feature = "procfs_check"))]
    println!("[2/3] procfs_check   ... skipped (feature not enabled)");

    #[cfg(feature = "breakpoint_scan")]
    {
        print!("[3/3] breakpoint_scan ... ");
        let hit = detect_breakpoint::is_debugger_attached();
        if hit { println!("DETECTED"); } else { println!("clean"); }
        detected |= hit;
    }
    #[cfg(not(feature = "breakpoint_scan"))]
    println!("[3/3] breakpoint_scan ... skipped (feature not enabled)");

    println!();
    if detected {
        eprintln!("┌─────────────────────────────────────────┐");
        eprintln!("│  ⚠  DEBUGGER DETECTED — exiting now.   │");
        eprintln!("└─────────────────────────────────────────┘");
        std::process::exit(1);
    } else {
        println!("✓ No debugger detected. Running normally.");
        println!("  Sleeping for 30s — try attaching a debugger now...");
        std::thread::sleep(std::time::Duration::from_secs(30));
        println!("Done.");
    }
}

fn enabled_features() -> &'static str {
    #[cfg(all(feature = "ptrace_check", feature = "procfs_check", feature = "breakpoint_scan"))]
    return "all_checks";

    #[cfg(all(feature = "ptrace_check", not(feature = "procfs_check"), not(feature = "breakpoint_scan")))]
    return "ptrace_check";

    #[cfg(all(not(feature = "ptrace_check"), feature = "procfs_check", not(feature = "breakpoint_scan")))]
    return "procfs_check";

    #[cfg(all(not(feature = "ptrace_check"), not(feature = "procfs_check"), feature = "breakpoint_scan"))]
    return "breakpoint_scan";

    #[cfg(not(any(feature = "ptrace_check", feature = "procfs_check", feature = "breakpoint_scan")))]
    return "none (build with --features <feature>)";

    #[allow(unreachable_code)]
    "custom combination"
}
