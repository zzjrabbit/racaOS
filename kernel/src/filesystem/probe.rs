use alloc::{sync::Arc, vec::Vec};
use spin::RwLock;

use crate::filesystem::{File, FileSystemError};

type FileSystemProbe = fn(Arc<File>) -> Result<Arc<File>, FileSystemError>;

static FILE_SYSTEMS: RwLock<Vec<FileSystemProbe>> = RwLock::new(Vec::new());

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
