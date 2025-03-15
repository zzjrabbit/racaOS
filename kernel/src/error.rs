pub type RcResult<T> = Result<T, RcError>;

#[allow(dead_code)]
#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum RcError {
    Ok = 0,
    BadHandle = -1,
    WrongType = -2,
    AccessDenied = -3,
    AllocationFailed = -4,
    FailedToMap = -5,
    FailedToUnmap = -6,
}
