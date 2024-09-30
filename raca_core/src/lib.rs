#![no_std]
#![feature(get_mut_unchecked)]
extern crate alloc;

pub mod error;
pub mod ipc;
pub mod object;
pub mod task;

#[cfg(test)]
mod tests {}
