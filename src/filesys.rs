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

pub fn detect_fs(_first_sector: u64) -> Option<FileSystem> {
    None // [TODO]
}