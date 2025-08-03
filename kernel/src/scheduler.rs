use zodiac::task::{FifoScheduler, set_scheduler};

static SCHEDULER: FifoScheduler = FifoScheduler::new();

pub fn init() {
    set_scheduler(&SCHEDULER);
}
