use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::collections::btree_map::BTreeMap;
use spin::Lazy;

use crate::arch::{cpu_num, current_cpu, disable_int, enable_int};

pub struct DisabledLocalIrqGuard;

impl DisabledLocalIrqGuard {
    pub fn new() -> Self {
        if IRQ_GUARD_COUNT.increment() == 1 {
            disable_int();
        }
        Self
    }

    pub fn count() -> usize {
        IRQ_GUARD_COUNT.fetch()
    }
}

impl Drop for DisabledLocalIrqGuard {
    fn drop(&mut self) {
        if IRQ_GUARD_COUNT.decrement() == 0 {
            enable_int();
        }
    }
}

static IRQ_GUARD_COUNT: Lazy<IrqGuardCount> = Lazy::new(IrqGuardCount::new);

struct IrqGuardCount {
    values: BTreeMap<u64, AtomicUsize>,
}

impl IrqGuardCount {
    pub fn new() -> Self {
        let mut values = BTreeMap::new();
        for cpu_id in 0..cpu_num() {
            values.insert(cpu_id, AtomicUsize::new(0));
        }
        Self { values }
    }
}

impl IrqGuardCount {
    pub fn increment(&self) -> usize {
        let cpu_id = current_cpu();
        self.values
            .get(&cpu_id)
            .unwrap()
            .fetch_add(1, Ordering::Relaxed)
            + 1
    }

    pub fn decrement(&self) -> usize {
        let cpu_id = current_cpu();
        self.values
            .get(&cpu_id)
            .unwrap()
            .fetch_sub(1, Ordering::Relaxed)
            - 1
    }

    pub fn fetch(&self) -> usize {
        let cpu_id = current_cpu();
        self.values.get(&cpu_id).unwrap().load(Ordering::Relaxed)
    }
}
