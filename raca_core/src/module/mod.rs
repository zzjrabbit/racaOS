use core::{alloc::Layout, mem::transmute};

use alloc::{alloc::alloc, collections::btree_map::BTreeMap, string::String};
use object::{File, Object, ObjectSegment, ObjectSymbol, ObjectSymbolTable};
use x86_64::{
    structures::paging::{Mapper, Page, Size4KiB},
    VirtAddr,
};

use crate::memory::{ExtendedPageTable, MappingType, KERNEL_PAGE_TABLE};
use operations::*;

mod operations;

#[repr(C)]
pub struct InfoStruct {
    _magic: [u8; 8],
    name: [u8; 8],
    kernel_function_address: usize,
}

pub struct Module {
    name: String,
    function_addresses: BTreeMap<String, u64>,
}

impl Module {
    pub fn load(data: &[u8]) -> Self {
        //object::write::elf::Writer::

        let binary = File::parse(data).unwrap();

        let max_addr = binary
            .segments()
            .map(|seg| seg.address() + seg.size())
            .max()
            .unwrap() as u64;
        let base =
            unsafe { alloc(Layout::from_size_align(max_addr as usize, 4096).unwrap()) } as u64;

        let mut page_table = KERNEL_PAGE_TABLE.lock();

        for page_i in 0..(max_addr + 4095) / 4096 {
            let page =
                Page::<Size4KiB>::containing_address(VirtAddr::new(base + page_i as u64 * 4096));
            unsafe {
                page_table
                    .update_flags(page, MappingType::KernelCode.flags())
                    .unwrap()
                    .flush();
            }
        }

        for segment in binary.segments() {
            if let Ok(data) = segment.data() {
                page_table.write_to_mapped_address(data, VirtAddr::new(segment.address() + base));
            }
        }

        //for mut reloc in binary.dynamic_relocations().unwrap() {
        //    reloc.1.set_addend(base as i64);
        //}

        let get_info_address = binary
            .dynamic_symbol_table()
            .unwrap()
            .symbols()
            .find(|sym| {
                if let Ok(name) = sym.name() {
                    name == "get_info"
                } else {
                    false
                }
            })
            .unwrap()
            .address()
            + base;

        let mut function_addresses = BTreeMap::new();
        binary.dynamic_symbols().for_each(
            |sym| {
                if let Ok(name) = sym.name() {
                    function_addresses.insert(String::from(name), sym.address() + base);
                }
            }
        );

        let func: extern "C" fn() -> &'static mut InfoStruct =
            unsafe { transmute(get_info_address) };
        let info = func();

        info.kernel_function_address = kernel_function as usize;

        Self {
            name: String::from_utf8(info.name.to_vec()).unwrap(),
            function_addresses,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_function_address(&self, name: &str) -> u64 {
        self.function_addresses
            .get(name)
            .unwrap_or(&0)
            .clone()
    }

    pub fn exec(&self) -> usize {
        let func: extern "C" fn() -> usize = unsafe { transmute(self.get_function_address("init")) };
        func()
    }
}

