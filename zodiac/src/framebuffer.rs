use limine::request::FramebufferRequest;

use crate::mem::{PhysicalAddress, VirtualAddress, convert_virtual_to_physical};

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

pub struct FrameBufferInfo {
    pub address: PhysicalAddress,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
    /// The size of the red mask, in bits. This part of the mask can be applied
    /// with `red_value & ((1 << red_mask_size) - 1)`.
    pub red_mask_size: u8,
    /// The number of bits to shift the red mask to the left. This part of the
    /// mask can be applied with `red_value << red_mask_shift`.
    pub red_mask_shift: u8,
    /// The size of the green mask, in bits. This part of the mask can be
    /// applied with `green_value & ((1 << green_mask_size) - 1)`.
    pub green_mask_size: u8,
    /// The number of bits to shift the green mask to the left. This part of the
    /// mask can be applied with `green_value << green_mask_shift`.
    pub green_mask_shift: u8,
    /// The size of the blue mask, in bits. This part of the mask can be applied
    /// with `blue_value & ((1 << blue_mask_size) - 1)`.
    pub blue_mask_size: u8,
    /// The number of bits to shift the blue mask to the left. This part of the
    /// mask can be applied with `blue_value << blue_mask_shift`.
    pub blue_mask_shift: u8,
}

impl FrameBufferInfo {
    pub fn get() -> Self {
        let response = FRAMEBUFFER_REQUEST.get_response().unwrap();
        let framebuffer = response.framebuffers().next().unwrap();
        log::info!("address: {:p}", framebuffer.addr());
        Self {
            address: convert_virtual_to_physical(framebuffer.addr() as VirtualAddress),
            width: framebuffer.width(),
            height: framebuffer.height(),
            bpp: framebuffer.bpp(),
            pitch: framebuffer.pitch(),
            red_mask_size: framebuffer.red_mask_size(),
            red_mask_shift: framebuffer.red_mask_shift(),
            green_mask_size: framebuffer.green_mask_size(),
            green_mask_shift: framebuffer.green_mask_shift(),
            blue_mask_size: framebuffer.blue_mask_size(),
            blue_mask_shift: framebuffer.blue_mask_shift(),
        }
    }
}
