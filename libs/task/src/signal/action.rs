use bitflags::bitflags;
use ostd::mm::Vaddr;

use crate::{Signal, SignalMask};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SignalAction {
    #[default]
    Default,
    Ignore,
    User {
        handler: Vaddr,
        flags: SignalActionFlags,
        restorer: Vaddr,
        mask: SignalMask,
    },
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SignalActionFlags: u32 {
        const SA_NOCLDSTOP  = 1;
        const SA_NOCLDWAIT  = 2;
        const SA_SIGINFO    = 4;
        const SA_ONSTACK    = 0x08000000;
        const SA_RESTART    = 0x10000000;
        const SA_NODEFER    = 0x40000000;
        const SA_RESETHAND  = 0x80000000;
        const SA_RESTORER   = 0x04000000;
    }
}

/// The default action to signals
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SignalDefaultAction {
    Terminate, // Default action is to terminate the process.
    Ignore,    // Default action is to ignore the signal.
    Core,      // Default action is to terminate the process and dump core (see core(5)).
    Stop,      // Default action is to stop the process.
    Continue,  // Default action is to continue the process if it is currently stopped.
}

impl SignalDefaultAction {
    pub fn from_signal(signal: Signal) -> SignalDefaultAction {
        match signal {
            Signal::SIGABRT | // = SIGIOT
            Signal::SIGBUS  |
            Signal::SIGFPE  |
            Signal::SIGILL  |
            Signal::SIGQUIT |
            Signal::SIGSEGV |
            Signal::SIGSYS  | // = SIGUNUSED
            Signal::SIGTRAP |
            Signal::SIGXCPU |
            Signal::SIGXFSZ
                => SignalDefaultAction::Core,
            Signal::SIGCHLD |
            Signal::SIGURG  |
            Signal::SIGWINCH
                => SignalDefaultAction::Ignore,
            Signal::SIGCONT
                => SignalDefaultAction::Continue,
            Signal::SIGSTOP |
            Signal::SIGTSTP |
            Signal::SIGTTIN |
            Signal::SIGTTOU
                => SignalDefaultAction::Stop,
            _
                => SignalDefaultAction::Terminate,
        }
    }
}
