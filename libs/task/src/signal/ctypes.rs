#![allow(dead_code)]
#![allow(non_camel_case_types)]

use credentials::Uid;
use inherit_methods_macro::inherit_methods;
use ostd::{Pod, arch::cpu::context::UserContext, mm::Vaddr};

use crate::Signal;
#[cfg(target_arch = "x86_64")]
use crate::stack_t;

macro_rules! read_union_field {
    ($container:expr, $type:ty, $($field:tt)+) => {{
        // Perform type checking first.
        let container: &$type = $container;
        let reader = Reader::new(container);

        let field_offset = core::mem::offset_of!($type, $($field)*);
        let type_infer = ostd::ptr_null_of!({
            // This is not safe, but the code won't be executed.
            &raw const container.$($field)*
        });

        reader.read_at(field_offset, type_infer)
    }}
}

#[derive(Clone, Copy, Pod)]
#[repr(C)]
pub struct siginfo_t {
    pub si_signo: i32,
    pub si_errno: i32,
    pub si_code: i32,
    // In x86_64, there will be a 4-bytes padding here automatically, the offset of `siginfo_fields` is `0x10`.
    // Yet in other architectures like arm64, there is no padding here and the offset of `siginfo_fields` is `0x0c`.
    //_padding: i32,
    /// siginfo_fields should be a union type ( See occlum definition ). But union type have unsafe interfaces.
    /// Here we use a simple byte array.
    siginfo_fields: siginfo_fields_t,
}

impl siginfo_t {
    pub fn new(num: Signal, code: i32) -> Self {
        siginfo_t {
            si_signo: u8::from(num) as i32,
            si_errno: 0,
            si_code: code,
            siginfo_fields: siginfo_fields_t::zero_fields(),
        }
    }

    pub fn set_si_addr(&mut self, si_addr: Vaddr) {
        self.siginfo_fields.sigfault.addr = si_addr;
    }

    pub fn set_pid_uid(&mut self, pid: usize, uid: Uid) {
        let pid_uid = siginfo_common_first_t {
            piduid: siginfo_piduid_t { pid, uid },
        };

        self.siginfo_fields.common.first = pid_uid;
    }

    pub fn set_status(&mut self, status: i32) {
        self.siginfo_fields.common.second.sigchild.status = status;
    }

    pub fn si_addr(&self) -> Vaddr {
        read_union_field!(self, Self, siginfo_fields.sigfault.addr)
    }
}

#[derive(Clone, Copy, Pod)]
#[repr(C)]
union siginfo_fields_t {
    bytes: [u8; 128 - size_of::<i32>() * 4],
    common: siginfo_common_t,
    sigfault: siginfo_sigfault_t,
}

impl siginfo_fields_t {
    fn zero_fields() -> Self {
        Self {
            bytes: [0; 128 - size_of::<i32>() * 4],
        }
    }
}

#[derive(Clone, Copy, Pod)]
#[repr(C)]
struct siginfo_common_t {
    first: siginfo_common_first_t,
    second: siginfo_common_second_t,
}

#[derive(Clone, Copy, Pod)]
#[repr(C)]
union siginfo_common_first_t {
    piduid: siginfo_piduid_t,
    timer: siginfo_timer_t,
}

#[derive(Clone, Copy, Pod)]
#[repr(C)]
struct siginfo_piduid_t {
    pid: usize,
    uid: Uid,
}

#[derive(Clone, Copy, Pod)]
#[repr(C)]
struct siginfo_timer_t {
    timerid: i32,
    overrun: i32,
}

#[derive(Clone, Copy, Pod)]
#[repr(C)]
union siginfo_common_second_t {
    value: sigval_t,
    sigchild: siginfo_sigchild_t,
}

#[derive(Clone, Copy, Pod)]
#[repr(C)]
pub union sigval_t {
    sigval_int: i32,
    sigval_ptr: Vaddr, //*mut c_void
}

impl sigval_t {
    pub fn read_int(&self) -> i32 {
        read_union_field!(self, Self, sigval_int)
    }

    pub fn read_ptr(&self) -> Vaddr {
        read_union_field!(self, Self, sigval_ptr)
    }
}

pub type clock_t = i64;

#[derive(Clone, Copy, Pod)]
#[repr(C)]
union siginfo_sigchild_t {
    status: i32,
    utime: clock_t,
    stime: clock_t,
}

#[derive(Clone, Copy, Pod)]
#[repr(C)]
struct siginfo_sigfault_t {
    addr: Vaddr, //*const c_void
    addr_lsb: i16,
    first: siginfo_sigfault_first_t,
}

#[derive(Clone, Copy, Pod)]
#[repr(C)]
union siginfo_sigfault_first_t {
    addr_bnd: siginfo_addr_bnd_t,
    pkey: u32,
}

#[derive(Clone, Copy, Pod)]
#[repr(C)]
union siginfo_addr_bnd_t {
    lower: Vaddr, // *const c_void
    upper: Vaddr, // *const c_void,
}

