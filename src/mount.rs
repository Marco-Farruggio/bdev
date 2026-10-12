pub fn mount(dev_name: &str, mount_point: &str) -> Result<(), String> {
    let status = std::process::Command::new("mount")
        .arg(dev_name)
        .arg(mount_point)
        .status()
        .map_err(|e| format!("Failed to execute mount command: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Mount command failed with exit code: {}",
            status.code().unwrap_or(-1)
        ))
    }
}

pub fn unmount(dev_name: &str) -> Result<(), String> {
    let status = std::process::Command::new("umount")
        .arg(dev_name)
        .status()
        .map_err(|e| format!("Failed to execute umount command: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Umount command failed with exit code: {}",
            status.code().unwrap_or(-1)
        ))
    }
}

use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::fs;
use std::io;

fn dev_major(dev: u64) -> u32 {
    (((dev >> 8) & 0xfff) | ((dev >> 32) & !0xfff)) as u32
}

fn dev_minor(dev: u64) -> u32 {
    ((dev & 0xff) | ((dev >> 12) & !0xff)) as u32
}

/// Ok(vec![]) = checked, not mounted. Err = couldn't check.
pub fn mount_points(dev: &Path) -> io::Result<Vec<PathBuf>> {
    let meta = fs::metadata(dev)?; // follows symlinks
    if !meta.file_type().is_block_device() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "not a block device"));
    }
    let rdev = meta.rdev();
    let (maj, min) = (dev_major(rdev), dev_minor(rdev));
    let want = format!("{maj}:{min}");

    // read as bytes as mount points aren't strictly guaranteed to be valid UTF-8
    let info = fs::read("/proc/self/mountinfo")?;
    let mut out = Vec::new();

    for line in info.split(|&b| b == b'\n') {
        if line.is_empty() {
            continue;
        }
        // fields: id parent major:minor root mountpoint options ...
        let mut fields = line.split(|&b| b == b' ');
        let (Some(devno), Some(mp)) = (fields.nth(2), fields.nth(1)) else {
            continue;
        };
        if devno == want.as_bytes() {
            out.push(PathBuf::from(OsStr::from_bytes(&unescape(mp))));
        }
    }
    Ok(out)
}

/// mountinfo escapes certain whitespace characteres:
/// space     \040
/// tab       \011
/// newline   \012
/// backslash \134
fn unescape(s: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    let mut rest = s;

    while let Some((&b, tail)) = rest.split_first() {
        match (b, parse_octal_escape(tail)) {
            (b'\\', Some(byte)) => {
                out.push(byte);
                rest = &tail[3..];
            }
            _ => {
                out.push(b);
                rest = tail;
            }
        }
    }
    out
}

/// Parses exactly three octal digits at the start of `s`.
fn parse_octal_escape(s: &[u8]) -> Option<u8> {
    let digits = s.get(..3)?;
    let mut value: u32 = 0;
    for &d in digits {
        if !(b'0'..=b'7').contains(&d) {
            return None;
        }
        value = value * 8 + (d - b'0') as u32;
    }
    u8::try_from(value).ok()
}