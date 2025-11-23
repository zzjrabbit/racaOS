#![no_std]

pub use addr::*;

mod addr;
pub mod instructions;
pub mod registers;
pub mod structures;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivilegeLevel {
    Privilege0 = 0,
    Privilege1 = 1,
    Privilege2 = 2,
    Privilege3 = 3,
}
