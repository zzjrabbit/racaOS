use alloc::vec::Vec;
use filesystem::{FileDescriptor, IoEvent};
use ostd::{Pod, mm::Vaddr, sync::RwArc, task::Task};

use crate::{AsThread, Process, UserThreadData, syscall::SyscallResult};

pub fn poll(fds: Vaddr, nfds: u32, _time_out: i32) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();
    let vmar = data.memory_info().vmar();

    if Process::current().id() == 1 {
        return Ok(0);
    }

    let mut read_addr = fds;
    let mut poll_fds = Vec::with_capacity(nfds as usize);

    for _ in 0..nfds {
        let c_poll_fd = vmar.read_val::<CPollFd>(read_addr)?;
        poll_fds.push(PollFd::from(c_poll_fd));
        read_addr += core::mem::size_of::<CPollFd>();
    }

    let result = data.wait_with_waker(
        || {
            let good = poll_fds
                .iter()
                .filter(|poll_fd| poll_fd.revents().get_cloned().contains(poll_fd.events()))
                .count();
            if good == 0 { None } else { Some(good) }
        },
        |waker| {
            for poll_fd in &poll_fds {
                if let Some(fd) = poll_fd.fd() {
                    let _ = data.fs_info().with_file(fd, |_, _, _, file| {
                        file.register_waker(
                            poll_fd.events,
                            poll_fd.revents().clone(),
                            waker.clone(),
                        );
                    });
                }
            }
        },
    )?;

    let mut write_addr = fds;
    for poll_fd in &poll_fds {
        let c_poll_fd = CPollFd {
            fd: poll_fd.fd().unwrap_or(-1),
            events: poll_fd.events().bits() as i16,
            revents: poll_fd.revents().get_cloned().bits() as i16,
        };
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

#[derive(Clone)]
pub struct PollFd {
    fd: Option<FileDescriptor>,
    events: IoEvent,
    revents: RwArc<IoEvent>,
}

impl PollFd {
    pub fn fd(&self) -> Option<FileDescriptor> {
        self.fd
    }

    pub fn events(&self) -> IoEvent {
        self.events
    }

    pub fn revents(&self) -> &RwArc<IoEvent> {
        &self.revents
    }
}

impl From<CPollFd> for PollFd {
    fn from(raw: CPollFd) -> Self {
        let fd = if raw.fd >= 0 {
            Some(raw.fd as FileDescriptor)
        } else {
            None
        };
        let events = IoEvent::from_bits_truncate(raw.events as _);
        let revents = RwArc::new(IoEvent::empty());
        Self {
            fd,
            events,
            revents,
        }
    }
}

impl From<PollFd> for CPollFd {
    fn from(raw: PollFd) -> Self {
        let fd = raw.fd().unwrap_or(-1);
        let events = raw.events().bits() as i16;
        let revents = raw.revents().get_cloned().bits() as i16;
        Self {
            fd,
            events,
            revents,
        }
    }
}
