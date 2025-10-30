use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{format, string::String, sync::Arc, vec::Vec};
use errors::{Errno, Result};
use spin::RwLock;

use crate::{File, FileType, InodeOperation, Path, dev::dev_dir, open_file, part::parse_partitions};

type FileSystemProbe = fn(Arc<File>) -> Result<Arc<File>>;

static FILE_SYSTEMS: RwLock<Vec<FileSystemProbe>> = RwLock::new(Vec::new());

static PART_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn add_block_device(
    device: impl InodeOperation,
    name: String,
    part_name: impl Fn(usize) -> String,
) {
    let device = File::new(Path::from(""), device, FileType::BlockDevice);

    let dev_fs = open_file(&Path::from("/dev")).unwrap();
    let device_file = dev_fs.create(name, FileType::File).unwrap();

    device.mount(device_file.clone());

    let probe_part = |part_file: Arc<File>| {
        if let Ok(part) = probe(part_file.clone()) {
            let id = PART_COUNT.fetch_add(1, Ordering::SeqCst);
            let root = open_file(&Path::from("/")).unwrap();
            let part_dir = root.create(format!("part{}", id), FileType::File).unwrap();
            part.mount(part_dir.clone());
            
            if id == 0 {
                let target_dev_dir = part_dir.lookup("dev".into()).unwrap();
                dev_dir().mount(target_dev_dir);
            }
        }
    };

    probe_part(device_file.clone());

    if let Ok(partitions) = parse_partitions(device_file) {
        for (id, partition) in partitions.iter().enumerate() {
            let part_file = dev_fs.create(part_name(id), FileType::File).unwrap();

            let part = File::new(Path::from(""), partition.clone(), FileType::BlockDevice);
            part.mount(part_file.clone());

            probe_part(part_file);
        }
    }
}

pub fn register_probe(probe: FileSystemProbe) {
    FILE_SYSTEMS.write().push(probe);
}

pub(super) fn probe(device: Arc<File>) -> Result<Arc<File>> {
    for probe in FILE_SYSTEMS.read().iter() {
        if let Ok(file) = probe(device.clone()) {
            return Ok(file);
        }
    }
    Err(Errno::EINVAL.with_message("Invalid device for filesystem prober."))
}
