use crate::{RcResult, syscall};

pub struct EventPair(u32);

impl EventPair {
    pub fn new() -> RcResult<(EventPair, EventPair)> {
        let mut handle0 = 0;
        let mut handle1 = 0;

        syscall!(22, &mut handle0, &mut handle1)?;

        Ok((EventPair(handle0), EventPair(handle1)))
    }

    pub fn as_handle(&self) -> u32 {
        self.0
    }
}

pub fn set_signal(handle: u32, signal: Signal) -> RcResult<()> {
    syscall!(25, handle, signal.bits())?;

    Ok(())
}

pub fn clear_signal(handle: u32, signal: Signal) -> RcResult<()> {
    syscall!(27, handle, signal.bits())?;

    Ok(())
}

pub fn wait_for_signal(handles: &[u32], signal: Signal) -> RcResult<u32> {
    let mut source = 0u32;
    syscall!(
        28,
        handles.as_ptr(),
        handles.len(),
        signal.bits(),
        &mut source
    )?;
    Ok(source)
}

bitflags::bitflags! {
    #[derive(Default, Clone, Copy, Debug)]
    pub struct Signal: u32 {
        const READABLE = 1 << 0;

        const INTERRUPT_PRESENT = 1 << 1;

        const PEER_CLOSED = 1 << 2;

        const TASK_DEAD = 1 << 3;

        const USER_SIGNAL0 = 1 << 24;
        const USER_SIGNAL1 = 1 << 25;
        const USER_SIGNAL2 = 1 << 26;
        const USER_SIGNAL3 = 1 << 27;
        const USER_SIGNAL4 = 1 << 28;
        const USER_SIGNAL5 = 1 << 29;
        const USER_SIGNAL6 = 1 << 30;
        const USER_SIGNAL7 = 1 << 31;
    }
}
