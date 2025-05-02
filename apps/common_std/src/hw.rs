use crate::{RcError, RcResult, syscall};

pub struct IoPort(u32);

impl IoPort {
    pub fn new(port: u16) -> RcResult<Self> {
        let mut handle = 0;
        syscall!(18, port, 0b11, &mut handle)?;
        Ok(Self(handle))
    }

    pub fn new_readonly(port: u16) -> RcResult<Self> {
        let mut handle = 0;
        syscall!(18, port, 0b01, &mut handle)?;
        Ok(Self(handle))
    }

    pub fn new_writeonly(port: u16) -> RcResult<Self> {
        let mut handle = 0;
        syscall!(18, port, 0b10, &mut handle)?;
        Ok(Self(handle))
    }
}

impl IoPort {
    pub fn read<T: PortRWType>(&self) -> RcResult<T> {
        if T::size() > 4 {
            return Err(RcError::InvalidArguments);
        }

        Ok(T::from(syscall!(19, self.0, T::size())?))
    }

    pub fn write<T: PortRWType>(&self, value: &T) -> RcResult<()> {
        if T::size() > 4 {
            return Err(RcError::InvalidArguments);
        }

        syscall!(20, self.0, value, T::size())?;
        Ok(())
    }
}

pub trait PortRWType {
    fn size() -> usize;
    fn from(value: usize) -> Self;
}

impl PortRWType for u8 {
    fn size() -> usize {
        1
    }
    fn from(value: usize) -> Self {
        value as u8
    }
}

impl PortRWType for u16 {
    fn size() -> usize {
        2
    }
    fn from(value: usize) -> Self {
        value as u16
    }
}

impl PortRWType for u32 {
    fn size() -> usize {
        4
    }
    fn from(value: usize) -> Self {
        value as u32
    }
}

pub struct Irq(u32);

impl Irq {
    pub fn new(irq: u8) -> RcResult<Self> {
        let mut handle = 0;
        syscall!(21, irq as usize, &mut handle)?;
        Ok(Self(handle))
    }

    pub fn wait(&self) -> RcResult<()> {
        syscall!(22, self.0)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct IntFrame {
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rbx: usize,

    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,

    pub int_num: usize,
    pub error_code: usize,

    // Pushed by CPU
    pub rip: usize,
    pub cs: usize,
    pub rflags: usize,

    // Pushed by CPU when Ring3->0
    pub rsp: usize,
    pub ss: usize,
}
