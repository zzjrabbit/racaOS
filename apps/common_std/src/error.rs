pub type RcResult<T> = Result<T, RcError>;

#[allow(dead_code)]
#[repr(isize)]
#[derive(Debug, Clone, Copy)]
pub enum RcError {
    Ok = 0,
    BadHandle = -1,
    WrongType = -2,
    AccessDenied = -3,
    AllocationFailed = -4,
    FailedToMap = -5,
    FailedToUnmap = -6,
    NotSupported = -7,
    NotFound = -8,
    BadState = -9,
    AlreadyExists = -10,
    InvalidSyscall = -11,
    InvalidArguments = -12,
    PeerClosed = -13,
    ShouldWait = -14,
}
