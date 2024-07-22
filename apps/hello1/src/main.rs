#![no_std]
#![no_main]
#![feature(naked_functions)]

#[no_mangle]
pub fn main() {
    //let fd = raca_std::fs::open("/RACA/app64/hello2.rae", raca_std::fs::OpenMode::Read).unwrap();
    raca_std::println!("OK");
    //let mut buf = [0;8192];
    //let buf = [b'H',b'e',b'l',b'l',b'o',b',',b'y',b'o',b'u',b'r',b' ',b'd',b'i',b's',b'k',b' ',b'i',b's',b' ',b'b',b'r',b'o',b'k',b'e',b'n',b'!'];
    //write2(fd,buf.as_ptr(),buf.len());
    //let len = buf.len();
    //let mut buf = [0u8;40];
    //raca_std::fs::lseek(fd, 0);
    //raca_std::fs::read(fd,&mut buf);
    //raca_std::fs::close(fd);
    //raca_std::debug::dump_hex_buffer(&buf);
    //for i in 0..buf.len() {
    //    buf[i] += b'0';
    //}
    //raca_std::dump_hex_buffer(buf.as_ptr(), buf.len());
    //raca_std::task::create_process("Hello2",&buf);
    loop {
        //write("[racaOS]".as_ptr(),6);
        //syscall3(1);
    }
}
