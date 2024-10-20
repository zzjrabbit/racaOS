use core::{alloc::Layout, mem::transmute};

use alloc::{
    alloc::alloc,
    collections::btree_map::BTreeMap,
    string::{String, ToString},
};
use goblin::elf::Elf;
use x86_64::{
    structures::paging::{Mapper, Page, Size4KiB},
    VirtAddr,
};

use crate::memory::{ExtendedPageTable, MappingType, KERNEL_PAGE_TABLE};
use operations::*;

mod operations;

#[repr(C)]
pub struct InfoStruct {
    name: &'static str,
}

pub struct Module {
    name: String,
    symbol_addresses: BTreeMap<String, u64>,
    entry_address: u64,
}

impl Module {
    pub fn load(data: &[u8]) -> Self {
        let binary = Elf::parse(data).unwrap();
        let base = unsafe { alloc(Layout::from_size_align(data.len(), 4096).unwrap()) } as u64;

        let mut section_addresses = BTreeMap::new();

        let mut current_adress = base;

        let mut page_table = KERNEL_PAGE_TABLE.lock();

        let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(base));
        let end_page =
            Page::<Size4KiB>::containing_address(VirtAddr::new(base + data.len() as u64));

        for page in start_page..=end_page {
            //log::info!("mapping {:x?}",page);
            unsafe {
                page_table
                    .update_flags(page, MappingType::KernelCode.flags())
                    .unwrap()
                    .flush();
            }
        }

        for (id, section) in binary.section_headers.iter().enumerate() {
            if section.is_relocation() {
                continue;
            }

            section_addresses.insert(id, current_adress);
            current_adress += section.sh_size;
        }

        for (section_index, relocation_section) in binary.shdr_relocs {

            let relocationed_section = binary
                .section_headers
                .get(section_index)
                .unwrap();
            let relocationed_section = binary.section_headers.get(relocationed_section.sh_info as usize).unwrap();

            for relocation in relocation_section.iter() {
                let symbol = binary.syms.get(relocation.r_sym).unwrap();

                let section_address = if symbol.is_import() {
                    let symbol_name = binary.strtab.get_at(symbol.st_name).unwrap();
                    if let Some(address) = KERNEL_SYMBOL_TABLE.get(symbol_name) {
                        *address
                    } else {
                        panic!("unknow symbol {}!",symbol_name);
                    }
                } else {
                    *section_addresses.get(&symbol.st_shndx).unwrap()
                };
                let offset = relocation.r_offset + relocationed_section.sh_offset;
                let addend = relocation.r_addend.unwrap_or(0);

                let target_address = VirtAddr::new(if addend >= 0 {
                    section_address + addend as u64
                } else {
                    section_address - (-addend as u64)
                });

                unsafe {
                    ((data.as_ptr() as u64 + offset as u64) as *mut u64)
                        .write(target_address.as_u64());
                }
            }
        }

        for (&section_id, &section_address) in section_addresses.iter() {
            let section = &binary.section_headers[section_id];

            page_table.write_to_mapped_address(
                &data[section.sh_offset as usize
                    ..section.sh_offset as usize + section.sh_size as usize],
                VirtAddr::new(section_address),
            );
        }

        let symbol_addresses = binary
            .syms
            .iter()
            .filter(|symbol| section_addresses.get(&symbol.st_shndx).is_some())
            .map(|symbol| {
                (
                    binary.strtab.get_at(symbol.st_name).unwrap().to_string(),
                    *section_addresses.get(&symbol.st_shndx).unwrap(),
                )
            })
            .collect::<BTreeMap<_, _>>();

        let entry_address = *symbol_addresses.get("init").unwrap();

        let info_address = *symbol_addresses.get("MODULE_INFO").unwrap();
        let module_info = unsafe { &mut *(info_address as *mut InfoStruct) };
        let name = module_info.name;

        Self {
            name: name.into(),
            symbol_addresses,
            entry_address,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_function_address(&self, name: &str) -> u64 {
        self.symbol_addresses.get(name).unwrap_or(&0).clone()
    }

    pub fn exec(&self) -> usize {
        let func: extern "C" fn() -> usize = unsafe { transmute(self.entry_address) };
        func()
    }
}
