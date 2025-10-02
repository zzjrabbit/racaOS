use core::{
    any::Any,
    sync::atomic::{AtomicBool, Ordering},
};

use alloc::{
    boxed::Box,
    sync::{Arc, Weak},
};

use ostd::task::Task;

pub use kernel::*;
pub use user::*;

mod kernel;
mod user;

#[allow(dead_code)]
pub struct Thread {
    task: Weak<Task>,
    data: Box<dyn Any + Send + Sync>,
    callbacks: Box<dyn CallBacks>,
    dead: AtomicBool,
}

impl Thread {
    fn new(
        task: Weak<Task>,
        data: Box<dyn Any + Send + Sync>,
        callbacks: Box<dyn CallBacks>,
    ) -> Self {
        Self {
            task,
            data,
            callbacks,
            dead: AtomicBool::new(false),
        }
    }
}

trait CallBacks: Sync + Send {
    fn on_exit(&self);
    fn on_kill(&self);

    fn pre_execute(&self);
}

impl Thread {
    pub fn data(&self) -> &Box<dyn Any + Send + Sync> {
        &self.data
    }
}

impl Thread {
    pub fn pre_execute(&self) {
        self.callbacks.pre_execute();
    }
}

impl Thread {
    pub fn exit(&self) {
        self.dead.store(true, Ordering::SeqCst);

        self.callbacks.on_exit();
    }

    pub fn on_kill(&self) {
        self.dead.store(true, Ordering::SeqCst);

        self.callbacks.on_kill();
    }

    pub fn is_dead(&self) -> bool {
        self.dead.load(Ordering::SeqCst)
    }
}

pub trait AsThread {
    fn as_thread(&self) -> Option<&Arc<Thread>>;
    fn direct_downcast<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.as_thread()
            .and_then(|thread| thread.data().downcast_ref::<T>())
    }
}

impl AsThread for Task {
    fn as_thread(&self) -> Option<&Arc<Thread>> {
        self.data().downcast_ref::<Arc<Thread>>()
    }
}
