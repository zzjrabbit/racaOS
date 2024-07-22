pub fn create_process(name: &str, binary: &[u8]) {
    const CREATE_PROCESS_SYSCALL_ID: u64 = 6;
    crate::syscall(
        CREATE_PROCESS_SYSCALL_ID,
        binary.as_ptr() as usize,
        binary.len(),
        name.as_ptr() as usize,
        name.len(),
        0,
    );
}
