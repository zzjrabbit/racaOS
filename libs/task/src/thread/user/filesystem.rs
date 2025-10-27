use core::sync::atomic::{AtomicI32, Ordering};

use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use errors::{Errno, Result};
use spin::RwLock;

use filesystem::{AccessMode, File, FileDescriptor, OpenFlags};

type FileDescription = (u64, AccessMode, OpenFlags, Arc<File>);
type FileDescriptorTable = RwLock<BTreeMap<FileDescriptor, FileDescription>>;

pub struct FileSystemInfo {
    fd_table: FileDescriptorTable,
    next_fd: AtomicI32,
}

impl FileSystemInfo {
    pub fn new(stdin: Arc<File>, stdout: Arc<File>, stderr: Arc<File>) -> Self {
        let mut fd_table = BTreeMap::new();
        fd_table.insert(0, (0, AccessMode::O_RDONLY, OpenFlags::empty(), stdin));
        fd_table.insert(1, (0, AccessMode::O_WRONLY, OpenFlags::empty(), stdout));
        fd_table.insert(2, (0, AccessMode::O_WRONLY, OpenFlags::empty(), stderr));

        FileSystemInfo {
            fd_table: RwLock::new(fd_table),
            next_fd: AtomicI32::new(3),
        }
    }

    pub fn deep_clone(&self) -> Self {
        FileSystemInfo {
            fd_table: RwLock::new(self.fd_table.read().clone()),
            next_fd: AtomicI32::new(self.next_fd.load(Ordering::SeqCst)),
        }
    }
}

#[allow(dead_code)]
impl FileSystemInfo {
    pub fn add_file(
        &self,
        file: Arc<File>,
        access_mode: AccessMode,
        open_flags: OpenFlags,
    ) -> FileDescriptor {
        let descriptor = self.next_fd.fetch_add(1, Ordering::SeqCst);
        self.fd_table.write().insert(
            descriptor,
            (
                if open_flags.contains(OpenFlags::O_APPEND) {
                    file.len()
                } else {
                    0
                },
                access_mode,
                open_flags,
                file,
            ),
        );
        descriptor
    }

    pub fn remove_file(&self, descriptor: FileDescriptor) -> Option<()> {
        self.fd_table.write().remove(&descriptor).map(|_| ())
    }

    pub fn duplicate(
        &self,
        descriptor: FileDescriptor,
        new_descriptor: FileDescriptor,
        flags: OpenFlags,
    ) -> Result<FileDescriptor> {
        let mut fd_table = self.fd_table.write();

        let description = fd_table
            .get(&descriptor)
            .ok_or(Errno::ENOENT.no_message())?
            .clone();

        let fd = if let None = fd_table.get(&new_descriptor) {
            new_descriptor
        } else {
            self.next_fd.fetch_add(1, Ordering::SeqCst)
        };

        fd_table.insert(
            fd,
            (description.0, description.1, flags, description.3.clone()),
        );

        Ok(fd)
    }

    pub fn close_on_execve(&self) {
        self.fd_table
            .write()
            .retain(|_, (_, _, open_flags, _)| !open_flags.contains(OpenFlags::O_CLOEXEC));
    }

    pub fn with_file<R>(
        &self,
        descriptor: FileDescriptor,
        f: impl Fn(u64, AccessMode, OpenFlags, Arc<File>) -> R,
    ) -> Result<R> {
        let fd_table = self.fd_table.read();
        let (offset, access_mode, open_flags, file) = fd_table
            .get(&descriptor)
            .ok_or(Errno::ENOENT.no_message())?;
        Ok(f(*offset, *access_mode, *open_flags, file.clone()))
    }

    pub fn with_file_mut<R>(
        &self,
        descriptor: FileDescriptor,
        f: impl Fn(&mut u64, &mut AccessMode, &mut OpenFlags, Arc<File>) -> R,
    ) -> Result<R> {
        let mut fd_table = self.fd_table.write();
        let (offset, access_mode, open_flags, file) = fd_table
            .get_mut(&descriptor)
            .ok_or(Errno::ENOENT.no_message())?;
        Ok(f(offset, access_mode, open_flags, file.clone()))
    }

    pub fn with_open_flags_mut<R>(
        &self,
        descriptor: FileDescriptor,
        f: impl Fn(&mut OpenFlags) -> R,
    ) -> Result<R> {
        let mut fd_table = self.fd_table.write();
        let (_, _, open_flags, _) = fd_table
            .get_mut(&descriptor)
            .ok_or(Errno::ENOENT.no_message())?;
        Ok(f(open_flags))
    }

    pub fn get_open_flags(&self, descriptor: FileDescriptor) -> Result<OpenFlags> {
        let fd_table = self.fd_table.read();
        fd_table
            .get(&descriptor)
            .map(|(_, _, flags, _)| *flags)
            .ok_or(Errno::ENOENT.no_message())
    }
}
