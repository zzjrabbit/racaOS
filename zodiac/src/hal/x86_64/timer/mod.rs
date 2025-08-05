use core::time::Duration;

mod apic;
#[allow(dead_code)]
mod hpet;

pub(crate) fn init() {
    hpet::init();
    apic::init();
}

pub(crate) fn ap_init() {
    apic::ap_init();
}

/// Returns the elapsed time since the system was booted.
pub fn elapsed() -> Duration {
    hpet::HPET.elapsed()
}

#[allow(dead_code)]
pub(crate) fn set_timer(duration: Duration) {
    hpet::HPET.set_timer(hpet::HPET.estimate(duration));
}
