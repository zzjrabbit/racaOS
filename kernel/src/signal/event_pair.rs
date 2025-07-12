use crate::error::*;
use crate::object::{KernelObject, KoID, Signal};
use alloc::sync::{Arc, Weak};

crate::kernel_object! {
    pub struct EventPair {
        peer: Weak<EventPair>,
    }

    fn peer(&self) -> RcResult<Arc<dyn KernelObject>> {
        let peer = self.peer.upgrade().ok_or(RcError::PeerClosed)?;
        Ok(peer)
    }

    fn related_koid(&self) -> KoID {
        self.peer.upgrade().map(|p| p.id()).unwrap_or(0)
    }
}

impl EventPair {
    pub fn new() -> (Arc<Self>, Arc<Self>) {
        let event_pair0 = Arc::new(Self {
            base: Default::default(),
            peer: Weak::default(),
        });
        let event_pair1 = Arc::new(EventPair {
            base: Default::default(),
            peer: Arc::downgrade(&event_pair0),
        });
        // no other reference of `channel0`
        unsafe { &mut *(Arc::as_ptr(&event_pair0) as *mut EventPair) }.peer =
            Arc::downgrade(&event_pair1);

        (event_pair0, event_pair1)
    }

    pub fn peer(&self) -> RcResult<Arc<Self>> {
        self.peer.upgrade().ok_or(RcError::PeerClosed)
    }
}

impl Drop for EventPair {
    fn drop(&mut self) {
        if let Some(peer) = self.peer.upgrade() {
            peer.set_signal(Signal::PEER_CLOSED);
        }
    }
}
