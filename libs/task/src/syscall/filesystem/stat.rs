use ::filesystem::{File, open_file};
use alloc::sync::Arc;
use ostd::{Pod, task::Task};

use crate::{AsThread, UserThreadData};

use super::*;

#[derive(Debug, Clone, Copy, Pod, Default)]
#[repr(C)]
struct Stat {
    dev: u64,
    ino: u64,
    nlink: u64,
    mode: u32,
    uid: u32,
    gid: u32,
    __pad0: u32,
    rdev: u64,
    size: i64,
    blksize: i64,
    blocks: i64,
    atime: timespec_t,
    mtime: timespec_t,
    ctime: timespec_t,
    __unused: [i64; 3],
}

bitflags::bitflags! {
    struct StatFlags: u32 {
        const AT_EMPTY_PATH = 1 << 12;
        const AT_NO_AUTOMOUNT = 1 << 11;
        const AT_SYMLINK_NOFOLLOW = 1 << 8;
    }
}

pub fn fstat(fd: FileDescriptor, address: Vaddr) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    let stat = data
        .fs_info()
        .with_file(fd, |_, _, _, file| get_stat(file))
        .ok_or(Errno::EBADFD.no_message())?;
    data.memory_info().vmar().write_val(address, &stat)?;

    Ok(0)
}

pub fn stat(file_name_address: Vaddr, stat_address: Vaddr) -> SyscallResult {
    fstatat(None, file_name_address, stat_address, 0)
}

pub fn lstat(file_name_address: Vaddr, stat_address: Vaddr) -> SyscallResult {
    fstatat(
        None,
        file_name_address,
        stat_address,
        StatFlags::AT_SYMLINK_NOFOLLOW.bits(),
    )
}

pub fn fstatat(
    fd: Option<FileDescriptor>,
    file_name_address: Vaddr,
    stat_address: Vaddr,
    flags: u32,
) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();
    let flags = StatFlags::from_bits(flags).ok_or(Errno::EINVAL.no_message())?;
    let file_name = data
        .memory_info()
        .vmar()
        .read_cstring(file_name_address, None)?;

    if file_name.is_empty() {
        if !flags.contains(StatFlags::AT_EMPTY_PATH) {
            return Err(Errno::ENOENT.with_message("Path is empty."));
        }
        return fstat(fd.unwrap(), stat_address);
    }

    let path = {
        let file_name = file_name.to_string_lossy().into_owned();
        let path = fd
            .map(|fd| {
                data.fs_info()
                    .with_file(fd, |_, _, _, file| file.path())
                    .ok_or(Errno::EBADFD.no_message())
            })
            .unwrap_or(Ok(data.cwd()))?;
        path.join(&file_name)
    };
    let file = open_file(&path).ok_or(Errno::ENOENT.no_message())?;

    let stat = get_stat(file);
    data.memory_info().vmar().write_val(stat_address, &stat)?;
    Ok(0)
}

fn get_stat(file: Arc<File>) -> Stat {
    let metadata = file.metadata();
    let inode_id = file.inode_id();
    let link_num = file.link_num();

    let size = file.len() as i64;
    let block_num = size.div_ceil(512);

    Stat {
        dev: metadata.dev,
        ino: inode_id,
        nlink: link_num as u64,
        mode: metadata.inode_mode.bits() as u32,
        uid: u32::from(metadata.uid),
        gid: u32::from(metadata.gid),
        __pad0: 0,
        rdev: metadata.rdev,
        size,
        blksize: 512,
        blocks: block_num,
        atime: timespec_t::from(metadata.access_time),
        mtime: timespec_t::from(metadata.modification_time),
        ctime: timespec_t::from(metadata.creation_time),
        __unused: [0; 3],
    }
}
