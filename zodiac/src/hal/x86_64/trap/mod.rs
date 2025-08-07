use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use spin::{Lazy, RwLock};
use x86_64::VirtAddr;

use crate::{
    hal::{
        context::{CpuException, TrapFrame},
        cpu::Cpu,
        smp::CPUS,
    },
    mem::VirtualAddress,
    task::{schedule, Thread},
    trap::IRQ_MANAGER,
};

pub(super) mod gdt;
pub(super) mod idt;
pub(super) mod syscall;

pub(crate) use syscall::init;
pub use syscall::set_syscall_handler;

pub(crate) enum ContextSaveAction {
    Yield,
    Fork,
    Clone(Arc<Thread>, VirtualAddress),
}

static CONTEXT_SAVE_ACTION: Lazy<RwLock<BTreeMap<Cpu, ContextSaveAction>>> = Lazy::new(|| {
    let mut map = BTreeMap::new();
    for cpu in Cpu::all() {
        map.insert(cpu, ContextSaveAction::Yield);
    }
    RwLock::new(map)
});

pub(crate) fn change_context_save_action(action: ContextSaveAction) {
    *CONTEXT_SAVE_ACTION.write().get_mut(&Cpu::current()).unwrap() = action;
}

#[unsafe(no_mangle)]
extern "C" fn rust_entry(frame: &mut TrapFrame) {
    if let Some(cpu_exception) = CpuException::new(frame.int_num, frame.error_code) {
        log::warn!(
            "CPU Exception on {}: {:x?}",
            Cpu::current().id(),
            cpu_exception
        );
        log::warn!("Trap frame: {:x?}", frame);

        loop {
            x86_64::instructions::hlt();
        }
    } else {
        if frame.int_num == 32 {
            match CONTEXT_SAVE_ACTION.read().get(&Cpu::current()).unwrap() {
                ContextSaveAction::Yield => schedule(frame),
                ContextSaveAction::Clone(thread, stack) => {
                    thread.clone_impl(frame.clone(), *stack);
                    schedule(frame);
                }
                _ => unimplemented!()
            }
            return;
        }
        IRQ_MANAGER.handle_irq(frame);
    }
}

pub(crate) fn set_kernel_stack(stack: VirtualAddress) {
    CPUS.with_cpu_info_mut(Cpu::current(), |cpu_info| {
        cpu_info.set_ring0_rsp(VirtAddr::new(stack as u64))
    });
}
