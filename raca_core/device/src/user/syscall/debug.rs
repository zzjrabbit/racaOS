use crate::user::get_current_process;
use alloc::vec;

use x86_64::VirtAddr;

pub fn write(buf_addr: usize, buf_len: usize) -> usize {
    let mut buf = vec![0; buf_len];

    if let Err(_) = get_current_process().read().page_table.read(
        VirtAddr::new(buf_addr as u64),
        buf_len,
        &mut buf,
    ) {
        panic!("Read error at {:x}!", buf_addr);
    }

    framework::print!("{}", core::str::from_utf8(buf.as_slice()).unwrap());
    buf_len
}

pub fn show_cpu_id() -> usize {
    let id = framework::arch::apic::get_lapic_id();
    framework::print!("[{}]", id);
    // 在这里输出当前线程所在的CPU的lapic id
    id as usize
}

pub fn dump_hex_buffer(buf_addr: usize, buf_len: usize) -> usize {
    let mut buf = vec![0; buf_len];

    if let Err(_) = get_current_process().read().page_table.read(
        VirtAddr::new(buf_addr as u64),
        buf_len,
        &mut buf,
    ) {
        panic!("Read error at {:x}!", buf_addr);
    }

    for i in 0..buf_len {
        framework::serial_print!("{:02x} ", buf[i] as u8);
    }

    buf.len()
}
