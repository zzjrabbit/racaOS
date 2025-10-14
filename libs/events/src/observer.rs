use crate::Event;

pub trait Observer<E: Event>: Send + Sync {
    fn on_event(&self, event: E);
}

impl<E: Event> Observer<E> for () {
    fn on_event(&self, _event: E) {}
}
