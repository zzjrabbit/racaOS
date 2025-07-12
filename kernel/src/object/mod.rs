use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::sync::atomic::{AtomicU64, Ordering};
use downcast_rs::{DowncastSync, impl_downcast};
use spin::Mutex;

mod handle;
mod rights;
mod signal;

pub use handle::*;
pub use rights::*;
pub use signal::*;

use crate::error::{RcError, RcResult};

pub trait KernelObject: DowncastSync + Debug {
    fn id(&self) -> KoID;
    fn type_name(&self) -> &str;
    fn name(&self) -> String;
    fn set_name(&self, name: &str);

    fn set_signal(&self, signal: Signal);
    fn clear_signal(&self, signal: Signal);
    fn signal_present(&self, signal: Signal) -> bool;
    fn add_signal_callback(&self, callback: SignalHandler);

    fn peer(&self) -> RcResult<Arc<dyn KernelObject>> {
        Err(RcError::NotSupported)
    }
    fn related_koid(&self) -> KoID {
        0
    }
    fn get_child(&self, _id: KoID) -> RcResult<Arc<dyn KernelObject>> {
        Err(RcError::WrongType)
    }
}
impl_downcast!(sync KernelObject);

pub type KoID = u64;

pub struct KObjectBase {
    pub id: KoID,
    inner: Mutex<KObjectBaseInner>,
}

pub type SignalHandler = Box<dyn Fn(Signal) -> bool + Send>;

#[derive(Default)]
struct KObjectBaseInner {
    name: String,
    signal: Signal,
    signal_callbacks: Vec<SignalHandler>,
}

impl KObjectBaseInner {
    pub fn with_name(name: String) -> Self {
        Self {
            name,
            signal: Signal::empty(),
            signal_callbacks: Vec::new(),
        }
    }
}

impl Default for KObjectBase {
    fn default() -> Self {
        KObjectBase {
            id: Self::new_koid(),
            inner: Default::default(),
        }
    }
}

impl KObjectBase {
    pub fn with_name(name: &str) -> Self {
        Self {
            id: Self::new_koid(),
            inner: Mutex::new(KObjectBaseInner::with_name(name.to_string())),
        }
    }
}

impl KObjectBase {
    fn new_koid() -> KoID {
        static NEXT_KOID: AtomicU64 = AtomicU64::new(1);
        NEXT_KOID.fetch_add(1, Ordering::Relaxed)
    }

    pub fn name(&self) -> String {
        self.inner.lock().name.clone()
    }

    pub fn set_name(&self, name: &str) {
        self.inner.lock().name = name.to_string();
    }

    pub fn set_signal(&self, signal: Signal) {
        if !self.signal_present(signal) {
            let mut inner = self.inner.lock();

            inner.signal.insert(signal);

            let signal = inner.signal;
            inner.signal_callbacks.retain(|f| !f(signal));
        }
    }

    pub fn clear_signal(&self, signal: Signal) {
        self.inner.lock().signal.remove(signal);
    }

    pub fn signal_present(&self, signal: Signal) -> bool {
        self.inner.lock().signal.contains(signal)
    }

    pub fn add_signal_callback(&self, callback: SignalHandler) {
        let mut inner = self.inner.lock();

        if !callback(inner.signal) {
            inner.signal_callbacks.push(callback);
        }
    }
}

#[macro_export]
macro_rules! kernel_object {
    {$($vis:tt)? struct $name: ident {
        $($field: ident: $field_type: ty = $field_new: expr),* $(,)?
    }

    fn new($($arg: ident: $arg_type: ty),* $(,)?) {
    }

    $($fn: tt)*

    } => {
        $($vis)? struct $name {
            base: $crate::object::KObjectBase,
            $($field: $field_type),*
        }

        impl $name {
            pub fn new($($arg: $arg_type),*) -> alloc::sync::Arc<Self> {
                alloc::sync::Arc::new(Self {
                    base: Default::default(),
                    $($field: $field_new),*
                })
            }
        }

        impl $crate::object::KernelObject for $name {
            fn id(&self) -> $crate::object::KoID {
                self.base.id
            }

            fn type_name(&self) -> &str {
                stringify!($name)
            }

            fn name(&self) -> alloc::string::String {
                self.base.name()
            }

            fn set_name(&self, name: &str){
                self.base.set_name(name)
            }

            fn set_signal(&self, signal: $crate::object::Signal) {
                self.base.set_signal(signal)
            }

            fn clear_signal(&self, signal: $crate::object::Signal) {
                self.base.clear_signal(signal)
            }

            fn signal_present(&self, signal: $crate::object::Signal) -> bool {
                self.base.signal_present(signal)
            }

            fn add_signal_callback(
                &self,
                callback: $crate::object::SignalHandler
            ) {
                self.base.add_signal_callback(callback);
            }

            $($fn)*
        }

        impl core::fmt::Debug for $name {
            fn fmt(
                &self,
                f: &mut core::fmt::Formatter<'_>,
            ) -> core::result::Result<(), core::fmt::Error> {

                f.debug_tuple(&stringify!($name))
                    .field(&$crate::object::KernelObject::id(self))
                    .field(&$crate::object::KernelObject::name(self))
                    .finish()
            }
        }
    };

    {$($vis:tt)? struct $name: ident {
        $($field: ident: $field_type: ty),* $(,)?
    }

    $($fn: tt)*

    } => {
        $($vis)? struct $name {
            base: $crate::object::KObjectBase,
            $($field: $field_type),*
        }

        impl $crate::object::KernelObject for $name {
            fn id(&self) -> $crate::object::KoID {
                self.base.id
            }

            fn type_name(&self) -> &str {
                stringify!($name)
            }

            fn name(&self) -> alloc::string::String {
                self.base.name()
            }

            fn set_name(&self, name: &str){
                self.base.set_name(name)
            }

            fn set_signal(&self, signal: $crate::object::Signal) {
                self.base.set_signal(signal)
            }

            fn clear_signal(&self, signal: $crate::object::Signal) {
                self.base.clear_signal(signal)
            }

            fn signal_present(&self, signal: $crate::object::Signal) -> bool {
                self.base.signal_present(signal)
            }

            fn add_signal_callback(&self, callback: $crate::object::SignalHandler) {
                self.base.add_signal_callback(callback);
            }

            $($fn)*
        }

        impl core::fmt::Debug for $name {
            fn fmt(
                &self,
                f: &mut core::fmt::Formatter<'_>,
            ) -> core::result::Result<(), core::fmt::Error> {

                f.debug_tuple(&stringify!($name))
                    .field(&$crate::object::KernelObject::id(self))
                    .field(&$crate::object::KernelObject::name(self))
                    .finish()
            }
        }
    };
}

kernel_object! {
    pub struct Dummy {}

    fn new() {
    }
}
