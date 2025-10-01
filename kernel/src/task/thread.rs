use core::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

use alloc::{
    collections::btree_map::BTreeMap,
    sync::{Arc, Weak},
    vec::Vec,
};
use ostd::{
    mm::{PageProperty, Vaddr, VmSpace},
    task::{Task, TaskOptions},
    Error as OstdError,
};
use spin::{Once, RwLock};

use crate::{
    filesystem::{AccessMode, File, FileDescriptor, OpenFlags},
    task::Process,
};

type FileDescriptorInfo = (u64, AccessMode, OpenFlags, Arc<File>);

static THREADS: RwLock<Vec<Arc<Task>>> = RwLock::new(Vec::new());

pub fn create_kernel_thread(entry: fn()) -> Arc<Task> {
    let task = Arc::new(TaskOptions::new(entry).build().unwrap());
    task.run();
    THREADS.write().push(task.clone());
    task
}

/// The base of kernel address space.
pub const KERNEL_ASPACE_BASE: usize = 0xffff_ff80_0000_0000;
/// The base of user address space.
pub const USER_ASPACE_BASE: usize = 0x0000_0001_0000_0000;
/// The size of user address space.
pub const USER_ASPACE_SIZE: usize = KERNEL_ASPACE_BASE - USER_ASPACE_BASE;

pub struct ThreadData {
    pub vm_space: Arc<VmSpace>,
    fd_table: Arc<RwLock<BTreeMap<FileDescriptor, FileDescriptorInfo>>>,
    pub process: Weak<Process>,
    next_fd: Arc<AtomicI32>,
    free_memory_space: RwLock<Vec<MemoryRegion>>,
    allocated: RwLock<Vec<MemoryRegion>>,
    pub unused_region: RwLock<Vec<(MemoryRegion, PageProperty)>>,
    pub tid_address: RwLock<Option<Vaddr>>,
    pub entry: Once<usize>,
    pub stack: Once<usize>,
    tid: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    start: usize,
    end: usize,
}

#[allow(dead_code)]
impl MemoryRegion {
    pub fn new(start: usize, len: usize) -> Self {
        MemoryRegion {
            start,
            end: start + len,
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn start_address(&self) -> usize {
        self.start
    }

    pub fn end_address(&self) -> usize {
        self.end
    }

    pub fn contains(&self, addr: usize) -> bool {
        self.start <= addr && addr < self.end
    }

    pub fn overlap(&self, other: &MemoryRegion) -> bool {
        self.start <= other.start && self.end >= other.end
    }
}

impl ThreadData {
    pub fn new(
        stdin: Arc<File>,
        stdout: Arc<File>,
        stderr: Arc<File>,
        process: &Arc<Process>,
    ) -> Self {
        let mut fd_table = BTreeMap::new();
        fd_table.insert(0, (0, AccessMode::O_RDONLY, OpenFlags::empty(), stdin));
        fd_table.insert(1, (0, AccessMode::O_WRONLY, OpenFlags::empty(), stdout));
        fd_table.insert(2, (0, AccessMode::O_WRONLY, OpenFlags::empty(), stderr));

        static TID: AtomicUsize = AtomicUsize::new(0);

        Self {
            vm_space: Arc::new(VmSpace::new()),
            fd_table: Arc::new(RwLock::new(fd_table)),
            process: Arc::downgrade(process),
            next_fd: Arc::new(AtomicI32::new(3)),
            free_memory_space: RwLock::new(alloc::vec![MemoryRegion::new(
                USER_ASPACE_BASE,
                USER_ASPACE_SIZE
            )]),
            allocated: RwLock::new(Vec::new()),
            unused_region: RwLock::new(Vec::new()),
            tid_address: RwLock::new(None),
            entry: Once::new(),
            stack: Once::new(),
            tid: TID.fetch_add(1, Ordering::SeqCst),
        }
    }
}

impl ThreadData {
    pub fn tid(&self) -> usize {
        self.tid
    }
}

#[allow(dead_code)]
impl ThreadData {
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

impl ThreadData {
    pub fn allocate(&self, len: usize) -> Result<MemoryRegion, OstdError> {
        let mut free_memory_space = self.free_memory_space.write();

        let mut region = None;

        for free_region in free_memory_space.iter_mut() {
            if free_region.end - free_region.start >= len {
                let allocated_region = MemoryRegion::new(free_region.start, len);
                free_region.start = allocated_region.end;
                region = Some(allocated_region);
                break;
            }
        }

        if let Some(region) = region {
            self.allocated.write().push(region);
        }

        region.ok_or(OstdError::NoMemory)
    }

    pub fn allocate_at(&self, address: Vaddr, len: usize) -> Result<MemoryRegion, OstdError> {
        let mut allocated = self.allocated.write();
        let mut free_memory_space = self.free_memory_space.write();
        let required_region = MemoryRegion::new(address, len);

        for mapped in allocated.iter() {
            if mapped.overlap(&required_region) {
                return Err(OstdError::NoMemory);
            }
        }

        allocated.push(required_region);

        let mut new_region = None;

        for region in free_memory_space.iter_mut() {
            if region.overlap(&required_region) {
                if region.start == required_region.start {
                    region.end = required_region.end;
                } else if region.end == required_region.end {
                    region.start = required_region.start;
                } else {
                    region.end = required_region.start;
                    new_region = Some(MemoryRegion::new(
                        required_region.end,
                        region.end - required_region.end,
                    ));
                }
            }
        }

        if let Some(region) = new_region {
            free_memory_space.push(region);
        }

        Ok(required_region)
    }
}
