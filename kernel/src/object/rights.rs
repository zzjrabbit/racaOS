use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy)]
    pub struct Rights: u32 {
        const READ = 1 << 0;
        const WRTIE = 1 << 1;
        const EXECUTE = 1 << 2;
    }
}
