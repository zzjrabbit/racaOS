use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZodiacError {
    #[error("Not found.")]
    NotFound,
    #[error("Invalid arguments.")]
    InvalidArguments,
    #[error("No memory left.")]
    NoMemory,
    #[error("Out of bounds.")]
    OutOfBounds,
    #[error("Arguments not enough.")]
    ArgumentsNotEnough,
    #[error("Failed to map.")]
    FailedToMap(#[from] MapError),
    #[error("Failed to unmap.")]
    FailedToUnmap(#[from] UnmapError),
    #[error("Failed to update.")]
    FailedToUpdate(#[from] UpdateError),
    #[error("Failed to query.")]
    FailedToFlush(#[from] QueryError),
    #[error("Physical memory error.")]
    PhysicalMemoryError(#[from] PhyscialMemoryError),
}

#[derive(Error, Debug)]
pub enum MapError {
    #[error("Page already mapped.")]
    PageAlreadyMapped,
    #[error("Parent entry is a huge page.")]
    ParentEntryHugePage,
    #[error("Unable to allocate frame.")]
    FrameAllocationFailed,
    #[error("Virtual address is not aligned.")]
    VirtualAddressNotAligned,
}

#[derive(Error, Debug)]
pub enum UnmapError {
    #[error("Attempt to unmap an unmapped page.")]
    NotMappedYet,
    #[error("This page is mapped to invalid frame address.")]
    InvalidFrameAddress,
}

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Attempt to query an unmapped page.")]
    NotMappedYet,
    #[error("This page is mapped to invalid frame address.")]
    InvalidFrameAddress,
}

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Attempt to update an unmapped page.")]
    NotMappedYet,
}

#[derive(Error, Debug)]
pub enum PhyscialMemoryError {
    #[error("Unable to allocate {0} pages of memory.")]
    AllocateFailed(usize),
}
