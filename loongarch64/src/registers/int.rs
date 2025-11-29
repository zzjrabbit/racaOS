use crate::define_csr;

define_csr!(ExceptionEntry, 0xc);
define_csr!(ExceptionConfig, 0x4);
define_csr!(ExceptionStatus, 0x5);
define_csr!(ExceptionReturnAddress, 0x6);
define_csr!(BadVirtAddr, 0x7);

define_csr!(TimerConfig, 0x41);
define_csr!(TimerIntClear, 0x44);
