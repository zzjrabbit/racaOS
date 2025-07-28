use core::{sync::atomic::{AtomicBool, AtomicU32, Ordering}, time::Duration};

use x2apic::lapic::TimerMode;

use crate::hal::{kernel::LAPIC, timer::hpet::HPET};

const TIMER_FREQUENCY_HZ: u32 = 200;
const TIMER_CALIBRATION_ITERATION: u32 = 50;

static LAPIC_TIMER_INIT: AtomicBool = AtomicBool::new(false);
static LAPIC_TIMER_INITIAL: AtomicU32 = AtomicU32::new(0);

pub fn init() {
    LAPIC_TIMER_INIT.store(true, Ordering::SeqCst);
    unsafe {
        calibrate_timer();
        LAPIC.lock().enable_timer();
    }
}

pub fn ap_init() {
    while !LAPIC_TIMER_INIT.load(Ordering::SeqCst) {
        core::hint::spin_loop();
    }
    let timer_initial = LAPIC_TIMER_INITIAL.load(Ordering::Relaxed);
    unsafe {
        LAPIC.lock().set_timer_initial(timer_initial);
        LAPIC.lock().enable_timer();
    }
}

unsafe fn calibrate_timer() {
    let mut lapic = LAPIC.lock();
    let mut lapic_total_ticks = 0;

    for _ in 0..TIMER_CALIBRATION_ITERATION {
        let last_time = HPET.elapsed();
        unsafe {
            lapic.set_timer_initial(u32::MAX);
        }
        while HPET.elapsed() - last_time < Duration::from_millis(1) {}
        lapic_total_ticks += u32::MAX - unsafe{lapic.timer_current()};
    }

    let average_ticks_per_ms = lapic_total_ticks / TIMER_CALIBRATION_ITERATION;
    let calibrated_timer_initial = average_ticks_per_ms * 1000 / TIMER_FREQUENCY_HZ;
    log::debug!("Calibrated timer initial: {calibrated_timer_initial}");

    unsafe {
        lapic.set_timer_mode(TimerMode::Periodic);
        lapic.set_timer_initial(calibrated_timer_initial);
    }
    
    LAPIC_TIMER_INITIAL.store(calibrated_timer_initial, Ordering::Relaxed);
}