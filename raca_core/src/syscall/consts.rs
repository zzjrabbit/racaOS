// Generated by build.rs. DO NOT EDIT.
use numeric_enum_macro::numeric_enum;

numeric_enum! {
    #[repr(u32)]
    #[derive(Debug, Eq, PartialEq)]
    #[allow(non_camel_case_types)]
    #[allow(clippy::upper_case_acronyms)]
    pub enum SyscallType {
        DEBUG = 0,
    }
}
