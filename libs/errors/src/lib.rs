#![no_std]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid arguments.")]
    InvalidArguments,
    #[error("Access denied.")]
    AccessDenied,
    #[error("Not found.")]
    NotFound,
    #[error("Already exists.")]
    AlreadyExists,
    #[error("I/O Operation failed.")]
    IoOperationFailed,
    #[error("Not enough resources.")]
    NotEnoughResources,
    #[error("No memory available.")]
    NoMemory,
    #[error("Something overflowed.")]
    Overflow,
    #[error("Page fault.")]
    PageFault,
}

impl From<ostd::Error> for Error {
    fn from(value: ostd::Error) -> Self {
        match value {
            ostd::Error::AccessDenied => Error::AccessDenied,
            ostd::Error::IoError => Error::IoOperationFailed,
            ostd::Error::NotEnoughResources => Error::NotEnoughResources,
            ostd::Error::NoMemory => Error::NoMemory,
            ostd::Error::Overflow => Error::Overflow,
            ostd::Error::PageFault => Error::PageFault,
            ostd::Error::InvalidArgs => Error::InvalidArguments,
        }
    }
}
