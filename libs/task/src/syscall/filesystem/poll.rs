use alloc::{sync::Arc, vec::Vec};
use filesystem::{IoEvent, Poller};
use ostd::{Pod, mm::Vaddr, task::Task};
use spin::RwLock;

use crate::{AsThread, Process, UserThreadData, syscall::SyscallResult};

pub fn poll(fds: Vaddr, nfds: u32, _time_out: i32) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();
    let vmar = data.memory_info().vmar();

    if Process::current().id() == 1 {
        return Ok(0);
    }

    let mut read_addr = fds;
    let pollers = RwLock::new(Vec::<Arc<Poller>>::with_capacity(nfds as usize));
    let mut c_poll_fds = Vec::with_capacity(nfds as usize);

    for _ in 0..nfds {
        let c_poll_fd = vmar.read_val::<CPollFd>(read_addr)?;
        c_poll_fds.push(c_poll_fd);
        read_addr += core::mem::size_of::<CPollFd>();
    }

    let result = data.wait_with_waker(
        || {
            let good = pollers
                .read()
                .iter()
                .filter(|poller| poller.finished())
                .count();
            if good == 0 { None } else { Some(good) }
        },
        |waker| {
            for c_poll_fd in &c_poll_fds {
                let poller = Poller::new(
                    IoEvent::from_bits_truncate(c_poll_fd.events as u32),
                    waker.clone(),
                );
                pollers.write().push(poller.clone());
                let _ = data.fs_info().with_file(c_poll_fd.fd, |_, _, _, file| {
                    file.register_poller(poller.clone());
                });
            }
        },
    )?;

    let mut write_addr = fds;
    for (poller, c_poll_fd) in pollers.read().iter().zip(c_poll_fds.iter()) {
        let mut c_poll_fd = *c_poll_fd;

        c_poll_fd.revents = poller.event().bits() as i16;

        vmar.write_val(write_addr, &c_poll_fd)?;
        write_addr += core::mem::size_of::<CPollFd>();
    }

    Ok(result as isize)
}

#[derive(Debug, Clone, Copy, Pod)]
#[repr(C)]
struct CPollFd {
    fd: i32,
    events: i16,
    revents: i16,
}
