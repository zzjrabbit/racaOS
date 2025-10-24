use credentials::{Gid, Uid};
use elf::{
    ElfBytes,
    abi::{ET_DYN, PF_R, PF_W, PF_X, PT_LOAD},
    endian::LittleEndian,
};
use errors::Result;
use memory::{align_down_by_page_size, align_up_by_page_size};
use ostd::{
    Error as OstdError,
    mm::{CachePolicy, PAGE_SIZE, PageFlags, PageProperty, Vaddr},
};

use crate::MemoryInfo;

use crate::process::user_stack::{AuxKey, AuxVec};

impl MemoryInfo {
    pub fn load(&self, data: &[u8]) -> Result<(Vaddr, Vaddr, AuxVec)> {
        let file =
            ElfBytes::<LittleEndian>::minimal_parse(data).map_err(|_| OstdError::InvalidArgs)?;

        let len = {
            let segments = file.segments().ok_or(OstdError::InvalidArgs)?;
            let end = segments
                .iter()
                .map(|seg| seg.p_vaddr + seg.p_memsz)
                .max()
                .unwrap();
            align_up_by_page_size(end as usize)
        };
        let base = if file.ehdr.e_type & ET_DYN == 1 {
            log::info!("Loading PIE");
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
                )
                .unwrap();

            self.vmar().write(address, data)?;
        }

        let len = align_up_by_page_size(data.len());
        let region = self.allocate(len)?;
        let file_address = region.start_address();
        self.vmar().map(
            file_address,
            len,
            PageProperty::new_user(PageFlags::R | PageFlags::W, CachePolicy::Writeback),
        )?;
        self.vmar().write(file_address, data)?;

        let mut aux_vec = AuxVec::new();
        aux_vec.set(AuxKey::Phnum, file.ehdr.e_phnum as u64);
        aux_vec.set(AuxKey::Entry, file.ehdr.e_entry + base as u64);
        aux_vec.set(AuxKey::Phent, file.ehdr.e_phentsize as u64);
        aux_vec.set(AuxKey::Phdr, file_address as u64 + file.ehdr.e_phoff);
        aux_vec.set(AuxKey::PageSize, PAGE_SIZE as u64);
        aux_vec.set(AuxKey::Gid, u32::from(Gid::new_root()) as u64);
        aux_vec.set(AuxKey::Uid, u32::from(Uid::new_root()) as u64);

        Ok((base, file.ehdr.e_entry as Vaddr + base, aux_vec))
    }
}
