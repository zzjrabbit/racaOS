use std::{env::args, hash::Hasher, io::Write};

fn main() {
    let mut args = args();
    let _bin = args.next().unwrap();

    let mut user_name = String::new();

    let mut pass_word = String::new();

    std::io::stdin().read_line(&mut user_name).expect("Unable to read user name.");
    std::io::stdin().read_line(&mut pass_word).expect("Unable to read pass word.");
    let user_name = user_name.trim().to_string();
    let pass_word = pass_word.trim().to_string();

    //print!("group")

    let mut hasher = rs_sha3_512::Sha3_512Hasher::default();
    hasher.write(pass_word.as_bytes());
    //hasher.finish();

    let pass_word_result = rs_sha3_512::HasherContext::finish(&mut hasher);

    let passwd_file = args.next().expect("Expected passwd file");
    let mut passwd_file = std::fs::File::create(passwd_file).expect("Unable to create passwd file");

    let line = format!("{}:{pass_word_result:02X}\n", user_name);
    passwd_file.write_all(line.as_bytes()).expect("Unable to write to passwd file");
}
