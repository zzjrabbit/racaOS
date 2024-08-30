use std::{env::args, hash::Hasher, io::Write};

fn main() {
    let mut args = args();
    let _bin = args.next().unwrap();

    let passwd_file = args.next().expect("Expected passwd file");
    let mut passwd_file = std::fs::File::create(passwd_file).expect("Unable to create passwd file");

    println!("User num: ");

    let mut user_num = String::new();
    std::io::stdin()
        .read_line(&mut user_num)
        .expect("Unable to read user number.");
    let user_num = user_num
        .trim()
        .parse::<u32>()
        .expect("Invalid user number.");

    for uid in 1000..1000 + user_num {
        let mut user_name = String::new();
        let mut pass_word = String::new();

        println!("User name: ");
        std::io::stdin()
            .read_line(&mut user_name)
            .expect("Unable to read user name.");
        println!("User password: ");
        std::io::stdin()
            .read_line(&mut pass_word)
            .expect("Unable to read pass word.");
        let user_name = user_name.trim().to_string();
        let pass_word = pass_word.trim().to_string();

        println!("Chose a user group: 0: ROOT, 1: USER :");

        let mut group = String::new();
        std::io::stdin()
            .read_line(&mut group)
            .expect("Unable to read user group.");
        let group = group.trim().parse::<u32>().expect("Invalid user group.");

        if group != 0 && group != 1 {
            eprintln!("Invalid user group. Only 0 and 1 are allowed.");
            continue;
        }

        let mut hasher = rs_sha3_512::Sha3_512Hasher::default();
        hasher.write(pass_word.as_bytes());
        //hasher.finish();

        let pass_word_result = rs_sha3_512::HasherContext::finish(&mut hasher);

        let line = format!("{} ({uid},{group}) : {pass_word_result:02X}\n", user_name);
        passwd_file
            .write_all(line.as_bytes())
            .expect("Unable to write to passwd file");
    }
}
