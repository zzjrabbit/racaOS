pub mod hpet;
pub mod keyboard;
pub mod mouse;
pub mod serial;

pub fn init() {
    hpet::init();
    keyboard::init();
    mouse::init();
}
