use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{collections::btree_map::BTreeMap, sync::Weak, vec::Vec};
use ostd::sync::{LocalIrqDisabled, SpinLock};

use crate::{Event, EventFilter, Observer};

pub struct Subject<E: Event, F: EventFilter<E> = ()> {
    observers: SpinLock<BTreeMap<ObserverWrapper<E>, F>, LocalIrqDisabled>,
    num_observers: AtomicUsize,
}

impl<E: Event, F: EventFilter<E>> Subject<E, F> {
    pub fn new() -> Self {
        Self {
            observers: SpinLock::new(BTreeMap::new()),
            num_observers: AtomicUsize::new(0),
        }
    }
}

impl<E: Event, F: EventFilter<E>> Subject<E, F> {
    pub fn register_observer(&self, observer: Weak<dyn Observer<E>>, filter: F) {
        let mut observers = self.observers.lock();
        let is_new = observers
            .insert(ObserverWrapper(observer), filter)
            .is_none();
        if is_new {
            self.num_observers.fetch_add(1, Ordering::Acquire);
        }
    }

    pub fn unregister_observer(
        &self,
        observer: &Weak<dyn Observer<E>>,
    ) -> Option<Weak<dyn Observer<E>>> {
        let observer = ObserverWrapper(observer.clone());
        let mut observers = self.observers.lock();
        let observer = observers
            .remove_entry(&observer)
            .map(|(observer, _)| observer.0);
        if observer.is_some() {
            self.num_observers.fetch_sub(1, Ordering::Relaxed);
        }
        observer
    }

    pub fn notify_observers(&self, event: E) {
        if self.num_observers.fetch_add(0, Ordering::Release) == 0 {
            return;
        }

        let mut active_observers = Vec::new();
        let mut num_freed = 0;
        let mut observers = self.observers.lock();
        observers.retain(|observer, filter| {
            if let Some(observer) = observer.0.upgrade() {
                if filter.filter(event) {
                    active_observers.push(observer.clone());
                }
                true
            } else {
                num_freed += 1;
                false
            }
        });
        if num_freed > 0 {
            self.num_observers.fetch_sub(num_freed, Ordering::Relaxed);
        }
        drop(observers);

        for observer in active_observers {
            observer.on_event(event);
        }
    }
}

struct ObserverWrapper<E: Event>(Weak<dyn Observer<E>>);

impl<E: Event> PartialEq for ObserverWrapper<E> {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr().cast::<()>() == other.0.as_ptr().cast::<()>()
    }
}

impl<E: Event> PartialOrd for ObserverWrapper<E> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.0
            .as_ptr()
            .cast::<()>()
            .partial_cmp(&other.0.as_ptr().cast::<()>())
    }
}

impl<E: Event> Ord for ObserverWrapper<E> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0
            .as_ptr()
            .cast::<()>()
            .cmp(&other.0.as_ptr().cast::<()>())
    }
}

impl<E: Event> Eq for ObserverWrapper<E> {}
