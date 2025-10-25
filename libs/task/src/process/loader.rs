use alloc::{ffi::CString, string::ToString};
use credentials::{Gid, Uid};
use elf::{
    ElfBytes,
    abi::{PF_R, PF_W, PF_X, PT_INTERP, PT_LOAD},
    endian::LittleEndian,
};
use errors::{Errno, Result};
use filesystem::Path;
use memory::{align_down_by_page_size, align_up_by_page_size};
use ostd::{
    Error as OstdError,
    mm::{CachePolicy, PAGE_SIZE, PageFlags, PageProperty, Vaddr},
};

use crate::{MemoryInfo, UserThreadData};

use crate::process::user_stack::{AuxKey, AuxVec};

impl MemoryInfo {
    pub fn load(
        &self,
        exec_file_name: Vaddr,
        fs_resolver: Option<&UserThreadData>,
        data: &[u8],
    ) -> Result<(Vaddr, AuxVec)> {
        let file =
            ElfBytes::<LittleEndian>::minimal_parse(data).map_err(|_| OstdError::InvalidArgs)?;

        let mut interp = None;

        for segment in file.segments().unwrap() {
            if segment.p_type != PT_INTERP {
                continue;
            }

            let file_offset = segment.p_offset as usize;
            let file_size = segment.p_filesz as usize;

            log::info!("interp path len: {}", file_size);
            let file_path = data[file_offset..file_offset + file_size].to_vec();
            let file_path = CString::from_vec_with_nul(file_path).unwrap();
            let file_path = file_path.to_string_lossy().to_string();

            log::info!("interp path: {}", file_path);

            interp = Some(file_path);
            break;
        }

        let (base, entry) = if let None = fs_resolver {
            (0, file.ehdr.e_entry as Vaddr)
        } else if let Some(interp) = interp {
            let file = fs_resolver
                .unwrap()
                .open_file(&Path::from(interp))
                .ok_or(Errno::ENOENT.no_message())?;
            let mut buffer = alloc::vec![0u8; file.len() as usize];
            file.read_at(0, &mut buffer)?;

            let file = ElfBytes::minimal_parse(&buffer).unwrap();
            let base = self.map_elf_file(true, &file)?;

            (base, file.ehdr.e_entry as Vaddr + base)
        } else {
            (0, file.ehdr.e_entry as Vaddr)
        };

        self.map_elf_file(false, &file)?;

        let load_start = file
            .segments()
            .unwrap()
            .into_iter()
            .map(|segment| align_down_by_page_size(segment.p_vaddr as Vaddr))
            .min()
            .unwrap();

        let mut aux_vec = AuxVec::new();
        aux_vec.set(AuxKey::Base, base as u64);
        aux_vec.set(AuxKey::Phnum, file.ehdr.e_phnum as u64);
        aux_vec.set(AuxKey::Entry, file.ehdr.e_entry);
        aux_vec.set(AuxKey::Phent, file.ehdr.e_phentsize as u64);
        aux_vec.set(AuxKey::Phdr, load_start as u64 + file.ehdr.e_phoff);
        aux_vec.set(AuxKey::PageSize, PAGE_SIZE as u64);
        aux_vec.set(AuxKey::Gid, u32::from(Gid::new_root()) as u64);
        aux_vec.set(AuxKey::Uid, u32::from(Uid::new_root()) as u64);
        aux_vec.set(AuxKey::ExecFileName, exec_file_name as u64);

        log::info!("aux vec: {:x?}", aux_vec);

        Ok((entry, aux_vec))
    }

    fn map_elf_file(&self, ldso: bool, file: &ElfBytes<LittleEndian>) -> Result<Vaddr> {
        let len = {
            let segments = file.segments().ok_or(OstdError::InvalidArgs)?;
            let end = segments
                .iter()
                .map(|seg| seg.p_vaddr + seg.p_memsz)
                .max()
                .unwrap();
            align_up_by_page_size(end as usize)
        };
        let base = if ldso {
            log::info!("Loading LDSO");
            self.allocate(align_up_by_page_size(len))?.start_address()
        } else {
            0
        };

        for segment in file.segments().ok_or(OstdError::InvalidArgs)? {
            if segment.p_type & PT_LOAD == 0 || segment.p_memsz == 0 {
                continue;
            }

            let data = file
                .segment_data(&segment)
                .map_err(|_| OstdError::InvalidArgs)?;

            let address = segment.p_vaddr as Vaddr + base;

            let mut page_flags = PageFlags::empty();
            if segment.p_flags & PF_R != 0 {
                page_flags |= PageFlags::R;
            }
            if segment.p_flags & PF_W != 0 {
                page_flags |= PageFlags::W;
            }
            if segment.p_flags & PF_X != 0 {
                page_flags |= PageFlags::X;
            }

            let aligned_address = align_down_by_page_size(address);
            let aligned_size =
                align_up_by_page_size(segment.p_memsz as usize + address % PAGE_SIZE);

            self.vmar()
                .map(
                    aligned_address,
                    aligned_size,
                    PageProperty::new_user(page_flags, CachePolicy::Writeback),
                    false,
                )
                .unwrap();

            self.vmar().write(address, data)?;
        }

        Ok(base)
    }
}
