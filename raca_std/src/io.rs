use alloc::string::String;

use crate::fs::FileDescriptor;

impl FileDescriptor {
    pub fn stdin_read_line(&self, buf: &mut String) {
        buf.clear(); // make sure that the buf is clean

        let mut tmp_buf = [0; 1];
        self.read(&mut tmp_buf);

        while tmp_buf[0] != b'\n' {
            if tmp_buf[0] == 8 {
                // backspace
                let _ = buf.pop();
            } else {
                buf.push(tmp_buf[0] as char);
            }
            self.read(&mut tmp_buf);
        }
    }
}
