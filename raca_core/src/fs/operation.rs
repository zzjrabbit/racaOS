use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};
use framework::{ref_to_mut, task::process::ProcessId};
use spin::{Mutex, RwLock};

use crate::user::get_current_process_id;

use super::{
    vfs::{
        inode::{FileInfo, InodeRef, InodeTy},
        pipe::Pipe,
    },
    ROOT,
};

static FILE_DESCRIPTOR_MANAGERS: Mutex<BTreeMap<ProcessId, Arc<FileDescriptorManager>>> =
    Mutex::new(BTreeMap::new());

pub enum OpenMode {
    Read,
    Write,
}

type FileDescriptor = usize;
type FileTuple = (InodeRef, OpenMode, usize);

struct FileDescriptorManager {
    file_descriptors: BTreeMap<FileDescriptor, FileTuple>,
    file_descriptor_allocator: AtomicUsize,
    cwd: Mutex<InodeRef>,
}

impl FileDescriptorManager {
    pub fn new(file_descriptors: BTreeMap<FileDescriptor, FileTuple>) -> Self {
        Self {
            file_descriptors,
            file_descriptor_allocator: AtomicUsize::new(3), // 0, 1, and 2 are reserved for stdin, stdout, and stderr
            cwd: Mutex::new(ROOT.lock().clone()),
        }
    }

    pub fn get_new_fd(&self) -> FileDescriptor {
        self.file_descriptor_allocator
            .fetch_add(1, Ordering::Relaxed)
    }

    pub fn add_inode(&self, inode: InodeRef, mode: OpenMode) -> FileDescriptor {
        let new_fd = self.get_new_fd();
        ref_to_mut(self)
            .file_descriptors
            .insert(new_fd, (inode, mode, 0));
        new_fd
    }

    pub fn change_cwd(&self, path: String) {
        *self.cwd.lock() = get_inode_by_path(path).unwrap();
    }

    pub fn get_cwd(&self) -> String {
        self.cwd.lock().read().get_path()
    }
}

fn get_file_descriptor_manager<'a>() -> Option<Arc<FileDescriptorManager>> {
    let pid = get_current_process_id();

    FILE_DESCRIPTOR_MANAGERS.lock().get_mut(&pid).cloned()
}

pub fn init_file_descriptor_manager(pid: ProcessId) {
    let mut file_descriptor_managers = FILE_DESCRIPTOR_MANAGERS.lock();
    file_descriptor_managers.insert(pid, Arc::new(FileDescriptorManager::new(BTreeMap::new())));
}

pub fn init_file_descriptor_manager_with_stdin_stdout(
    pid: ProcessId,
    stdin: InodeRef,
    stdout: InodeRef,
) {
    let mut file_descriptor_managers = FILE_DESCRIPTOR_MANAGERS.lock();

    let mut file_descriptors = BTreeMap::new();
    file_descriptors.insert(0, (stdin.clone(), OpenMode::Read, 0));
    file_descriptors.insert(1, (stdout.clone(), OpenMode::Write, 0));

    file_descriptor_managers.insert(pid, Arc::new(FileDescriptorManager::new(file_descriptors)));
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

pub fn get_inode_by_fd(file_descriptor: usize) -> Option<InodeRef> {
    let current_file_descriptor_manager = get_file_descriptor_manager()?;

    let (inode, _, _) = current_file_descriptor_manager
        .file_descriptors
        .get(&file_descriptor)?;

    Some(inode.clone())
}

pub fn open(path: String, open_mode: OpenMode) -> Option<usize> {
    let current_file_descriptor_manager = get_file_descriptor_manager()?;

    let inode = if path.starts_with("/") {
        get_inode_by_path(path.clone())?
    } else {
        get_inode_by_path(alloc::format!(
            "{}{}",
            current_file_descriptor_manager.get_cwd(),
            path.clone()
        ))?
    };

    let file_descriptor = current_file_descriptor_manager.add_inode(inode, open_mode);

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

pub fn fsize(fd: FileDescriptor) -> Option<usize> {
    let current_file_descriptor_manager = get_file_descriptor_manager()?;

    let (inode, _, _) = ref_to_mut(current_file_descriptor_manager.as_ref())
        .file_descriptors
        .get_mut(&fd)?;

    let size = inode.read().size();

    Some(size)
}

pub fn open_pipe(buffer: &mut [usize]) -> Option<()> {
    if buffer.len() != 2 {
        return None;
    }

    let current_file_descriptor_manager = get_file_descriptor_manager()?;

    let inode = Arc::new(RwLock::new(Pipe::new()));

    let file_descriptor_read =
        current_file_descriptor_manager.add_inode(inode.clone(), OpenMode::Read);

    let file_descriptor_write =
        current_file_descriptor_manager.add_inode(inode.clone(), OpenMode::Write);

    buffer[0] = file_descriptor_read;
    buffer[1] = file_descriptor_write;

    log::info!("{:?}", buffer);

    Some(())
}

pub fn list_dir(path: String) -> Vec<FileInfo> {
    if let Some(inode) = get_inode_by_path(path) {
        if inode.read().inode_type() == InodeTy::Dir {
            return inode.read().list();
        }
    }
    Vec::new()
}

pub fn change_cwd(path: String) {
    if let Some(current_file_descriptor_manager) = get_file_descriptor_manager() {
        if path.starts_with("/") {
            current_file_descriptor_manager.change_cwd(path);
        } else {
            let current = current_file_descriptor_manager.get_cwd();
            let new = alloc::format!("{}{}",current,path);
            current_file_descriptor_manager.change_cwd(new);
        }
    }
}

pub fn get_cwd() -> String {
    if let Some(current_file_descriptor_manager) = get_file_descriptor_manager(){
        current_file_descriptor_manager.get_cwd()
    }else {
        String::from("/")
    }
}
