use crate::{Signal, SignalAction};

#[derive(Clone, Copy)]
pub struct SignalDisposition {
    map: [SignalAction; Signal::SIGNAL_NUM],
}

impl Default for SignalDisposition {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalDisposition {
    pub fn new() -> Self {
        Self {
            map: [SignalAction::default(); Signal::SIGNAL_NUM],
        }
    }
}

impl SignalDisposition {
    pub fn get(&self, signal: Signal) -> SignalAction {
        let id = Self::signal_to_id(signal);
        self.map[id]
    }

    pub fn set(&mut self, signal: Signal, action: SignalAction) -> SignalAction {
        let id = Self::signal_to_id(signal);
        core::mem::replace(&mut self.map[id], action)
    }

    pub fn set_default(&mut self, signal: Signal) {
        let id = Self::signal_to_id(signal);
        self.map[id] = SignalAction::default();
    }

    pub fn inherit(&mut self) {
        for signal_action in &mut self.map {
            if let SignalAction::User { .. } = signal_action {
                *signal_action = SignalAction::default();
            }
        }
    }

    fn signal_to_id(signal: Signal) -> usize {
        u8::from(signal - Signal::MIN_STD_SIGNAL) as usize
    }
}
