use alloc::sync::Arc;
use task::ProcessCreationArguments;

use crate::{
    error::RcError,
    object::{HandleValue, Rights, Signal},
    task::{
        job_policy::BasicPolicy,
        process::{Process, ProcessInfo},
        scheduler::SCHEDULER,
    },
};

mod debug;
mod fb;
mod handle;
mod hw;
mod ipc;
mod memory;
mod object;
mod signal;
mod task;

#[allow(unused_variables)]
pub extern "C" fn syscall_matcher(
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> isize {
    let syscall_index: usize;
    unsafe { core::arch::asm!("mov {0}, rax", out(reg) syscall_index) };

    //log::info!("syscall{} {} {} {} {} {} {}", syscall_index, arg1, arg2, arg3, arg4, arg5, arg6);

    let ret = match syscall_index {
        0 => debug::debug(arg1, arg2),
        1 => task::process_create(arg1, arg2, arg3 as *const ProcessCreationArguments, arg4),
        2 => memory::allocate_child(arg1 as HandleValue, arg2, arg3),
        3 => memory::get_vm_start_address(arg1),
        4 => memory::create_physical_memory(arg1, arg2),
        5 => memory::get_pm_start_address(arg1),
        6 => memory::map(arg1, arg2, arg3),
        7 => memory::unmap(arg1),
        8 => memory::create_child(arg1, arg2, arg3, arg4),
        9 => memory::root_virtual_memory(arg1),
        10 => fb::fb_width(arg1),
        11 => fb::fb_height(arg1),
        12 => fb::fb_ptr(arg1),
        13 => fb::get_fb(arg1),
        14 => handle::duplicate_handle(
            arg1 as u32,
            Rights::from_bits_truncate(arg2 as u32),
            arg3 as *mut HandleValue,
        ),
        15 => ipc::channel_create(arg1 as *mut HandleValue, arg2 as *mut HandleValue),
        16 => ipc::channel_read(
            arg1 as HandleValue,
            arg2 as *mut u8,
            arg3 as *mut usize,
            arg4 as *mut HandleValue,
            arg5 as *mut usize,
        ),
        17 => ipc::channel_write(
            arg1 as HandleValue,
            arg2 as *const u8,
            arg3 as usize,
            arg4 as *const HandleValue,
            arg5 as usize,
        ),
        18 => hw::create_io_port(arg1 as u16, arg2, arg3 as *mut HandleValue),
        19 => hw::io_port_read(arg1 as HandleValue, arg2),
        20 => hw::io_port_write(arg1 as HandleValue, arg2, arg3),
        21 => hw::irq_register(arg1 as u8, arg2 as *mut HandleValue),
        22 => signal::event_pair_create(arg1 as *mut HandleValue, arg2 as *mut HandleValue),
        23 => task::job_create(arg1 as HandleValue, arg2 as *mut HandleValue),
        24 => task::job_set_policy(arg1 as HandleValue, arg2 as *const BasicPolicy, arg3),
        25 => object::set_signal(arg1 as HandleValue, Signal::from_bits_truncate(arg2 as u32)),
        26 => task::exit(arg1 as i64),
        27 => object::clear_signal(arg1 as HandleValue, Signal::from_bits_truncate(arg2 as u32)),
        28 => signal::wait_for_signal(
            arg1 as *const HandleValue,
            arg2,
            Signal::from_bits_truncate(arg3 as u32),
            arg4 as *mut HandleValue,
        ),
        29 => task::get_process_info(arg1 as HandleValue, arg2 as *mut ProcessInfo),
        30 => handle::close_handle(arg1 as HandleValue),
        31 => task::spawn_thread(
            arg1 as *const u8,
            arg2,
            arg3,
            arg4 as HandleValue,
            arg5 as *mut HandleValue,
        ),
        32 => task::exit_thread(),
        33 => task::job_kill(arg1 as HandleValue),
        34 => task::process_kill(arg1 as HandleValue),
        35 => task::kill_thread(arg1 as HandleValue),
        36 => task::increase_nice(arg1),
        _ => Err(RcError::InvalidSyscall),
    };

    //log::info!("done syscall with {:?}", ret);

    match ret {
        Ok(ret) => ret as isize,
        Err(error) => error as isize,
    }
}

pub fn current_process() -> Arc<Process> {
    SCHEDULER
        .current_thread()
        .upgrade()
        .unwrap()
        .process()
        .upgrade()
        .unwrap()
}
