pub fn show_cpu_id() -> usize {
    const SHOW_CPU_ID_SYSCALL_ID: u64 = 1;
    crate::syscall(SHOW_CPU_ID_SYSCALL_ID, 0, 0, 0, 0, 0)
}

pub fn dump_hex_buffer(buffer: &[u8]) {
    const DUMP_HEX_BUFFER_SYSCALL_ID: u64 = 5;
    crate::syscall(
        DUMP_HEX_BUFFER_SYSCALL_ID,
        buffer.as_ptr() as usize,
        buffer.len(),
        0,
        0,
        0,
    );
}
