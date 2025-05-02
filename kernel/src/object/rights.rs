use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct Rights: u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXECUTE = 1 << 2;
        const MAP = 1 << 3;
        const DUPLICATE = 1 << 4;
        const TRANSFER = 1 << 5;
        const DESTROY = 1 << 6;
        const MANAGE_JOB = 1 << 7;
        const MANAGE_PROCESS = 1 << 8;
        const MANAGE_THREAD = 1 << 9;
        const SAME_RIGHTS = 1 << 10;
        const GET_INFO = 1 << 11;

        const IO = Self::READ.bits() | Self::WRITE.bits();

        const BASIC = Self::TRANSFER.bits() | Self::DUPLICATE.bits() | Self::GET_INFO.bits();

        const DEFAULT_PROCESS = Self::BASIC.bits() | Self::DESTROY.bits() | Self::MANAGE_PROCESS.bits() | Self::MANAGE_THREAD.bits() | Self::IO.bits();
        const DEFAULT_THREAD = Self::BASIC.bits() | Self::DESTROY.bits() | Self::MANAGE_THREAD.bits() | Self::IO.bits();
        const DEFAULT_JOB = Self::BASIC.bits() | Self::DESTROY.bits() | Self::MANAGE_JOB.bits() | Self::IO.bits() | Self::MANAGE_PROCESS.bits() | Self::MANAGE_THREAD.bits();

        const DEFAULT_PHYSICAL_MEMORY = Self::BASIC.bits() | Self::IO.bits() | Self::MAP.bits();
        const DEFAULT_VIRTUAL_MEMORY = Self::BASIC.bits();

        const DEFAULT_FB = Self::BASIC.bits() | Self::IO.bits();

        const DEFAULT_CHANNEL = Self::BASIC.bits() & !Self::DUPLICATE.bits() | Self::IO.bits();
    }
}
