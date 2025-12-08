use alloc::{sync::Arc, vec::Vec};
use elf::{
    ElfBytes,
    abi::{ET_DYN, PF_R, PF_W, PF_X, PT_LOAD, SHT_RELA, STB_GLOBAL, STV_DEFAULT},
    endian::LittleEndian,
    segment::ProgramHeader,
};
use mostd_core::ModuleInfo;
use ruzstd::{decoding::StreamingDecoder, io::Read};
use spin::{Lazy, Mutex};
use zodiac::{
    ZodiacError,
    mem::{
        CachePolicy, MMUFlags, PageProperty, PageSize, PhysicalMemoryAllocOptions, Privilege,
        VirtualAddress, VmSpace,
    },
};

use crate::module::{
    MODULES, Module,
    symbols::{insert_symbol, search_global_symbol},
};

const MODULE_END: usize = 0xffff_ffff_8000_0000usize;
pub const MODULE_START: usize = MODULE_END - MODULE_SIZE;
pub const MODULE_SIZE: usize = 1 * 1024 * 1024 * 1024 * 1024;

const R_LARCH_64: u32 = 2;
const R_LARCH_RELATIVE: u32 = 3;
const R_LARCH_JUMP_SLOT: u32 = 5;

impl Module {
    pub(super) fn load_module(mut data: &[u8]) -> Result<Arc<Self>, ZodiacError> {
        let mut decoder =
            StreamingDecoder::new(&mut data).map_err(|_| ZodiacError::InvalidArguments)?;
        let mut data = Vec::new();
        decoder
            .read_to_end(&mut data)
            .map_err(|_| ZodiacError::InvalidArguments)?;

        let binary = ElfBytes::<LittleEndian>::minimal_parse(&data)
            .map_err(|_| ZodiacError::InvalidArguments)?;

        if binary.ehdr.e_type != ET_DYN {
            log::warn!("expected ET_DYN!");
            return Err(ZodiacError::InvalidArguments);
        }

        let (base, _) = Self::alloc_mem(&binary)?;
        let kernel_vm_space = VmSpace::kernel();

        for segment in binary.segments().unwrap() {
            Self::map_segment(&kernel_vm_space, base, &binary, &segment)?;
        }

        Self::relocate(base, &binary)?;

        let module = Arc::new(Self::load_symbols(base, &binary)?);

        log::info!(
            "Module {} loaded at {:x}, entry: {:p}.",
            module.name(),
            base,
            module.entry
        );

        MODULES.lock().push(module.clone());

        Ok(module)
    }

    fn alloc_mem(binary: &ElfBytes<LittleEndian>) -> Result<(usize, usize), ZodiacError> {
        let size = binary
            .segments()
            .ok_or(ZodiacError::InvalidArguments)?
            .into_iter()
            .filter(|segment| segment.p_type == PT_LOAD)
            .map(|segment| segment.p_vaddr + segment.p_memsz)
            .max()
            .ok_or(ZodiacError::InvalidArguments)? as usize;
        let size = PageSize::Size4K.align_up(size);

        let vaddr_start = MODULE_ALLOCATOR.lock().allocate(size)?;

        Ok((vaddr_start, size))
    }

    fn relocate(base: VirtualAddress, binary: &ElfBytes<LittleEndian>) -> Result<(), ZodiacError> {
        let common = binary
            .find_common_data()
            .map_err(|_| ZodiacError::InvalidArguments)?;
        let dyn_syms = common.dynsyms.ok_or(ZodiacError::NotFound)?;
        let dyn_strtab = common.dynsyms_strs.ok_or(ZodiacError::NotFound)?;

        let kernel_vm_space = VmSpace::kernel();

        for section in binary
            .section_headers()
            .ok_or(ZodiacError::InvalidArguments)?
        {
            if section.sh_type != SHT_RELA {
                continue;
            }

            for rela in binary
                .section_data_as_relas(&section)
                .map_err(|_| ZodiacError::InvalidArguments)?
            {
                let reloc_addr = rela.r_offset as usize + base;
                let sym_idx = rela.r_sym;
                let r_type = rela.r_type;

                let symbol = dyn_syms
                    .get(sym_idx as usize)
                    .map_err(|_| ZodiacError::NotFound)?;

                match r_type {
                    R_LARCH_RELATIVE => {
                        let value = base as i64 + rela.r_addend;

                        kernel_vm_space
                            .writer(reloc_addr, size_of::<usize>())
                            .write(&(value as usize))?;
                    }
                    R_LARCH_JUMP_SLOT => {
                        let symbol_name = dyn_strtab.get(symbol.st_name as usize).unwrap_or("");

                        let s_addr = if symbol.is_undefined() {
                            let Some(addr) = search_global_symbol(symbol_name) else {
                                continue;
                            };
                            addr as i64
                        } else {
                            (symbol.st_value as i64) + (base as i64)
                        };

                        let value = s_addr.wrapping_add(rela.r_addend) as usize;

                        kernel_vm_space
                            .writer(reloc_addr, size_of::<usize>())
                            .write(&(value))?;
                    }
                    R_LARCH_64 => {
                        let symbol_name = dyn_strtab.get(symbol.st_name as usize).unwrap_or("");

                        let Some(s_addr) = (if symbol.is_undefined() {
                            search_global_symbol(symbol_name).map(|v| v as i64)
                        } else {
                            Some((symbol.st_value as i64) + (base as i64))
                        }) else {
                            continue;
                        };

                        let value = s_addr.wrapping_add(rela.r_addend) as usize;

                        kernel_vm_space
                            .writer(reloc_addr, size_of::<usize>())
                            .write(&(value))?;
                    }
                    _ => log::warn!("Unsupported relocation type {}!", r_type),
                }
            }
        }

        Ok(())
    }

