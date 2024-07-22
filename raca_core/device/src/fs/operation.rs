use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{collections::BTreeMap, string::String, sync::Arc};
use framework::{ref_to_mut, task::process::ProcessId};
use spin::Mutex;

use crate::user::get_current_process_id;

use super::{vfs::inode::InodeRef, ROOT};

static FILE_DESCRIPTOR_MANAGERS: Mutex<BTreeMap<ProcessId, Arc<FileDescriptorManager>>> =
    Mutex::new(BTreeMap::new());

pub enum OpenMode {
    Read,
    Write,
}

type FileDescriptor = usize;

struct FileDescriptorManager {
    file_descriptors: BTreeMap<FileDescriptor, (InodeRef, OpenMode, usize)>,
    file_descriptor_allocator: AtomicUsize,
}

fn get_file_descriptor_manager<'a>() -> Option<Arc<FileDescriptorManager>> {
    let pid = get_current_process_id();

    FILE_DESCRIPTOR_MANAGERS.lock().get_mut(&pid).cloned()
}

pub fn init_file_descriptor_manager(pid: ProcessId) {
    let mut file_descriptor_managers = FILE_DESCRIPTOR_MANAGERS.lock();
    file_descriptor_managers.insert(
        pid,
        Arc::new(FileDescriptorManager {
            file_descriptors: BTreeMap::new(),
            file_descriptor_allocator: AtomicUsize::new(3), // 0, 1, and 2 are reserved for stdin, stdout, and stderr
        }),
    );
}

fn get_inode_by_path(path: String) -> Option<InodeRef> {
    let root = ROOT.lock().clone();

    let path = path.split("/");

    let node = root;

    for path_node in path {
        if path_node.len() > 0 {
            if let Some(child) = node.read().open(String::from(path_node)) {
                core::mem::drop(core::mem::replace(ref_to_mut(&node), child));
            } else {
                return None;
            }
        }
    }

    Some(node.clone())
}

pub fn kernel_open(path: String) -> Option<InodeRef> {
    get_inode_by_path(path)
}

pub fn open(path: String, open_mode: OpenMode) -> Option<usize> {
    let current_file_descriptor_manager = get_file_descriptor_manager()?;
    let file_descriptor = current_file_descriptor_manager
        .file_descriptor_allocator
        .fetch_add(1, Ordering::Relaxed);

    let node = get_inode_by_path(path.clone())?;

    ref_to_mut(current_file_descriptor_manager.as_ref())
        .file_descriptors
        .insert(file_descriptor, (node.clone(), open_mode, 0));

    Some(file_descriptor)
}

pub fn read(fd: FileDescriptor, buf: &mut [u8]) -> Option<()> {
    let current_file_descriptor_manager = get_file_descriptor_manager()?;

    let (inode, _, offset) = current_file_descriptor_manager.file_descriptors.get(&fd)?;

    inode.read().read_at(*offset, buf);

    Some(())
}

pub fn write(fd: FileDescriptor, buf: &[u8]) -> Option<()> {
    let current_file_descriptor_manager = get_file_descriptor_manager()?;

    let (inode, mode, offset) = current_file_descriptor_manager.file_descriptors.get(&fd)?;

    match mode {
        OpenMode::Write => {
            inode.read().write_at(*offset, buf);
        }
        _ => return None,
    }

    Some(())
}

pub fn lseek(fd: FileDescriptor, offset: usize) -> Option<()> {
    let current_file_descriptor_manager = get_file_descriptor_manager()?;

    let (_, _, old_offset) = ref_to_mut(current_file_descriptor_manager.as_ref())
        .file_descriptors
        .get_mut(&fd)?;
    *old_offset = offset;

    Some(())
}

pub fn close(fd: FileDescriptor) -> Option<()> {
    let current_file_descriptor_manager = get_file_descriptor_manager()?;
    ref_to_mut(current_file_descriptor_manager.as_ref())
        .file_descriptors
        .remove(&fd)?;
    Some(())
}
