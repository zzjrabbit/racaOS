pub use crate::arch::context::{TaskContext, UserContext};

pub trait TaskContextApi {
    fn set_program_counter(&mut self, pc: usize);
    fn set_stack_pointer(&mut self, sp: usize);
}
