use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{boxed::Box, format, sync::Arc, vec::Vec};
use block::register_callback;
use spin::RwLock;

use crate::filesystem::{block::BlockInode, open_file, File, FileSystemError, FileType, Path};

type FileSystemProbe = fn(Arc<File>) -> Result<Arc<File>, FileSystemError>;

static FILE_SYSTEMS: RwLock<Vec<FileSystemProbe>> = RwLock::new(Vec::new());

pub(super) fn init() {
    static PART_COUNT: AtomicUsize = AtomicUsize::new(0);
    register_callback(Box::new(|device| {
        let dev_fs = open_file(&Path::from("/dev")).unwrap();

        let disk_file = dev_fs
            .create(format!("{}", device.metadata().device_type), FileType::File)
            .unwrap();
        let disk = File::new(
            Path::from(""),
            BlockInode::new(device),
            FileType::BlockDevice,
        );

        disk.mount(disk_file.clone());

        if let Ok(part) = probe(disk_file) {
            let id = PART_COUNT.fetch_add(1, Ordering::SeqCst);
            let root = open_file(&Path::from("/")).unwrap();
            let part_dir = root.create(format!("part{}", id), FileType::File).unwrap();
            part.mount(part_dir.clone());
        }
    }));
}

pub(super) fn register_probe(probe: FileSystemProbe) {
    FILE_SYSTEMS.write().push(probe);
}

pub(super) fn probe(device: Arc<File>) -> Result<Arc<File>, FileSystemError> {
    for probe in FILE_SYSTEMS.read().iter() {
        match probe(device.clone()) {
            Ok(file) => return Ok(file),
            Err(_) => {}
        }
    }
    Err(FileSystemError::InvalidArguments)
}
