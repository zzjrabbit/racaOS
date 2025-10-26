mod action;
mod ctypes;
mod disposition;
mod event;
mod mask;
mod queues;
mod signal;
mod stack;

pub use action::*;
use align_ext::AlignExt;
use alloc::sync::Arc;
use bitflags::bitflags;
pub(crate) use ctypes::*;
pub use disposition::*;
use errors::{Errno, Result};
pub use event::*;
pub use mask::*;
use memory::Vmar;
use ostd::{arch::cpu::context::UserContext, mm::Vaddr, user::UserContextApi};
pub use queues::*;
pub use signal::*;
pub use stack::*;

use crate::{Process, UserThreadData};

pub(crate) fn set_new_stack(data: &UserThreadData, stack: stack_t, sp: usize) -> Result<()> {
    fn check_new_ss_flags(ss_flags: u32) -> Result<SignalStackFlags> {
        let ss_flags = SignalStackFlags::from_bits(ss_flags)
            .ok_or_else(|| Errno::EINVAL.with_message("unknown signal stack flags"))?;

        let status_flags = SignalStackStatusFlags::from_bits_truncate(ss_flags.bits());

        // Linux permits SS_ONSTACK to be set on a new stack, so we follow Linux's behavior here.
        // However, this may be considered a BUG.
        // Reference: <https://man7.org/linux/man-pages/man2/sigaltstack.2.html#BUGS>.
        if status_flags != SignalStackStatusFlags::SS_DISABLE
            && status_flags != SignalStackStatusFlags::SS_ONSTACK
            && status_flags != SignalStackStatusFlags::empty()
        {
            return Err(Errno::EINVAL.no_message());
        }

        Ok(ss_flags)
    }

    let old_stack = data.signal_stack();
    let mut old_stack = old_stack.write();

    if old_stack.contains(sp) {
        return Err(Errno::EPERM.with_message("the old stack is active now"));
    }

    let flags = check_new_ss_flags(stack.ss_flags as u32)?;

    let new_stack = if flags.contains(SignalStackFlags::SS_DISABLE) {
        SignalStack::new(0, flags, 0)
    } else {
        const MINSTKSZ: usize = 2048;

        if stack.ss_size < MINSTKSZ {
            return Err(Errno::ENOMEM.with_message("stack size is less than MINSTKSZ"));
        }

        if stack.ss_sp.checked_add(stack.ss_size).is_none() {
            return Err(Errno::EINVAL.with_message("overflow for given stack addr and size"));
        }

        SignalStack::new(stack.ss_sp, flags, stack.ss_size)
    };

    *old_stack = new_stack;

    Ok(())
}

