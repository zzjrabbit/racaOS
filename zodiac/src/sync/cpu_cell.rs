use alloc::collections::btree_map::BTreeMap;
use spin::Mutex;

use crate::arch::{cpu_num, current_cpu};

pub struct CpuCell<T> {
    values: BTreeMap<u64, Mutex<T>>,
}

impl<T: Default> CpuCell<T> {
    pub fn new() -> Self {
        let mut values = BTreeMap::new();
        for cpu_id in 0..cpu_num() {
            values.insert(cpu_id, Mutex::new(T::default()));
        }
        CpuCell { values }
    }
}

impl<T: Clone> CpuCell<T> {
    pub fn new_with(default: T) -> Self {
        let mut values = BTreeMap::new();
        for cpu_id in 0..cpu_num() {
            values.insert(cpu_id, Mutex::new(default.clone()));
        }
        CpuCell { values }
    }
}

impl<T: Clone> CpuCell<T> {
    pub fn get_current(&self) -> T {
        let cpu_id = current_cpu();
        let value = self.values.get(&cpu_id).unwrap();
        value.lock().clone()
    }

    pub fn set_current(&self, new: T) {
        let cpu_id = current_cpu();
        let old = self.values.get(&cpu_id).unwrap();
        *old.lock() = new;
    }

    pub fn get_cpu(&self, cpu_id: u64) -> T {
        let value = self.values.get(&cpu_id).unwrap();
        value.lock().clone()
    }

    pub fn set_cpu(&self, cpu_id: u64, new: T) {
        let value = self.values.get(&cpu_id).unwrap();
        *value.lock() = new;
    }
}

impl<T> CpuCell<T> {
    pub fn with_current<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let cpu_id = current_cpu();
        let value = self.values.get(&cpu_id).unwrap();
        f(&value.lock())
    }

    pub fn with_current_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let cpu_id = current_cpu();
        let value = self.values.get(&cpu_id).unwrap();
        f(&mut value.lock())
    }

    pub fn with<R>(&self, cpu_id: u64, f: impl FnOnce(&T) -> R) -> R {
        let value = self.values.get(&cpu_id).unwrap();
        f(&value.lock())
    }

    pub fn with_mut<R>(&self, cpu_id: u64, f: impl FnOnce(&mut T) -> R) -> R {
        let value = self.values.get(&cpu_id).unwrap();
        f(&mut value.lock())
    }
}
