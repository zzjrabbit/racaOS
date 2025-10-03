use alloc::{collections::btree_map::BTreeMap, string::String, sync::Arc, vec::Vec};
use ostd::sync::RwLock;

use crate::BlockDevice;

pub struct DeviceManager {
    devices: RwLock<BTreeMap<String, Arc<dyn BlockDevice>>>,
}

impl DeviceManager {
    pub const fn new() -> Self {
        Self {
            devices: RwLock::new(BTreeMap::new()),
        }
    }
}

impl DeviceManager {
    pub fn register_device<S>(&self, name: S, device: Arc<dyn BlockDevice>)
    where
        String: From<S>,
    {
        self.devices.write().insert(name.into(), device);
    }

    pub fn get_device<S>(&self, name: S) -> Option<Arc<dyn BlockDevice>>
    where
        String: From<S>,
    {
        let name: String = name.into();
        self.devices.read().get(&name).cloned()
    }

    pub fn all_devices(&self) -> Vec<(String, Arc<dyn BlockDevice>)> {
        self.devices
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}