bitflags! {
    struct SignalStackAttrFlags: u32 {
        const SS_AUTODISARM = SignalStackFlags::SS_AUTODISARM.bits();
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct SignalStackStatusFlags: u32 {
        const SS_ONSTACK = SignalStackFlags::SS_ONSTACK.bits();
        const SS_DISABLE = SignalStackFlags::SS_DISABLE.bits();
    }
}

impl From<SignalStackStatus> for SignalStackStatusFlags {
    fn from(value: SignalStackStatus) -> Self {
        match value {
            SignalStackStatus::Inactive => SignalStackStatusFlags::empty(),
            SignalStackStatus::Active => SignalStackStatusFlags::SS_ONSTACK,
            SignalStackStatus::Disable => SignalStackStatusFlags::SS_DISABLE,
        }
    }
}

pub(crate) fn handle_pending_signal(
    context: &mut UserContext,
    data: &UserThreadData,
    pre_syscall_ret: Option<usize>,
) -> Result<()> {
    let syscall_restart = if let Some(pre_syscall_ret) = pre_syscall_ret
        && context.rax() == (-(Errno::ERESTARTSYS as i32)) as usize
    {
        context.set_rax(-(Errno::EINTR as i32) as usize);
        Some(pre_syscall_ret)
    } else {
        None
    };

    let process = Process::current();

    let signal_kind = {
        let signal_mask = data.blocked_signals();
        if let Some(signal) = data.dequeue_signal(&signal_mask) {
            signal
        } else {
            return Ok(());
        }
    };
    let signal = signal_kind.signal();

    let signal_action = {
        let signal_disposition = process.signal_disposition();
        let mut signal_disposition = signal_disposition.lock();
        let signal_action = signal_disposition.get(signal);

        if let SignalAction::User { flags, .. } = signal_action
            && flags.contains(SignalActionFlags::SA_RESETHAND)
        {
            signal_disposition.set_default(signal);
        }

        signal_action
    };

    match signal_action {
        SignalAction::Ignore => {
            log::trace!("Ignoring signal {}!", signal);
        }
        SignalAction::User {
            handler,
            flags,
            restorer,
            mask,
        } => {
            if let Some(pre_syscall_ret) = syscall_restart
                && flags.contains(SignalActionFlags::SA_RESTART)
            {
                const SYSCALL_INST_LEN: usize = 2;

                context.set_rax(pre_syscall_ret);
                context.set_instruction_pointer(context.instruction_pointer() - SYSCALL_INST_LEN);
            }

            if let Err(e) =
                handle_user_signal(data, signal_kind, handler, flags, restorer, mask, context)
            {
                log::warn!("Signal handling failed with error {}!", e);
                process.exit(-1);
            }
        }
        SignalAction::Default => {
            let signal_default_action = SignalDefaultAction::from_signal(signal);
            match signal_default_action {
                SignalDefaultAction::Core | SignalDefaultAction::Terminate => {
                    log::warn!("Process{} terminating on signal {}!", process.id(), signal);
                    process.exit(-1);
                }
                SignalDefaultAction::Ignore => {}
                _ => unimplemented!(),
            }
        }
    }

    Ok(())
}

fn handle_user_signal(
    data: &UserThreadData,
    signal_kind: SignalKind,
    handler: Vaddr,
    flags: SignalActionFlags,
    restorer: Vaddr,
    mut mask: SignalMask,
    context: &mut UserContext,
) -> Result<()> {
    const SI_KERNEL: i32 = 128;
    const SI_QUEUE: i32 = -1;
    const SI_USER: i32 = 0;
    const SI_TKILL: i32 = -6;

    let signal = signal_kind.signal();
    let signal_info = siginfo_t::new(
        signal,
        match signal_kind {
            SignalKind::Kernel { .. } => SI_KERNEL,
            SignalKind::User { kind, .. } => match kind {
                UserSignalKind::Kill => SI_USER,
                UserSignalKind::TKill => SI_TKILL,
                UserSignalKind::SignalQueue => SI_QUEUE,
            },
            SignalKind::Fault { code, .. } => code,
        },
    );

    if !flags.contains(SignalActionFlags::SA_NODEFER) {
        mask += signal;
    }

    let old_mask = data.blocked_signals();
    data.replace_signal_mask(mask + old_mask);

    let mut stack_pointer =
        if let Some(sp) = use_alternate_signal_stack(data, flags, context.stack_pointer()) {
            sp as u64
        } else {
            context.stack_pointer() as u64
        };

    stack_pointer -= 128;

    let vmar = data.memory_info().vmar();

    stack_pointer -= size_of::<siginfo_t>() as u64;
    vmar.write_val(stack_pointer as _, &signal_info)?;
    let signal_info_addr = stack_pointer;

    let uc_stack = {
        let signal_stack = data.signal_stack();
        let mut signal_stack = signal_stack.write();
        let stack = stack_t::from(&*signal_stack);

        if signal_stack
            .flags()
            .contains(SignalStackFlags::SS_AUTODISARM)
        {
            signal_stack.reset();
        }

        stack
    };

    let mut ucontext = ucontext_t {
        uc_sigmask: mask.into(),
        uc_stack,
        ..Default::default()
    };

    ucontext.uc_mcontext.copy_user_regs_from(context);
    let signal_context = data.signal_context();
    let mut signal_context = signal_context.write();
    if let Some(signal_context_addr) = *signal_context {
        ucontext.uc_link = signal_context_addr;
    } else {
        ucontext.uc_link = 0;
    }

    let ucontext_addr = alloc_aligned_in_user_stack(
        stack_pointer,
        size_of::<ucontext_t>(),
        align_of::<ucontext_t>(),
    )?;

    vmar.write_val(ucontext_addr as Vaddr, &ucontext)?;
    *signal_context = Some(ucontext_addr as Vaddr);

    stack_pointer = ucontext_addr;
    if flags.contains(SignalActionFlags::SA_RESTORER) {
        stack_pointer = write_u64_to_user_stack(&vmar, stack_pointer, restorer as u64)?;
    }

    context.set_instruction_pointer(handler);
    context.set_stack_pointer(stack_pointer as _);

    if flags.contains(SignalActionFlags::SA_SIGINFO) {
        set_arguments(context, signal, signal_info_addr as _, ucontext_addr as _);
    } else {
        set_arguments(context, signal, 0, 0);
    }

    const X86_RFLAGS_DF: usize = 1 << 10;
    context.general_regs_mut().rflags &= !X86_RFLAGS_DF;

    Ok(())
}

fn use_alternate_signal_stack(
    data: &UserThreadData,
    flags: SignalActionFlags,
    sp: usize,
) -> Option<usize> {
    if !flags.contains(SignalActionFlags::SA_ONSTACK) {
        return None;
    }

    let sig_stack = data.signal_stack();

    if sig_stack.read().active_status(sp) != SignalStackStatus::Inactive {
        return None;
    }

    // Make sp align at 16. FIXME: is this required?
    let stack_pointer = (sig_stack.read().base() + sig_stack.read().size()).align_down(16);
    Some(stack_pointer)
}

fn alloc_aligned_in_user_stack(rsp: u64, size: usize, align: usize) -> Result<u64> {
    if !align.is_power_of_two() {
        return Err(Errno::EINVAL.with_message("align must be power of two"));
    }
    let start = (rsp - size as u64).align_down(align as u64);
    Ok(start)
}

fn write_u64_to_user_stack(vmar: &Arc<Vmar>, rsp: u64, value: u64) -> Result<u64> {
    let rsp = rsp - 8;
    vmar.write_val(rsp as Vaddr, &value)?;
    Ok(rsp)
}

fn set_arguments(
    context: &mut UserContext,
    signal: Signal,
    siginfo_addr: usize,
    ucontext_addr: usize,
) {
    context.set_rdi(u8::from(signal) as usize);
    context.set_rsi(siginfo_addr);
    context.set_rdx(ucontext_addr);
}
