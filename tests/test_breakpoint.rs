#[cfg(feature = "breakpoint_scan")]
mod breakpoint_tests {
    use dont_ptrace_me::detect_breakpoint::{
        is_debugger_attached, matches_breakpoint_pattern, parse_executable_regions,
        read_disk_bytes, read_process_memory, scan_for_breakpoints,
    };

    #[test]
    fn test_breakpoint_pattern_detected_at_start() {
        println!("\n  [bp] Testing breakpoint pattern detection at offset 0 ...");

        #[cfg(target_arch = "x86_64")]
        let buf = vec![0xCC_u8, 0x90, 0x90];
        #[cfg(target_arch = "aarch64")]
        let buf = vec![0x00_u8, 0x00, 0x20, 0xD4, 0xFF, 0xFF];
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        let buf: Vec<u8> = vec![];

        if buf.is_empty() {
            println!("  [bp] Unsupported architecture — skipping.");
            return;
        }

        println!("  [bp] Buffer: {:02X?}", &buf);
        let result = matches_breakpoint_pattern(&buf, 0);
        println!("  [bp] matches_breakpoint_pattern at 0 = {}", result);
        assert!(result, "Should detect breakpoint at offset 0");
        println!("  [bp] Breakpoint pattern recognised at offset 0 ✓");
    }

    #[test]
    fn test_no_false_positive_on_clean_bytes() {
        println!("\n  [bp] Testing that all-zero bytes do NOT match breakpoint pattern ...");
        let buf = vec![0x00_u8; 16];
        println!("  [bp] Buffer: {:02X?}", &buf[..4]);

        #[cfg(target_arch = "x86_64")]
        {
            let result = matches_breakpoint_pattern(&buf, 0);
            println!("  [bp] x86_64: 0x00 matches INT3 (0xCC) = {} (expected false)", result);
            assert!(!result, "0x00 should not match INT3");
        }
        #[cfg(target_arch = "aarch64")]
        {
            let result = matches_breakpoint_pattern(&buf, 0);
            println!("  [bp] aarch64: all-zeros matches BRK #0 = {} (expected false)", result);
            assert!(!result, "All-zeros should not match BRK #0 [0x00,0x00,0x20,0xD4]");
        }
        println!("  [bp] No false positive on zero bytes ✓");
    }

    #[test]
    fn test_breakpoint_pattern_not_detected_mid_buffer() {
        println!("\n  [bp] Scanning all-zero 16-byte buffer for breakpoints at every offset ...");
        let buf = vec![0x00_u8; 16];
        let mut found = false;
        for i in 0..16 {
            if matches_breakpoint_pattern(&buf, i) {
                println!("  [bp] Unexpected match at offset {}", i);
                found = true;
            }
        }
        println!("  [bp] Any match found = {} (expected false)", found);
        assert!(!found, "Should not find breakpoints in all-zero buffer");
        println!("  [bp] No breakpoints in zero buffer ✓");
    }

    #[test]
    fn test_parse_executable_regions() {
        println!("\n  [bp] Parsing /proc/self/maps for executable (r-xp) regions ...");
        let regions = parse_executable_regions()
            .expect("Should parse /proc/self/maps");
        println!("  [bp] Found {} executable region(s):", regions.len());
        for r in &regions {
            println!("       0x{:x}–0x{:x}  {}  offset=0x{:x}  {}",
                r.addr_range.start, r.addr_range.end,
                r.perms, r.file_offset,
                if r.pathname.is_empty() { "<anonymous>" } else { &r.pathname }
            );
            assert!(r.perms.contains('x'), "Region must be executable: {}", r.perms);
        }
        assert!(!regions.is_empty(), "Must find at least one executable region");
        println!("  [bp] /proc/self/maps parsed successfully ✓");
    }

    #[test]
    fn test_read_process_memory() {
        println!("\n  [bp] Reading first 16 bytes of executable region from /proc/self/mem ...");
        let regions = parse_executable_regions().expect("Should parse /proc/self/maps");
        let first = &regions[0];
        let small_range = first.addr_range.start..(first.addr_range.start + 16);
        println!("  [bp] Reading range 0x{:x}–0x{:x} ...", small_range.start, small_range.end);
        let bytes = read_process_memory(&small_range)
            .expect("Should read process memory");
        println!("  [bp] Read {} bytes: {:02X?}", bytes.len(), bytes);
        assert_eq!(bytes.len(), 16, "Should read exactly 16 bytes");
        println!("  [bp] Process memory readable ✓");
    }

    #[test]
    fn test_read_disk_bytes() {
        println!("\n  [bp] Reading first 4 bytes of /proc/self/exe (expect ELF magic) ...");
        let bytes = read_disk_bytes(0, 4).expect("Should read disk bytes");
        println!("  [bp] Read bytes: {:02X?}", bytes);
        assert_eq!(bytes.len(), 4);
        assert_eq!(&bytes[0..4], &[0x7F, b'E', b'L', b'F'],
            "First 4 bytes must be ELF magic 7F 45 4C 46");
        println!("  [bp] ELF magic confirmed: 7F 45 4C 46 ✓");
    }

    #[test]
    fn test_no_breakpoints_without_debugger() {
        println!("\n  [bp] Scanning .text segment for debugger-injected breakpoints ...");
        let hits = scan_for_breakpoints();
        if hits.is_empty() {
            println!("  [bp] No breakpoints found in memory ✓");
        } else {
            println!("  [bp] Unexpected breakpoints at: {:x?}", hits);
        }
        assert!(hits.is_empty(),
            "Should find no breakpoints without a debugger. Found {} at: {:x?}",
            hits.len(), hits);
    }

    #[test]
    fn test_not_detected_without_debugger() {
        println!("\n  [bp] Running is_debugger_attached() (full pipeline check) ...");
        let detected = is_debugger_attached();
        println!("  [bp] Result: {}", if detected { "DETECTED (unexpected!)" } else { "clean ✓" });
        assert!(!detected, "False positive: breakpoint_scan triggered without a debugger");
        println!("  [bp] No debugger-injected breakpoints found ✓");
    }
}
