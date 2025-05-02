use core::slice::from_raw_parts_mut;

use alloc::vec::Vec;

use crate::{
    error::RcResult,
    ipc::{Channel, MessagePacket},
    object::{Handle, HandleValue, Rights},
};

use super::current_process;

pub fn channel_create(
    handle0_ptr: *mut HandleValue,
    handle1_ptr: *mut HandleValue,
) -> RcResult<usize> {
    let current_process = current_process();

    let (channel0, channel1) = Channel::create();

    let handle0 = current_process.add_handle(Handle::new(channel0, Rights::DEFAULT_CHANNEL));
    let handle1 = current_process.add_handle(Handle::new(channel1, Rights::DEFAULT_CHANNEL));

    unsafe {
        *handle0_ptr = handle0;
        *handle1_ptr = handle1;
    }

    Ok(0)
}

pub fn channel_read(
    handle_value: HandleValue,
    bytes_ptr: *mut u8,
    bytes_len: *mut usize,
    handle_ptr: *mut HandleValue,
    handle_len: *mut usize,
) -> RcResult<usize> {
    let current_process = current_process();
    let channel = current_process.get_object_with_rights::<Channel>(handle_value, Rights::READ)?;

    let message_packet = channel.read()?;

    unsafe {
        *bytes_len = message_packet.data.len();
        *handle_len = message_packet.handles.len();
    }

    let bytes = unsafe { from_raw_parts_mut(bytes_ptr, message_packet.data.len()) };
    let handles = unsafe { from_raw_parts_mut(handle_ptr, message_packet.handles.len()) };

    bytes.copy_from_slice(&message_packet.data);
    handles.copy_from_slice(
        &message_packet
            .handles
            .iter()
            .map(|handle| current_process.add_handle(handle.clone()))
            .collect::<Vec<_>>(),
    );

    Ok(0)
}

pub fn channel_write(
    handle_value: HandleValue,
    bytes_ptr: *const u8,
    bytes_len: usize,
    handle_ptr: *const HandleValue,
    handle_len: usize,
) -> RcResult<usize> {
    let current_process = current_process();
    let channel = current_process.get_object_with_rights::<Channel>(handle_value, Rights::WRITE)?;

    let bytes = unsafe { core::slice::from_raw_parts(bytes_ptr, bytes_len) };
    let handles = unsafe { core::slice::from_raw_parts(handle_ptr, handle_len) };

    channel.write(MessagePacket::new(
        bytes.to_vec(),
        handles
            .iter()
            .map(|handle| current_process.remove_handle(*handle))
            .collect::<RcResult<Vec<_>>>()?,
    ))?;

    Ok(0)
}
