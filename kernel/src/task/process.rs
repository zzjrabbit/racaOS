use alloc::{collections::btree_map::BTreeMap, sync::Arc};

use crate::{
    error::{RcError, RcResult},
    object::{Handle, HandleValue, KernelObject, Rights},
};

crate::kernel_object! {
    pub struct Process {
        inner: spin::Mutex<ProcessInner> = spin::Mutex::new(ProcessInner::new()),
    }

    fn new() {}
}

impl Process {
    pub fn add_handle(&self, handle: Handle) -> HandleValue {
        let mut inner = self.inner.lock();
        let value = (0 as HandleValue..)
            .find(|idx| !inner.handles.contains_key(idx))
            .unwrap();

        inner.handles.insert(value, handle);
        value
    }

    pub fn remove_handle(&self, handle_value: HandleValue) {
        self.inner.lock().handles.remove(&handle_value);
    }

    pub fn get_object_with_rights<T: KernelObject>(
        &self,
        handle_value: HandleValue,
        desired_rights: Rights,
    ) -> RcResult<Arc<T>> {
        let handle = self
            .inner
            .lock()
            .handles
            .get(&handle_value)
            .ok_or(RcError::BadHandle)?
            .clone();
        // check type before rights
        let object = handle
            .object
            .downcast_arc::<T>()
            .map_err(|_| RcError::WrongType)?;
        if !handle.rights.contains(desired_rights) {
            return Err(RcError::AccessDenied);
        }
        Ok(object)
    }
}

struct ProcessInner {
    handles: BTreeMap<HandleValue, Handle>,
}

impl ProcessInner {
    pub fn new() -> Self {
        Self {
            handles: BTreeMap::new(),
        }
    }
}
