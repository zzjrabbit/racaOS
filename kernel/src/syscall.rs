use zodiac::hal::trap::set_syscall_handler;

pub fn init() {
    set_syscall_handler(syscall_handler);
}

fn syscall_handler(syscall_id: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize, arg5: usize, arg6: usize) -> isize {
    log::info!("syscall{}({},{},{},{},{},{})", syscall_id, arg1, arg2, arg3, arg4, arg5, arg6);
    isize::MIN
}

