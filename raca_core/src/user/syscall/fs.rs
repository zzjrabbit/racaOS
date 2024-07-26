use crate::{fs::operation::OpenMode, user::get_current_process};
use alloc::{string::String, vec};
use framework::memory::{addr_to_array, write_for_syscall};

use x86_64::VirtAddr;

pub fn open(buf_addr: usize, buf_len: usize, open_mode: usize) -> usize {
    let mut buf = vec![0; buf_len];

    if let Err(_) = get_current_process().read().page_table.read(
        VirtAddr::new(buf_addr as u64),
        buf_len,
        &mut buf,
    ) {
        panic!("Read error at {:x}!", buf_addr);
    }

    let path = String::from(core::str::from_utf8(buf.as_slice()).unwrap());

    let open_mode = match open_mode {
        0 => OpenMode::Read,
        1 => OpenMode::Write,
        _ => return 0,
    };

    let fd = crate::fs::operation::open(path.clone(), open_mode);
    if let Some(fd) = fd {
        fd
    } else {
        0
    }
}

pub fn write(fd: usize, buf_addr: usize, buf_len: usize) -> usize {
    let mut buf = vec![0; buf_len];

    if let Err(_) = get_current_process().read().page_table.read(
        VirtAddr::new(buf_addr as u64),
        buf_len,
        &mut buf,
    ) {
        panic!("Read error at {:x}!", buf_addr);
    }

    if let Some(_) = crate::fs::operation::write(fd, buf.as_slice()) {
        buf_len
    } else {
        0
    }
}

pub fn read(fd: usize, buf_addr: usize, buf_len: usize) -> usize {
    let mut buf = vec![0; buf_len];

    let ok = crate::fs::operation::read(fd, buf.as_mut()).is_some();

    write_for_syscall(VirtAddr::new(buf_addr as u64), buf.as_slice());

    let mut buf = vec![0; buf_len];

    if let Err(_) = get_current_process().read().page_table.read(
        VirtAddr::new(buf_addr as u64),
        buf_len,
        &mut buf,
    ) {
        panic!("Read error at {:x}!", buf_addr);
    }

    ok as usize * buf_len
}

pub fn close(fd: usize) -> usize {
    if let Some(_) = crate::fs::operation::close(fd) {
        1
    } else {
        0
    }
}

pub fn lseek(fd: usize, offset: usize) -> usize {
    if let Some(_) = crate::fs::operation::lseek(fd, offset) {
        1
    } else {
        0
    }
}

pub fn fsize(fd: usize) -> usize {
    if let Some(size) = crate::fs::operation::fsize(fd) {
        size
    } else {
        0
    }
}

pub fn open_pipe(buf_addr: usize) -> usize {
    let buffer: &mut [usize] = addr_to_array::<usize>(VirtAddr::new(buf_addr as u64), 2);
    if let Some(_) = crate::fs::operation::open_pipe(buffer) {
        1
    } else {
        0
    }
}
