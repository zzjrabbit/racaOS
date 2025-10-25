#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Alive,
    Zombie(i32),
}

impl ProcessStatus {
    pub fn is_alive(&self) -> bool {
        matches!(self, ProcessStatus::Alive)
    }

    pub fn is_zombie(&self) -> bool {
        matches!(self, ProcessStatus::Zombie(_))
    }

    pub fn exit_code(&self) -> Option<i32> {
        match self {
            ProcessStatus::Alive => None,
            ProcessStatus::Zombie(code) => Some(*code),
        }
    }
}
