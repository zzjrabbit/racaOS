use super::*;

impl Readline {
    pub(super) fn handle_control_key(&mut self, key: char, buf: &mut String) {
        match key {
            '\x5b' => {
                // Directions
                let key = self.read_char();
                match key {
                    'C' => {
                        let idx = self.insert_point;

                        if idx < buf.len() {
                            print!("\x1b\x5bC");
                            self.insert_point = idx + 1;
                        }
                    }
                    'D' => {
                        let idx = self.insert_point;

                        if idx > 0 {
                            print!("\x1b\x5bD");
                            self.insert_point = idx - 1;
                        }
                    }
                    'A' => {
                        if self.history_index >= 1 {
                            self.history_index -= 1;
                            for _ in 0..self.insert_point {
                                print!("\x08");
                            }
                            for _ in 0..self.insert_point {
                                print!(" ");
                            }
                            for _ in 0..self.insert_point {
                                print!("\x08");
                            }
                            *buf = self.history[self.history_index].clone();
                            for index in 0..buf.len() {
                                print!("{}", buf.chars().nth(index).unwrap());
                            }
                            self.insert_point = buf.len();
                        }
                    }
                    'B' => {
                        if self.history_index < self.history.len() {
                            self.history_index += 1;
                            for _ in 0..self.insert_point {
                                print!("\x08");
                            }
                            for _ in 0..self.insert_point {
                                print!(" ");
                            }
                            for _ in 0..self.insert_point {
                                print!("\x08");
                            }
                            *buf = self.history[self.history_index].clone();
                            for index in 0..buf.len() {
                                print!("{}", buf.chars().nth(index).unwrap());
                            }
                            self.insert_point = buf.len();
                        }
                    }
                    _ => {
                        println!("Unhandled Direction: {:x}", key as u32);
                    }
                }
            }
            _ => {
                println!("Unhandled Control: {:x}", key as u32);
            }
        }
    }
}
