use core::sync::atomic::{AtomicI32, Ordering};

use alloc::{
    collections::btree_map::BTreeMap,
    sync::{Arc, Weak},
    vec::Vec,
};
use spin::{Once, RwLock};
use zodiac::{
    ZodiacError,
    hal::mem::{USER_ASPACE_BASE, USER_ASPACE_SIZE},
    mem::{Cursor, MMUFlags, PageSize, VirtualAddress, VirtualMemorySpace},
};

use crate::{
    filesystem::{AccessMode, File, FileDescriptor, OpenFlags},
    task::Process,
};

type FileDescriptorInfo = (u64, AccessMode, OpenFlags, Arc<File>);

pub struct ThreadData {
    pub vm_space: Arc<VirtualMemorySpace>,
    fd_table: Arc<RwLock<BTreeMap<FileDescriptor, FileDescriptorInfo>>>,
    pub process: Weak<Process>,
    next_fd: Arc<AtomicI32>,
    free_memory_space: RwLock<Vec<MemoryRegion>>,
    allocated: RwLock<Vec<MemoryRegion>>,
    pub unused_region: RwLock<Vec<(MemoryRegion, MMUFlags, PageSize)>>,
    pub fs: RwLock<Option<VirtualAddress>>,
    pub gs: RwLock<Option<VirtualAddress>>,
    pub tid_address: RwLock<Option<VirtualAddress>>,
    pub entry: Once<usize>,
    pub stack: Once<usize>,
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

        Self {
            vm_space: Arc::new(VirtualMemorySpace::new_user()),
            fd_table: Arc::new(RwLock::new(fd_table)),
            process: Arc::downgrade(process),
            next_fd: Arc::new(AtomicI32::new(3)),
            free_memory_space: RwLock::new(alloc::vec![MemoryRegion::new(
                USER_ASPACE_BASE,
                USER_ASPACE_SIZE
            )]),
            allocated: RwLock::new(Vec::new()),
            unused_region: RwLock::new(Vec::new()),
            fs: RwLock::new(None),
            gs: RwLock::new(None),
            tid_address: RwLock::new(None),
            entry: Once::new(),
            stack: Once::new(),
        }
    }
}

fn infer_page_size_by_len(len: usize) -> PageSize {
    let wasted_1g = PageSize::Size1G.align_up(len) - len;
    if wasted_1g > (len >> 10) {
        PageSize::Size2M
    } else {
        PageSize::Size1G
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
    pub fn allocate(
        &self,
        len: usize,
        huge_page: bool,
    ) -> Result<(VirtualAddress, Cursor, PageSize), ZodiacError> {
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

        let page_size = if huge_page {
            infer_page_size_by_len(len)
        } else {
            PageSize::Size4K
        };

        if let Some(region) = region {
            self.allocated.write().push(region);
        }

        region
            .map(|region| {
                self.vm_space
                    .cursor(page_size.align_down(region.start), page_size)
                    .map(|cursor| (region.start, cursor, page_size))
            })
            .unwrap_or(Err(ZodiacError::NoMemory))
    }

    pub fn allocate_at(
        &self,
        address: VirtualAddress,
        len: usize,
        huge_page: bool,
    ) -> Result<(VirtualAddress, Cursor, PageSize), ZodiacError> {
        let mut allocated = self.allocated.write();
        let mut free_memory_space = self.free_memory_space.write();
        let required_region = MemoryRegion::new(address, len);

        for mapped in allocated.iter() {
            if mapped.overlap(&required_region) {
                return Err(ZodiacError::NoMemory);
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

        let page_size = if huge_page {
            infer_page_size_by_len(len)
        } else {
            PageSize::Size4K
        };

        Ok((
            address,
            self.vm_space
                .cursor(page_size.align_down(required_region.start), page_size)?,
            page_size,
        ))
    }
}
