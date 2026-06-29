use nix::sys::ptrace;

pub fn is_debugger_attached() -> bool {
    match ptrace::traceme() {
        Ok(_) => false,
        Err(errno) => {
            eprintln!(
                "[ptrace_check] PTRACE_TRACEME failed: {} (errno {})",
                errno,
                errno as i32
            );
            true
        }
    }
}
