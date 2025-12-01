#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

mod errno;
mod from;

use core::fmt::{Debug, Display};

use alloc::string::String;
pub use errno::*;

#[mostd::entry(errors)]
pub fn main() {}

pub type Result<T> = core::result::Result<T, Error>;

impl Errno {
    pub fn with_message<S: Into<String>>(&self, message: S) -> Error {
        Error {
            errno: *self,
            message: message.into(),
        }
    }

    pub fn no_message(&self) -> Error {
        Error {
            errno: *self,
            message: String::new(),
        }
    }
}

pub struct Error {
    errno: Errno,
    message: String,
}

impl From<Error> for i32 {
    fn from(error: Error) -> Self {
        error.errno as i32
    }
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
