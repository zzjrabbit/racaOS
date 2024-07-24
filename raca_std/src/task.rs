#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Process {
    binary_addr: usize,
    binary_len: usize,
    name_addr: usize,
    name_len: usize,
    stdin: usize,
    stdout: usize,
}

impl Process {
    pub fn new(binary: &[u8], name: &str, stdin: usize, stdout: usize) -> Self {
        Self {
            binary_addr: binary.as_ptr() as usize,
            binary_len: binary.len(),
            name_addr: name.as_ptr() as usize,
            name_len: name.len(),
            stdin,
            stdout,
        }
    }

    pub fn run(&self) {
        const CREATE_PROCESS_SYSCALL_ID: u64 = 6;
        crate::syscall(
            CREATE_PROCESS_SYSCALL_ID,
            self as *const Self as usize,
            0,
            0,
            0,
            0,
        );
    }
}
