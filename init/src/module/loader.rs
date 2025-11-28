use alloc::{sync::Arc, vec::Vec};
use elf::{
    ElfBytes,
    abi::{ET_DYN, PF_R, PF_W, PF_X, PT_LOAD, SHT_RELA, STB_GLOBAL, STT_NOTYPE, STV_DEFAULT},
    endian::LittleEndian,
    segment::ProgramHeader,
};
use spin::{Lazy, Mutex};
use zodiac::{
    ZodiacError,
    mem::{
        CachePolicy, MMUFlags, PageProperty, PageSize, PhysicalMemoryAllocOptions, Privilege,
        VirtualAddress, VmSpace,
    },
};

use crate::module::{MODULES, Module, symbols::SYMBOLS};

pub const MODULE_START: usize = 0xffff_c000_0000_0000usize;
pub const MODULE_SIZE: usize = 64 * 1024 * 1024;

const R_LARCH_RELATIVE: u32 = 3;
const R_LARCH_JUMP_SLOT: u32 = 5;

impl Module {
    pub(super) fn load_module(data: &[u8]) -> Result<Arc<Self>, ZodiacError> {
        let binary = ElfBytes::<LittleEndian>::minimal_parse(data)
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

        let ModuleFnSet { entry } = Self::load_symbols(base, &binary)?;

        log::info!("Module loaded at {:x}, entry: {:p}.", base, entry);

        let module = Arc::new(Self { entry });
        MODULES.lock().push(module.clone());

        Ok(module)
    }

    fn alloc_mem(binary: &ElfBytes<LittleEndian>) -> Result<(usize, usize), ZodiacError> {
        let size = binary
            .segments()
            .unwrap()
            .into_iter()
            .filter(|segment| segment.p_type == PT_LOAD)
            .map(|segment| segment.p_vaddr + segment.p_memsz)
            .max()
            .unwrap() as usize;
        let vaddr_start = MODULE_ALLOCATOR.lock().allocate(size)?;

        Ok((vaddr_start, size))
    }

    fn relocate(base: VirtualAddress, binary: &ElfBytes<LittleEndian>) -> Result<(), ZodiacError> {
        let common = binary.find_common_data().unwrap();
        let dyn_syms = common.dynsyms.unwrap();
        let dyn_strtab = common.dynsyms_strs.unwrap();

        let symbols = SYMBOLS.lock();

        let kernel_vm_space = VmSpace::kernel();

        for section in binary.section_headers().unwrap() {
            if section.sh_type != SHT_RELA {
                continue;
            }

            for rela in binary.section_data_as_relas(&section).unwrap() {
                let reloc_addr = rela.r_offset as usize + base;
                let sym_idx = rela.r_sym;
                let r_type = rela.r_type;

                let symbol = dyn_syms.get(sym_idx as usize).unwrap();

                match r_type {
                    R_LARCH_RELATIVE => {
                        let value = base as i64 + rela.r_addend;

                        kernel_vm_space
                            .writer(reloc_addr, size_of::<usize>())
                            .write(&(value as usize))?;
                    }
                    R_LARCH_JUMP_SLOT => {
                        let symbol_name = dyn_strtab.get(symbol.st_name as usize).unwrap();

                        let Some(value) = symbols
                            .get(symbol_name)
                            .map(|value| *value as i64 + rela.r_addend)
                        else {
                            log::error!("Symbol {} not found!", symbol_name);
                            return Err(ZodiacError::NotFound);
                        };

                        kernel_vm_space
                            .writer(reloc_addr, size_of::<usize>())
                            .write(&(value as usize))?;
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

        let size = segment.p_memsz as usize;
        let aligned_size = PageSize::Size4K.align_up(size);

        let mut pm = PhysicalMemoryAllocOptions::new()
            .count(aligned_size / PageSize::Size4K as usize)
            .allocate()?;

        let data = binary.segment_data(&segment).unwrap();
        pm.writer(vaddr - aligned_vaddr, data.len())
            .write_bytes(data)?;

        let mut cursor = kernel_vm_space.cursor(aligned_vaddr)?;
        cursor.map(&pm, property)
    }

    fn load_symbols(
        base: VirtualAddress,
        binary: &ElfBytes<LittleEndian>,
    ) -> Result<ModuleFnSet, ZodiacError> {
        let common = binary
            .find_common_data()
            .map_err(|_| ZodiacError::NotFound)?;
        let symboltab = common.symtab.ok_or(ZodiacError::NotFound)?;
        let symbol_strtab = common.symtab_strs.ok_or(ZodiacError::NotFound)?;

        let mut entry = None;
        let mut symbols = SYMBOLS.lock();

        for symbol in symboltab.iter() {
            if symbol.st_vis() == STV_DEFAULT
                && symbol.st_bind() == STB_GLOBAL
                && symbol.st_symtype() != STT_NOTYPE
            {
                let name = symbol_strtab.get(symbol.st_name as usize).unwrap();
                let addr = symbol.st_value as VirtualAddress + base;

                log::info!("scanned symbol[{}] at {:x}!", name, addr);

                if name == "init" {
                    entry = Some(unsafe { core::mem::transmute(addr) });
                } else {
                    symbols.insert(name.into(), addr);
                }
            }
        }

        Ok(ModuleFnSet {
            entry: entry.unwrap(),
        })
    }
}

struct ModuleFnSet {
    entry: fn(),
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
