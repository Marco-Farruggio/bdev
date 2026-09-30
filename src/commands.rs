//! A command is any form of change affecting a disk, partition, etc.
//! 
//! A command contains its command itself, but also its last attempt status,
//! to be shown by the gui before pushing the writes to the disk(s)
//! 
//! A command can only affect one disk at a time, (e.g. not cross-disk copying)

use crate::{
    disks::Disk,
    filesys::FileSystem,
    parts::PartitionScheme,
};

use std::fmt::{Display, Formatter};

#[derive(Hash, Clone)]
#[derive(PartialEq, Eq)]
pub enum Change {
    DeletePartitionTable {
        name: String,
    },
    DeletePartition {
        name: String,
    },
    ReformatPartitionTable {
        name: String,
        scheme: PartitionScheme,
    },
    ReformatPartition {
        partition: String,
        file_sys: FileSystem,
    },
}

impl Display for Change {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeletePartitionTable { name } => {
                write!(f, "Delete partition table on {name}")
            }
            Self::DeletePartition { name } => write!(f, "Delete partition {name}"),
            Self::ReformatPartitionTable { name, scheme } => {
                write!(f, "Reformat {name} to {scheme}")
            }
            Self::ReformatPartition { partition, file_sys } => {
                write!(f, "Reformat {partition} to {file_sys}")
            }
        }
    }
}
impl Change {
    pub fn relates_to(&self, device: &str) -> bool {
        match self {
            Self::DeletePartitionTable { name } => name == device,
            Self::DeletePartition { name } => name == device,
            Self::ReformatPartitionTable { name, .. } => name == device,
            Self::ReformatPartition { partition, .. } => partition == device,
        }
    }
}

pub struct Command {
    pub change: Change,
    pub error: Option<String>,
}

impl Command {
    pub fn new(change: Change) -> Self {
        Self {
            change,
            error: None,
        }
    }

    pub fn try_mem_apply(&mut self, disks: &mut Vec<Disk>) {
        match self.change {
            Change::DeletePartitionTable { ref name } => {
                // find the disk ('name'), set its .partition_scheme to None,
                // and set the error to None, if we can't find the disk (only
                // possible error at this stage, caused by a refresh), then
                // we set Error to a nice human readable string.
                if let Some(disk) = disks.iter_mut().find(|d| &d.name == name) {
                    disk.partition_scheme = Ok(None);
                    disk.partitions = Vec::new(); // Delete all partition entries
                    self.error = None;
                } else {
                    self.error = Some(format!("Can't find {name}. Try refreshing."));
                }
            }
            Change::DeletePartition { ref name } => {
                // Find the partition by full name, delete it from the disks list of partitions,
                // if we can't find said partition, set the error string
                // ...existing code...
                let mut found = false;

                for disk in disks.iter_mut() {
                    if let Some(pos) = disk.partitions.iter().position(|p| &p.name == name) {
                        disk.partitions.remove(pos);
                        found = true;
                        break;
                    }
                }

                if found {
                    self.error = None;
                } else {
                    self.error = Some(format!("Can't find {name}. Try refreshing."));
                }
            }
            Change::ReformatPartitionTable { ref name, scheme } => {
                // Find the disk, if it exists assume thats okay, otherwise, error string it is
                if let Some(disk) = disks.iter_mut().find(|d| &d.name == name) {
                    disk.partition_scheme = Ok(Some(scheme));
                    self.error = None;
                } else {
                    self.error = Some(format!("Could not find disk {name}"));
                }
            }
            Change::ReformatPartition { ref partition, file_sys } => {
                // Find the partition (by full name, I may IDed by disk AND partition soon),
                // and remove it, again, only error is if we can't find the partition (after a refresh),
                // and delete it from the list of the disk's partitions, which is why its crucial they
                // aren't IDed by a crude index into the vec. Same error logic applies.
                let mut part = None;
                for disk in disks.iter_mut() {
                    part = disk.partitions.iter_mut().find(|p| &p.name == partition);
                    if part.is_some() {
                        break;
                    }
                }
                
                if let Some(partition) = part {
                    partition.fs = Some(file_sys);
                    self.error = None;
                } else {
                    self.error = Some(format!("Can't find {partition}. Try refreshing."));
                }
            }
            // _ => {
            //     self.error = Some("This feature is not yet implemented.".into());
            // }
        }
    }

    /// dispatch to the relevant bdev subsytem, disks/parts/file_sys etc
    /// to perform the action, setting command.error as it goes, and 
    /// returning the same error.
    /// 
    /// Sets self.error to None unless the shell-out failed, in which
    /// case self.error is set to the stderr of the process
    pub fn perform(&mut self) {
        match self.change {
            Change::DeletePartitionTable { ref name } => {
                // shell out to wipefs
            }
            Change::DeletePartition { ref name } => {
                // shell out to wipefs
            }
            Change::ReformatPartitionTable { ref name, scheme } => {
                // this is a lot more difficult
            }
            Change::ReformatPartition { ref partition, file_sys } => {
                // shell out to mkfs
                //
                // I have considered hand rolling this, but have decided against it,
                // MKFS is already polished and maintained to a high standard,
                // and will receive any updates and fixes automatically, the main point
                // of bdev is to replace fdisk/lsblk and unify them into a simple, yet
                // powerfull TUI based (for now?) cli
            }
        }
    }
}