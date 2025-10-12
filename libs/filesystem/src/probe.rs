use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{boxed::Box, format, sync::Arc, vec::Vec};
use block::register_callback;
use spin::RwLock;

use crate::{
    File, FileSystemError, FileType, Path, block::BlockInode, open_file, part::parse_partitions,
};

type FileSystemProbe = fn(Arc<File>) -> Result<Arc<File>, FileSystemError>;

static FILE_SYSTEMS: RwLock<Vec<FileSystemProbe>> = RwLock::new(Vec::new());

pub(super) fn init() {
    static PART_COUNT: AtomicUsize = AtomicUsize::new(0);
    register_callback(Box::new(|device| {
        let dev_fs = open_file(&Path::from("/dev")).unwrap();
        let device_type = device.metadata().device_type;

        let disk_file = dev_fs
            .create(format!("{}", device_type), FileType::File)
            .unwrap();
        let disk = File::new(
            Path::from(""),
            BlockInode::new(device),
            FileType::BlockDevice,
        );

        disk.mount(disk_file.clone());

        let probe_part = |part_file: Arc<File>| {
            if let Ok(part) = probe(part_file.clone()) {
                let id = PART_COUNT.fetch_add(1, Ordering::SeqCst);
                let root = open_file(&Path::from("/")).unwrap();
                let part_dir = root.create(format!("part{}", id), FileType::File).unwrap();
                part.mount(part_dir.clone());
            }
        };

        probe_part(disk_file.clone());

        if let Ok(partitions) = parse_partitions(disk_file) {
            for (id, partition) in partitions.iter().enumerate() {
                let part_file = dev_fs
                    .create(device_type.partition_name(id), FileType::File)
                    .unwrap();

                let part = File::new(Path::from(""), partition.clone(), FileType::BlockDevice);
                part.mount(part_file.clone());

                probe_part(part_file);
            }
        }
    }));
}

pub(super) fn register_probe(probe: FileSystemProbe) {
    FILE_SYSTEMS.write().push(probe);
}

pub(super) fn probe(device: Arc<File>) -> Result<Arc<File>, FileSystemError> {
    for probe in FILE_SYSTEMS.read().iter() {
        if let Ok(file) = probe(device.clone()) {
            return Ok(file);
        }
    }
    Err(FileSystemError::InvalidArguments)
}
