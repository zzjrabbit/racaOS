use alloc::vec::Vec;
use ostd::{mm::Vaddr, task::Task};

use {
    crate::{AsThread, UserThreadData},
    ::filesystem::{AccessMode, FileDescriptor, FileType, InodeMode, OpenFlags, Path, open_file},
};

use super::*;

pub fn open(address: Vaddr, flags: i32, mode: u32) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    let access_mode = AccessMode::try_from(flags)
        .map_err(|()| Errno::EINVAL.with_message("Invalid access mode."))?;
    let open_flags = OpenFlags::from(flags);

    let _mode = InodeMode::from_bits_truncate(mode as u16);

    let mut path = Vec::new();
    loop {
        let byte = data.memory_info().vmar().read_val(address + path.len())?;
        if byte == 0 {
            break;
        }
        path.push(byte);
    }

    let path = Path::new(
        core::str::from_utf8(&path)
            .map_err(|_| Errno::EINVAL.with_message("Unable to parse path with utf-8."))?,
    );

    if let Some(file) = data.open_file(&path) {
        let fd = data.fs_info().add_file(file, access_mode, open_flags);
        Ok(fd as isize)
    } else if open_flags.contains(OpenFlags::O_CREAT) {
        let parent = path.parent().ok_or(Errno::EACCES.no_message())?;
        let file = open_file(&parent).ok_or(Errno::ENOENT.no_message())?;

        let file = file
            .create(
                path.name(),
                if open_flags.contains(OpenFlags::O_DIRECTORY) {
                    FileType::Directory
                } else {
                    FileType::File
                },
            )
            .ok_or(Errno::ENOENT.no_message())?;

        let fd = data.fs_info().add_file(file, access_mode, open_flags);
        Ok(fd as isize)
    } else {
        Err(Errno::ENOENT.no_message())
    }
}

pub fn close(fd: FileDescriptor) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    data.fs_info()
        .remove_file(fd)
        .ok_or(Errno::EBADFD.no_message())?;
    Ok(0)
}
