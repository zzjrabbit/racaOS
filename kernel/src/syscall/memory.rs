use core::range::Range;

use alloc::sync::Arc;

use crate::{
    error::{RcError, RcResult},
    mm::{MMUFlags, PhysicalMemory, VirtualMemory, VmMapping},
    object::{Handle, HandleValue, Rights},
    task::job_policy::PolicyCondition,
};

use super::current_process;

pub fn allocate_child(father: HandleValue, handle_ptr: usize, page_count: usize) -> RcResult<usize> {
    let current_process = current_process();

    let father = current_process.get_object_with_rights::<VirtualMemory>(father, Rights::GET_INFO)?;
    let virtual_memory = father.allocate_child(page_count)?;
    let handle =
        current_process.add_handle(Handle::new(virtual_memory, Rights::DEFAULT_VIRTUAL_MEMORY));

    unsafe {
        *(handle_ptr as *mut HandleValue) = handle as HandleValue;
    }

    Ok(0)
}

pub fn get_vm_start_address(handle: usize) -> RcResult<usize> {
    let current_process = current_process();

    let vm = current_process
        .get_object_with_rights::<VirtualMemory>(handle as HandleValue, Rights::GET_INFO)?;

    Ok(vm.start_address())
}

pub fn create_child(
    handle: usize,
    page_start: usize,
    page_count: usize,
    child_handle_ptr: usize,
) -> RcResult<usize> {
    let current_process = current_process();

    let vm = current_process
        .get_object_with_rights::<VirtualMemory>(handle as HandleValue, Rights::GET_INFO)?;

    if (page_start + page_count) * 4096 > vm.len() {
        return Err(RcError::InvalidArguments);
    }

    let child = vm.create_child(Range::from(page_start..page_start + page_count));
    let child_handle =
        current_process.add_handle(Handle::new(child, Rights::DEFAULT_VIRTUAL_MEMORY));

    unsafe {
        *(child_handle_ptr as *mut HandleValue) = child_handle as HandleValue;
    }

    Ok(0)
}

pub fn root_virtual_memory(handle_ptr: usize) -> RcResult<usize> {
    let current_process = current_process();

    let vm = current_process.vmar();
    let handle = current_process.add_handle(Handle::new(vm, Rights::DEFAULT_VIRTUAL_MEMORY));

    unsafe {
        *(handle_ptr as *mut HandleValue) = handle as HandleValue;
    }

    Ok(0)
}

pub fn create_physical_memory(count: usize, handle_ptr: usize) -> RcResult<usize> {
    let physical_memory = PhysicalMemory::allocate(count)?;
    let handle = current_process().add_handle(Handle::new(
        physical_memory,
        Rights::DEFAULT_PHYSICAL_MEMORY,
    ));

    unsafe {
        *(handle_ptr as *mut HandleValue) = handle as HandleValue;
    }

    Ok(0)
}

pub fn get_pm_start_address(handle: usize) -> RcResult<usize> {
    let current_process = current_process();

    let pm = current_process
        .get_object_with_rights::<PhysicalMemory>(handle as HandleValue, Rights::GET_INFO)?;

    Ok(pm.start_address())
}

pub fn map(vm: usize, pm: usize, flags: usize) -> RcResult<usize> {
    let current_process = current_process();

    let flags = MMUFlags::from_bits_truncate(flags) | MMUFlags::USER;

    if flags.contains(MMUFlags::EXECUTE | MMUFlags::WRITE) {
        current_process.check_policy(PolicyCondition::VmarWx)?;
    }

    let vm = current_process.get_object::<VirtualMemory>(vm as HandleValue)?;
    let pm =
        current_process.get_object_with_rights::<PhysicalMemory>(pm as HandleValue, Rights::MAP)?;

    let vm_mapping = Arc::new(VmMapping::new(flags, vm, pm));
    vm_mapping.map()?;

    Ok(0)
}

pub fn unmap(vm: usize) -> RcResult<usize> {
    let current_process = current_process();

    let vm = current_process.get_object::<VirtualMemory>(vm as HandleValue)?;

    let vm_mapping = Arc::new(VmMapping::new(
        MMUFlags::empty(),
        vm,
        PhysicalMemory::allocate(0)?,
    ));
    vm_mapping.unmap()?;

    Ok(0)
}
