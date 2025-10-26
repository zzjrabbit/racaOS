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
        waker_fn(waker);
        waiter.wait();

        cond().ok_or(Errno::EINTR.with_message("Interrupted by signal."))
    }
}
