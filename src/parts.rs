use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};

use crate::filesys::FileSystem;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] // Temporarily
pub enum PartitionScheme {
    Gpt,
    Mbr,
}
impl std::fmt::Display for PartitionScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mbr => write!(f, "MBR"),
            Self::Gpt => write!(f, "GPT"),
        }
    }
}

#[derive(Clone)]
pub struct Partition {
    pub name: String,
    pub index: Option<u32>,
    pub size: Option<u64>,
    pub fs: Option<FileSystem>,
    // bootable: Result<bool, std::io::Error>,
    // part_type: crate::mbr::MbrPartitionType,
}

pub fn disk_scheme_label(scheme: &Result<Option<PartitionScheme>, String>) -> String {
    match scheme {
        Ok(Some(PartitionScheme::Gpt)) => "[GPT]".to_string(),
        Ok(Some(PartitionScheme::Mbr)) => "[MBR]".to_string(),
        Ok(None) => "[RAW]".to_string(),
        Err(error) => format!("[{error}]"),
    }
}

pub fn detect_partition_scheme(disk_name: &str) -> Result<Option<PartitionScheme>, String> {
    let path = format!("/dev/{disk_name}");

    //  todo: duplicate this everywhere
    let mut f = match File::open(&path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            // return Err(format!("bdev: cannot open {path}: permission denied (try running with sudo)"));
            return Err("Permission Denied (Try sudo)".to_string());
        }
        Err(e) => return Err(e.to_string()),
    };

    let mut sector0 = [0u8; 512];
    f.read_exact(&mut sector0)
        .map_err(|error| error.to_string())?;

    if sector0[510] != 0x55 || sector0[511] != 0xAA {
        return Ok(None); // no partition table detected
    }

    let first_entry_type = sector0[450];

    if first_entry_type == 0xEE {
        f.seek(SeekFrom::Start(512))
            .map_err(|error| error.to_string())?;
        let mut sector1 = [0u8; 8];
        f.read_exact(&mut sector1)
            .map_err(|error| error.to_string())?;
        if &sector1 == b"EFI PART" {
            return Ok(Some(PartitionScheme::Gpt));
        }
    }

    Ok(Some(PartitionScheme::Mbr))
}

pub fn is_partition(name: &str) -> bool {
    fs::metadata(format!("/sys/class/block/{name}/partition")).is_ok()
}

/// takes a device's full name and attempts to detect its
/// partition index
pub fn detect_partition_index(name: &str) -> Option<u32> {
    fs::read_to_string(format!("/sys/class/block/{name}/partition"))
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
}

use std::path::Path;

pub fn get_parent_disk_name(name: &str) -> Option<String> {
    let base = format!("/sys/class/block/{name}");

    // Whole disks have no "partition" attribute, so treat them as their own parent.
    if !Path::new(&format!("{base}/partition")).exists() {
        return Some(name.to_owned());
    }

    // Resolve the symlink to the real device path, then take its parent directory's name.
    fs::canonicalize(&base)
        .ok()?
        .parent()?
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
}

use std::io::Write;
use std::process::{Command, Stdio};

/// Returns Ok(Some("gpt" | "dos" | ...)) if the disk has a table, Ok(None) if it has none.
fn partition_table_type(disk: &str) -> Result<Option<String>, String> {
    let output = Command::new("lsblk")
        .args(["-dno", "PTTYPE"])
        .arg(format!("/dev/{disk}"))
        .output()
        .map_err(|e| format!("failed to run lsblk: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }

    let label = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    Ok(if label.is_empty() { None } else { Some(label) })
}

/// Creates a partition of `size` bytes on `parent_disk` (e.g. "sdb").
/// Returns None on success, or Some(error message) on failure.
pub fn create_partition(parent_disk: &str, size: u64, file_sys: &FileSystem) -> Option<String> {
    let table = match partition_table_type(parent_disk) {
        Ok(t) => t,
        Err(e) => return Some(e),
    };

    // Type codes differ by label: hex for dos, GUIDs for gpt.
    let is_dos = table.as_deref() == Some("dos");
    let type_code = match (is_dos, file_sys) {
        (true, FileSystem::Fat32) => "c",
        (true, FileSystem::Ntfs) => "7",
        (true, FileSystem::Ext4) => "83",
        (false, FileSystem::Fat32) | (false, FileSystem::Ntfs) => {
            "EBD0A0A2-B9E5-4433-87C0-68B6B72699C7" // Microsoft basic data
        }
        (false, FileSystem::Ext4) => "0FC63DAF-8483-4772-8E79-3D69D8477DE4", // Linux filesystem
    };

    // Bytes -> KiB. Prints "1048576" for whole KiB, a decimal point only when needed.
    let kib = size as f64 / 1024.0;
    let line = format!("size={kib}KiB, type={type_code}");

    // No table yet: nothing to append to, so create a GPT one. Otherwise keep
    // the existing partitions.
    // todo: possible error for creating a new partition on a disk with no table?
    // ensure they set a parition scheme first
    let (script, append) = match table {
        Some(_) => (line, true),
        None => (format!("label: gpt\n{line}"), false),
    };

    let mut cmd = Command::new("sfdisk");
    if append {
        cmd.arg("--append");
    }
    cmd.arg(format!("/dev/{parent_disk}"))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return Some(format!("failed to run sfdisk: {e}")),
    };

    match child.stdin.take() {
        Some(mut stdin) => {
            if let Err(e) = writeln!(stdin, "{script}") {
                return Some(e.to_string());
            }
        } // stdin dropped here, which closes the pipe so sfdisk sees EOF
        None => return Some("Failed to open sfdisk stdin".to_owned()),
    }

    match child.wait_with_output() {
        Ok(output) if output.status.success() => None,
        Ok(output) => Some(String::from_utf8_lossy(&output.stderr).trim().to_owned()),
        Err(e) => Some(e.to_string()),
    }
}