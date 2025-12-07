use bit_field::BitField;

use crate::define_csr;

define_csr!(read IpiStatus, 0x1000);
define_csr!(IpiEnabled, 0x1004);
define_csr!(write IpiSet, 0x1008);
define_csr!(write IpiClear, 0x100c);

define_csr!(MailBox0, 0x1020);
define_csr!(MailBox1, 0x1028);
define_csr!(MailBox2, 0x1030);
define_csr!(MailBox3, 0x1038);

define_csr!(write IpiSend, 0x1040);
define_csr!(write MailSend, 0x1048);
define_csr!(write FreqSend, 0x1058);

impl IpiSend {
    pub fn send_ipi(&self, cpu: u64, vector: u8, wait_till_written: bool) {
        let mut ipi_value = 0u64;
        ipi_value.set_bits(0..=4, vector as u64);
        ipi_value.set_bits(16..=25, cpu);
        ipi_value.set_bit(31, wait_till_written);

        self.write(ipi_value);
    }
}

impl MailSend {
    /// 0b0000 for mask => send all data
    pub fn send_data(
        &self,
        cpu: u64,
        data: u32,
        target_mailbox: u8,
        mask: u8,
        wait_till_written: bool,
    ) {
        assert!(target_mailbox < 3);

        let mut mail_box_value = 0u64;
        mail_box_value.set_bits(2..=4, target_mailbox as u64);
        mail_box_value.set_bits(16..=25, cpu);
        mail_box_value.set_bits(27..=30, mask as u64);
        mail_box_value.set_bit(31, wait_till_written);
        mail_box_value.set_bits(32..64, data as u64);

        self.write(mail_box_value);
    }
}
