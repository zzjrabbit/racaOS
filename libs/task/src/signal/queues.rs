use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{collections::vec_deque::VecDeque, sync::Weak, vec::Vec};
use events::{Observer, Subject};
use ostd::sync::Mutex;

use crate::{Signal, SignalEvent, SignalEventFilter, SignalKind, SignalMask, SignalSet};

pub struct SignalQueue {
    count: AtomicUsize,
    inner: Mutex<SignalQueueInner>,
    subject: Subject<SignalEvent, SignalEventFilter>,
}

struct SignalQueueInner {
    std_signals: Vec<Option<SignalKind>>,
    rt_signals: Vec<VecDeque<SignalKind>>,
}

impl Default for SignalQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalQueue {
    pub fn new() -> Self {
        let std_signals = (0..Signal::STD_SIGNAL_NUM).map(|_| None).collect();
        let rt_signals = (0..Signal::RT_SIGNAL_NUM)
            .map(|_| VecDeque::new())
            .collect();
        SignalQueue {
            count: AtomicUsize::new(0),
            inner: Mutex::new(SignalQueueInner {
                std_signals,
                rt_signals,
            }),
            subject: Subject::new(),
        }
    }
}

impl SignalQueue {
    pub fn is_empty(&self) -> bool {
        self.count.load(Ordering::Relaxed) == 0
    }

    pub fn enqueue(&self, signal_kind: SignalKind) {
        let signal = signal_kind.signal();

        let mut inner = self.inner.lock();
        if signal.is_std() {
            if inner.std_signals[Self::signal_to_id(signal)].is_none() {
                inner.std_signals[Self::signal_to_id(signal)] = Some(signal_kind);
                self.count.fetch_add(1, Ordering::Acquire);
            }
        } else {
            let id = Self::signal_to_id(signal);
            inner.rt_signals[id].push_back(signal_kind);
            self.count.fetch_add(1, Ordering::Acquire);
        }

        self.subject.notify_observers(SignalEvent::new(signal));
    }

    pub fn dequeue(&self, blocked: &SignalMask) -> Option<SignalKind> {
        if self.is_empty() {
            return None;
        }

        let mut inner = self.inner.lock();

        const ORDERED_STD_SIGS: [Signal; Signal::STD_SIGNAL_NUM] = [
            Signal::SIGKILL,
            Signal::SIGTERM,
            Signal::SIGSTOP,
            Signal::SIGSEGV,
            Signal::SIGILL,
            Signal::SIGHUP,
            Signal::SIGCONT,
            Signal::SIGINT,
            Signal::SIGQUIT,
            Signal::SIGTRAP,
            Signal::SIGABRT,
            Signal::SIGBUS,
            Signal::SIGFPE,
            Signal::SIGUSR1,
            Signal::SIGUSR2,
            Signal::SIGPIPE,
            Signal::SIGALRM,
            Signal::SIGSTKFLT,
            Signal::SIGCHLD,
            Signal::SIGTSTP,
            Signal::SIGTTIN,
            Signal::SIGTTOU,
            Signal::SIGURG,
            Signal::SIGXCPU,
            Signal::SIGXFSZ,
            Signal::SIGVTALRM,
            Signal::SIGPROF,
            Signal::SIGWINCH,
            Signal::SIGIO,
            Signal::SIGPWR,
            Signal::SIGSYS,
        ];

        for &signal in &ORDERED_STD_SIGS {
            if blocked.contains(signal) {
                continue;
            }

            if let Some(signal_kind) = inner.std_signals[Self::signal_to_id(signal)].take() {
                self.count.fetch_sub(1, Ordering::Acquire);
                return Some(signal_kind);
            }
        }

        for signal in *Signal::MIN_RT_SIGNAL..=*Signal::MAX_RT_SIGNAL {
            let signal = Signal::try_from(signal).unwrap();
            if blocked.contains(signal) {
                continue;
            }

            let queue = &mut inner.rt_signals[Self::signal_to_id(signal)];
            if let Some(signal_kind) = queue.pop_front() {
                self.count.fetch_sub(1, Ordering::Acquire);
                return Some(signal_kind);
            }
        }

        None
    }

    pub fn register_observer(
        &self,
        observer: Weak<dyn Observer<SignalEvent>>,
        filter: SignalEventFilter,
    ) {
        self.subject.register_observer(observer, filter);
    }

    pub fn unregister_observer(&self, observer: &Weak<dyn Observer<SignalEvent>>) {
        self.subject.unregister_observer(observer);
    }

    pub fn signal_pending(&self) -> SignalSet {
        let mut pending = SignalSet::new_empty();

        let inner = self.inner.lock();

        // Process standard signal queues
        for (idx, signal) in inner.std_signals.iter().enumerate() {
            if signal.is_some() {
                pending += Signal::try_from(idx as u8 + u8::from(Signal::MIN_STD_SIGNAL)).unwrap();
            }
        }

        // Process real-time signal queues
        for (idx, signals) in inner.rt_signals.iter().enumerate() {
            if !signals.is_empty() {
                pending += Signal::try_from(idx as u8 + u8::from(Signal::MIN_RT_SIGNAL)).unwrap();
            }
        }

        pending
    }

    pub fn has_pending(&self, blocked: SignalMask) -> bool {
        self.inner.lock().std_signals.iter().any(|signal| {
            signal
                .as_ref()
                .is_some_and(|signal| !blocked.contains(signal.signal()))
        }) || self
            .inner
            .lock()
            .rt_signals
            .iter()
            .any(|rt_queue| !rt_queue.is_empty())
    }
}

impl SignalQueue {
    fn signal_to_id(signal: Signal) -> usize {
        u8::from(signal - Signal::MIN_STD_SIGNAL) as usize
    }
}
