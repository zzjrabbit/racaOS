#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Gid(u32);

impl Gid {
    pub const INVALID: Self = Self(u32::MAX);
    pub const OVERFLOW: Self = Self(65534);

    pub const fn new(gid: u32) -> Self {
        Self(gid)
    }

    pub const fn new_root() -> Self {
        Self(0)
    }

    pub const fn is_root(&self) -> bool {
        self.0 == Self::new_root().0
    }
}

impl From<u32> for Gid {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl From<Gid> for u32 {
    fn from(value: Gid) -> Self {
        value.0
    }
}
