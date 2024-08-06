use alloc::{string::String, vec};
use raca_std::{fs::{FileDescriptor, FileType, OpenMode}, task::{wait, Process}};

pub fn try_run(path: String) -> Option<()> {
    if let Ok(mut file) = FileDescriptor::open(&path, OpenMode::Read) {
        if file.get_type() == FileType::Dir {
            return None;
        }

        let mut buf = vec![0; file.size()];
        file.read(&mut buf);
        file.close();
        //let (pipe1_read,pipe1_write) = FileDescriptor::open_pipe().unwrap();
        //let (pipe2_read,pipe2_write) = FileDescriptor::open_pipe().unwrap();

        let process = Process::new(&buf, "temp", 0, 0);
        process.run();
        //loop {
        //    let mut buf = [0;1];
        //    pipe2_read.read(&mut buf);
        //    write!(fd, "{}", buf[0] as char).unwrap();
        //}
        //loop{}
        wait();
        // loop{}
        Some(())
    }else {
        None
    }
}