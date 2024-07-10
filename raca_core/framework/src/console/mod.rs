pub mod display;
pub mod log;
pub mod printk;

pub fn init() {
    log::init();
    printk::init();
}
