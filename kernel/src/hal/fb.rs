use alloc::sync::Arc;
use limine::request::FramebufferRequest;
use x86_64::{
    VirtAddr,
    structures::paging::{Mapper, Page, PageTableFlags, PhysFrame, Size4KiB, Translate},
};

use crate::{
    kernel_object,
    mm::{VirtualMemory, page_count},
    object::KObjectBase,
};

use super::FRAME_ALLOCATOR;

#[used]
#[unsafe(link_section = ".requests")]
pub static FRAME_BUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

kernel_object! {
    pub struct FrameBuffer {
        width: u64 = 0,
        height: u64 = 0,
        ptr: u64 = 0,
    }

    fn new() {}
}

impl FrameBuffer {
    pub fn get(vm: Arc<VirtualMemory>) -> Arc<Self> {
        let response = FRAME_BUFFER_REQUEST.get_response().unwrap();

        let framebuffer = response.framebuffers().next().unwrap();

        let ptr = framebuffer.addr() as u64;
        let width = framebuffer.width();
        let height = framebuffer.height();

        let len = width as usize * height as usize * 4;

        let page_count = page_count(len);
        let vm = vm.allocate_child(page_count).unwrap();

        for count in 0..page_count {
            let start_address = VirtAddr::new(ptr + count as u64 * 4096);
            let frame = PhysFrame::containing_address(
                vm.page_table()
                    .lock()
                    .translate_addr(start_address)
                    .unwrap(),
            );
            let page = Page::<Size4KiB>::containing_address(VirtAddr::new(
                vm.start_address() as u64 + count as u64 * 4096,
            ));
            unsafe {
                vm.page_table()
                    .lock()
                    .map_to(
                        page,
                        frame,
                        PageTableFlags::USER_ACCESSIBLE
                            | PageTableFlags::WRITABLE
                            | PageTableFlags::PRESENT,
                        &mut *FRAME_ALLOCATOR.lock(),
                    )
                    .unwrap()
                    .flush();
            }
        }

        Arc::new(Self {
            base: KObjectBase::default(),
            width,
            height,
            ptr: vm.start_address() as u64,
        })
    }

    pub fn ptr(&self) -> u64 {
        self.ptr
    }

    pub fn width(&self) -> u64 {
        self.width
    }

    pub fn height(&self) -> u64 {
        self.height
    }
}
