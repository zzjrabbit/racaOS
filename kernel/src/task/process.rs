use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use spin::RwLock;
use zodiac::{
    ZodiacError,
    hal::mem::{USER_ASPACE_BASE, USER_ASPACE_SIZE},
    mem::{
        Cursor, MMUFlags, PageSize, PhysicalMemoryAllocOptions, VirtualAddress, VirtualMemorySpace,
    },
    task::{ProcessBuilder, Thread, ThreadBuilder},
};

use crate::filesystem::{File, FileDescriptor};

static PROCESSES: RwLock<Vec<Arc<Process>>> = RwLock::new(Vec::new());

pub struct Process {
    inner: Arc<zodiac::task::Process>,
    info: RwLock<ProcessInfo>,
}

struct ProcessInfo {
    free_memory_space: Vec<MemoryRegion>,
    mapped: Vec<MemoryRegion>,
    next_descriptor: FileDescriptor,
    descriptors: BTreeMap<FileDescriptor, (u64, OpenMode, Arc<File>)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OpenMode {
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug, Clone, Copy)]
struct MemoryRegion {
    start: usize,
    end: usize,
}

impl MemoryRegion {
    pub fn new(start: usize, len: usize) -> Self {
        MemoryRegion {
            start,
            end: start + len,
        }
    }

    pub fn overlap(&self, other: &MemoryRegion) -> bool {
        self.start <= other.start && self.end >= other.end
    }
}

impl Process {
    pub fn new(binary: &[u8], stdin: Arc<File>, stdout: Arc<File>) -> Result<Arc<Self>, ZodiacError> {
        let vm_space = VirtualMemorySpace::new_user();

        let entry = vm_space.binary_file_mapper().map(binary)?;

        let inner = ProcessBuilder::default().vm_space(vm_space).build()?;

        let mut descriptors = BTreeMap::new();
        descriptors.insert(0, (0, OpenMode::Read, stdin));
        descriptors.insert(1, (0, OpenMode::Write, stdout));
        
        let new_self = Arc::new(Self {
            inner,
            info: RwLock::new(ProcessInfo {
                free_memory_space: alloc::vec![MemoryRegion::new(
                    USER_ASPACE_BASE,
                    USER_ASPACE_SIZE
                )],
                mapped: Vec::new(),
                next_descriptor: 2,
                descriptors,
            }),
        });

        PROCESSES.write().push(new_self.clone());

        let (stack_address, mut cursor, page_size) = new_self.allocate(USER_STACK_SIZE, true)?;
        const USER_STACK_SIZE: usize = 8 * 1024 * 1024;

        let user_stack_end = stack_address + USER_STACK_SIZE;

        let physical_memory = PhysicalMemoryAllocOptions::default()
            .count(USER_STACK_SIZE / page_size as usize)
            .page_size(page_size)
            .allocate()
            .unwrap();
        cursor
            .map(
                &physical_memory,
                MMUFlags::READ | MMUFlags::WRITE | MMUFlags::USER,
            )
            .unwrap();
        new_self
            .inner()
            .vm_space()
            .writer(user_stack_end - size_of::<usize>(), size_of::<usize>())
            .write(&0usize.to_le_bytes())
            .unwrap();

        let envp = user_stack_end;
        new_self
            .inner()
            .vm_space()
            .writer(user_stack_end - 4 * size_of::<usize>(), size_of::<usize>())
            .write(&envp.to_le_bytes())
            .unwrap();

        let thread = ThreadBuilder::default()
            .entry(entry)
            .process(new_self.inner())
            .stack(user_stack_end - 6 * size_of::<usize>())
            .build()?;
        thread.spawn();

        Ok(new_self)
    }

    pub fn current() -> Arc<Self> {
        let current_thread = Thread::current();
        let pid = current_thread.process().unwrap().process_id();
        PROCESSES
            .read()
            .iter()
            .find(|p| p.inner().process_id() == pid)
            .unwrap()
            .clone()
    }
}

impl Process {
    pub fn inner(&self) -> Arc<zodiac::task::Process> {
        self.inner.clone()
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

impl Process {
    pub fn add_file(&mut self, file: Arc<File>, open_mode: OpenMode) {
        let mut info = self.info.write();
        let descriptor = info.next_descriptor;
        info.descriptors.insert(descriptor, (0, open_mode, file));
        info.next_descriptor += 1;
    }
    
    pub fn remove_file(&mut self, descriptor: u32) {
        let mut info = self.info.write();
        info.descriptors.remove(&descriptor);
    }
    
    pub fn get_file(&self, descriptor: u32) -> Option<(u64, OpenMode, Arc<File>)> {
        let info = self.info.read();
        info.descriptors.get(&descriptor).cloned()
    }
}

impl Process {
    pub fn allocate(
        &self,
        len: usize,
        huge_page: bool,
    ) -> Result<(VirtualAddress, Cursor, PageSize), ZodiacError> {
        let mut info = self.info.write();

        let mut region = None;

        for free_region in info.free_memory_space.iter_mut() {
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

        region
            .map(|region| {
                self.inner()
                    .vm_space()
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
        let mut info = self.info.write();
        let required_region = MemoryRegion::new(address, len);

        for mapped in info.mapped.iter() {
            if mapped.overlap(&required_region) {
                return Err(ZodiacError::NoMemory);
            }
        }

        info.mapped.push(required_region);

        let mut new_region = None;

        for region in info.free_memory_space.iter_mut() {
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
            info.free_memory_space.push(region);
        }

        let page_size = if huge_page {
            infer_page_size_by_len(len)
        } else {
            PageSize::Size4K
        };

        Ok((
            address,
            self.inner
                .vm_space()
                .cursor(page_size.align_down(required_region.start), page_size)?,
            page_size,
        ))
    }
}
