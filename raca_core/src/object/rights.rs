use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy)]
    pub struct Rights: u32 {
        const DUPLICATE = 1 << 0;
        const TRANSFER = 1 << 1;
        const READ = 1 << 2;
        const WRITE = 1 << 3;
        const EXECUTE = 1 << 4;
        const DEFAULT_PROCESS = 1 << 5;
    }


}
