pub mod job;
pub mod job_policy;
pub mod process;
pub mod scheduler;
pub mod thread;

/// The return code set when a task is killed via rc_task_kill().
pub const TASK_RETCODE_SYSCALL_KILL: i64 = -1028;
