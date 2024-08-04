use crate::print;

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

    pub fn run(&self) -> usize {
        const CREATE_PROCESS_SYSCALL_ID: u64 = 6;
        crate::syscall(
            CREATE_PROCESS_SYSCALL_ID,
            self as *const Self as usize,
            0,
            0,
            0,
            0,
        )
    }
}

pub fn exit(code: usize) -> ! {
    print!("OK");
    const EXIT_SYSCALL_ID: u64 = 21;
    crate::syscall(EXIT_SYSCALL_ID, code, 0, 0, 0, 0);
    print!("OK");

    loop {} // Never return
}

pub fn wait() -> usize {
    start_wait_for_signal(0);
    while !has_signal(0) {}
    let signal = get_signal(0).unwrap();
    done_signal(signal.ty);
    signal.data[0] as usize
}

#[derive(Debug, Clone, Copy)]
pub struct Signal {
    pub ty: usize,
    pub data: [u64;8],
}

pub fn done_signal(ty: usize) {
    const DONE_SIGNAL_SYSCALL_ID: u64 = 22;
    crate::syscall(DONE_SIGNAL_SYSCALL_ID, ty, 0, 0, 0, 0);
}

pub fn has_signal(ty: usize) -> bool {
    const HAS_SIGNAL_SYSCALL_ID: u64 = 23;
    crate::syscall(HAS_SIGNAL_SYSCALL_ID, ty, 0, 0, 0, 0) as usize > 0
}

pub fn start_wait_for_signal(ty: usize) {
    const START_WAIT_FOR_SIGNAL_SYSCALL_ID: u64 = 24;
    crate::syscall(START_WAIT_FOR_SIGNAL_SYSCALL_ID, ty, 0, 0, 0, 0);
}

pub fn get_signal(ty: usize) -> Option<Signal> {
    const GET_SIGNAL_SYSCALL_ID: u64 = 25;
    let signal_ptr = crate::syscall(GET_SIGNAL_SYSCALL_ID, ty, 0, 0, 0, 0) as usize;
    if signal_ptr == 0 {
        None
    } else {
        Some(unsafe { *(signal_ptr as *const Signal) })
    }
}
