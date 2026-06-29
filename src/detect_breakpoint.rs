use std::fs;
use std::io::{self, Read, Seek, SeekFrom};
use std::ops::Range;

#[cfg(target_arch = "x86_64")]
const BREAKPOINT_PATTERN: &[u8] = &[0xCC];

#[cfg(target_arch = "aarch64")]
const BREAKPOINT_PATTERN: &[u8] = &[0x00, 0x00, 0x20, 0xD4];

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const BREAKPOINT_PATTERN: &[u8] = &[];

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub addr_range: Range<usize>,
    pub file_offset: usize,
    pub perms: String,
    pub pathname: String,
}

pub fn parse_executable_regions() -> io::Result<Vec<MemoryRegion>> {
    let content = fs::read_to_string("/proc/self/maps")?;
    let mut regions = Vec::new();

    let exe_path = fs::read_link("/proc/self/exe")
        .ok()
        .and_then(|p| p.to_str().map(str::to_owned))
        .unwrap_or_default();

    for line in content.lines() {
        let mut parts = line.splitn(6, ' ');
        let addr_part = parts.next().unwrap_or("");
        let perms_part = parts.next().unwrap_or("");
        let offset_part = parts.next().unwrap_or("");
        let _dev = parts.next();
        let _inode = parts.next();
        let pathname = parts.next().unwrap_or("").trim().to_owned();

        if !perms_part.contains('x') {
            continue;
        }
        if !pathname.is_empty() && pathname != exe_path {
            continue;
        }

        let (start_str, end_str) = match addr_part.split_once('-') {
            Some(pair) => pair,
            None => continue,
        };
        let start = usize::from_str_radix(start_str, 16).unwrap_or(0);
        let end = usize::from_str_radix(end_str, 16).unwrap_or(0);
        let file_offset = usize::from_str_radix(offset_part, 16).unwrap_or(0);

        if start >= end {
            continue;
        }

        regions.push(MemoryRegion {
            addr_range: start..end,
            file_offset,
            perms: perms_part.to_owned(),
            pathname,
        });
    }

    Ok(regions)
}

pub fn read_process_memory(addr_range: &Range<usize>) -> io::Result<Vec<u8>> {
    let len = addr_range.end - addr_range.start;
    let mut file = fs::File::open("/proc/self/mem")?;
    file.seek(SeekFrom::Start(addr_range.start as u64))?;
    let mut buf = vec![0u8; len];
    file.read_exact(&mut buf)?;
    Ok(buf)
}

pub fn read_disk_bytes(file_offset: usize, len: usize) -> io::Result<Vec<u8>> {
    let exe_path = fs::read_link("/proc/self/exe")?;
    let mut file = fs::File::open(exe_path)?;
    file.seek(SeekFrom::Start(file_offset as u64))?;
    let mut buf = vec![0u8; len];
    let n = file.read(&mut buf)?;
    for b in &mut buf[n..] {
        *b = 0;
    }
    Ok(buf)
}

pub fn matches_breakpoint_pattern(bytes: &[u8], pos: usize) -> bool {
    matches_breakpoint_at(bytes, pos)
}

fn matches_breakpoint_at(bytes: &[u8], pos: usize) -> bool {
    let pat = BREAKPOINT_PATTERN;
    if pat.is_empty() {
        return false;
    }
    if pos + pat.len() > bytes.len() {
        return false;
    }
    &bytes[pos..pos + pat.len()] == pat
}

pub fn scan_for_breakpoints() -> Vec<usize> {
    if BREAKPOINT_PATTERN.is_empty() {
        return vec![];
    }

    let regions = match parse_executable_regions() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[breakpoint_scan] Could not parse /proc/self/maps: {}", e);
            return vec![];
        }
    };

    let mut hits: Vec<usize> = Vec::new();

    for region in &regions {
        let len = region.addr_range.end - region.addr_range.start;

        let mem_bytes = match read_process_memory(&region.addr_range) {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "[breakpoint_scan] Could not read memory {:x?}: {}",
                    region.addr_range, e
                );
                continue;
            }
        };

        let disk_bytes = match read_disk_bytes(region.file_offset, len) {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "[breakpoint_scan] Could not read disk bytes at offset {:x}: {}",
                    region.file_offset, e
                );
                continue;
            }
        };

        let pat_len = BREAKPOINT_PATTERN.len();
        let mut pos = 0;
        while pos + pat_len <= len {
            if matches_breakpoint_at(&mem_bytes, pos) && !matches_breakpoint_at(&disk_bytes, pos) {
                let vaddr = region.addr_range.start + pos;
                eprintln!(
                    "[breakpoint_scan] Breakpoint detected at virtual address 0x{:x}",
                    vaddr
                );
                hits.push(vaddr);
            }
            pos += pat_len;
        }
    }

    hits
}

pub fn is_debugger_attached() -> bool {
    let hits = scan_for_breakpoints();
    if !hits.is_empty() {
        eprintln!(
            "[breakpoint_scan] {} breakpoint(s) detected in memory.",
            hits.len()
        );
        true
    } else {
        false
    }
}