    fn map_segment(
        kernel_vm_space: &Arc<VmSpace>,
        base: VirtualAddress,
        binary: &ElfBytes<LittleEndian>,
        segment: &ProgramHeader,
    ) -> Result<(), ZodiacError> {
        if segment.p_type != PT_LOAD {
            return Ok(());
        }

        let mut flags = MMUFlags::empty();
        if segment.p_flags & PF_R != 0 {
            flags |= MMUFlags::READ;
        }
        if segment.p_flags & PF_W != 0 {
            flags |= MMUFlags::WRITE;
        }
        if segment.p_flags & PF_X != 0 {
            flags |= MMUFlags::EXECUTE;
        }

        let property = PageProperty::new(flags, CachePolicy::CacheCoherent, Privilege::KernelOnly);

        let vaddr = base + segment.p_vaddr as VirtualAddress;
        let aligned_vaddr = PageSize::Size4K.align_down(vaddr);

        let size = segment.p_memsz as usize + vaddr - aligned_vaddr;
        let aligned_size = PageSize::Size4K.align_up(size);

        let pm = PhysicalMemoryAllocOptions::new()
            .count(aligned_size / PageSize::Size4K as usize)
            .allocate()?;
        pm.zero()?;

        let data = binary.segment_data(&segment).unwrap();
        pm.writer(vaddr - aligned_vaddr, data.len())
            .write_bytes(data)?;

        let mut cursor = kernel_vm_space.cursor(aligned_vaddr)?;
        cursor.map(&pm, property)
    }

    fn load_symbols(
        base: VirtualAddress,
        binary: &ElfBytes<LittleEndian>,
    ) -> Result<Self, ZodiacError> {
        let common = binary
            .find_common_data()
            .map_err(|_| ZodiacError::NotFound)?;
        let symboltab = common.dynsyms.ok_or(ZodiacError::NotFound)?;
        let symbol_strtab = common.dynsyms_strs.ok_or(ZodiacError::NotFound)?;

        let mut info: Option<&ModuleInfo> = None;
        let mut entry = None;

        for symbol in symboltab.iter() {
            if symbol.st_vis() == STV_DEFAULT
                && symbol.st_bind() == STB_GLOBAL
                && !symbol.is_undefined()
            {
                let Ok(name) = symbol_strtab.get(symbol.st_name as usize) else {
                    continue;
                };
                let addr = symbol.st_value as VirtualAddress + base;

                if name == "_module_init_" {
                    entry = Some(unsafe { core::mem::transmute(addr) });
                } else if name == "_MODULE_INFO" {
                    info = Some(unsafe { &*core::ptr::with_exposed_provenance(addr) });
                } else {
                    insert_symbol(name, addr);
                }
            }
        }

        Ok(Self {
            name: info.ok_or(ZodiacError::NotFound)?.name,
            entry: entry.ok_or(ZodiacError::NotFound)?,
        })
    }
}

static MODULE_ALLOCATOR: Lazy<Mutex<ModuleAllocator>> =
    Lazy::new(|| Mutex::new(ModuleAllocator::new(MODULE_START, MODULE_SIZE)));

struct ModuleAllocator {
    regions: Vec<(usize, usize)>,
}

impl ModuleAllocator {
    pub fn new(start: usize, size: usize) -> Self {
        Self {
            regions: alloc::vec![(start, size)],
        }
    }
}

impl ModuleAllocator {
    pub fn allocate(&mut self, size: usize) -> Result<usize, ZodiacError> {
        for (start, rsize) in &mut self.regions {
            if size <= *rsize {
                let allocated_start = *start;
                *start += size;
                return Ok(allocated_start);
            }
        }
        Err(ZodiacError::NoMemory)
    }

    pub fn deallocate(&mut self, start: usize, size: usize) -> Result<(), ZodiacError> {
        self.regions.push((start, size));
        self.merge();
        Err(ZodiacError::InvalidArguments)
    }

    pub fn merge(&mut self) {
        self.regions.sort_by(|a, b| a.0.cmp(&b.0));
        let mut merged = Vec::new();
        let mut current = self.regions[0];
        for next in &self.regions[1..] {
            if current.0 + current.1 == next.0 {
                current.1 += next.1;
            } else {
                merged.push(current);
                current = *next;
            }
        }
        merged.push(current);
        self.regions = merged;
    }
}
