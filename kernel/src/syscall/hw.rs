use crate::{
    error::{RcError, RcResult},
    hal::{IoPort, int::Irq},
    object::{Handle, HandleValue, Rights},
};

use super::current_process;

pub fn create_io_port(port: u16, option: usize, handle_ptr: *mut HandleValue) -> RcResult<usize> {
    let current_process = current_process();
    let io_port = IoPort::new(port);

    let mut right = Rights::BASIC;
    if option & 0b1 != 0 {
        right |= Rights::READ;
    }
    if option & 0b10 != 0 {
        right |= Rights::WRITE;
    }

    let handle = current_process.add_handle(Handle::new(io_port, right));
    unsafe {
        *handle_ptr = handle;
    }

    Ok(0)
}

pub fn io_port_read(handle: HandleValue, size: usize) -> RcResult<usize> {
    let current_process = current_process();
    let io_port = current_process.get_object_with_rights::<IoPort>(handle, Rights::READ)?;

    match size {
        1 => Ok(io_port.read::<u8>() as usize),
        2 => Ok(io_port.read::<u16>() as usize),
        4 => Ok(io_port.read::<u32>() as usize),
        _ => Err(RcError::InvalidArguments),
    }
}

pub fn io_port_write(handle: HandleValue, value: usize, size: usize) -> RcResult<usize> {
    let current_process = current_process();
    let io_port = current_process.get_object_with_rights::<IoPort>(handle, Rights::WRITE)?;

    match size {
        1 => io_port.write::<u8>(value as u8),
        2 => io_port.write::<u16>(value as u16),
        4 => io_port.write::<u32>(value as u32),
        _ => return Err(RcError::InvalidArguments),
    }

    Ok(0)
}

pub fn irq_register(irq: u8, handle_ptr: *mut HandleValue) -> RcResult<usize> {
    let current_process = current_process();
    let handle = current_process.add_handle(Handle::new(
        Irq::register(irq)?,
        Rights::BASIC & !Rights::DUPLICATE,
    ));

    unsafe {
        *handle_ptr = handle;
    }

    Ok(0)
}

pub fn irq_wait(handle: HandleValue) -> RcResult<usize> {
    let current_process = current_process();
    let irq = current_process.get_object_with_rights::<Irq>(handle, Rights::empty())?;

    x86_64::instructions::interrupts::enable();

    irq.wait();

    x86_64::instructions::interrupts::disable();

    Ok(0)
}
