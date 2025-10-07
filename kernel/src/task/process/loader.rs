use elf::{
    abi::{PF_R, PF_W, PF_X, PT_LOAD},
    endian::LittleEndian,
    ElfBytes,
};
use ostd::{
    mm::{
        tlb::TlbFlushOp, CachePolicy, FrameAllocOptions, PageFlags, PageProperty, Vaddr, VmSpace,
        PAGE_SIZE,
    },
    task::disable_preempt,
    Error as OstdError,
};

use crate::{mem::{VmReadWrite, align_down_by_page_size, align_up_by_page_size}, task::process::user_stack::{AuxKey, AuxVec}};

pub trait BinaryLoader {
    fn load(&self, data: &[u8]) -> Result<(Vaddr, AuxVec), OstdError>;
}

impl BinaryLoader for VmSpace {
    fn load(&self, data: &[u8]) -> Result<(Vaddr, AuxVec), OstdError> {
        let guard = disable_preempt();

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
            let aligned_address = align_down_by_page_size(address);

            let page_offset = address - aligned_address;

            let page_count = align_up_by_page_size(data.len() + page_offset) / PAGE_SIZE;

            for i in 0..page_count {
                let frame = FrameAllocOptions::new().alloc_frame()?;
                let page_vaddr = aligned_address + i * PAGE_SIZE;

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

                let mut cursor = self.cursor_mut(&guard, &(page_vaddr..page_vaddr + PAGE_SIZE))?;
                cursor.map(
                    frame.into(),
                    PageProperty::new_user(page_flags, CachePolicy::Writeback),
                );

                cursor
                    .flusher()
                    .issue_tlb_flush(TlbFlushOp::for_range(page_vaddr..page_vaddr + PAGE_SIZE));
                cursor.flusher().dispatch_tlb_flush();
            }

            self.write(address, data).unwrap();
        }
        
        let mut aux_vec = AuxVec::new();
        aux_vec.set(AuxKey::Phnum, file.ehdr.e_phnum as u64);
        aux_vec.set(AuxKey::Entry, file.ehdr.e_entry);
        
        Ok((file.ehdr.e_entry as Vaddr, aux_vec))
    }
}
