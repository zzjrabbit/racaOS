use bitflags::bitflags;
use events::{Event, EventFilter};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct IoEvent: u32 {
        const IN    = 0x0001;
        const PRI   = 0x0002;
        const OUT   = 0x0004;
        const ERR   = 0x0008;
        const HUP   = 0x0010;
        const NVAL  = 0x0020;
        const RDHUP = 0x2000;
        /// Events that are always polled even without specifying them.
        const ALWAYS_POLL = Self::ERR.bits() | Self::HUP.bits();
    }
}

impl Event for IoEvent {}

impl EventFilter<IoEvent> for IoEvent {
    fn filter(&self, events: &IoEvent) -> bool {
        self.intersects(*events)
    }
}
