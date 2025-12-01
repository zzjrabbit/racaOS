use alloc::{collections::btree_map::BTreeMap, format, string::String};
use elf::{
    ElfBytes,
    abi::{ET_DYN, STB_GLOBAL, STV_DEFAULT},
    endian::LittleEndian,
};
use rustc_demangle::demangle;
use spin::{Lazy, Mutex};
use zodiac::{ZodiacError, kernel_base, kernel_file, mem::VirtualAddress};

use crate::panic_handler;

pub(super) fn search_global_symbol(name: &str) -> Option<VirtualAddress> {
    let demangled = format!("{:#}", demangle(name));
    if demangled.len() > 6
        && let Some(addr) = KERNEL_SYMBOLS.lock().get_function(&demangled[6..])
    {
        Some(addr)
    } else {
        SYMBOLS.lock().get(name).cloned()
    }
}

static KERNEL_SYMBOLS: Mutex<SymbolTableNode> = Mutex::new(SymbolTableNode::new());

pub fn init() -> Result<(), ZodiacError> {
    let kernel_file = kernel_file();
    let file = ElfBytes::<LittleEndian>::minimal_parse(kernel_file)
        .map_err(|_| ZodiacError::InvalidArguments)?;
    log::info!("kernel parsed");

    let base = if file.ehdr.e_type == ET_DYN {
        kernel_base()
    } else {
        0
    };

    let mut symbols = KERNEL_SYMBOLS.lock();

    let common = file.find_common_data().map_err(|_| ZodiacError::NotFound)?;
    let symtab = common.symtab.ok_or(ZodiacError::NotFound)?;
    let strtab = common.symtab_strs.ok_or(ZodiacError::NotFound)?;

    log::info!("len: {}", symtab.len());

    for symbol in symtab.iter() {
        if symbol.is_undefined() || symbol.st_bind() != STB_GLOBAL || symbol.st_vis() != STV_DEFAULT
        {
            continue;
        }
        let Ok(name) = strtab.get(symbol.st_name as usize) else {
            continue;
        };

        let name = format!("{:#}", demangle(name));

        if !name.starts_with("zodiac") {
            continue;
        }

        symbols.insert(&name[6..], symbol.st_value as VirtualAddress + base);
    }
    log::info!("scanned");

    Ok(())
}

macro_rules! symbols {
    ($(fn $name: ident $func: ident);* $(;)?) => {
        [$((stringify!($name).into(), VirtualAddress::from($func as *const () as usize))),*]
    };
}

pub(super) static SYMBOLS: Lazy<Mutex<BTreeMap<String, VirtualAddress>>> = Lazy::new(|| {
    Mutex::new(
        symbols!(
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

fn rust_eh_personality() {}

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

struct SymbolTableNode {
    function: Option<VirtualAddress>,
    next: BTreeMap<char, SymbolTableNode>,
}

impl SymbolTableNode {
    pub const fn new() -> Self {
        SymbolTableNode {
            function: None,
            next: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, name: &str, address: VirtualAddress) {
        let mut current = self;
        for ch in name.chars() {
            let entry = current.next.entry(ch);
            let next = entry.or_insert(SymbolTableNode::new());
            current = next;
        }

        current.function = Some(address);
    }

    pub fn get_function(&self, name: &str) -> Option<VirtualAddress> {
        let mut current = self;
        for ch in name.chars() {
            if let Some(node) = current.next.get(&ch) {
                current = node;
            } else {
                return None;
            }
        }
        current.function
    }
}
