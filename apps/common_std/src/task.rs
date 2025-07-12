use alloc::vec::Vec;

use crate::{
    ipc::{Channel, MessagePacket}, memory::{MMUFlags, PhysicalMemory, VirtualMemory}, signal::{wait_for_signal, Signal}, syscall, RcResult
};

/// Information of a process.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Default)]
pub struct ProcessInfo {
    pub return_code: i64,
    pub started: bool,
    pub has_exited: bool,
}

#[repr(C)]
struct ProcessCreationArguments {
    name_ptr: *const u8,
    name_len: usize,
    binary_ptr: *const u8,
    binary_len: usize,
}

pub struct Process(u32);

impl Process {
    pub fn create(job: &Job, name: &str, binary: &[u8], handles: Vec<u32>) -> RcResult<Self> {
        let (channel0, channel1) = Channel::create()?;

        let args = ProcessCreationArguments {
            name_ptr: name.as_ptr(),
            name_len: name.len(),
            binary_ptr: binary.as_ptr(),
            binary_len: binary.len(),
        };

        let mut handle = 0u32;
        syscall!(
            1,
            job.as_handle(),
            &raw mut handle,
            &args,
            channel0.as_handle()
        )?;

        channel1.write(&MessagePacket::new(
            handles.len().to_le_bytes().to_vec(),
            alloc::vec![],
        ))?;

        if handles.len() != 0 {
            channel1.write(&MessagePacket::new(alloc::vec![], handles))?;
        }

        Ok(Self(handle))
    }

    pub fn as_handle(&self) -> u32 {
        self.0
    }

    pub fn get_info(&self) -> RcResult<ProcessInfo> {
        let mut info = ProcessInfo::default();

        syscall!(29, self.as_handle(), &mut info)?;

        Ok(info)
    }

    pub fn wait(&self) -> RcResult<()> {
        wait_for_signal(&[self.as_handle()], Signal::TASK_DEAD)?;
        Ok(())
    }

    pub fn kill(&self) -> RcResult<()> {
        syscall!(34, self.as_handle())?;
        Ok(())
    }
}

pub struct Job(u32);

impl Job {
    pub unsafe fn from_handle(handle: u32) -> Self {
        Self(handle)
    }

    pub fn as_handle(&self) -> u32 {
        self.0
    }

    pub fn create_child(&self) -> RcResult<Self> {
        let mut handle_value = 0u32;
        syscall!(23, self.as_handle(), &mut handle_value)?;
        Ok(Self(handle_value))
    }

    pub fn set_basic_policy(&self, basic_policies: &[BasicPolicy]) -> RcResult<()> {
        syscall!(
            24,
            self.as_handle(),
            basic_policies.as_ptr(),
            basic_policies.len()
        )?;

        Ok(())
    }

    pub fn kill(&self) -> RcResult<()> {
        syscall!(33, self.as_handle())?;
        Ok(())
    }
}

/// The policy type.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BasicPolicy {
    /// Condition when the policy is applied.
    pub condition: PolicyCondition,
    ///
    pub action: PolicyAction,
}

/// The condition when a policy is applied.
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum PolicyCondition {
    /// A process under this job is attempting to map an address region with write-execute access.
    VmarWx = 1,
    // A process under this job is attempting to create a new DDK( Driver Development Kit ) Object.
    NewDdkObject = 2,
}

/// The action taken when the condition happens specified by a policy.
#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PolicyAction {
    /// Allow condition.
    Allow = 0,
    /// Prevent condition.
    Deny = 1,
    /// Terminate the process.
    Kill = 2,
}

pub struct Thread(u32);

impl Thread {
    pub fn new(name: &str, thread_func: fn() -> !) -> RcResult<Self> {
        const STACK_SIZE: usize = 8 * 1024 * 1024;
        let page_count = STACK_SIZE / 4096;

        let stack = VirtualMemory::root_virtual_memory().unwrap().allocate_child(page_count)?;
        
        for id in 0..page_count {
            let child = stack.create_child(id, 1).unwrap();
            let pm = PhysicalMemory::create(1).unwrap();
            child.map(pm, MMUFlags::READ | MMUFlags::WRITE).unwrap();
        }

        let mut handle = 0;
        syscall!(
            31,
            name.as_ptr(),
            name.len(),
            thread_func as usize,
            stack.as_handle(),
            &mut handle
        )?;

        Ok(Self(handle))
    }

    pub fn as_handle(&self) -> u32 {
        self.0
    }
}

impl Thread {
    pub fn join(&self) -> RcResult<()> {
        wait_for_signal(&[self.as_handle()], Signal::TASK_DEAD)?;
        Ok(())
    }

    pub fn kill(&self) -> RcResult<()> {
        syscall!(35, self.as_handle())?;
        Ok(())
    }
}

pub fn thread_exit() -> ! {
    syscall!(32).unwrap();
    loop {}
}
