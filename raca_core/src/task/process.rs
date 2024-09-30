use crate::{
    error::{RcError, RcResult},
    object::*,
};
use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use rc_object_derive::kernel_object;
use spin::Mutex;

/// 进程对象
#[kernel_object]
pub struct Process {
    inner: Mutex<ProcessInner>,
}

struct ProcessInner {
    handles: BTreeMap<HandleValue, Handle>,
}

pub type HandleValue = u32;

impl Process {
    /// 创建一个新的进程对象
    pub fn new() -> Arc<Self> {
        Arc::new(Process {
            base: KObjectBase::default(),
            inner: Mutex::new(ProcessInner {
                handles: BTreeMap::default(),
            }),
        })
    }

    pub fn add_handle(&self, handle: Handle) -> HandleValue {
        let mut inner = self.inner.lock();
        let value = (0 as HandleValue..)
            .find(|idx| !inner.handles.contains_key(idx))
            .unwrap();
        // 插入BTreeMap
        inner.handles.insert(value, handle);
        value
    }

    pub fn remove_handle(&self, handle_value: HandleValue) {
        self.inner.lock().handles.remove(&handle_value);
    }

    /// 根据句柄值查找内核对象，并检查权限
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
            .ok_or(RcError::BAD_HANDLE)?
            .clone();
        // check type before rights
        let object = handle
            .object
            .downcast_arc::<T>()
            .map_err(|_| RcError::WRONG_TYPE)?;
        if !handle.rights.contains(desired_rights) {
            return Err(RcError::ACCESS_DENIED);
        }
        Ok(object)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn new_proc() {
        let proc = Process::new();
        assert_eq!(proc.type_name(), "Process");
        assert_eq!(proc.name(), "");
        proc.set_name("proc1");
        assert_eq!(proc.name(), "proc1");
        assert_eq!(
            format!("{:?}", proc),
            format!("Process({}, \"proc1\")", proc.id())
        );

        let obj: Arc<dyn KernelObject> = proc;
        assert_eq!(obj.type_name(), "Process");
        assert_eq!(obj.name(), "proc1");
        obj.set_name("proc2");
        assert_eq!(obj.name(), "proc2");
        assert_eq!(
            format!("{:?}", obj),
            format!("Process({}, \"proc2\")", obj.id())
        );
    }

    #[test]
    fn proc_handle() {
        let proc = Process::new();
        let handle = Handle::new(proc.clone(), Rights::DEFAULT_PROCESS);
        let handle_value = proc.add_handle(handle);

        let object1: Arc<Process> = proc
            .get_object_with_rights(handle_value, Rights::DEFAULT_PROCESS)
            .expect("failed to get object");
        assert!(Arc::ptr_eq(&object1, &proc));

        proc.remove_handle(handle_value);
    }
}
