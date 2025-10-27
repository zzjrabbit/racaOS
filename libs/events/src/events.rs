pub trait Event: Clone + Copy + Send + Sync + 'static {}

impl Event for () {}

pub trait EventFilter<E: Event>: Send + Sync + 'static {
    fn filter(&self, event: &E) -> bool;
}

impl<E: Event> EventFilter<E> for () {
    fn filter(&self, _event: &E) -> bool {
        true
    }
}

impl<E: Event> EventFilter<E> for fn(&E) -> bool {
    fn filter(&self, event: &E) -> bool {
        self(event)
    }
}

impl<E: Event, I: EventFilter<E>> EventFilter<E> for Option<I> {
    fn filter(&self, event: &E) -> bool {
        self.as_ref().is_none_or(|f| f.filter(event))
    }
}
