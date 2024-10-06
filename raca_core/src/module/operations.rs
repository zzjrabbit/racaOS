pub fn kernel_function(id: usize) -> usize {
    match id {
        0 => print as usize,
        _ => panic!("Unknown kernel function"),
    }
}

fn print(msg: &str) -> usize {
    crate::print!("{}", msg);
    0
}
