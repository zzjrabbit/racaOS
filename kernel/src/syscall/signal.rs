use core::slice::from_raw_parts;

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use spin::Mutex;

use crate::{
    error::RcResult,
    object::{Handle, HandleValue, Rights, Signal},
    signal::EventPair,
    task::{scheduler::SCHEDULER, thread::ThreadState},
};

use super::current_process;

pub fn wait_for_signal(
    objects_ptr: *const HandleValue,
    objects_len: usize,
    signal: Signal,
    source_ptr: *mut HandleValue,
) -> RcResult<usize> {
    let current_process = current_process();
    let current_thread = SCHEDULER.current_thread();

    let handles = unsafe { from_raw_parts(objects_ptr, objects_len) };
    let objects = handles
        .iter()
        .map(|handle| current_process.get_object_with_rights_no_downgrade(*handle, Rights::empty()))
        .collect::<RcResult<Vec<_>>>()?;

    for object in objects.iter() {
        if object.signal_present(signal) {
            handles.iter().for_each(|handle| {
                let this_object = current_process
                    .get_object_with_rights_no_downgrade(*handle, Rights::empty())
                    .unwrap();

                if Arc::ptr_eq(&this_object, &object) {
                    unsafe {
                        source_ptr.write(*handle);
                    }
                }
            });
            return Ok(0);
        }
    }

    let source = Arc::new(Mutex::new(None));

    objects.iter().for_each(|object| {
        let current_thread = current_thread.clone();
        let source = Arc::downgrade(&source);
        let another_object = object.clone();
        object.add_signal_callback(Box::new(move |got_signal| {
            if got_signal.contains(signal) {
                if let (Some(current_thread), Some(source)) =
                    (current_thread.upgrade(), source.upgrade())
                {
                    if source.lock().is_some() || current_thread.state().is_awake() {
                        return true;
                    }

                    *source.lock() = Some(another_object.clone());
                    current_thread.set_state(ThreadState::Ready);
                    SCHEDULER.add_thread(&current_thread);
                }
                return true;
            }
            false
        }))
    });

    let current_thread = current_thread.upgrade().unwrap();

    while source.lock().is_none() {
        current_thread.set_state(ThreadState::Blocked);

        unsafe {
            core::arch::asm!("int 0x21");
        }
    }

    let source = source.lock().clone().unwrap();

    handles.iter().for_each(|handle| {
        let this_object = current_process
            .get_object_with_rights_no_downgrade(*handle, Rights::empty())
            .unwrap();

        if Arc::ptr_eq(&this_object, &source) {
            unsafe {
                source_ptr.write(*handle);
            }
        }
    });

    Ok(0)
}

pub fn event_pair_create(
    handle0_ptr: *mut HandleValue,
    handle1_ptr: *mut HandleValue,
) -> RcResult<usize> {
    let current_process = current_process();

    let (event_pair0, event_pair1) = EventPair::new();
    let handle0 = current_process.add_handle(Handle::new(event_pair0, Rights::DEFAULT_EVENT_PAIR));
    let handle1 = current_process.add_handle(Handle::new(event_pair1, Rights::DEFAULT_EVENT_PAIR));

    unsafe {
        handle0_ptr.write(handle0);
        handle1_ptr.write(handle1);
    }

    Ok(0)
}
