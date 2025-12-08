use loongarch64::{instructions::interrupt, registers::TimerConfigBuilder};

mod trap;

pub fn init() {
    TimerConfigBuilder::new()
        .initial_value(1000000 >> 2)
        .set_enabled(true)
        .set_periodic(true)
        .done();

    trap::init();
}

pub fn enable_int() {
    interrupt::enable();
}

pub fn disable_int() {
    interrupt::disable();
}
