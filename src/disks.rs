/// Reads a block device's size in bytes (via sysfs).
/// The `size` file *always* reports the size as a count of 512-byte sectors,
/// regardless of the device's real physical sector size, a longstanding kernel
/// convention.
pub fn read_size_bytes(name: &str) -> Option<u64> {
    let raw = std::fs::read_to_string(format!("/sys/class/block/{name}/size")).ok()?;
    let sectors: u64 = raw.trim().parse().ok()?;
    Some(sectors * 512)
}

/// Examples:
/// 24.3 GiB
/// 730.9 MiB
/// 138.2 KiB
/// 7 B
pub fn format_nbytes(nbytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

    if nbytes < 1024 {
        return format!("{nbytes} B");
    }

    let mut size = nbytes as f64;
    let mut unit_index = 0;
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{size:.1} {}", UNITS[unit_index])
}