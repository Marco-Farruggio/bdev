use crate::parts::PartitionScheme;
use crate::filesys::FileSystem;

#[derive(Debug, Hash, Clone)] // Temporarily
#[derive(PartialEq, Eq)]
pub enum Command {
    DeletePartitionTable { name: String },
    DeletePartition { name: String },
    ReformatPartitionTable { name: String, scheme: PartitionScheme },
    ReformatPartition { partition: String, file_sys: FileSystem },
}

impl Command {
    pub fn relates_to(&self, device: &str) -> bool {
        match self {
            Self::DeletePartitionTable { name} => name == device,
            Self::DeletePartition { name } => name == device,
            Self::ReformatPartitionTable { name , .. } => name == device,
            Self::ReformatPartition { partition, .. } => partition == device,
        }
    }
}