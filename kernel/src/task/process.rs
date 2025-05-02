use core::range::Range;

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use elf::{
    ElfBytes,
    abi::{PF_R, PF_W, PF_X, PT_LOAD},
    endian::NativeEndian,
};
use spin::RwLock;
use x86_64::{
    VirtAddr,
    structures::paging::{Page, Size4KiB},
};

use super::{
    job::Job,
    job_policy::{JobPolicy, PolicyAction, PolicyCondition},
    thread::Thread,
};
use crate::{
    error::{RcError, RcResult},
    mm::{MMUFlags, PhysicalMemory, VirtualMemory, VmMapping, page_count},
    object::{Handle, HandleValue, KernelObject, KoID, Rights},
};

const MAIN_THREAD_STACK_SIZE: usize = 256; // 1MB

static PROCESSES: RwLock<Vec<Arc<Process>>> = RwLock::new(Vec::new());

crate::kernel_object! {
    pub struct Process {
        inner: spin::Mutex<ProcessInner> = spin::Mutex::new(ProcessInner::new()),
        job: Arc<Job> = Job::root(),
        policy: JobPolicy = Default::default(),
        virtual_memory: Arc<VirtualMemory> = VirtualMemory::new_root(),
    }

    fn new() {}

    fn get_child(&self, id: crate::object::KoID) -> RcResult<Arc<dyn KernelObject>> {
        let inner = self.inner.lock();
        let thread = inner.threads.iter().find(|o| o.id() == id).ok_or(RcError::NotFound)?;
        Ok(thread.clone())
    }

    fn related_koid(&self) -> KoID {
        self.job.id()
    }

}

impl Process {
    pub fn create(name: &str, binary: &'static [u8]) -> RcResult<Arc<Self>> {
        let process = Self::new();

        let file = ProcessBinary::parse(binary);
        ProcessBinary::map_segments(&file, process.vmar());

        let stack = process.vmar().allocate_child(MAIN_THREAD_STACK_SIZE)?;
        for count in 0..MAIN_THREAD_STACK_SIZE {
            let stack_page = stack.create_child(Range::from(count..count + 1));
            let stack_memory = PhysicalMemory::allocate(1)?;
            let mapping = Arc::new(VmMapping::new(
                MMUFlags::READ | MMUFlags::WRITE | MMUFlags::USER,
                stack_page.clone(),
                stack_memory.clone(),
            ));
            mapping.map()?;
        }

        let stack_address = stack.as_ptr() as usize + 4096 * MAIN_THREAD_STACK_SIZE;

        Thread::create(&process, name, file.ehdr.e_entry as usize, stack_address)?;
        PROCESSES.write().push(process.clone());

        Ok(process)
    }
}

impl Process {
    pub fn add_handle(&self, handle: Handle) -> HandleValue {
        self.inner.lock().add_handle(handle)
    }

    pub fn remove_handle(&self, handle_value: HandleValue) -> RcResult<Handle> {
        self.inner.lock().remove_handle(handle_value)
    }

    /// Get a handle from the process
    fn get_handle(&self, handle_value: HandleValue) -> RcResult<Handle> {
        self.inner.lock().get_handle(handle_value)
    }

    /// Get the kernel object corresponding to this `handle_value`
    pub fn get_object<T: KernelObject>(&self, handle_value: HandleValue) -> RcResult<Arc<T>> {
        let handle = self.get_handle(handle_value)?;
        let object = handle
            .object
            .downcast_arc::<T>()
            .map_err(|_| RcError::WrongType)?;
        Ok(object)
    }

    /// Get the kernel object corresponding to this `handle_value` and this handle's rights.
    pub fn get_object_and_rights<T: KernelObject>(
        &self,
        handle_value: HandleValue,
    ) -> RcResult<(Arc<T>, Rights)> {
        let handle = self.get_handle(handle_value)?;
        let object = handle
            .object
            .downcast_arc::<T>()
            .map_err(|_| RcError::WrongType)?;
        Ok((object, handle.rights))
    }

