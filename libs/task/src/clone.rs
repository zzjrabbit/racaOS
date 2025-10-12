#![allow(dead_code)]

use alloc::sync::Arc;
use bitflags::bitflags;
use ostd::{Error as OstdError, arch::cpu::context::UserContext, mm::Vaddr, task::Task};

use crate::{AsThread, Process, Signal, UserThreadData, spawn_user_thread};

bitflags! {
    #[derive(Default, Clone, Copy, Debug)]
    pub struct CloneFlags: u32 {
        const CLONE_NEWTIME = 0x00000080;       /* New time namespace */
        const CLONE_VM      = 0x00000100;       /* Set if VM shared between processes.  */
        const CLONE_FS      = 0x00000200;       /* Set if fs info shared between processes.  */
        const CLONE_FILES   = 0x00000400;       /* Set if open files shared between processes.  */
        const CLONE_SIGHAND = 0x00000800;       /* Set if signal handlers shared.  */
        const CLONE_PIDFD   = 0x00001000;       /* Set if a pidfd should be placed in parent.  */
        const CLONE_PTRACE  = 0x00002000;       /* Set if tracing continues on the child.  */
        const CLONE_VFORK   = 0x00004000;       /* Set if the parent wants the child to wake it up on mm_release.  */
        const CLONE_PARENT  = 0x00008000;       /* Set if we want to have the same parent as the cloner.  */
        const CLONE_THREAD  = 0x00010000;       /* Set to add to same thread group.  */
        const CLONE_NEWNS   = 0x00020000;       /* Set to create new namespace.  */
        const CLONE_SYSVSEM = 0x00040000;       /* Set to shared SVID SEM_UNDO semantics.  */
        const CLONE_SETTLS  = 0x00080000;       /* Set TLS info.  */
        const CLONE_PARENT_SETTID = 0x00100000; /* Store TID in userlevel buffer before MM copy.  */
        const CLONE_CHILD_CLEARTID = 0x00200000;/* Register exit futex and memory location to clear.  */
        const CLONE_DETACHED = 0x00400000;      /* Create clone detached.  */
        const CLONE_UNTRACED = 0x00800000;      /* Set if the tracing process can't force CLONE_PTRACE on this clone.  */
        const CLONE_CHILD_SETTID = 0x01000000;  /* Store TID in userlevel buffer in the child.  */
        const CLONE_NEWCGROUP   = 0x02000000;	/* New cgroup namespace.  */
        const CLONE_NEWUTS	= 0x04000000;	    /* New utsname group.  */
        const CLONE_NEWIPC	= 0x08000000;	    /* New ipcs.  */
        const CLONE_NEWUSER	= 0x10000000;	    /* New user namespace.  */
        const CLONE_NEWPID	= 0x20000000;	    /* New pid namespace.  */
        const CLONE_NEWNET	= 0x40000000;	    /* New network namespace.  */
        const CLONE_IO	= 0x80000000;	        /* Clone I/O context.  */

        /// A bitmask of all `CloneFlags` related to namespace creation.
        const CLONE_NS_FLAGS = Self::CLONE_NEWTIME.bits() |
            Self::CLONE_NEWNS.bits() |
            Self::CLONE_NEWCGROUP.bits() |
            Self::CLONE_NEWUTS.bits() |
            Self::CLONE_NEWIPC.bits() |
            Self::CLONE_NEWUSER.bits() |
            Self::CLONE_NEWPID.bits() |
            Self::CLONE_NEWNET.bits();
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CloneArgs {
    pub flags: CloneFlags,
    pub child_tid: Vaddr,
    pub stack: Vaddr,
    pub stack_size: usize,
    pub tls: u64,
    pub child_signal: Signal,
}

impl CloneArgs {
    pub fn for_fork() -> Self {
        Self {
            child_signal: Signal::SIGCHLD,
            ..Self::default()
        }
    }
}

pub fn clone_child(
    clone_args: CloneArgs,
    parent: Arc<Task>,
    context: &UserContext,
) -> Result<(Arc<Task>, Arc<Process>), OstdError> {
    let parent_data = parent.direct_downcast::<UserThreadData>().unwrap();
    let parent_process = parent_data.process.upgrade().unwrap();

    let process = parent_process.fork(clone_args.child_signal,parent_process.signal_disposition());

    let fs_info = if clone_args.flags.contains(CloneFlags::CLONE_FILES) {
        parent_data.fs_info().clone()
    } else {
        Arc::new(parent_data.fs_info().deep_clone())
    };
    let memory_info = if clone_args.flags.contains(CloneFlags::CLONE_VM) {
        parent_data.memory_info().clone()
    } else {
        Arc::new(parent_data.memory_info().deep_clone())
    };
    let tid_address = parent_data.tid_address.read().clone();

    let child_data = UserThreadData::new_all(&process, memory_info.clone(), fs_info, tid_address);

    let mut child_context = context.clone();
    child_context.set_rax(0);
    if clone_args.flags.contains(CloneFlags::CLONE_SETTLS) {
        child_context.set_tls_pointer(clone_args.tls as usize);
    }

    let child_thread = spawn_user_thread(&process, child_context, memory_info, Some(child_data));

    Ok((child_thread, process))
}
