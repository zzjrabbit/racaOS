use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use alloc::{sync::Arc, vec::Vec};
use ostd::{
    arch::cpu::context::{CpuException, UserContext},
    mm::{CachePolicy, FrameAllocOptions, PageFlags, PageProperty, PAGE_SIZE},
    task::{disable_preempt, Task, TaskOptions},
    user::{ReturnReason, UserMode},
    Error as OstdError,
};
use spin::RwLock;

use crate::{
    filesystem::File,
    mem::VmReadWrite,
    syscall::syscall_handler,
    task::ThreadData,
    trap::user_page_fault_handler,
};
use loader::BinaryLoader;

mod loader;

static PROCESSES: RwLock<Vec<Arc<Process>>> = RwLock::new(Vec::new());

pub struct Process {
    threads: RwLock<Vec<Arc<Task>>>,
    is_child_process: AtomicBool,
    exit_code: AtomicI32,
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

                data.memory_info().vm_space().activate();
                let mut user_context = UserContext::default();
                user_context.set_rip(*data.entry.get().unwrap());
                user_context.set_rsp(*data.stack.get().unwrap());

                UserMode::new(user_context)
            };
            
            loop {
                {
                    let current = Task::current().unwrap();
                    let data = current.data().downcast_ref::<ThreadData>().unwrap();
                    
                    if data.is_dead() {
                        break;
                    }
                }
                
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
            exit_code: AtomicI32::new(0),
        });

        PROCESSES.write().push(new_self.clone());

        let thread_data = ThreadData::new(stdin, stdout, stderr, &new_self);

        let entry = thread_data.memory_info().vm_space().load(binary)?;

        let stack_region = thread_data.memory_info().allocate(USER_STACK_SIZE)?;
        const USER_STACK_SIZE: usize = 8 * 1024 * 1024;

        let user_stack_end = stack_region.end_address();

        let disable_preempt_guard = disable_preempt();
        
        let vm_space = thread_data
            .memory_info()
            .vm_space();
        
        let mut cursor = vm_space.cursor_mut(
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
        vm_space.write_val(envp, &0usize).unwrap();

        let path = c"hello";
        let argv = envp - path.count_bytes() - 1;
        for (id, byte) in path.to_bytes_with_nul().iter().enumerate() {
            vm_space.write_val(argv + id, byte).unwrap();
        }

        let aligned_argv = argv - 2;

        let auxv_ptr = aligned_argv - 2 * size_of::<usize>();
        vm_space.write_val(auxv_ptr, &0usize).unwrap();
        vm_space.write_val(auxv_ptr + size_of::<usize>(), &0usize)
            .unwrap();

        // write envp
        let envp_ptr = auxv_ptr - size_of::<usize>();
        vm_space.write_val(envp_ptr, &envp).unwrap();

        // write argv
        let argv_ptr = envp_ptr - size_of::<usize>();
        vm_space.write_val(argv_ptr, &argv).unwrap();

        // write argc
        let argc_ptr = argv_ptr - size_of::<usize>();
        vm_space.write_val(argc_ptr, &1usize).unwrap();

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
    
    pub fn remove_thread(&self, tid: usize) {
        self.threads.write().retain(|t| {
            let t_data = t.data().downcast_ref::<ThreadData>().unwrap();
            t_data.tid() != tid
        });
    }
}

#[allow(dead_code)]
impl Process {
    pub fn exit(&self, exit_code: i32) {
        self.exit_code.store(exit_code, Ordering::SeqCst);
        for thread in self.threads.read().clone().iter() {
            let thread_data = thread.data().downcast_ref::<ThreadData>().unwrap();
            thread_data.on_exit();
        }
    }

    pub fn kill(&self) {
        self.exit_code.store(-1, Ordering::SeqCst);
        for thread in self.threads.read().iter() {
            let thread_data = thread.data().downcast_ref::<ThreadData>().unwrap();
            thread_data.on_kill();
        }
    }
    
    pub fn exit_code(&self) -> Option<i32> {
        if self.threads.read().is_empty() {
            Some(self.exit_code.load(Ordering::SeqCst))
        } else {
            None
        }
    }
}
