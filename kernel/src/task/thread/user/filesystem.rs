use core::sync::atomic::{AtomicI32, Ordering};

use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use spin::RwLock;

use crate::filesystem::{AccessMode, File, FileDescriptor, OpenFlags};

type FileDescription = (u64, AccessMode, OpenFlags, Arc<File>);
type FileDescriptorTable = Arc<RwLock<BTreeMap<FileDescriptor, FileDescription>>>;

pub struct FileSystemInfo {
    fd_table: FileDescriptorTable,
    next_fd: Arc<AtomicI32>,
}

impl FileSystemInfo {
    pub fn new(stdin: Arc<File>, stdout: Arc<File>, stderr: Arc<File>) -> Self {
        let mut fd_table = BTreeMap::new();
        fd_table.insert(0, (0, AccessMode::O_RDONLY, OpenFlags::empty(), stdin));
        fd_table.insert(1, (0, AccessMode::O_WRONLY, OpenFlags::empty(), stdout));
        fd_table.insert(2, (0, AccessMode::O_WRONLY, OpenFlags::empty(), stderr));

        FileSystemInfo {
            fd_table: Arc::new(RwLock::new(fd_table)),
            next_fd: Arc::new(AtomicI32::new(3)),
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

    pub fn with_file<R>(
        &self,
        descriptor: FileDescriptor,
        f: impl Fn(u64, AccessMode, OpenFlags, Arc<File>) -> R,
    ) -> Option<R> {
        let fd_table = self.fd_table.read();
        let (offset, access_mode, open_flags, file) = fd_table.get(&descriptor)?.clone();
        Some(f(offset, access_mode, open_flags, file))
    }

    pub fn with_file_mut<R>(
        &self,
        descriptor: FileDescriptor,
        f: impl Fn(&mut u64, &mut AccessMode, &mut OpenFlags, Arc<File>) -> R,
    ) -> Option<R> {
        let mut fd_table = self.fd_table.write();
        let (offset, access_mode, open_flags, file) = fd_table.get_mut(&descriptor)?;
        Some(f(offset, access_mode, open_flags, file.clone()))
    }
}
