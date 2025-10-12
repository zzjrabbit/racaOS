use derive_more::{Add, AddAssign, Deref, DerefMut, Display, Sub, SubAssign};

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Display,
    Deref,
    DerefMut,
    Add,
    AddAssign,
    Sub,
    SubAssign,
)]
pub struct Signal(u8);

#[allow(dead_code)]
impl Signal {
    pub(crate) const STD_SIGNAL_NUM: usize = 31;
    pub(crate) const RT_SIGNAL_NUM: usize = 33;
    pub(crate) const SIGNAL_NUM: usize = 64;

    pub(crate) const MIN_STD_SIGNAL: Self = Self(1);
    /// Inclusive
    pub(crate) const MAX_STD_SIGNAL: Self = Self(31);

    pub(crate) const MIN_RT_SIGNAL: Self = Self(32);
    /// Inclusive
    pub(crate) const MAX_RT_SIGNAL: Self = Self(64);
}

impl TryFrom<u8> for Signal {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > Self::SIGNAL_NUM as u8 {
            Err(())
        } else {
            Ok(Self(value))
        }
    }
}

impl From<Signal> for u8 {
    fn from(signal: Signal) -> Self {
        signal.0
    }
}

macro_rules! define_std_signums {
    ( $( $name: ident = $num: expr ),+, ) => {
        $(
            pub const $name : Signal = Signal($num);
        )*
    }
}

impl Signal {
    define_std_signums! {
        SIGHUP    = 1, // Hangup detected on controlling terminal or death of controlling process
        SIGINT    = 2, // Interrupt from keyboard
        SIGQUIT   = 3, // Quit from keyboard
        SIGILL    = 4, // Illegal Instruction
        SIGTRAP   = 5, // Trace/breakpoint trap
        SIGABRT   = 6, // Abort signal from abort(3)
        SIGBUS    = 7, // Bus error (bad memory access)
        SIGFPE    = 8, // Floating-point exception
        SIGKILL   = 9, // Kill signal
        SIGUSR1   = 10, // User-defined signal 1
        SIGSEGV   = 11, // Invalid memory reference
        SIGUSR2   = 12, // User-defined signal 2
        SIGPIPE   = 13, // Broken pipe: write to pipe with no readers; see pipe(7)
        SIGALRM   = 14, // Timer signal from alarm(2)
        SIGTERM   = 15, // Termination signal
        SIGSTKFLT = 16, // Stack fault on coprocessor (unused)
        SIGCHLD   = 17, // Child stopped or terminated
        SIGCONT   = 18, // Continue if stopped
        SIGSTOP   = 19, // Stop process
        SIGTSTP   = 20, // Stop typed at terminal
        SIGTTIN   = 21, // Terminal input for background process
        SIGTTOU   = 22, // Terminal output for background process
        SIGURG    = 23, // Urgent condition on socket (4.2BSD)
        SIGXCPU   = 24, // CPU time limit exceeded (4.2BSD); see setrlimit(2)
        SIGXFSZ   = 25, // File size limit exceeded (4.2BSD); see setrlimit(2)
        SIGVTALRM = 26, // Virtual alarm clock (4.2BSD)
        SIGPROF   = 27, // Profiling timer expired
        SIGWINCH  = 28, // Window resize signal (4.3BSD, Sun)
        SIGIO     = 29, // I/O now possible (4.2BSD)
        SIGPWR    = 30, // Power failure (System V)
        SIGSYS    = 31, // Bad system call (SVr4); see also seccomp(2)
    }
}
