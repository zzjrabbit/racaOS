use driver::Mmio;
use ostd::{Pod, mm::Paddr};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CommandHeader {
    pub flags: u16,
    pub prdt_length: u16,
    prd_byte_count: Paddr,
    pub command_table_base_address: u64,
    pub reserved: [u32; 4],
}

unsafe impl Pod for CommandHeader {}

impl CommandHeader {
    pub fn prdt_byte_count(&self) -> Mmio<u32> {
        Mmio::new(self.prd_byte_count).unwrap()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CommandTable {
    pub cfis: [u8; 64],
    pub acmd: [u8; 16],
    pub reserved: [u8; 48],
    pub prdt: [PrdtEntry; PRDT_ENTRIES],
}

const CMD_TBL_SIZE: usize = 4 * 4096;
const PRDT_ENTRIES: usize = (CMD_TBL_SIZE - 128) / size_of::<PrdtEntry>();

unsafe impl Pod for CommandTable {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PrdtEntry {
    pub data_base_address: u64,
    pub reserved: u32,
    pub byte_count_i: u32,
}

unsafe impl Pod for PrdtEntry {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FisRegH2D {
    pub fis_type: u8,
    pub cflags: u8,
    pub command: u8,
    pub feature_lo: u8,
    pub lba_0: u8,
    pub lba_1: u8,
    pub lba_2: u8,
    pub device: u8,
    pub lba_3: u8,
    pub lba_4: u8,
    pub lba_5: u8,
    pub feature_hi: u8,
    pub sector_count: u16,
    pub icc: u8,
    pub control: u8,
    pub _padding: [u8; 4],
}

unsafe impl Pod for FisRegH2D {}

impl FisRegH2D {
    pub fn set_lba(&mut self, lba: u64) {
        self.lba_0 = lba as u8;
        self.lba_1 = (lba >> 8) as u8;
        self.lba_2 = (lba >> 16) as u8;
        self.lba_3 = (lba >> 24) as u8;
        self.lba_4 = (lba >> 32) as u8;
        self.lba_5 = (lba >> 40) as u8;
    }
}
