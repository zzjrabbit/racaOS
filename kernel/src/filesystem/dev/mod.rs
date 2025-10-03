mod null;
mod terminal;
mod zero;

use null::*;
use terminal::*;
use zero::*;

use crate::filesystem::{File, Path, ROOT_FS};

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

    let null_device = File::new(
        Path::new("/dev/null"),
        NullDevice,
        super::FileType::CharDevice,
    );
    let zero_device = File::new(
        Path::new("/dev/zero"),
        ZeroDevice,
        super::FileType::CharDevice,
    );
    let terminal_device = File::new(
        Path::new("/dev/tty"),
        TerminalInode,
        super::FileType::CharDevice,
    );

    null_device.mount(null);
    zero_device.mount(zero);
    terminal_device.mount(terminal);
}
