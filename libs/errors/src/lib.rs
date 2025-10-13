#![no_std]

extern crate alloc;

mod errno;
mod from;

use core::fmt::{Debug, Display};

use alloc::string::String;
pub use errno::*;

pub type Result<T> = core::result::Result<T, Error>;

impl Errno {
    pub fn with_message(&self, message: String) -> Error {
        Error { errno: *self, message }
    }
    
    pub fn no_message(&self) -> Error {
        Error { errno: *self, message: String::new() }
    }
}

pub struct Error {
    errno: Errno,
    message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}: {}", self.errno, self.message)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}: {}", self.errno, self.message)
    }
}

impl core::error::Error for Error {}
