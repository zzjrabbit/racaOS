use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use ostd::{
    Pod,
    mm::{CachePolicy, PageFlags, PageProperty},
};

use {crate::MemoryInfo, memory::Vmar};

pub struct UserStack {
    vmar: Arc<Vmar>,
    stack_pointer: usize,
}

const USER_STACK_SIZE: usize = 8 * 1024 * 1024;

impl UserStack {
    pub fn new(memory_info: &MemoryInfo) -> Self {
        let stack_region = memory_info.allocate(USER_STACK_SIZE).unwrap();
        let vmar = memory_info.vmar();

        vmar.map(
            stack_region.start_address(),
            USER_STACK_SIZE,
            PageProperty::new_user(PageFlags::RW, CachePolicy::Writeback),
        )
        .unwrap();

        let stack_pointer = stack_region.end_address();

        Self {
            vmar,
            stack_pointer,
        }
    }
}

impl UserStack {
    pub fn push<T: Pod>(&mut self, value: T) -> usize {
        let value_ptr = self.stack_pointer - core::mem::size_of::<T>();
        self.vmar.write_val(value_ptr, &value).unwrap();
        self.stack_pointer = value_ptr;
        value_ptr
    }

    pub fn push_a_lot<T: Pod>(&mut self, values: &[T]) -> usize {
        for value in values.iter().rev() {
            self.push(*value);
        }
        self.stack_pointer
    }

    pub fn push_zero_until_aligned(&mut self, alignment: usize) {
        let remainder = self.stack_pointer % alignment;
        self.push_a_lot(&alloc::vec![0u8; remainder]);
    }

    pub fn stack_pointer(&self) -> usize {
        self.stack_pointer
    }
}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuxKey {
    Ignore = 1,
    ExecFd = 2,
    Phdr = 3,
    Phent = 4,
    Phnum = 5,
    PageSize = 6,
    Base = 7,
    Flags = 8,
    Entry = 9,
    NotElf = 10,
    Uid = 11,
    Euid = 12,
    Gid = 13,
    Egid = 14,
    Platform = 15,
    HwCap = 16,
    ClockTick = 17,

    Secure = 23,
    BasePlatform = 24,
    Random = 25,
    HwCap2 = 26,

    ExecFileName = 31,
    SysInfo = 32,
    SysInfoEhdr = 33,
}

impl AuxKey {
    pub fn as_u64(&self) -> u64 {
        *self as u64
    }
}

pub struct AuxVec {
    table: BTreeMap<AuxKey, u64>,
}

impl AuxVec {
    pub const fn new() -> Self {
        Self {
            table: BTreeMap::new(),
        }
    }
}

#[allow(dead_code)]
impl AuxVec {
    pub fn set(&mut self, key: AuxKey, value: u64) {
        self.table.insert(key, value);
    }

    pub fn get(&self, key: AuxKey) -> Option<u64> {
        self.table.get(&key).copied()
    }

    pub fn del(&mut self, key: AuxKey) {
        self.table.remove(&key);
    }
}

impl AuxVec {
    pub fn as_slice(&self) -> Vec<u64> {
        let mut vec = Vec::new();
        for (key, value) in self.table.iter() {
            vec.push(key.as_u64());
            vec.push(*value);
        }
        vec.push(0);
        vec.push(0);
        vec
    }
}
