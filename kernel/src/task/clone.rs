#![allow(dead_code)]

use alloc::sync::Arc;
use bitflags::bitflags;
use ostd::{mm::Vaddr, task::Task, Error as OstdError};

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
}

pub fn clone_child(_clone_args: CloneArgs, _parent: Arc<Task>) -> Result<Arc<Task>, OstdError> {
    unimplemented!()
}
