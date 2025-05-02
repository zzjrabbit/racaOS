use alloc::sync::Arc;

use super::{KernelObject, Rights};

#[derive(Clone)]
pub struct Handle {
    pub object: Arc<dyn KernelObject>,
    pub rights: Rights,
}

pub type HandleValue = u32;

impl Handle {
    pub fn new(object: Arc<dyn KernelObject>, rights: Rights) -> Self {
        Handle { object, rights }
    }
}

pub const INVALID_HANDLE: u32 = u32::MAX;
