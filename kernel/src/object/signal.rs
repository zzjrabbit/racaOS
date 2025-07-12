bitflags::bitflags! {
    #[derive(Default, Clone, Copy, Debug)]
    pub struct Signal: u32 {
        const READABLE = 1 << 0;

        const INTERRUPT_PRESENT = 1 << 1;

        const PEER_CLOSED = 1 << 2;

        const TASK_DEAD = 1 << 3;
        
        const USER_SIGNAL0 = 1 << 24;
        const USER_SIGNAL1 = 1 << 25;
        const USER_SIGNAL2 = 1 << 26;
        const USER_SIGNAL3 = 1 << 27;
        const USER_SIGNAL4 = 1 << 28;
        const USER_SIGNAL5 = 1 << 29;
        const USER_SIGNAL6 = 1 << 30;
        const USER_SIGNAL7 = 1 << 31;
    }
}
