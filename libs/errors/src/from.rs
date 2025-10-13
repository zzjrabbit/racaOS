use crate::{Errno, Error};

impl From<ostd::Error> for Error {
    fn from(value: ostd::Error) -> Self {
        match value {
            ostd::Error::AccessDenied => Errno::EACCES,
            ostd::Error::InvalidArgs => Errno::EINVAL,
            ostd::Error::IoError => Errno::EIO,
            ostd::Error::NoMemory => Errno::ENOMEM,
            ostd::Error::NotEnoughResources => Errno::ENOSPC,
            ostd::Error::Overflow => Errno::EOVERFLOW,
            ostd::Error::PageFault => Errno::EFAULT,
        }.no_message()
    }
}
