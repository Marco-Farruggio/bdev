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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PartitionID(u32);

#[derive(Clone)]
pub struct Partition {
    pub name: String,
    pub index: Option<PartitionID>,
    pub size: Option<u64>,
    pub fs: Option<FileSystem>,
    // bootable: Result<bool, std::io::Error>,
    // part_type: crate::mbr::MbrPartitionType,
}

pub fn scheme_to_str(scheme: &Option<PartitionScheme>) -> &'static str {
    match scheme {
        Some(PartitionScheme::Gpt) => "GPT",
        Some(PartitionScheme::Mbr) => "MBR",
        None => "RAW",
    }
}

pub fn detect_partition_scheme(disk_name: &str) -> Result<Option<PartitionScheme>, String> {
    let mut file = File::open(format!("/dev/{disk_name}"))
        .map_err(|error| error.to_string())?;

    let mut sector0 = [0u8; 512];
    file.read_exact(&mut sector0)
        .map_err(|error| error.to_string())?;

    if sector0[510] != 0x55 || sector0[511] != 0xAA {
        return Ok(None); // no partition table detected
    }

    let first_entry_type = sector0[450];

    if first_entry_type == 0xEE {
        file.seek(SeekFrom::Start(512))
            .map_err(|error| error.to_string())?;
        let mut sector1 = [0u8; 8];
        file.read_exact(&mut sector1)
            .map_err(|error| error.to_string())?;
        if &sector1 == b"EFI PART" {
            return Ok(Some(PartitionScheme::Gpt));
        }
    }

    Ok(Some(PartitionScheme::Mbr))
}

pub fn maybe_scheme_to_str(scheme: &Result<Option<PartitionScheme>, String>) -> String {
    match scheme {
        Err(e) => e.to_string(),
        Ok(s) => match s {
            Some(s) => match s {
                PartitionScheme::Gpt => "GPT".into(),
                PartitionScheme::Mbr => "MBR".into(),
            },
            None => "RAW".into(),
        },
    }
}

pub fn is_partition(name: &str) -> bool {
    fs::metadata(format!("/sys/class/block/{name}/partition")).is_ok()
}

/// takes a device's full name and attempts to detect its
/// partition index directly by reading the partition table
pub fn detect_partition_index(name: &str) -> Option<PartitionID> {
    fs::read_to_string(format!("/sys/class/block/{name}/partition"))
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok().map(|u| PartitionID { 0: u })) // [TODO] PartitionID: new
}
