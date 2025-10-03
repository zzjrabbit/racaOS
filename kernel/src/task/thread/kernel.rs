use alloc::{boxed::Box, sync::Arc};
use ostd::task::{Task, TaskOptions};

use crate::task::{thread::CallBacks, Thread};

struct KernelThreadData;

struct KernelThreadCallbacks;

impl CallBacks for KernelThreadCallbacks {
    fn on_exit(&self) {}

    fn on_kill(&self) {}

    fn pre_execute(&self) {}
}

pub fn create_kernel_thread(entry: fn()) -> Arc<Task> {
    let thread = Arc::new_cyclic(|weak_task| {
        let thread = Thread::new(
            weak_task.clone(),
            Box::new(KernelThreadData),
            Box::new(KernelThreadCallbacks),
        );

        TaskOptions::new(entry)
            .data(Arc::new(thread))
            .build()
            .unwrap()
    });

    thread.run();

    thread
}
