use alloc::{collections::btree_map::BTreeMap, string::String};
use spin::{Lazy, Mutex};
use zodiac::{mem::VirtualAddress, print};

use crate::panic_handler;

macro_rules! symbols {
    ($(fn $name: ident $func: ident);* $(;)?) => {
        [$((stringify!($name).into(), VirtualAddress::from($func as *const () as usize))),*]
    };
}

pub(super) static SYMBOLS: Lazy<Mutex<BTreeMap<String, VirtualAddress>>> = Lazy::new(|| {
    Mutex::new(
        symbols!(
            fn print_str print_str;
            fn kernel_panic_handler panic_handler;
        )
        .into(),
    )
});

fn print_str(msg: &str) {
    print!("{}", msg);
}
