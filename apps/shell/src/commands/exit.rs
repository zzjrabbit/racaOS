use alloc::{string::String, vec::Vec};
use raca_std::fs::FileDescriptor;

pub fn exit(_stdio: &mut FileDescriptor, _args: Vec<String>) {
    raca_std::task::exit(0);
}

