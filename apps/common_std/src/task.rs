use alloc::vec::Vec;

use crate::{
    RcResult,
    ipc::{Channel, MessagePacket},
    syscall,
};

pub struct Process(u32);

impl Process {
    pub fn create(name: &str, binary: &[u8], handles: Vec<u32>) -> RcResult<Self> {
        let (channel0, channel1) = Channel::create()?;

        let mut handle = 0u32;
        syscall!(
            1,
            &raw mut handle,
            name.as_ptr() as usize,
            name.len(),
            binary.as_ptr() as usize,
            binary.len(),
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
}
