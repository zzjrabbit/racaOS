#![no_std]
#![no_main]

use alloc::vec;
use raca_std::fs::FileDescriptor;

extern crate alloc;

#[no_mangle]
pub fn main() {
    let mut fd = FileDescriptor::open("/dev/terminal", raca_std::fs::OpenMode::Write).unwrap();
    fd.write("Kernel jumped into the init user program.".as_bytes());
    fd.close();
    let fd = FileDescriptor::open("/RACA/app64/shell.rae", raca_std::fs::OpenMode::Read).unwrap();
    let mut buf = vec![0; fd.size()];
    fd.read(&mut buf);
    let process = raca_std::task::Process::new(&buf, "shell", 0, 0);
    process.run();

    loop {}
}
