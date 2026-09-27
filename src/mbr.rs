pub enum MbrPartitionType {
    Empty = 0x00,
    Fat12 = 0x01,
    XenixRoot = 0x02,
    ExtendedPartition = 0x05,
    Fat32 = 0x0B,
    LinuxSwapPartition = 0x82,
    LinuxNativeFileSystem = 0x83,
    OpenBsd = 0xA6,
    NetBsd = 0xA9,
    SolarisBootPartition = 0xBE,
    GptProtective = 0xEE,
    // Other(u8) // Temporary
}

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

#[derive(Debug, Clone, Copy)]
pub struct MbrPartitionInfo {
    pub index: u8,        // 1-4
    pub bootable: bool,   // true if *and only if* boot flag == 0x80
    pub part_type: u8,    // partition type byte (see `MbrPartitionType`)
}

/// Reads the 4 primary partition entries from a MBR disk,
/// *note*: it is important that the disk is assuredly pure-MBR
/// beforehand, not protective GPT, as this function performs no checks,
/// and will read & interpret data blindy.
/// 
/// This includes the bootable flag, index, and the type byte
pub fn read_mbr_partition_info(devname: &str) -> io::Result<Vec<MbrPartitionInfo>> {
    const PARTITION_TABLE_OFFSET: u64 = 0x1BE;
    const ENTRY_SIZE: usize = 16;
    const NUM_ENTRIES: usize = 4;
    const SIGNATURE_OFFSET: u64 = 0x1FE;

    let mut f = File::open(devname)?;

    // Verify boot signature 0x55AA.
    f.seek(SeekFrom::Start(SIGNATURE_OFFSET))?;
    let mut sig = [0u8; 2];
    f.read_exact(&mut sig)?;
    if sig != [0x55, 0xAA] {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("missing MBR boot signature (got {:02x?})", sig),
        ));
    }

    // read the 4 partition table entries
    f.seek(SeekFrom::Start(PARTITION_TABLE_OFFSET))?;
    let mut table = [0u8; ENTRY_SIZE * NUM_ENTRIES];
    f.read_exact(&mut table)?;

    let mut partitions = Vec::with_capacity(NUM_ENTRIES);
    for i in 0..NUM_ENTRIES {
        let entry = &table[i * ENTRY_SIZE..(i + 1) * ENTRY_SIZE];
        let boot_flag = entry[0];
        let part_type = entry[4];

        partitions.push(MbrPartitionInfo {
            index: (i + 1) as u8,
            bootable: boot_flag == 0x80,
            part_type,
        });
    }

    Ok(partitions)
}