use core::{alloc::Layout, slice::from_raw_parts};

use alloc::{format, string::ToString};
use rustc_demangle::demangle;
use spin::{Lazy, Mutex};
use stringmap::StringMap;
use zodiac::mem::VirtualAddress;

use crate::{module::BOOT_FILES, panic_handler};

pub(super) fn search_global_symbol(name: &str) -> Option<VirtualAddress> {
    let demangled = format!("{:#}", demangle(name)).trim().to_string();

    if let Some(addr) = KERNEL_SYMBOLS.lock().get(&demangled) {
        Some(*addr)
    } else {
        let addr = SYMBOLS.lock().get(&demangled).cloned();
        if addr.is_none() {
            log::warn!("Lost of symbol: [{}]!", demangled);
        }
        addr
    }
}

pub(super) fn insert_symbol(name: &str, address: VirtualAddress) {
    let demangled = format!("{:#}", demangle(name));
    SYMBOLS.lock().insert(&demangled, address);
}

macro_rules! symbols {
    ($(fn $name: literal [$func: path]);* $(;)?) => {
        [$(($name.into(), VirtualAddress::from($func as *const () as usize))),*]
    };

    ($(static $name: literal [$var: path]);* $(;)?) => {
        [$(($name.into(), VirtualAddress::from(&$var as *const _ as usize))),*]
    };
}

static KERNEL_SYMBOLS: Lazy<Mutex<StringMap<VirtualAddress>>> = Lazy::new(|| {
    let boot_files = BOOT_FILES.get_response().unwrap();
    let file = boot_files
        .modules()
        .iter()
        .find(|file| file.path().to_string_lossy().ends_with("symbols.sym"))
        .unwrap();

    let file_content = unsafe { from_raw_parts(file.addr(), file.size() as usize) };
    let content = core::str::from_utf8(file_content).unwrap();

    let mut map = StringMap::new();

    for line in content.lines() {
        if line.is_empty() {
            continue;
        }
        let mut items = line.split(";");
        let name = items.next().unwrap().trim();
        let address = items.next().unwrap().trim().parse().unwrap();
        map.insert(name, address);
    }

    Mutex::new(map)
});

static SYMBOLS: Lazy<Mutex<StringMap<VirtualAddress>>> = Lazy::new(|| {
    let mut map = StringMap::new();

    let lists: &[&[(&str, VirtualAddress)]] = &[&symbols!(
        fn "kernel_panic_handler" [panic_handler];
        fn "rust_eh_personality" [empty_fn];
        fn "memcpy" [memcpy];
        fn "memset" [memset];
        fn "bcmp" [bcmp];
        fn "memcmp" [memcmp];
        fn "alloc" [alloc];
        fn "dealloc" [dealloc];
        fn "_Unwind_Resume" [_Unwind_Resume];
        fn "strlen" [strlen];
        fn "memmove" [memmove];
    )];

    for list in lists {
        for (name, addr) in list.iter() {
            map.insert(name, *addr);
        }
    }

    Mutex::new(map)
});

fn empty_fn() {}

extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) {
    unsafe {
        core::ptr::copy_nonoverlapping(src, dest, n);
    }
}

extern "C" fn memset(dest: *mut u8, c: u8, n: usize) {
    unsafe {
        core::ptr::write_bytes(dest, c, n);
    }
}

extern "C" fn bcmp(lhs: *const u8, rhs: *const u8, n: usize) -> i32 {
    for i in 0..n {
        if unsafe { *lhs.add(i) } != unsafe { *rhs.add(i) } {
            return unsafe { *lhs.add(i) as i32 - *rhs.add(i) as i32 };
        }
    }
    0
}

extern "C" fn memcmp(lhs: *const u8, rhs: *const u8, n: usize) -> i32 {
    for i in 0..n {
        if unsafe { *lhs.add(i) } != unsafe { *rhs.add(i) } {
            return unsafe { *lhs.add(i) as i32 - *rhs.add(i) as i32 };
        }
    }
    0
}

fn alloc(layout: Layout) -> *mut u8 {
    unsafe { alloc::alloc::alloc(layout) }
}

fn dealloc(ptr: *mut u8, layout: Layout) {
    unsafe {
        alloc::alloc::dealloc(ptr, layout);
    }
}

#[allow(non_snake_case)]
extern "C" fn _Unwind_Resume() {}

extern "C" fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    unsafe {
        while *s.add(len) != 0 {
            len += 1;
        }
    }
    len
}

extern "C" fn memmove(dst: *mut u8, src: *const u8, count: usize) {
    unsafe {
        core::ptr::copy(src, dst, count);
    }
}