    /// 根据句柄值查找内核对象，并检查权限
    pub fn get_object_with_rights<T: KernelObject>(
        &self,
        handle_value: HandleValue,
        desired_rights: Rights,
    ) -> RcResult<Arc<T>> {
        let handle = self.get_handle(handle_value)?;
        // check type before rights
        let object = handle
            .object
            .downcast_arc::<T>()
            .map_err(|_| RcError::WrongType)?;
        if !handle.rights.contains(desired_rights) {
            return Err(RcError::AccessDenied);
        }
        Ok(object)
    }

    pub fn dup_handle_operating_rights(
        &self,
        handle_value: HandleValue,
        operation: impl FnOnce(Rights) -> RcResult<Rights>,
    ) -> RcResult<HandleValue> {
        let mut inner = self.inner.lock();
        let mut handle = match inner.handles.get(&handle_value) {
            Some(handle) => handle.clone(),
            None => return Err(RcError::BadHandle),
        };
        handle.rights = operation(handle.rights)?;
        let new_handle_value = inner.add_handle(handle);
        Ok(new_handle_value)
    }

    /// Exit current process with `retcode`.
    /// The process do not terminate immediately when exited.
    /// It will terminate after all its child threads are terminated.
    pub fn exit(&self, retcode: i64) {
        let mut inner = self.inner.lock();
        if let Status::Exited(_) = inner.status {
            return;
        }
        inner.status = Status::Exited(retcode);
        inner.handles.clear();
    }

    /// The process finally terminates.
    fn terminate(&self) {
        let mut inner = self.inner.lock();
        let _retcode = match inner.status {
            Status::Exited(retcode) => retcode,
            _ => {
                inner.status = Status::Exited(0);
                0
            }
        };
        self.job.remove_process(self.base.id);
    }

    /// Check whether `condition` is allowed in the parent job's policy.
    pub fn check_policy(&self, condition: PolicyCondition) -> RcResult<()> {
        match self
            .policy
            .get_action(condition)
            .unwrap_or(PolicyAction::Allow)
        {
            PolicyAction::Allow => Ok(()),
            PolicyAction::Deny => Err(RcError::AccessDenied),
            _ => unimplemented!(),
        }
    }

    /// Get process status.
    pub fn status(&self) -> Status {
        self.inner.lock().status
    }

    /// Get the `VmAddressRegion` of the process.
    pub fn vmar(&self) -> Arc<VirtualMemory> {
        self.virtual_memory.clone()
    }

    /// Get the job of the process.
    pub fn job(&self) -> Arc<Job> {
        self.job.clone()
    }

    /// Add a thread to the process.
    pub(super) fn add_thread(&self, thread: Arc<Thread>) -> RcResult<()> {
        let mut inner = self.inner.lock();
        if let Status::Exited(_) = inner.status {
            return Err(RcError::BadState);
        }
        inner.threads.push(thread);
        Ok(())
    }

    /// Remove a thread from the process.
    ///
    /// If no more threads left, exit the process.
    pub(super) fn remove_thread(&self, tid: KoID) {
        let mut inner = self.inner.lock();
        inner.threads.retain(|t| t.id() != tid);
        if inner.threads.is_empty() {
            drop(inner);
            self.terminate();
        }
    }

    /// Get KoIDs of Threads.
    pub fn thread_ids(&self) -> Vec<KoID> {
        self.inner.lock().threads.iter().map(|t| t.id()).collect()
    }

    /// Get information of this process.
    pub fn get_info(&self) -> ProcessInfo {
        let mut info = ProcessInfo {
            ..Default::default()
        };
        match self.inner.lock().status {
            Status::Init => {
                info.started = false;
                info.has_exited = false;
            }
            Status::Running => {
                info.started = true;
                info.has_exited = false;
            }
            Status::Exited(ret) => {
                info.return_code = ret;
                info.has_exited = true;
                info.started = true;
            }
        }
        info
    }
}

