use crate::{print, println};
use alloc::{string::String, vec, vec::Vec};

mod control;
mod rw;

pub struct Readline {
    insert_point: usize,
    history: Vec<String>,
    history_index: usize,
}

impl Readline {
    pub fn new() -> Self {
        Self {
            insert_point: 0,
            history: vec!["".into()],
            history_index: 0,
        }
    }

    pub fn read_line(&mut self, buf: &mut String) {
        buf.clear();
        self.insert_point = 0;
        self.history_index = self.history.len() - 1;

        loop {
            let key = self.read_char();

            if self.handle_key(key, buf) {
                let last_index = self.history.len() - 1;
                self.history[last_index] = buf.clone();
                self.history.push("".into());

                break;
            }
        }
    }

    fn handle_key(&mut self, key: char, buf: &mut String) -> bool {
        //println!("handle_key called with key: {:x}", key as u8);
        match key {
            '\x7f' => {
                // Backspace
                let old_length = buf.len();

                if old_length == 0 || self.insert_point == 0 {
                    return false;
                }

                buf.remove(self.insert_point - 1);
                let new_length = buf.len();
                self.insert_point -= 1;
                crate::print("\x08").unwrap();
                for index in self.insert_point..new_length {
                    print!("{}", buf.chars().nth(index).unwrap());
                }
                print!(" ");
                for _ in self.insert_point..old_length {
                    print!("\x08");
                }

                false
            }
            '\n' => {
                println!();
                true
            }
            '\x1b' => {
                // control key
                let key = self.read_char();
                self.handle_control_key(key, buf);
                false
            }
            '\x03' => {
                println!("^C");
                true
            }
            '\x07' => {
                print!("\x07");
                false
            }
            _ => {
                self.show_char(key, buf);
                false
            }
        }
    }
}
