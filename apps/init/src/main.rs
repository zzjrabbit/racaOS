#![no_std]
#![no_main]

extern crate alloc;

#[no_mangle]
pub fn main() {
    raca_std::println!("Kernel jumped into the init user program.");
    let fd = raca_std::fs::open("/dev/terminal", raca_std::fs::OpenMode::Write).unwrap();
    raca_std::fs::write(fd, "Kernel jumped into the init user program.".as_bytes());

    loop {
        let mut buf = [0; 1];
        raca_std::fs::read(fd, &mut buf);
        raca_std::println!("{}", buf[0] as char);
    }
}
