use crate::{
    error::RcResult,
    hal::fb::FrameBuffer,
    object::{Handle, HandleValue, Rights},
};

use super::current_process;

pub fn get_fb(handle_ptr: usize) -> RcResult<usize> {
    let current_process = current_process();

    let fb = crate::hal::fb::FrameBuffer::get(current_process.vmar());
    let handle = current_process.add_handle(Handle::new(fb, Rights::DEFAULT_FB));

    unsafe {
        *(handle_ptr as *mut HandleValue) = handle as HandleValue;
    }

    Ok(0)
}

pub fn fb_width(fb: usize) -> RcResult<usize> {
    let current_process = current_process();
    let fb =
        current_process.get_object_with_rights::<FrameBuffer>(fb as HandleValue, Rights::READ)?;
    Ok(fb.width() as usize)
}

pub fn fb_height(fb: usize) -> RcResult<usize> {
    let current_process = current_process();
    let fb =
        current_process.get_object_with_rights::<FrameBuffer>(fb as HandleValue, Rights::READ)?;
    Ok(fb.height() as usize)
}

pub fn fb_ptr(fb: usize) -> RcResult<usize> {
    let current_process = current_process();
    let fb =
        current_process.get_object_with_rights::<FrameBuffer>(fb as HandleValue, Rights::READ)?;
    Ok(fb.ptr() as usize)
}
