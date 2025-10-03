#![no_std]
#![deny(unsafe_code)]

use alloc::{string::String, sync::Arc, vec::Vec};
use component::{ComponentInitError, init_component};
use ostd::mm::PAGE_SIZE;

mod device;
mod io;
mod manager;

extern crate alloc;

pub use device::{BlockDevice, BlockDeviceError, BlockMetadata};
pub use io::{BlockIo, BlockOperation};

use crate::manager::DeviceManager;

pub const BLOCK_SIZE: usize = PAGE_SIZE;
pub const SECTOR_SIZE: usize = 512;

static MANAGER: DeviceManager = DeviceManager::new();

pub fn register_device<S>(name: S, device: Arc<dyn BlockDevice>)
where
    String: From<S>,
{
    MANAGER.register_device(name, device);
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
