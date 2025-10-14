use events::{Event, EventFilter};

use crate::{Signal, SignalMask};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalEvent(Signal);

impl SignalEvent {
    pub fn new(signal: Signal) -> Self {
        SignalEvent(signal)
    }

    pub fn signal(&self) -> Signal {
        self.0
    }
}

impl Event for SignalEvent {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalEventFilter(SignalMask);

impl SignalEventFilter {
    pub fn new(mask: SignalMask) -> Self {
        SignalEventFilter(mask)
    }
}

impl EventFilter<SignalEvent> for SignalEventFilter {
    fn filter(&self, event: SignalEvent) -> bool {
        !self.0.contains(event.0)
    }
}
