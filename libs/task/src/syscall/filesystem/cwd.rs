use ::filesystem::FileType;
use alloc::vec::Vec;
use ostd::{mm::Vaddr, task::Task};

use {
    crate::{AsThread, UserThreadData},
    ::filesystem::{FileDescriptor, Path},
};

use super::*;

pub fn getcwd(buffer: Vaddr, len: usize) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();
    let vmar = data.memory_info().vmar();
    let cwd = data.cwd();

    if cwd.len() >= len {
        Ok(0)
    } else if let Err(_) = vmar.write(buffer, cwd.as_bytes()) {
        Ok(0)
    } else {
        Ok(buffer as isize)
    }
}

pub fn chdir(file_name: Vaddr) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();
    let vmar = data.memory_info().vmar();

    let mut buffer = Vec::new();
    buffer.push(vmar.read_val(file_name)?);

    while *buffer.last().unwrap() != 0u8 {
        buffer.push(vmar.read_val(file_name + buffer.len() as Vaddr)?);
    }
    buffer.pop().unwrap();

    let path = core::str::from_utf8(&buffer).map_err(|_| Errno::EINVAL.no_message())?;
    let path = Path::from(path);

    if data.open_file(&path).is_none() {
        return Err(Errno::ENOENT.no_message());
    }

    data.set_cwd(path);

    Ok(0)
}

pub fn fchdir(fd: FileDescriptor) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    let path = data
        .fs_info()
        .with_file_mut(fd, |_, _access_mode, _open_flags, file| file.path())
        .ok_or(Errno::EBADFD.no_message())?;

    data.set_cwd(path);

    Ok(0)
}

pub fn chroot(file_name: Vaddr) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();
    let vmar = data.memory_info().vmar();

    let path = vmar.read_cstring(file_name, None)?;

    let path = path.to_string_lossy();
    let path = Path::from(path);

    match data.open_file(&path) {
        Some(file) => {
            if file.r#type() != FileType::Directory {
                return Err(Errno::ENOTDIR.no_message());
            }
        }
        None => return Err(Errno::ENOENT.no_message()),
    }

    data.set_root(path);

    Ok(0)
}
