use core::sync::atomic::{AtomicBool, Ordering};

use alloc::{sync::Arc, vec::Vec};
use ostd::{
    arch::cpu::context::{CpuException, UserContext},
    mm::{CachePolicy, FrameAllocOptions, PageFlags, PageProperty, PAGE_SIZE},
    task::{disable_preempt, halt_cpu, Task, TaskOptions},
    user::{ReturnReason, UserMode},
    Error as OstdError,
};
use spin::RwLock;

use crate::{
    filesystem::File,
    mem::VmReadWrite,
    syscall::syscall_handler,
    task::{BinaryLoader, ThreadData},
    trap::user_page_fault_handler,
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
    ) -> Result<Arc<Self>, OstdError> {
        fn thread_entry() {
            let mut user_mode = {
                let task = Task::current().unwrap();
                let data = task.data().downcast_ref::<ThreadData>().unwrap();

                data.vm_space.activate();
                let mut user_context = UserContext::default();
                user_context.set_rip(*data.entry.get().unwrap());
                user_context.set_rsp(*data.stack.get().unwrap());

                ostd::early_println!(
                    "program entry: {:x} {:x}",
                    user_context.rip(),
                    user_context.rsp()
                );

                UserMode::new(user_context)
            };

            loop {
                let return_reason = user_mode.execute(|| false);

                match return_reason {
                    ReturnReason::UserSyscall => {
                        syscall_handler(user_mode.context_mut());
                    }
                    ReturnReason::UserException => {
                        let context = user_mode.context_mut();
                        let exception = context.take_exception().unwrap();

                        match exception {
                            CpuException::PageFault(_) => {
                                user_page_fault_handler(&exception).unwrap()
                            }
                            _ => panic!("Unhandled exception: {:?}", exception),
                        }
                    }
                    _ => panic!("Thread error: {:x?}", return_reason),
                }

                Task::yield_now();
            }
        }

        let new_self = Arc::new(Self {
            threads: RwLock::new(Vec::new()),
            is_child_process: AtomicBool::new(false),
        });

        PROCESSES.write().push(new_self.clone());

        let thread_data = ThreadData::new(stdin, stdout, stderr, &new_self);

        let entry = thread_data.vm_space.load(binary)?;

        let stack_region = thread_data.allocate(USER_STACK_SIZE)?;
        const USER_STACK_SIZE: usize = 8 * 1024 * 1024;

        let user_stack_end = stack_region.end_address();

        let disable_preempt_guard = disable_preempt();
        let mut cursor = thread_data
            .vm_space
            .cursor_mut(
                &disable_preempt_guard,
                &(stack_region.start_address()..stack_region.end_address()),
            )
            .unwrap();

        for _ in 0..USER_STACK_SIZE / PAGE_SIZE {
            let frame = FrameAllocOptions::new().alloc_frame().unwrap();

            let property = PageProperty::new_user(PageFlags::RW, CachePolicy::Writeback);
            cursor.map(frame.into(), property);
        }
        drop(cursor);
        drop(disable_preempt_guard);

        let envp = user_stack_end - size_of::<usize>();
        thread_data.vm_space.write_val(envp, &0usize).unwrap();

        let path = c"hello";
        let argv = envp - path.count_bytes() - 1;
        for (id, byte) in path.to_bytes_with_nul().iter().enumerate() {
            thread_data.vm_space.write_val(argv + id, byte).unwrap();
        }

        let aligned_argv = argv - 2;

        let auxv_ptr = aligned_argv - 2 * size_of::<usize>();
        thread_data.vm_space.write_val(auxv_ptr, &0usize).unwrap();
        thread_data
            .vm_space
            .write_val(auxv_ptr + size_of::<usize>(), &0usize)
            .unwrap();

        // write envp
        let envp_ptr = auxv_ptr - size_of::<usize>();
        thread_data.vm_space.write_val(envp_ptr, &envp).unwrap();

        // write argv
        let argv_ptr = envp_ptr - size_of::<usize>();
        thread_data.vm_space.write_val(argv_ptr, &argv).unwrap();

        // write argc
        let argc_ptr = argv_ptr - size_of::<usize>();
        thread_data.vm_space.write_val(argc_ptr, &1usize).unwrap();

        thread_data.entry.call_once(|| entry as usize);
        thread_data.stack.call_once(|| argc_ptr);

        let thread = Arc::new(TaskOptions::new(thread_entry).data(thread_data).build()?);

        new_self.add_thread(thread.clone());

        thread.run();

        Ok(new_self)
    }

    pub fn current() -> Arc<Self> {
        let current_thread = Task::current().unwrap();
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
        /*let current = Task::current().unwrap();
        for thread in self.threads.read().iter() {
            if thread.data().downcast_ref::<ThreadData>().unwrap().tid() != current.data().downcast_ref::<ThreadData>().unwrap().tid() {
                thread.();
            }
        }
        current.exit();*/
        loop {
            halt_cpu();
        }
    }

    pub fn kill(&self) {
        /*for thread in self.threads.read().iter() {
            thread.kill();
        }*/
        unimplemented!()
    }
}
