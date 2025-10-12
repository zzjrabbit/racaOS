use elf::{
    abi::{PF_R, PF_W, PF_X, PT_LOAD},
    endian::LittleEndian,
    ElfBytes,
};
use ostd::{
    mm::{CachePolicy, PageFlags, PageProperty, Vaddr},
    Error as OstdError,
};

use {
    memory::Vmar,
    crate::process::user_stack::{AuxKey, AuxVec},
};

pub(crate) trait ElfLoader {
    fn load(&self, data: &[u8]) -> Result<(Vaddr, AuxVec), OstdError>;
}

impl ElfLoader for Vmar {
    fn load(&self, data: &[u8]) -> Result<(Vaddr, AuxVec), OstdError> {
        let file =
            ElfBytes::<LittleEndian>::minimal_parse(data).map_err(|_| OstdError::InvalidArgs)?;

        for segment in file.segments().ok_or(OstdError::InvalidArgs)? {
            if segment.p_type & PT_LOAD == 0 {
                continue;
            }

            let data = file
                .segment_data(&segment)
                .map_err(|_| OstdError::InvalidArgs)?;

            let address = segment.p_vaddr as Vaddr;

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
            let _ = self.map(
                address,
                data.len(),
                PageProperty::new_user(page_flags, CachePolicy::Writeback),
            );

            self.write(address, data)?;
        }

        let mut aux_vec = AuxVec::new();
        aux_vec.set(AuxKey::Phnum, file.ehdr.e_phnum as u64);
        aux_vec.set(AuxKey::Entry, file.ehdr.e_entry);

        Ok((file.ehdr.e_entry as Vaddr, aux_vec))
    }
}
