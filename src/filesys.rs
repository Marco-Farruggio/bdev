use std::fmt::{Display, Formatter};

#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub enum FileSystem {
    Fat32,
    Ntfs,
    Ext4,
}

impl Display for FileSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fat32 => write!(f, "FAT32"),
            Self::Ntfs  => write!(f, "NTFS"),
            Self::Ext4  => write!(f, "ext4"),
        }
    }
}

pub fn detect_fs_raw(_first_sector: u64) -> Option<FileSystem> {
    None // [TODO]
}

/// detect the filesystem of a given device via libblkid,
/// this keeps and detection centralized and consistent.
pub fn detect_part_fs(dev_name: &str) -> Option<FileSystem> {
    None
}

/// detects if the disk is 'superfloppy' (filesystem directly
/// on the disk, with no partition table)
/// 
/// Note: This should only be called if it has been previously
/// ascertained that the disk has no partition table, as it will
/// read blindly under that assumption
/// 
/// uses libblkid, for justification, see `detect_part_fs`
pub fn detect_disk_fs(dev_name: &str) -> Option<FileSystem> {
    None
}