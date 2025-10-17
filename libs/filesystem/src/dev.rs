mod fs;
mod null;
mod terminal;
mod zero;

use alloc::sync::Arc;
use spin::Lazy;

pub use terminal::{Terminal, init_terminal};

use crate::{ROOT_FS, dev::fs::DevFs};

static DEV_FS: Lazy<Arc<DevFs>> = Lazy::new(|| DevFs::new());

pub fn init() {
    let root_fs = ROOT_FS.clone();
    let dev_fs = root_fs
        .create("dev".into(), super::FileType::Directory)
        .unwrap();

    let null = dev_fs
        .create("null".into(), super::FileType::CharDevice)
        .unwrap();
    let zero = dev_fs
        .create("zero".into(), super::FileType::CharDevice)
        .unwrap();
    let terminal = dev_fs
        .create("tty".into(), super::FileType::CharDevice)
        .unwrap();

    let null_device = DEV_FS.new_null();
    let zero_device = DEV_FS.new_zero();
    let terminal_device = DEV_FS.new_terminal();

    null_device.mount(null);
    zero_device.mount(zero);
    terminal_device.mount(terminal);
}
