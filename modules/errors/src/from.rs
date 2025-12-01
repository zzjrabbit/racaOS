use crate::{Errno, Error};

impl From<mostd::ZodiacError> for Error {
    fn from(value: mostd::ZodiacError) -> Self {
        match value {
            mostd::ZodiacError::AccessDenied => Errno::EACCES,
            mostd::ZodiacError::InvalidArgs => Errno::EINVAL,
            mostd::ZodiacError::IoError => Errno::EIO,
            mostd::ZodiacError::NoMemory => Errno::ENOMEM,
            mostd::ZodiacError::NotEnoughResources => Errno::ENOSPC,
            mostd::ZodiacError::Overflow => Errno::EOVERFLOW,
            mostd::ZodiacError::PageFault => Errno::EFAULT,
        }
        .no_message()
    }
}
