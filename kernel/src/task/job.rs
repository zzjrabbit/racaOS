use super::{
    job_policy::{BasicPolicy, JobPolicy, SetPolicyOptions},
    process::Process,
};

use crate::{
    error::{RcError, RcResult},
    kernel_object,
    object::{KObjectBase, KernelObject, KoID, Signal},
};
use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use spin::{Lazy, Mutex};

pub static ROOT_JOB: Lazy<Arc<Job>> = Lazy::new(Job::root);

kernel_object! {
    pub struct Job {
        parent: Option<Arc<Job>> = None,
        parent_policy: JobPolicy = JobPolicy::default(),
        inner: spin::Mutex<JobInner> = spin::Mutex::new(JobInner::default()),
    }

    fn new() {}

    fn get_child(&self, id: KoID) -> crate::error::RcResult<Arc<dyn crate::object::KernelObject>> {
        let inner = self.inner.lock();

        if let Some(job) = inner.children.iter().filter_map(|o|o.upgrade()).find(|o| o.id() == id) {
            return Ok(job);
        }
        if let Some(proc) = inner.processes.iter().find(|o| o.id() == id) {
            return Ok(proc.clone());
        }
        Err(RcError::NotFound)
    }

    fn related_koid(&self) -> KoID {
        self.parent.as_ref().map(|p| p.id()).unwrap_or(0)
    }

}

impl Job {
    pub fn root() -> Arc<Self> {
        let job = Arc::new(Self {
            base: KObjectBase::default(),
            parent: None,
            parent_policy: JobPolicy::default(),
            inner: Mutex::new(JobInner::default()),
        });
        job.inner.lock().self_ref = Arc::downgrade(&job);
        job
    }

    pub fn create_child(self: &Arc<Self>) -> RcResult<Arc<Self>> {
        let mut inner = self.inner.lock();
        if inner.killed {
            return Err(RcError::BadState);
        }
        let child = Arc::new(Job {
            base: KObjectBase::default(),
            parent: Some(self.clone()),
            parent_policy: inner.policy.merge(&self.parent_policy),
            inner: Mutex::new(JobInner::default()),
        });
        let child_weak = Arc::downgrade(&child);
        child.inner.lock().self_ref = child_weak.clone();
        inner.children.push(child_weak);
        Ok(child)
    }

    fn remove_child(&self, to_remove: &Weak<Job>) {
        {
            let mut inner = self.inner.lock();
            inner.children.retain(|child| !to_remove.ptr_eq(child));
        }
        if self.is_empty() {
            self.kill();
        }
    }

    /// Get the policy of the job.
    pub fn policy(&self) -> JobPolicy {
        self.inner.lock().policy.merge(&self.parent_policy)
    }

    /// Get the parent job.
    pub fn parent(&self) -> Option<Arc<Self>> {
        self.parent.clone()
    }

    /// Sets one or more security and/or resource policies to an empty job.
    ///
    /// The job's effective policies is the combination of the parent's
    /// effective policies and the policies specified in policy.
    ///
    /// After this call succeeds any new child process or child job will have
    /// the new effective policy applied to it.
    pub fn set_policy_basic(
        &self,
        options: SetPolicyOptions,
        policies: &[BasicPolicy],
    ) -> RcResult<()> {
        let mut inner = self.inner.lock();
        if !inner.is_empty() {
            return Err(RcError::BadState);
        }
        for policy in policies {
            if self.parent_policy.get_action(policy.condition).is_some() {
                match options {
                    SetPolicyOptions::Absolute => return Err(RcError::AlreadyExists),
                    SetPolicyOptions::Relative => {}
                }
            } else {
                inner.policy.apply(*policy);
            }
        }
        Ok(())
    }

    /// Add a process to the job.
    pub(super) fn add_process(&self, process: Arc<Process>) -> RcResult<()> {
        let mut inner = self.inner.lock();
        if inner.killed {
            return Err(RcError::BadState);
        }
        inner.processes.push(process);
        Ok(())
    }

    /// Remove a process from the job.
    pub(super) fn remove_process(&self, id: KoID) {
        {
            let mut inner = self.inner.lock();
            inner.processes.retain(|proc| proc.id() != id);
        }

        if self.is_empty() {
            self.kill();
        }
    }

    /// Check whether this job is root job.
    pub fn check_root_job(&self) -> RcResult<()> {
        if self.parent.is_some() {
            Err(RcError::AccessDenied)
        } else {
            Ok(())
        }
    }

    /// Get KoIDs of Processes.
    pub fn process_ids(&self) -> Vec<KoID> {
        self.inner.lock().processes.iter().map(|p| p.id()).collect()
    }

    /// Get KoIDs of children Jobs.
    pub fn children_ids(&self) -> Vec<KoID> {
        self.inner
            .lock()
            .children
            .iter()
            .filter_map(|j| j.upgrade())
            .map(|j| j.id())
            .collect()
    }

    /// Return true if this job has no processes and no child jobs.
    pub fn is_empty(&self) -> bool {
        self.inner.lock().is_empty()
    }

    pub fn kill(&self) {
        self.inner.lock().killed = true;
        self.set_signal(Signal::TASK_DEAD);
        for child_job in self.inner.lock().children.iter() {
            if let Some(child_job) = child_job.upgrade() {
                child_job.kill();
            }
        }

        for process in self.inner.lock().processes.iter() {
            process.kill();
        }
    }
}

#[derive(Default)]
struct JobInner {
    policy: JobPolicy,
    children: Vec<Weak<Job>>,
    processes: Vec<Arc<Process>>,
    killed: bool,
    self_ref: Weak<Job>,
}

impl JobInner {
    fn is_empty(&self) -> bool {
        self.processes.is_empty() && self.children.is_empty()
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        self.kill();
        //self.terminate();
    }
}
