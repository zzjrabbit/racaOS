#![no_std]
#![deny(unsafe_code)]

use alloc::{boxed::Box, format, string::String, sync::Arc, vec::Vec};
use component::{ComponentInitError, init_component};
use ostd::{mm::PAGE_SIZE, sync::RwLock};

mod device;
mod io;
mod manager;

extern crate alloc;

pub use device::{BlockDevice, BlockDeviceError, BlockDeviceType, BlockMetadata};
pub use io::{BlockIo, BlockOperation};

use crate::manager::DeviceManager;

pub const BLOCK_SIZE: usize = PAGE_SIZE;
pub const SECTOR_SIZE: usize = 512;

static MANAGER: DeviceManager = DeviceManager::new();

type Callback = Box<dyn Fn(Arc<dyn BlockDevice>) + Sync + Send>;

static REGISTER_CALL_BACKS: RwLock<Vec<Callback>> = RwLock::new(Vec::new());

pub fn register_callback(callback: Callback) {
    REGISTER_CALL_BACKS.write().push(callback);
}

pub fn register_device(device: Arc<dyn BlockDevice>) {
    let device_type = device.metadata().device_type;

    MANAGER.register_device(format!("{device_type}"), device.clone());

    let callbacks = REGISTER_CALL_BACKS.read();
    for callback in callbacks.iter() {
        callback(device.clone());
    }
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
