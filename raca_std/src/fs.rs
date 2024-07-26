use core::fmt;

use crate::println;

#[repr(C)]
pub enum OpenMode {
    Read = 0,
    Write = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileDescriptor(pub usize, bool);

impl FileDescriptor {
    pub fn open(path: &str, open_mode: OpenMode) -> Result<Self, ()> {
        const OPEN_SYSCALL_ID: u64 = 2;
        let fd = crate::syscall(
            OPEN_SYSCALL_ID,
            path.as_ptr() as usize,
            path.len(),
            open_mode as usize,
            0,
            0,
        );
        if fd == 0 {
            Err(())
        } else {
            Ok(Self(fd, false))
        }
    }

    /// this opens a pipe, the first FileDescriptor is the read side, and the next one is the write side.
    /// You can use one of them as stdin or stdout stream for the sub process.
    pub fn open_pipe() -> Result<(Self, Self), ()> {
        const OPEN_SYSCALL_ID: u64 = 12;
        let mut buf = [0usize; 2];
        let code = crate::syscall(OPEN_SYSCALL_ID, buf.as_mut_ptr() as usize, 0, 0, 0, 0);
        println!("{:?}", buf);
        if code == 0 {
            Err(())
        } else {
            Ok((Self(buf[0], false), Self(buf[1], false)))
        }
    }

    pub fn stdin() -> Self {
        Self(0, false)
    }

    pub fn stdout() -> Self {
        Self(1, false)
    }

    pub fn read(&self, buffer: &mut [u8]) -> usize {
        assert_ne!(self.1, true, "This File Descriptor had been closed!");

        const READ_SYSCALL_ID: u64 = 4;
        crate::syscall(
            READ_SYSCALL_ID,
            self.0,
            buffer.as_ptr() as usize,
            buffer.len(),
            0,
            0,
        )
    }

    pub fn write(&self, buffer: &[u8]) -> usize {
        assert_ne!(self.1, true, "This File Descriptor had been closed!");

        const WRITE_SYSCALL_ID: u64 = 3;
        crate::syscall(
            WRITE_SYSCALL_ID,
            self.0,
            buffer.as_ptr() as usize,
            buffer.len(),
            0,
            0,
        )
    }

    pub fn seek(&self, offset: usize) -> usize {
        assert_ne!(self.1, true, "This File Descriptor had been closed!");

        const LSEEK_SYSCALL_ID: u64 = 10;
        crate::syscall(LSEEK_SYSCALL_ID, self.0, offset, 0, 0, 0)
    }

    pub fn size(&self) -> usize {
        assert_ne!(self.1, true, "This File Descriptor had been closed!");

        const FSIZE_SYSCALL_ID: u64 = 11;
        crate::syscall(FSIZE_SYSCALL_ID, self.0, 0, 0, 0, 0)
    }

    pub fn close(&mut self) {
        self.1 = true;

        const CLOSE_SYSCALL_ID: u64 = 9;
        crate::syscall(CLOSE_SYSCALL_ID, self.0, 0, 0, 0, 0);
    }
}

impl fmt::Write for FileDescriptor {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.write(s.as_bytes()) != s.as_bytes().len() {
            fmt::Result::Err(fmt::Error::default())
        } else {
            Ok(())
        }
    }
}
