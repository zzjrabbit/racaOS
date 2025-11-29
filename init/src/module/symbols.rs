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
            fn rust_eh_personality rust_eh_personality;
            fn memcpy memcpy;
            fn memset memset;
            fn bcmp bcmp;
            fn memcmp memcmp;
        )
        .into(),
    )
});

fn print_str(msg: &str) {
    print!("{}", msg);
}

fn rust_eh_personality() {
    log::info!("call rust_eh_personality");
}

extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) {
    log::info!("call memcpy");
    unsafe {
        core::ptr::copy_nonoverlapping(src, dest, n);
    }
}

extern "C" fn memset(dest: *mut u8, c: u8, n: usize) {
    log::info!("call memset");
    unsafe {
        core::ptr::write_bytes(dest, c, n);
    }
}

extern "C" fn bcmp(lhs: *const u8, rhs: *const u8, n: usize) -> i32 {
    log::info!("call bcmp");
    for i in 0..n {
        if unsafe { *lhs.add(i) } != unsafe { *rhs.add(i) } {
            return unsafe { *lhs.add(i) as i32 - *rhs.add(i) as i32 };
        }
    }
    0
}

extern "C" fn memcmp(lhs: *const u8, rhs: *const u8, n: usize) -> i32 {
    log::info!("call memcmp");
    for i in 0..n {
        if unsafe { *lhs.add(i) } != unsafe { *rhs.add(i) } {
            return unsafe { *lhs.add(i) as i32 - *rhs.add(i) as i32 };
        }
    }
    0
}
