#![allow(dead_code)]
#![allow(unused_imports)]

mod ext2;
mod ext4;

use alloc::sync::Arc;
use lwext4_rust::{
    bindings::{EINVAL, SEEK_CUR, SEEK_END, SEEK_SET},
    KernelDevOp,
};

use crate::{probe::register_probe, File};

pub fn init() {
    //register_probe(ext4::parse_ext4_fs);
}

struct Lwext4Disk {
    inner: Arc<File>,
    offset: u64,
}

impl Lwext4Disk {
    fn new(inner: Arc<File>) -> Self {
        Self { inner, offset: 0 }
    }
}

impl KernelDevOp for Lwext4Disk {
    type DevType = Lwext4Disk;

    fn read(dev: &mut Self::DevType, buf: &mut [u8]) -> Result<usize, i32> {
        Ok(dev.inner.read_at(dev.offset, buf))
    }

    fn write(dev: &mut Self::DevType, buf: &[u8]) -> Result<usize, i32> {
        let r = dev.inner.write_at(dev.offset, buf);
        Ok(r)
    }

    fn flush(_dev: &mut Self::DevType) -> Result<usize, i32>
    where
        Self: Sized,
    {
        Ok(0)
    }

    fn seek(dev: &mut Self::DevType, off: i64, whence: i32) -> Result<i64, i32> {
        let new_offset = match whence as u32 {
            SEEK_CUR => dev.offset.checked_add_signed(off).ok_or(-(EINVAL as i32))?,
            SEEK_SET => off as u64,
            SEEK_END => {
                let len = dev.inner.len();
                len.checked_add_signed(off).ok_or(-(EINVAL as i32))?
            }
            _ => Err(-(EINVAL as i32))?,
        };
        dev.offset = new_offset;
        Ok(new_offset as i64)
    }
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
unsafe extern "C" fn __stack_chk_fail() {}
