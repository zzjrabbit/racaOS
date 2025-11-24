use loongarch64::structures::paging::{MapToError, PageSize, TranslateError};

use crate::{MapError, QueryError};

impl<S: PageSize> From<MapToError<S>> for MapError {
    fn from(value: MapToError<S>) -> Self {
        match value {
            MapToError::FrameAllocationFailed => Self::FrameAllocationFailed,
            MapToError::PageAlreadyMapped(_) => Self::PageAlreadyMapped,
            MapToError::ParentEntryHugePage => Self::ParentEntryHugePage,
        }
    }
}

impl From<TranslateError> for QueryError {
    fn from(value: TranslateError) -> Self {
        match value {
            TranslateError::InvalidFrameAddress(_) => Self::InvalidFrameAddress,
            TranslateError::PageNotMapped => Self::NotMappedYet,
            _ => unreachable!(),
        }
    }
}
