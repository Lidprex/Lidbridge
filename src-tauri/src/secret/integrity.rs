use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "windows")]
pub fn is_debugger_attached() -> bool {
    extern "system" {
        fn IsDebuggerPresent() -> bool;
    }
    unsafe { IsDebuggerPresent() }
}

#[cfg(not(target_os = "windows"))]
pub fn is_debugger_attached() -> bool {
    false
}

pub fn timing_check() -> bool {
    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let mut _sum = 0u64;
    for i in 0..1000 {
        _sum = _sum.wrapping_add(i);
    }

    let end = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let elapsed = end.saturating_sub(start);

    let elapsed_ms = elapsed / 1_000_000;

    elapsed_ms < 10
}

pub fn verify_blob_integrity(blob: &[u8], expected_magic: &[u8]) -> bool {
    if blob.len() < expected_magic.len() {
        return false;
    }

    let actual_magic = &blob[..expected_magic.len()];
    actual_magic == expected_magic
}

pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }

    diff == 0
}

pub fn secure_zero(data: &mut [u8]) {
    for byte in data.iter_mut() {
        unsafe {
            std::ptr::write_volatile(byte as *mut u8, 0);
        }
    }
}
