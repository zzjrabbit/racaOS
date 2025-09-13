use core::sync::atomic::AtomicBool;

use spin::Once;

pub(crate) static AP_ENTRY_SET: AtomicBool = AtomicBool::new(false);
static AP_ENTRY: Once<fn() -> !> = Once::new();

pub fn set_ap_entry(entry: fn() -> !) {
    AP_ENTRY.call_once(|| entry);
    AP_ENTRY_SET.store(true, core::sync::atomic::Ordering::SeqCst);
}

pub(crate) fn ap_entry() -> fn() -> ! {
    *AP_ENTRY.get().unwrap()
}
