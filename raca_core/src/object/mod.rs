mod handle;
mod rights;

pub use handle::*;
pub use rights::*;

use core::{
    fmt::Debug,
    sync::atomic::{AtomicU64, Ordering},
};

use alloc::{
    string::{String, ToString},
    sync::Arc,
};
use downcast_rs::{impl_downcast, DowncastSync};
use spin::Mutex;

use crate::error::{RcError, RcResult};

/// 内核对象公共接口x
pub trait KernelObject: DowncastSync + Debug {
    /// 获取对象 ID
    fn id(&self) -> KoID;
    /// 获取对象类型名
    fn type_name(&self) -> &str;
    /// 获取对象名称
    fn name(&self) -> String;
    /// 设置对象名称
    fn set_name(&self, name: &str);

    fn peer(&self) -> RcResult<Arc<dyn KernelObject>> {
        Err(RcError::NOT_SUPPORTED)
    }
    /// If the object is related to another (such as the other end of a channel, or the parent of
    /// a job), returns the KoID of that object, otherwise returns zero.
    fn related_koid(&self) -> KoID {
        0
    }
}
impl_downcast!(sync KernelObject);

/// 对象 ID 类型
pub type KoID = u64;

pub struct KObjectBase {
    /// 对象 ID
    pub id: KoID,
    inner: Mutex<KObjectBaseInner>,
}

/// `KObjectBase` 的内部可变部分
#[derive(Default)]
struct KObjectBaseInner {
    name: String,
}

impl Default for KObjectBase {
    /// 创建一个新 `KObjectBase`
    fn default() -> Self {
        KObjectBase {
            id: Self::new_koid(),
            inner: Default::default(),
        }
    }
}
impl KObjectBase {
    /// 生成一个唯一的 ID
    fn new_koid() -> KoID {
        static NEXT_KOID: AtomicU64 = AtomicU64::new(1024);
        NEXT_KOID.fetch_add(1, Ordering::SeqCst)
    }
    /// 获取对象名称
    pub fn name(&self) -> String {
        self.inner.lock().name.clone()
    }
    /// 设置对象名称
    pub fn set_name(&self, name: &str) {
        self.inner.lock().name = name.to_string()
    }
}
