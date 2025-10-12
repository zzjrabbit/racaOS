#![no_std]
#![deny(unsafe_code)]

use alloc::{format, string::String, sync::Arc, vec::Vec};
use component::{ComponentInitError, init_component};
use filesystem::add_block_device;
use ostd::mm::PAGE_SIZE;

mod device;
mod inode;
mod io;
mod manager;

extern crate alloc;

pub use device::{BlockDevice, BlockDeviceError, BlockDeviceType, BlockMetadata};
pub use io::{BlockIo, BlockOperation};

use crate::{inode::BlockInode, manager::DeviceManager};

pub const BLOCK_SIZE: usize = PAGE_SIZE;
pub const SECTOR_SIZE: usize = 512;

static MANAGER: DeviceManager = DeviceManager::new();

pub fn register_device(device: Arc<dyn BlockDevice>) {
    let device_type = device.metadata().device_type;

    MANAGER.register_device(format!("{device_type}"), device.clone());

    let device_type = device.metadata().device_type;
    let device = BlockInode::new(device);

    add_block_device(device, format!("{device_type}"), |id| {
        device_type.partition_name(id)
    });
}

pub fn get_device<S>(name: S) -> Option<Arc<dyn BlockDevice>>
where
    String: From<S>,
{
    MANAGER.get_device(name)
}

pub fn all_devices() -> Vec<(String, Arc<dyn BlockDevice>)> {
    MANAGER.all_devices()
}

#[init_component]
pub fn init() -> Result<(), ComponentInitError> {
    Ok(())
}
