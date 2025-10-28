use core::sync::atomic::{AtomicBool, Ordering};

use alloc::sync::Arc;
use errors::{Errno, Result};
use ostd::sync::{Waiter, Waker};

use crate::UserThreadData;

impl UserThreadData {
    pub fn wait_with_waker<R>(
        &self,
        cond: impl Fn() -> Option<R>,
        waker_fn: impl Fn(Arc<Waker>),
    ) -> Result<R> {
        let (waiter, waker) = Waiter::new_pair();

        self.set_signalled_waker(waker.clone());
        waker_fn(waker.clone());

        let first = AtomicBool::new(true);
        let res = waiter.wait_until_or_cancelled(&cond, || {
            if !first.swap(false, Ordering::SeqCst) {
                Err(Errno::EINTR.no_message())
            } else {
                Ok(())
            }
        });

        res
    }
}
