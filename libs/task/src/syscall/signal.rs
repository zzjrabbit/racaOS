use alloc::sync::Arc;
use errors::{Errno, Error};
use ostd::{Pod, mm::Vaddr, task::Task};

use crate::{
    AsThread, Process, Signal, SignalAction, SignalActionFlags, SignalMask, SignalSet,
    UserThreadData, syscall::SyscallResult,
};

pub fn rt_sigaction(
    signal: u8,
    signal_action_addr: Vaddr,
    old_signal_action_addr: Vaddr,
    signal_set_size: u64,
) -> SyscallResult {
    let signal = Signal::try_from(signal)?;

    if signal_set_size != 8 {
        return Err(Errno::EINVAL.no_message());
    }

    let process = Process::current();
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    let signal_disposition = process.signal_disposition();
    let mut signal_disposition = signal_disposition.lock();

    let old_action = if signal_action_addr != 0 {
        if signal == Signal::SIGKILL || signal == Signal::SIGSTOP {
            return Err(Errno::EINVAL.no_message());
        }

        let signal_action_c = data
            .memory_info()
            .vmar()
            .read_val::<sigaction_t>(signal_action_addr)?;
        let signal_action = SignalAction::from(signal_action_c);
        discard_signals_if_ignored(process.clone(), signal, &signal_action);
        signal_disposition.set(signal, signal_action.clone())
    } else {
        signal_disposition.get(signal)
    };

    if old_signal_action_addr != 0 {
        let old_signal_action_c = sigaction_t::from(old_action);
        data.memory_info()
            .vmar()
            .write_val(old_signal_action_addr, &old_signal_action_c)?;
    }

    Ok(0)
}

// be discarded, whether or not it is blocked
fn discard_signals_if_ignored(process: Arc<Process>, signal: Signal, signal_action: &SignalAction) {
    if !signal_action.will_ignore(signal) {
        return;
    }

    let mask = SignalSet::new_full() - signal;

    for task in process.threads() {
        let data = task.direct_downcast::<UserThreadData>().unwrap();
        data.dequeue_signal(&mask);
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Pod)]
#[repr(C)]
struct sigaction_t {
    pub handler_ptr: Vaddr,
    pub flags: u32,
    pub restorer_ptr: Vaddr,
    pub mask: u64,
}

const SIG_DFL: usize = 0;
const SIG_IGN: usize = 1;

impl From<sigaction_t> for SignalAction {
    fn from(input: sigaction_t) -> Self {
        match input.handler_ptr {
            SIG_DFL => SignalAction::Default,
            SIG_IGN => SignalAction::Ignore,
            _ => {
                let flags = SignalActionFlags::from_bits_truncate(input.flags);
                let mask = {
                    let mut sigset = SignalSet::from(input.mask);
                    // SIGSTOP and SIGKILL cannot be masked
                    sigset -= Signal::SIGSTOP;
                    sigset -= Signal::SIGKILL;
                    sigset
                };
                SignalAction::User {
                    handler: input.handler_ptr,
                    flags,
                    restorer: input.restorer_ptr,
                    mask,
                }
            }
        }
    }
}

impl From<SignalAction> for sigaction_t {
    fn from(input: SignalAction) -> Self {
        match input {
            SignalAction::Default => sigaction_t {
                handler_ptr: SIG_DFL,
                flags: 0,
                restorer_ptr: 0,
                mask: 0,
            },
            SignalAction::Ignore => sigaction_t {
                handler_ptr: SIG_IGN,
                flags: 0,
                restorer_ptr: 0,
                mask: 0,
            },
            SignalAction::User {
                handler,
                flags,
                restorer,
                mask,
            } => sigaction_t {
                handler_ptr: handler,
                flags: flags.bits(),
                restorer_ptr: restorer,
                mask: u64::from(mask),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MaskOp {
    Block = 0,
    Unblock = 1,
    SetMask = 2,
}

impl TryFrom<u32> for MaskOp {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MaskOp::Block),
            1 => Ok(MaskOp::Unblock),
            2 => Ok(MaskOp::SetMask),
            _ => Err(Errno::EINVAL.no_message()),
        }
    }
}

pub fn rt_sigprocmask(
    how: u32,
    set_addr: Vaddr,
    old_set_addr: Vaddr,
    signal_set_size: u64,
) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    if signal_set_size != 8 {
        return Err(Errno::EINVAL.no_message());
    }

    let mask_op = MaskOp::try_from(how)?;

    let old_signal_mask = data.blocked_signals();

    if old_set_addr != 0 {
        data.memory_info()
            .vmar()
            .write_val(old_set_addr, &u64::from(old_signal_mask))?;
    }

    if set_addr != 0 {
        let mut read_mask = data.memory_info().vmar().read_val::<SignalMask>(set_addr)?;
        match mask_op {
            MaskOp::Block => {
                read_mask -= Signal::SIGKILL;
                read_mask -= Signal::SIGSTOP;
                data.replace_signal_mask(old_signal_mask + read_mask);
            }
            MaskOp::Unblock => {
                data.replace_signal_mask(old_signal_mask - read_mask);
            }
            MaskOp::SetMask => {
                read_mask -= Signal::SIGKILL;
                read_mask -= Signal::SIGSTOP;
                data.replace_signal_mask(read_mask);
            }
        }
    }

    Ok(0)
}
