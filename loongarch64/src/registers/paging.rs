use crate::define_csr;

define_csr!(PwcLow, 0x1c);
define_csr!(PwcHigh, 0x1d);

define_csr!(PgdLow, 0x19);
define_csr!(PgdHigh, 0x1a);

pub fn init_pwc() {
    const PT_BASE: u64 = 12;
    const PT_WIDTH: u64 = 9;
    const DIR1_BASE: u64 = 21;
    const DIR1_WIDTH: u64 = 9;
    const DIR2_BASE: u64 = 30;
    const DIR2_WIDTH: u64 = 9;
    const PTE_WIDTH: u64 = 0;
    const DIR3_BASE: u64 = 39;
    const DIR3_WIDTH: u64 = 9;
    const DIR4_WIDTH: u64 = 0;

    const PWCL: u64 = (PTE_WIDTH << 30)
        | (DIR2_WIDTH << 25)
        | (DIR2_BASE << 20)
        | (DIR1_WIDTH << 15)
        | (DIR1_BASE << 10)
        | (PT_WIDTH << 5)
        | (PT_BASE << 0);

    const PWCH: u64 = (DIR4_WIDTH << 18) | (DIR3_WIDTH << 6) | (DIR3_BASE << 0);

    PwcLow.write(PWCL);
    PwcHigh.write(PWCH);
}
