use core::sync::atomic::{AtomicBool, Ordering};

use alloc::{sync::Arc, vec::Vec};
use spin::RwLock;
use zodiac::{
    ZodiacError,
    hal::context::CpuException,
    mem::{MMUFlags, PhysicalMemoryAllocOptions},
    task::{ReturnReason, Task, TaskBuilder, UserContext},
};

use crate::{
    filesystem::File, syscall::syscall_handler, task::ThreadData, trap::user_page_fault_handler,
};

static PROCESSES: RwLock<Vec<Arc<Process>>> = RwLock::new(Vec::new());

pub struct Process {
    threads: RwLock<Vec<Arc<Task>>>,
    is_child_process: AtomicBool,
}

impl Process {
    pub fn new(
        binary: &[u8],
        stdin: Arc<File>,
        stdout: Arc<File>,
        stderr: Arc<File>,
    ) -> Result<Arc<Self>, ZodiacError> {
        fn thread_entry() -> ! {
            let mut user_context = {
                let task = Task::current();
                let data = task.data().downcast_ref::<ThreadData>().unwrap();

                UserContext::new(*data.entry.get().unwrap(), *data.stack.get().unwrap())
            };

            loop {
                log::info!("execute");
                let return_reason = user_context.excute(|| false);
                log::info!("return_reason: {:?}", return_reason);

                match return_reason {
                    ReturnReason::Syscall => {
                        syscall_handler(user_context.trap_frame());
                    }
                    ReturnReason::Exception(exception) => match exception {
                        CpuException::PageFault(_, _) => {
                            user_page_fault_handler(user_context.trap_frame(), exception)
                        }
                        _ => panic!("Unhandled exception: {:?}", exception),
                    },
                    _ => panic!("Thread error: {:x?}", return_reason),
                }
            }
        }

        let new_self = Arc::new(Self {
            threads: RwLock::new(Vec::new()),
            is_child_process: AtomicBool::new(false),
        });

        PROCESSES.write().push(new_self.clone());

        let thread_data = ThreadData::new(stdin, stdout, stderr, &new_self);

        let entry = thread_data.vm_space.binary_file_mapper().map(binary)?;

        let (stack_address, mut cursor, page_size) = thread_data.allocate(USER_STACK_SIZE, true)?;
        const USER_STACK_SIZE: usize = 8 * 1024 * 1024;

        let user_stack_end = stack_address + USER_STACK_SIZE;

        let physical_memory = PhysicalMemoryAllocOptions::default()
            .count(USER_STACK_SIZE / page_size as usize)
            .page_size(page_size)
            .allocate()
            .unwrap();
        cursor
            .map(
                &physical_memory,
                MMUFlags::READ | MMUFlags::WRITE | MMUFlags::USER,
            )
            .unwrap();
        thread_data
            .vm_space
            .writer(user_stack_end - size_of::<usize>(), size_of::<usize>())
            .write(&0usize.to_le_bytes())
            .unwrap();

        let envp = user_stack_end;
        thread_data
            .vm_space
            .writer(user_stack_end - 4 * size_of::<usize>(), size_of::<usize>())
            .write(&envp.to_le_bytes())
            .unwrap();

        thread_data.entry.call_once(|| entry as usize);
        thread_data
            .stack
            .call_once(|| user_stack_end - 6 * size_of::<usize>());

        let thread = TaskBuilder::default()
            .entry(thread_entry)
            .data(thread_data)
            .build()?;

        new_self.add_thread(thread.clone());

        thread.spawn();

        Ok(new_self)
    }

    pub fn current() -> Arc<Self> {
        let current_thread = Task::current();
        current_thread
            .data()
            .downcast_ref::<ThreadData>()
            .unwrap()
            .process
            .upgrade()
            .unwrap()
    }
}

impl Process {
    pub fn is_child_process(&self) -> bool {
        self.is_child_process.load(Ordering::SeqCst)
    }
}

impl Process {
    pub fn add_thread(&self, thread: Arc<Task>) {
        self.threads.write().push(thread);
    }
}

#[allow(dead_code)]
impl Process {
    pub fn exit(&self) -> ! {
        let current = Task::current();
        for thread in self.threads.read().iter() {
            if thread.task_id() != current.task_id() {
                thread.kill();
            }
        }
        current.exit();
    }

    pub fn kill(&self) {
        for thread in self.threads.read().iter() {
            thread.kill();
        }
    }
}
