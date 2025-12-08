use crate::registers::{CurrentModeInfo, CurrentModeInfoBuilder};

pub fn enable() {
    CurrentModeInfoBuilder::new_with(CurrentModeInfo.read())
        .enable_global_interrupt()
        .done();
}

pub fn disable() {
    CurrentModeInfoBuilder::new_with(CurrentModeInfo.read())
        .disable_global_interrupt()
        .done();
}
