// SPDX-License-Identifier: GPL-2.0-only
// Adapted from os-findmy-e macOS-ADI-emulator/stubs.py; all paths remain virtual.
use crate::{
    Error,
    runtime::{Cpu, argument as arg},
};
pub(crate) fn cstr(cpu: &Cpu<'_>, address: u64) -> Result<String, Error> {
    let mut data = Vec::new();
    for i in 0..4096 {
        let b = cpu.mem_read_as_vec(
            address
                .checked_add(i)
                .ok_or(Error::Execution("string address overflow"))?,
            1,
        )?[0];
        if b == 0 {
            return String::from_utf8(data).map_err(|_| Error::Execution("invalid UTF-8 string"));
        }
        data.push(b);
    }
    Err(Error::Execution("unterminated guest string"))
}
fn errno(cpu: &mut Cpu<'_>, code: i32) -> Result<u64, Error> {
    let p = cpu.get_data().errno;
    cpu.mem_write(p, &code.to_le_bytes())?;
    Ok(u64::MAX)
}
pub(crate) fn call(cpu: &mut Cpu<'_>, name: &str) -> Result<Option<u64>, Error> {
    let a = arg(cpu, 0)?;
    let b = arg(cpu, 1)?;
    let c = arg(cpu, 2)?;
    let result = match name {
        "error" => cpu.get_data().errno,
        "open" => {
            let path = cstr(cpu, a)?;
            match cpu.get_data_mut().fs.open(path, b) {
                Ok(fd) => fd,
                Err(e) => errno(cpu, e)?,
            }
        }
        "close" => match cpu.get_data_mut().fs.close(a) {
            Ok(()) => 0,
            Err(e) => errno(cpu, e)?,
        },
        "read" => match cpu
            .get_data_mut()
            .fs
            .read(a, c.min(4 * 1024 * 1024) as usize)
        {
            Ok(bytes) => {
                cpu.mem_write(b, &bytes)?;
                bytes.len() as u64
            }
            Err(e) => errno(cpu, e)?,
        },
        "write" => {
            if c > 1024 * 1024 {
                return Ok(Some(errno(cpu, 28)?));
            }
            let bytes = cpu.mem_read_as_vec(b, c as usize)?;
            match cpu.get_data_mut().fs.write(a, &bytes) {
                Ok(n) => n as u64,
                Err(e) => errno(cpu, e)?,
            }
        }
        "ftruncate" => {
            let result = usize::try_from(b)
                .map_err(|_| 28)
                .and_then(|n| cpu.get_data_mut().fs.truncate(a, n));
            match result {
                Ok(()) => 0,
                Err(e) => errno(cpu, e)?,
            }
        }
        "lstat$INODE64" | "stat$INODE64" | "fstat$INODE64" => {
            let stat = if name == "fstat$INODE64" {
                cpu.get_data().fs.stat_fd(a)
            } else {
                cpu.get_data().fs.stat(&cstr(cpu, a)?)
            };
            match stat {
                Ok(bytes) => {
                    cpu.mem_write(b, &bytes)?;
                    0
                }
                Err(e) => errno(cpu, e)?,
            }
        }
        "statfs$INODE64" => {
            let mut bytes = [0u8; 2168];
            bytes[..4].copy_from_slice(&0x1000003u32.to_le_bytes());
            cpu.mem_write(b, &bytes)?;
            0
        }
        "mkdir" => {
            let path = cstr(cpu, a)?;
            match cpu.get_data_mut().fs.mkdir(path) {
                Ok(()) => 0,
                Err(e) => errno(cpu, e)?,
            }
        }
        "unlink" => {
            let path = cstr(cpu, a)?;
            match cpu.get_data_mut().fs.unlink(&path) {
                Ok(()) => 0,
                Err(e) => errno(cpu, e)?,
            }
        }
        "chmod" => {
            if cpu.get_data().fs.exists(&cstr(cpu, a)?) {
                0
            } else {
                errno(cpu, 2)?
            }
        }
        "fchmod"
        | "fcntl"
        | "lockf"
        | "notify_post"
        | "CFErrorGetCode"
        | "CFErrorGetDomain"
        | "CSBackupIsItemExcluded" => 0,
        "CFFileSecuritySetMode" | "CSBackupSetItemExcluded" | "CFURLSetResourcePropertyForKey" => 1,
        "NSVersionOfRunTimeLibrary" => 0x100000000,
        "gethostuuid" => {
            let uuid = cpu.get_data().host_uuid;
            cpu.mem_write(a, &uuid)?;
            0
        }
        "getenv" => {
            let name = cstr(cpu, a)?;
            let value = match name.as_str() {
                "HOME" => "/Users/aditest\0",
                "USER" => "aditest\0",
                "TMPDIR" => "/tmp/\0",
                _ => return Ok(Some(0)),
            };
            let p = cpu.get_data_mut().scratch(value.len() as u64)?;
            cpu.mem_write(p, value.as_bytes())?;
            p
        }
        _ => return Ok(None),
    };
    Ok(Some(result))
}
