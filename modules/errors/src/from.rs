use crate::{Errno, Error};

impl From<mostd::ZodiacError> for Error {
    fn from(value: mostd::ZodiacError) -> Self {
        match value {
            mostd::ZodiacError::PermissionDenied => Errno::EACCES,
            mostd::ZodiacError::InvalidArguments => Errno::EINVAL,
            mostd::ZodiacError::ArgumentsNotEnough => Errno::EINVAL,
            mostd::ZodiacError::FailedToFlush(_) => Errno::EINVAL,
            mostd::ZodiacError::FailedToMap(_) => Errno::EINVAL,
            mostd::ZodiacError::FailedToUnmap(_) => Errno::EINVAL,
            mostd::ZodiacError::FailedToUpdate(_) => Errno::EINVAL,
            mostd::ZodiacError::NoMemory => Errno::ENOMEM,
            mostd::ZodiacError::NotFound => Errno::ENOENT,
            mostd::ZodiacError::OutOfBounds => Errno::EINVAL,
            mostd::ZodiacError::PhysicalMemoryError(_) => Errno::ENOMEM,
        }
        .no_message()
    }
}
