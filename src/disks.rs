use crate::partitions::{
    Partition,
    PartitionScheme,
    is_partition,
    detect_partition_scheme,
    detect_partition_index,
};

use std::collections::HashMap;
use std::fs;


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

pub struct Disk {
    pub name: String,
    pub size: Option<u64>,
    pub partition_scheme: Result<Option<PartitionScheme>, std::io::Error>,
    pub partitions: Vec<Partition>,
}

pub fn read_block_devices() -> Vec<Disk> {
    let mut names: Vec<String> = match fs::read_dir("/sys/class/block") {
        Ok(read_dir) => read_dir
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect(),
        Err(_) => return Vec::new(),
    };
    names.sort();

    let mut disks: Vec<Disk> = Vec::new();
    let mut partitions_of: HashMap<String, Vec<Partition>> = HashMap::new();

    for name in &names {
        if is_partition(name) {
            let link_path = format!("/sys/class/block/{name}");
            if let Ok(real_path) = fs::canonicalize(&link_path) {
                if let Some(parent) = real_path
                    .parent()
                    .and_then(|p| p.file_name())
                    .map(|f| f.to_string_lossy().into_owned())
                {
                    partitions_of.entry(parent).or_default().push(
                        Partition {
                            name: name.clone(),
                            index: detect_partition_index(&name),
                            size: crate::disks::read_size_bytes(&name)
                        }
                    );
                }
            }
        } else {
            disks.push(Disk {
                name: name.clone(),
                size: crate::disks::read_size_bytes(name),
                partition_scheme: detect_partition_scheme(name),
                partitions: Vec::new(),
            });
        }
    }

    for disk in &mut disks {
        if let Some(parts) = partitions_of.remove(&disk.name) {
            disk.partitions = parts;
        }
    }

    disks
}