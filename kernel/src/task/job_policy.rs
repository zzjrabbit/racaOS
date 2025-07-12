/// Security and resource policies of a job.
#[derive(Default, Copy, Clone)]
pub struct JobPolicy {
    // TODO: use bitset
    action: [Option<PolicyAction>; 15],
}

impl JobPolicy {
    /// Get the action of a policy `condition`.
    pub fn get_action(&self, condition: PolicyCondition) -> Option<PolicyAction> {
        self.action[condition as usize]
    }

    /// Apply a basic policy.
    pub fn apply(&mut self, policy: BasicPolicy) {
        self.action[policy.condition as usize] = Some(policy.action);
    }

    /// Merge the policy with `parent`'s.
    pub fn merge(&self, parent: &Self) -> Self {
        let mut new = *self;
        for i in 0..15 {
            if parent.action[i].is_some() {
                new.action[i] = parent.action[i];
            }
        }
        new
    }
}

/// Control the effect in the case of conflict between
/// the existing policies and the new policies when setting new policies.
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum SetPolicyOptions {
    /// Policy is applied for all conditions in policy or the call fails.
    Absolute = 0,
    /// Policy is applied for the conditions not specifically overridden by the parent policy.
    Relative = 1,
}

/// The policy type.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BasicPolicy {
    /// Condition when the policy is applied.
    pub condition: PolicyCondition,
    ///
    pub action: PolicyAction,
}

/// The condition when a policy is applied.
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum PolicyCondition {
    /// A process under this job is attempting to map an address region with write-execute access.
    VmarWx = 1,
    // A process under this job is attempting to create a new DDK( Driver Development Kit ) Object.
    NewDdkObject = 2,
}

/// The action taken when the condition happens specified by a policy.
#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PolicyAction {
    /// Allow condition.
    Allow = 0,
    /// Prevent condition.
    Deny = 1,
    /// Terminate the process.
    Kill = 2,
}
