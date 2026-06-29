use std::fs;
use std::io;

pub fn get_tracer_pid() -> Option<u32> {
    let contents = fs::read_to_string("/proc/self/status").ok()?;
    parse_tracer_pid(&contents)
}

pub fn parse_tracer_pid(content: &str) -> Option<u32> {
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("TracerPid:") {
            return rest.trim().parse::<u32>().ok();
        }
    }
    None
}

pub fn is_debugger_attached() -> bool {
    match get_tracer_pid() {
        Some(pid) if pid != 0 => {
            eprintln!("[procfs_check] TracerPid is non-zero: {}", pid);
            true
        }
        Some(_) => false,
        None => {
            eprintln!("[procfs_check] Warning: could not read TracerPid from /proc/self/status");
            false
        }
    }
}

pub fn read_status_file() -> io::Result<String> {
    fs::read_to_string("/proc/self/status")
}
