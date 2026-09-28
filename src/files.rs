#[derive(Debug, Hash, Clone)] // Temporarily
#[derive(PartialEq, Eq)]
pub enum FileSystem {
    Fat32,
    Ntfs,
    Ext4,
}