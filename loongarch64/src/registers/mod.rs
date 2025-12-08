pub use dmw::*;
pub use int::*;
pub use ipi::*;
pub use mode::*;
pub use paging::*;

mod dmw;
mod int;
mod ipi;
mod mode;
mod paging;

#[macro_export]
macro_rules! define_csr {
    ($csr_ident: ident, $csr_number: literal) => {
        define_csr!(@define $csr_ident);

        impl $csr_ident {
            pub fn read(&self) -> u64 {
                unsafe {
                    let bits: u64;
                    core::arch::asm!("csrrd {}, {}", out(reg) bits, const $csr_number);
                    bits
                }
            }

            pub fn write(&self, value: u64) {
                unsafe {
                    core::arch::asm!("csrwr {}, {}", in(reg) value, const $csr_number);
                }
            }
        }
    };

    (read $csr_ident: ident, $csr_number: literal) => {
        define_csr!(@define $csr_ident);

        impl $csr_ident {
            pub fn read(&self) -> u64 {
                unsafe {
                    let bits: u64;
                    core::arch::asm!("csrrd {}, {}", out(reg) bits, const $csr_number);
                    bits
                }
            }
        }
    };

    (write $csr_ident: ident, $csr_number: literal) => {
        define_csr!(@define $csr_ident);

        impl $csr_ident {
            pub fn write(&self, value: u64) {
                unsafe {
                    core::arch::asm!("csrwr {}, {}", in(reg) value, const $csr_number);
                }
            }
        }
    };

    (@define $csr_ident: ident) => {
        #[derive(Clone, Copy)]
        #[doc = concat!("CSR ", stringify!($csr_ident))]
        pub struct $csr_ident;
    }
}

#[macro_export]
macro_rules! read_csr {
    ($csr_number: literal) => {
        unsafe {
            let bits: u64;
            core::arch::asm!("csrrd {}, {}", out(reg) bits, const $csr_number);
            bits
        }
    };
}

#[macro_export]
macro_rules! write_csr {
    ($csr_number: literal, $value: expr) => {
        unsafe {
            core::arch::asm!("csrwr {}, {}", in(reg) $value, const $csr_number);
        }
    };
}
