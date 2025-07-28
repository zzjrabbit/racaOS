use core::time::Duration;

mod apic;
mod hpet;

pub(crate) fn init() {
    hpet::init();
    apic::init();
}

pub(crate) fn ap_init() {
    apic::ap_init();
}

pub fn elapsed() -> Duration {
    hpet::HPET.elapsed()
}

pub(crate) fn set_timer(duration: Duration) {
    hpet::HPET.set_timer(hpet::HPET.estimate(duration));
}
