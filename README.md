# dont-ptrace-me

A Rust program demonstration of anti-debugging and anti-tracing techniques for Linux. It implements checks to detect when a process is running under a debugger or tracer.

## Features

- **Ptrace Check (`ptrace_check`)**: Detects if the process is already being ptraced.
- **Procfs Check (`procfs_check`)**: Scans system status via `/proc` to check for active tracers (`TracerPid`).
- **Breakpoint Scan (`breakpoint_scan`)**: Compares live in-memory executable bytes against the compiled binary on disk to detect injected software breakpoints (e.g., `0xCC` / `INT 3` on x86_64).

## Running Tests

To run the anti-debugging test suite with all checks enabled:

```bash
cargo test --features all_checks -- --nocapture
```
