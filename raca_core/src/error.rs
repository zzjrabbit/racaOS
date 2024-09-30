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
}
