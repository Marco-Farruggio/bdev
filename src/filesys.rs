#[derive(Debug, Hash, Clone)] // Temporarily
#[derive(PartialEq, Eq)]
pub enum FileSystem {
    Fat32,
    Ntfs,
    Ext4,
}

pub fn detect_fs(first_sector: u64) -> Option<FileSystem> {
    None // [TODO]
}