/// Reference: <https://elixir.bootlin.com/linux/v6.15.7/source/include/uapi/asm-generic/ucontext.h#L5>
#[cfg(target_arch = "x86_64")]
#[derive(Clone, Copy, Debug, Default, Pod)]
#[repr(C)]
pub struct ucontext_t {
    pub uc_flags: u64,
    pub uc_link: Vaddr, // *mut ucontext_t
    pub uc_stack: stack_t,
    pub uc_mcontext: mcontext_t,
    pub uc_sigmask: sigset_t,
}

pub type sigset_t = u64;

#[derive(Debug, Clone, Copy, Pod, Default)]
#[repr(C)]
pub struct mcontext_t {
    inner: SignalContext,
}

#[inherit_methods(from = "self.inner")]
impl mcontext_t {
    pub fn copy_user_regs_to(&self, context: &mut UserContext);
    pub fn copy_user_regs_from(&mut self, context: &UserContext);
}

#[derive(Clone, Copy, Pod)]
#[repr(C)]
pub struct _sigev_thread {
    pub function: Vaddr,
    pub attribute: Vaddr,
}

const SIGEV_MAX_SIZE: usize = 64;
/// The total size of the fields `sigev_value`, `sigev_signo` and `sigev_notify`.
const SIGEV_PREAMBLE_SIZE: usize = size_of::<i32>() * 2 + size_of::<sigval_t>();
const SIGEV_PAD_SIZE: usize = (SIGEV_MAX_SIZE - SIGEV_PREAMBLE_SIZE) / size_of::<i32>();

#[derive(Clone, Copy, Pod)]
#[repr(C)]
pub union _sigev_un {
    pub _pad: [i32; SIGEV_PAD_SIZE],
    pub _tid: i32,
    pub _sigev_thread: _sigev_thread,
}

impl _sigev_un {
    pub fn read_tid(&self) -> i32 {
        read_union_field!(self, Self, _tid)
    }

    pub fn read_function(&self) -> Vaddr {
        read_union_field!(self, Self, _sigev_thread.function)
    }

    pub fn read_attribute(&self) -> Vaddr {
        read_union_field!(self, Self, _sigev_thread.attribute)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(i32)]
pub enum SigNotify {
    SIGEV_SIGNAL = 0,
    SIGEV_NONE = 1,
    SIGEV_THREAD = 2,
    SIGEV_THREAD_ID = 4,
}

#[derive(Clone, Copy, Pod)]
#[repr(C)]
pub struct sigevent_t {
    pub sigev_value: sigval_t,
    pub sigev_signo: i32,
    pub sigev_notify: i32,
    pub sigev_un: _sigev_un,
}

pub struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    pub fn new<T: Pod>(object: &'a T) -> Self {
        Self {
            bytes: object.as_bytes(),
        }
    }

    pub fn read_at<F: Pod>(&self, field_offset: usize, _type_infer: *const F) -> F {
        F::from_bytes(&self.bytes[field_offset..])
    }
}

#[derive(Clone, Copy, Debug, Default, Pod)]
#[repr(C)]
pub struct SignalContext {
    r8: usize,
    r9: usize,
    r10: usize,
    r11: usize,
    r12: usize,
    r13: usize,
    r14: usize,
    r15: usize,
    rdi: usize,
    rsi: usize,
    rbp: usize,
    rbx: usize,
    rdx: usize,
    rax: usize,
    rcx: usize,
    rsp: usize,
    rip: usize,
    rflags: usize,
    cs: u16,
    gs: u16,
    fs: u16,
    ss: u16,
    error_code: usize,
    trap_num: usize,
    old_mask: u64,
    page_fault_addr: usize,
    reserved: [u64; 8],
}

macro_rules! copy_gp_regs {
    ($src: ident, $dst: ident) => {
        $dst.rax = $src.rax;
        $dst.rbx = $src.rbx;
        $dst.rcx = $src.rcx;
        $dst.rdx = $src.rdx;
        $dst.rsi = $src.rsi;
        $dst.rdi = $src.rdi;
        $dst.rbp = $src.rbp;
        $dst.rsp = $src.rsp;
        $dst.r8 = $src.r8;
        $dst.r9 = $src.r9;
        $dst.r10 = $src.r10;
        $dst.r11 = $src.r11;
        $dst.r12 = $src.r12;
        $dst.r13 = $src.r13;
        $dst.r14 = $src.r14;
        $dst.r15 = $src.r15;
        $dst.rip = $src.rip;
        $dst.rflags = $src.rflags;
    };
}

impl SignalContext {
    pub fn copy_user_regs_to(&self, dst: &mut UserContext) {
        let gp_regs = dst.general_regs_mut();
        copy_gp_regs!(self, gp_regs);
    }

    pub fn copy_user_regs_from(&mut self, src: &UserContext) {
        let gp_regs = src.general_regs();
        copy_gp_regs!(gp_regs, self);

        // TODO: Fill exception information in `SigContext`.
    }
}
