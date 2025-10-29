use core::sync::atomic::{AtomicBool, Ordering};

use alloc::{sync::Arc, vec::Vec};
use ostd::sync::{RwLock, Waker};

use crate::IoEvent;

pub struct Poller {
    inner: RwLock<PollerInner>,
    required: IoEvent,
    waker: Arc<Waker>,
    finished: AtomicBool,
}

struct PollerInner {
    event: IoEvent,
}

impl Poller {
    pub fn new(required: IoEvent, waker: Arc<Waker>) -> Arc<Self> {
        Arc::new(Self {
            inner: RwLock::new(PollerInner {
                event: IoEvent::empty(),
            }),
            required,
            waker,
            finished: AtomicBool::new(false),
        })
    }

    fn set_event(&self, event: IoEvent) {
        let event = {
            let mut inner = self.inner.write();
            inner.event = event;
            inner.event
        };
        if event.contains(self.required) {
            self.finished.store(true, Ordering::SeqCst);
            self.waker.wake_up();
        }
    }

    pub(crate) fn short_path(&self, event: IoEvent) {
        let mut inner = self.inner.write();
        inner.event = event;
        self.finished.store(true, Ordering::SeqCst);
    }

    pub(crate) fn required(&self) -> IoEvent {
        self.required
    }

    pub fn finished(&self) -> bool {
        self.finished.load(Ordering::SeqCst)
    }

    pub fn event(&self) -> IoEvent {
        self.inner.read().event
    }
}

pub struct Pollee {
    inner: RwLock<PolleeInner>,
}

struct PolleeInner {
    pollers: Vec<Arc<Poller>>,
    event: IoEvent,
}

impl Pollee {
    pub const fn new() -> Self {
        Self {
            inner: RwLock::new(PolleeInner {
                pollers: Vec::new(),
                event: IoEvent::empty(),
            }),
        }
    }

    pub fn register_poller(&self, poller: Arc<Poller>) {
        let event = self.inner.read().event;
        if event.contains(poller.required) {
            poller.short_path(event);
            return;
        }

        let mut inner = self.inner.write();
        inner.pollers.push(poller);
    }

    pub fn update_event(&self, f: impl FnOnce(&mut IoEvent)) {
        let mut inner = self.inner.write();
        f(&mut inner.event);
        for poller in &inner.pollers {
            poller.set_event(inner.event);
        }
    }
}
