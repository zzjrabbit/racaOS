use core::sync::atomic::{AtomicI32, Ordering};

use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use spin::RwLock;

use crate::filesystem::{AccessMode, File, FileDescriptor, OpenFlags, Path};

type FileDescription = (u64, AccessMode, OpenFlags, Arc<File>);
type FileDescriptorTable = RwLock<BTreeMap<FileDescriptor, FileDescription>>;

pub struct FileSystemInfo {
    fd_table: FileDescriptorTable,
    next_fd: AtomicI32,
    current_dir: RwLock<Path>,
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
            current_dir: RwLock::new(Path::new("/")),
        }
    }

    pub fn deep_clone(&self) -> Self {
        FileSystemInfo {
            fd_table: RwLock::new(self.fd_table.read().clone()),
            next_fd: AtomicI32::new(self.next_fd.load(Ordering::SeqCst)),
            current_dir: RwLock::new(self.current_dir.read().clone()),
        }
    }
}

impl FileSystemInfo {
    pub fn current_dir(&self) -> Path {
        self.current_dir.read().clone()
    }

    pub fn set_current_dir(&self, path: Path) {
        *self.current_dir.write() = path;
    }

    pub fn absolute_path(&self, path: Path) -> Path {
        if path.is_absolute() {
            path
        } else {
            self.current_dir.read().join(path)
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
