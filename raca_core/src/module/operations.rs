#![allow(improper_ctypes_definitions)]

use alloc::{collections::btree_map::BTreeMap, string::String};
use spin::Lazy;

pub static KERNEL_SYMBOL_TABLE: Lazy<BTreeMap<String, u64>> = Lazy::new(||{
    let mut symbol_table = BTreeMap::new();
    symbol_table.insert("print".into(), print as u64);
    symbol_table
});

pub extern "C" fn print(msg: &str) -> usize {
    crate::print!("{}", msg);
    0
}