/// Information of a process.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Default)]
pub struct ProcessInfo {
    pub return_code: i64,
    pub started: bool,
    pub has_exited: bool,
}

struct ProcessInner {
    max_handle_id: u32,
    status: Status,
    handles: BTreeMap<HandleValue, Handle>,
    threads: Vec<Arc<Thread>>,
}

/// Status of a process.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Status {
    /// Initial state, no thread present in process.
    Init,
    /// First thread has started and is running.
    Running,
    /// Process has exited with the code.
    Exited(i64),
}

impl ProcessInner {
    pub fn new() -> Self {
        Self {
            max_handle_id: 0,
            handles: BTreeMap::new(),
            status: Status::Init,
            threads: Vec::new(),
        }
    }

    /// Add a handle to the process
    fn add_handle(&mut self, handle: Handle) -> HandleValue {
        let key = (self.max_handle_id << 2) | 0x3u32;
        self.max_handle_id += 1;
        self.handles.insert(key, handle);
        key
    }

    fn remove_handle(&mut self, handle_value: HandleValue) -> RcResult<Handle> {
        let handle = self
            .handles
            .remove(&handle_value)
            .ok_or(RcError::BadHandle)?;
        Ok(handle)
    }

    fn get_handle(&mut self, handle_value: HandleValue) -> RcResult<Handle> {
        let handle = self.handles.get(&handle_value).ok_or(RcError::BadHandle)?;
        Ok(handle.clone())
    }

    /// Whether `thread` is in this process.
    fn contains_thread(&self, thread: &Arc<Thread>) -> bool {
        self.threads.iter().any(|t| Arc::ptr_eq(t, thread))
    }
}

struct ProcessBinary;

impl ProcessBinary {
    fn parse(bin: &'static [u8]) -> ElfBytes<'static, NativeEndian> {
        //File::parse(bin).expect("Failed to parse ELF binary!")
        ElfBytes::<NativeEndian>::minimal_parse(bin).expect("Failed to parse ELF binary!")
    }

    fn map_segments(elf_file: &ElfBytes<'static, NativeEndian>, vm: Arc<VirtualMemory>) {
        for segment in elf_file.segments().expect("ELF file contains no segment!") {
            if !segment.p_type == PT_LOAD {
                continue;
            }

            let start_address =
                Page::<Size4KiB>::containing_address(VirtAddr::new(segment.p_vaddr))
                    .start_address()
                    .as_u64();
            let page_cnt = page_count(
                segment.p_memsz as usize + segment.p_vaddr as usize - start_address as usize,
            );

            if page_cnt == 0 {
                continue;
            }

            let child = vm.create_child_absolute(start_address as usize, page_cnt);
            for count in 0..page_cnt {
                let child_page = child.create_child(Range::from(count..count + 1));
                let memory = PhysicalMemory::allocate(1).unwrap();

                let vm_mapping = Arc::new(VmMapping::new(
                    Self::elf_flags2mmu_flags(segment.p_flags),
                    child_page.clone(),
                    memory.clone(),
                ));
                if let Err(_) = vm_mapping.map() {
                    memory.deallocate();
                }
            }

            if let Ok(data) = elf_file.segment_data(&segment) {
                unsafe {
                    child.write((segment.p_vaddr - start_address) as usize, data);
                }
            }
        }
    }

    fn elf_flags2mmu_flags(elf_flags: u32) -> MMUFlags {
        let mut flags = MMUFlags::USER;
        if elf_flags & PF_R != 0 {
            flags |= MMUFlags::READ;
        }
        if elf_flags & PF_X != 0 {
            flags |= MMUFlags::EXECUTE;
        }
        if elf_flags & PF_W != 0 {
            flags |= MMUFlags::WRITE;
        }
        flags
    }
}
