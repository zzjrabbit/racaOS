use alloc::string::{String, ToString};
use core::fmt::Debug;
use core::sync::atomic::{AtomicU64, Ordering};
use downcast_rs::{DowncastSync, impl_downcast};
use spin::Mutex;

mod handle;
mod rights;

pub use handle::*;
pub use rights::*;

pub trait KernelObject: DowncastSync + Debug {
    fn id(&self) -> KoID;
    fn type_name(&self) -> &str;
    fn name(&self) -> String;
    fn set_name(&self, name: &str);
}
impl_downcast!(sync KernelObject);

pub type KoID = u64;

pub struct KObjectBase {
    pub id: KoID,
    inner: Mutex<KObjectBaseInner>,
}

#[derive(Default)]
struct KObjectBaseInner {
    name: String,
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
