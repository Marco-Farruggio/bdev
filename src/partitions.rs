use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PartitionScheme {
    Gpt,
    Mbr,
}

pub struct Partition {
    pub name: String,
    // pub index: Result<u8, std::io::Error>,
    pub size: Option<u64>,
    // bootable: Result<bool, std::io::Error>,
    // part_type: crate::mbr::MbrPartitionType,
}

impl std::fmt::Display for PartitionScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mbr => write!(f, "MBR"),
            Self::Gpt => write!(f, "GPT"),
        }
    }
}

pub fn scheme_to_str(scheme: &Option<PartitionScheme>) -> &'static str {
    match scheme {
        Some(PartitionScheme::Gpt) => "GPT",
        Some(PartitionScheme::Mbr) => "MBR",
        None                       => "RAW"
    }
}

pub fn read_partition_scheme(disk_name: &str) -> Result<Option<PartitionScheme>, std::io::Error> {
    let mut file = File::open(format!("/dev/{disk_name}"))?;

    let mut sector0 = [0u8; 512];
    file.read_exact(&mut sector0)?;

    if sector0[510] != 0x55 || sector0[511] != 0xAA {
        return Ok(None); // no partition table detected
    }

    let first_entry_type = sector0[450];

    if first_entry_type == 0xEE {
        file.seek(SeekFrom::Start(512))?;
        let mut sector1 = [0u8; 8];
        file.read_exact(&mut sector1)?;
        if &sector1 == b"EFI PART" {
            return Ok(Some(PartitionScheme::Gpt));
        }
    }

    Ok(Some(PartitionScheme::Mbr))
}

pub fn maybe_scheme_to_str(scheme: &Result<Option<PartitionScheme>, std::io::Error>) -> String {
    match scheme {
        Err(e) => e.to_string(),
        Ok(s) => {
            match s {
                Some(s) => {
                    match s {
                        PartitionScheme::Gpt => "GPT".into(),
                        PartitionScheme::Mbr => "MBR".into(),
                    }
                }
                None => "RAW".into()
            }
        }
    }
}