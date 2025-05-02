use alloc::vec::{self, Vec};

use crate::{ARG_HANDLE, RcResult, syscall};

pub struct Channel(u32);

impl Channel {
    pub fn with_father() -> Self {
        Channel(ARG_HANDLE)
    }

    pub fn create() -> RcResult<(Self, Self)> {
        let mut handle0 = 0;
        let mut handle1 = 0;
        syscall!(15, &mut handle0, &mut handle1)?;
        Ok((Channel(handle0), Channel(handle1)))
    }

    pub fn write(&self, packet: &MessagePacket) -> RcResult<()> {
        syscall!(
            17,
            self.0,
            packet.data.as_ptr(),
            packet.data.len(),
            packet.handles.as_ptr(),
            packet.handles.len()
        )?;
        Ok(())
    }

    pub fn read(&self) -> RcResult<MessagePacket> {
        let mut data = alloc::vec![0u8; 64*1024];
        let mut handles = alloc::vec![0u32; 1024];

        let mut data_len = 0;
        let mut handles_len = 0;

        syscall!(
            16,
            self.0,
            data.as_mut_ptr(),
            &mut data_len,
            handles.as_mut_ptr(),
            &mut handles_len
        )?;
        Ok(MessagePacket::new(
            data[..data_len].to_vec(),
            handles[..handles_len].to_vec(),
        ))
    }

    pub fn as_handle(&self) -> u32 {
        self.0
    }

    pub unsafe fn from_handle(handle: u32) -> Self {
        Channel(handle)
    }
}

#[derive(Default)]
pub struct MessagePacket {
    pub data: Vec<u8>,
    pub handles: Vec<u32>,
}

impl MessagePacket {
    pub fn new(data: Vec<u8>, handles: Vec<u32>) -> Self {
        Self {
            data: data,
            handles: handles,
        }
    }
}
