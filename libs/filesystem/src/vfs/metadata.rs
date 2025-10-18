use core::time::Duration;

use credentials::{Gid, Uid};
use time::DateTime;

use crate::vfs::InodeMode;

pub struct Metadata {
    pub dev: u64,
    pub rdev: u64,

    pub inode_mode: InodeMode,

    pub uid: Uid,
    pub gid: Gid,

    pub access_time: Duration,
    pub modification_time: Duration,
    pub creation_time: Duration,
}

impl Metadata {
    pub fn new(mode: InodeMode) -> Self {
        let now = DateTime::default().unix_timestamp();
        Self {
            dev: 0,
            rdev: 0,
            inode_mode: mode,
            uid: Uid::new_root(),
            gid: Gid::new_root(),
            access_time: now,
            modification_time: now,
            creation_time: now,
        }
    }
}
