use ostd::Pod;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Pod)]
#[repr(C)]
pub struct Uid(u32);

impl Uid {
    pub const INVALID: Self = Self(u32::MAX);
    pub const OVERFLOW: Self = Self(65534);

    pub const fn new(uid: u32) -> Self {
        Self(uid)
    }

    pub const fn new_root() -> Self {
        Self(0)
    }

    pub const fn is_root(&self) -> bool {
        self.0 == Self::new_root().0
    }
}

impl From<u32> for Uid {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl From<Uid> for u32 {
    fn from(value: Uid) -> Self {
        value.0
    }
}
