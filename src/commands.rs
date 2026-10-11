//! A `Change` is any form of change affecting a disk, partition, etc.
//!
//! A command contains both a, but also its last current error or lack therof,
//! to be shown by the gui before pushing the writes to the disk(s)
//!
//! A change can only affect one disk at a time, (e.g. no cross-disk copying),
//! (this is due to parallalelism, but may be revoked in a future update, though
//! unlikely).

use crate::{disks::Disk, filesys::FileSystem, parts::PartitionScheme};

use std::fmt::{Display, Formatter};
use std::io::Write;

#[derive(Hash, Clone, PartialEq, Eq)]
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
    CreatePartition {
        parent_disk: String,
        size: u64,
        file_sys: FileSystem, // optional?
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
            Self::ReformatPartition {
                partition,
                file_sys,
            } => {
                write!(f, "Reformat {partition} to {file_sys}")
            }
            Self::CreatePartition {
                parent_disk,
                size,
                file_sys,
            } => {
                write!(
                    f,
                    "Create partition on {parent_disk} of size {size} with filesystem {file_sys}"
                )
            }
        }
    }
}

pub struct Command {
    pub change: Change,
    pub error: Option<String>,
    pub written_successfully: bool,
}

impl Command {
    pub fn new(change: Change) -> Self {
        Self {
            change,
            error: None,
            written_successfully: false,
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
            Change::ReformatPartition {
                ref partition,
                file_sys,
            } => {
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
            Change::CreatePartition {
                ref parent_disk,
                size,
                file_sys,
            } => {
                // Find the disk, if it exists, create a new partition entry with the given size and filesystem,
                // and add it to the disk's partitions. If the disk doesn't exist, set an error string.

                if let Some(disk) = disks.iter_mut().find(|d| &d.name == parent_disk) {
                    // note: the partition.name is actually the kernel name eg sda1 or nvme0n1p1
                    // name: the full partition address, e.g. "
                    // construct a new name for the partition
                    let new_index = disk // = highest + 1
                        .partitions
                        .iter()
                        .filter_map(|p| p.index)
                        .max()
                        .map_or(0, |max_index| max_index + 1);

                    let full_part_addr = format!("{}{}", disk.name, new_index + 1);

                    let new_partition = crate::parts::Partition {
                        name: full_part_addr.clone(),
                        index: Some(new_index),
                        size: Some(size),
                        fs: Some(file_sys),
                    };
                    disk.partitions.push(new_partition);
                    self.error = None;
                } else {
                    self.error = Some(format!("Could not find disk {parent_disk}"));
                }
            }
        }
    }

    /// dispatch to the relevant bdev subsytem, disks/parts/file_sys etc
    /// to perform the action, setting command.error as it goes, and
    /// returning the same error.
    ///
    /// Sets self.error to None unless the shell-out failed, in which
    /// case self.error is set to the stderr of the process
    pub fn perform(&mut self) {
        self.written_successfully = false;

        self.error = match self.change {
            Change::DeletePartitionTable { ref name } => {
                // shell out to wipefs
                match std::process::Command::new("wipefs")
                    .args(["--all", &format!("/dev/{name}")])
                    .output()
                {
                    Ok(output) if output.status.success() => None,
                    Ok(output) => Some(String::from_utf8_lossy(&output.stderr).trim().to_owned()),
                    Err(error) => Some(error.to_string()),
                }
            }

            Change::DeletePartition { ref name } => {
                // shell out to sfdisk
                let sysfs_path = format!("/sys/class/block/{name}");

                let Some(parent) = std::fs::canonicalize(&sysfs_path).ok().and_then(|path| {
                    path.parent()
                        .and_then(|parent| parent.file_name())
                        .map(|name| name.to_string_lossy().into_owned())
                }) else {
                    self.written_successfully = false;
                    return self.error = Some(format!("Could not determine parent disk of {name}"));
                };

                let Some(index) = crate::parts::detect_partition_index(name) else {
                    self.written_successfully = false;
                    return self.error =
                        Some(format!("Could not determine partition index of {name}"));
                };

                match std::process::Command::new("sfdisk")
                    .args(["--delete", &format!("/dev/{parent}"), &index.to_string()])
                    .output()
                {
                    Ok(output) if output.status.success() => None,
                    Ok(output) => Some(String::from_utf8_lossy(&output.stderr).trim().to_owned()),
                    Err(error) => Some(error.to_string()),
                }
            }

            Change::ReformatPartitionTable { ref name, scheme } => {
                // this is a lot more difficult
                let label = match scheme {
                    PartitionScheme::Mbr => "dos",
                    PartitionScheme::Gpt => "gpt",
                };

                let mut child = match std::process::Command::new("sfdisk")
                    .arg(format!("/dev/{name}"))
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                {
                    Ok(child) => child,
                    Err(error) => {
                        self.written_successfully = false;
                        return self.error = Some(error.to_string());
                    }
                };

                if let Some(mut stdin) = child.stdin.take() {
                    if let Err(error) = writeln!(stdin, "label: {label}") {
                        self.written_successfully = false;
                        return self.error = Some(error.to_string());
                    }
                } else {
                    self.written_successfully = false;
                    return self.error = Some("Failed to open sfdisk stdin".to_owned());
                }

                match child.wait_with_output() {
                    Ok(output) if output.status.success() => None,
                    Ok(output) => Some(String::from_utf8_lossy(&output.stderr).trim().to_owned()),
                    Err(error) => Some(error.to_string()),
                }
            }
            Change::ReformatPartition {
                ref partition,
                file_sys,
            } => {
                // shell out to mkfs
                //
                // I have considered hand rolling this, but have decided against it,
                // MKFS is already polished and maintained to a high standard,
                // and will receive any updates and fixes automatically, the main point
                // of bdev is to replace fdisk/lsblk and unify them into a simple, yet
                // powerfull TUI based (for now?) cli
                let file_sys = match file_sys {
                    FileSystem::Fat32 => "vfat",
                    FileSystem::Ntfs => "ntfs",
                    FileSystem::Ext4 => "ext4",
                };

                match std::process::Command::new("mkfs")
                    .args(["-t", file_sys, &format!("/dev/{partition}")])
                    .output()
                {
                    Ok(output) if output.status.success() => None,
                    Ok(output) => Some(String::from_utf8_lossy(&output.stderr).trim().to_owned()),
                    Err(error) => Some(error.to_string()),
                }
            }
            Change::CreatePartition {
                ref parent_disk,
                size,
                file_sys,
            } => {
                // shell out to sfdisk to create a new partition
                crate::parts::create_partition(&parent_disk, size, &file_sys)
            }
        };

        self.written_successfully = self.error.is_none();
    }
}