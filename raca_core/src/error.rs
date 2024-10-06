use crate::arch::user::Error;

/// The type returned by kernel objects methods.
pub type RcResult<T = ()> = Result<T, RcError>;

/// Zircon statuses are signed 32 bit integers. The space of values is
/// divided as follows:
/// - The zero value is for the OK status.
/// - Negative values are defined by the system, in this file.
/// - Positive values are reserved for protocol-specific error values,
///   and will never be defined by the system.
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[repr(i32)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RcError {
    OK = 0,
    BAD_HANDLE = -1,
    WRONG_TYPE = -2,
    ACCESS_DENIED = -3,
    PEER_CLOSED = -4,
    NOT_SUPPORTED = -5,
    SHOULD_WAIT = -6,
    BUFFER_TOO_SMALL = -7,
    OUT_OF_MEMORY = -8,
    TIMED_OUT = -9,
    INVALID_ARGS = -10,
    ALREADY_EXISTS = -11,
    BAD_STATE = -12,
    OUT_OF_RANGE = -13,
    UNAVAILABLE = -14,
    NO_MEMORY = -15,
    NOT_FOUND = -16,
    ALREADY_BOUND = -17,
    NOT_BOUND = -18,
    FORBIDDEN = -19,
}

impl From<Error> for RcError {
    fn from(e: Error) -> Self {
        match e {
            Error::InvalidUtf8 => Self::INVALID_ARGS,
            Error::InvalidPointer => Self::INVALID_ARGS,
            Error::BufferTooSmall => Self::BUFFER_TOO_SMALL,
            Error::InvalidLength => Self::INVALID_ARGS,
            Error::InvalidVectorAddress => Self::NOT_FOUND,
        }
    }
}
