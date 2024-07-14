/*
 * @file    :   ahci.rs
 * @time    :   2023/08/28 13:51:54
 * @author  :   zzjcarrot
 */

// 来自rCore的AHCI驱动 见https://gitee.com/rcore-os/isomorphic_drivers/

use alloc::{collections::BTreeMap, vec::Vec};
use framework::{
    drivers::{alloc_for_dma, dealloc_for_dma, pci::find_device_with_class},
    memory::{addr_to_array, addr_to_mut_ref, convert_physical_to_virtual, read_from_addr},
};
//use alloc::string::String;
//use alloc::vec::Vec;
use bit_field::*;
use bitflags::*;
use core::mem::size_of;
use pci::BAR;
use spin::Mutex;
use volatile::*;
use x86_64::{PhysAddr, VirtAddr};

///
#[allow(dead_code)]
pub struct AHCI {
    //    header: usize,
    //    size: usize,
    ghc: &'static mut AHCIGenericHostControl,
    received_fis: &'static mut AHCIReceivedFIS,
    cmd_list: &'static mut [AHCICommandHeader],
    cmd_table: &'static mut AHCICommandTable,
    data: &'static mut [u8],
    port: Vec<&'static mut AHCIPort>,
}

/// AHCI Generic Host Control (3.1)
#[repr(C)]
struct AHCIGenericHostControl {
    /// Host capability
    capability: Volatile<AHCICap>,
    /// Global host control
    global_host_control: Volatile<u32>,
    /// Interrupt status
    interrupt_status: Volatile<u32>,
    /// Port implemented
    port_implemented: Volatile<u32>,
    /// Version
    version: Volatile<u32>,
    /// Command completion coalescing control
    ccc_control: Volatile<u32>,
    /// Command completion coalescing ports
    ccc_ports: Volatile<u32>,
    /// Enclosure management location
    em_location: Volatile<u32>,
    /// Enclosure management control
    em_control: Volatile<u32>,
    /// Host capabilities extended
    capabilities2: Volatile<u32>,
    /// BIOS/OS handoff control and status
    bios_os_handoff_control: Volatile<u32>,
}

bitflags! {
    #[derive(Copy, Clone)]
    struct AHCICap : u32 {
        const S64A = 1 << 31;
        const SNCQ = 1 << 30;
        const SSNTF = 1 << 29;
        const SMPS = 1 << 28;
        const SSS = 1 << 27;
        const SALP = 1 << 26;
        const SAL = 1 << 25;
        const SCLO = 1 << 24;
        const ISS_GEN_1 = 1 << 20;
        const ISS_GEN_2 = 2 << 20;
        const ISS_GEN_3 = 3 << 20;
        const SAM = 1 << 18;
        const SPM = 1 << 17;
        const FBSS = 1 << 16;
        const PMD = 1 << 15;
        const SSC = 1 << 14;
        const PSC = 1 << 13;
        const CCCS = 1 << 7;
        const EMS = 1 << 6;
        const SXS = 1 << 5;
        // number of ports - 1
        const NUM_MASK = 0b11111;
    }
}

impl AHCIGenericHostControl {
    fn enable_ahci(&mut self) {
        // ref: Linux ahci_enable_ahci
        self.global_host_control.update(|v| {
            // GHC.AE
            v.set_bit(31, true);
        });
        for _ in 0..1000 {
            if self.global_host_control.read().get_bit(31) {
                break;
            }
            self.global_host_control.update(|v| {
                // GHC.AE
                v.set_bit(31, true);
            });
        }
    }
    fn enable(&mut self) {
        // ref: Linux ahci_reset_controller
        self.enable_ahci();

        let ctl = self.global_host_control.read();
        if !ctl.get_bit(0) {
            self.global_host_control.update(|v| {
                // GHC.HR
                v.set_bit(0, true);
            });
            // flush
            self.global_host_control.read();
            while self.global_host_control.read().get_bit(0) {}
            self.enable_ahci();
        }
    }
    fn num_ports(&self) -> usize {
        self.capability.read().bits().get_bits(0..5) as usize + 1
    }
    fn has_port(&self, port_num: usize) -> bool {
        self.port_implemented.read().get_bit(port_num)
    }
    fn port_ptr(&self, port_num: usize) -> *mut AHCIPort {
        (self as *const _ as usize + 0x100 + 0x80 * port_num) as *mut AHCIPort
    }
}

/// AHCI Port Registers (3.3) (one set per port)
#[repr(C)]
#[derive(Clone)]
struct AHCIPort {
    command_list_base_address: Volatile<u64>,
    fis_base_address: Volatile<u64>,
    interrupt_status: Volatile<u32>,
    interrupt_enable: Volatile<u32>,
    command: Volatile<u32>,
    reserved: Volatile<u32>,
    task_file_data: Volatile<u32>,
    signature: Volatile<u32>,
    sata_status: Volatile<u32>,
    sata_control: Volatile<u32>,
    sata_error: Volatile<u32>,
    sata_active: Volatile<u32>,
    command_issue: Volatile<u32>,
    sata_notification: Volatile<u32>,
    fis_based_switch_control: Volatile<u32>,
}

impl AHCIPort {
    fn spin_on_slot(&mut self, slot: usize) {
        loop {
            let ci = self.command_issue.read();
            if !ci.get_bit(slot) {
                break;
            }
            //spin_loop();
        }
    }
    fn issue_command(&mut self, slot: usize) {
        assert!(slot < 32);
        self.command_issue.write(1 << (slot as u32));
    }
}

/// AHCI Received FIS Structure (4.2.1)
#[repr(C)]
struct AHCIReceivedFIS {
    dma: [u8; 0x20],
    pio: [u8; 0x20],
    d2h: [u8; 0x18],
    sdbfis: [u8; 0x8],
    ufis: [u8; 0x40],
    reserved: [u8; 0x60],
}

/// # AHCI Command List Structure (4.2.2)
///
/// Host sends commands to the device through Command List.
///
/// Command List consists of 1 to 32 command headers, each one is called a slot.
///
/// Each command header describes an ATA or ATAPI command, including a
/// Command FIS, an ATAPI command buffer and a bunch of Physical Region
/// Descriptor Tables specifying the data payload address and size.
///
/// https://wiki.osdev.org/images/e/e8/Command_list.jpg
#[repr(C)]
struct AHCICommandHeader {
    /// PMP R C B R P W A CFL
    flags: u16,
    /// Physical region descriptor table length in entries
    prdt_length: u16,
    /// Physical region descriptor byte count transferred
    prd_byte_count: u32,
    /// Command table descriptor base address
    command_table_base_address: u64,
    /// Reserved
    reserved: [u32; 4],
}

bitflags! {
    struct CommandHeaderFlags: u16 {
        /// Command FIS length in DWORDS, 2 ~ 16
        const CFL_MASK = 0b11111;
        /// ATAPI
        const ATAPI = 1 << 5;
        /// Write, 1: H2D, 0: D2H
        const WRITE = 1 << 6;
        /// Prefetchable
        const PREFETCHABLE = 1 << 7;
        /// Reset
        const RESET = 1 << 8;
        /// BIST
        const BIST = 1 << 9;
        /// Clear busy upon R_OK
        const CLEAR = 1 << 10;
        /// Port multiplier port
        const PORT_MULTIPLIER_PORT_MASK = 0b1111 << 12;
    }
}

/// AHCI Command Table (4.2.3)
#[repr(C)]
struct AHCICommandTable {
    /// Command FIS
    cfis: SATAFISRegH2D,
    /// ATAPI command, 12 or 16 bytes
    acmd: [u8; 16],
    /// Reserved
    reserved: [u8; 48],
    /// Physical region descriptor table entries, 0 ~ 65535
    prdt: [AHCIPrdtEntry; 1],
}

/// Physical region descriptor table entry
#[repr(C)]
struct AHCIPrdtEntry {
    /// Data base address
    data_base_address: u64,
    /// Reserved
    reserved: u32,
    /// Bit 21-0: Byte count, 4M max
    /// Bit 31:   Interrupt on completion
    byte_count_i: u32,
}

const FIS_REG_H2D: u8 = 0x27;

const CMD_READ_DMA_EXT: u8 = 0x25;
const CMD_WRITE_DMA_EXT: u8 = 0x35;
const CMD_IDENTIFY_DEVICE: u8 = 0xec;

/// SATA Register FIS - Host to Device
///
/// https://wiki.osdev.org/AHCI Figure 5-2
#[repr(C)]
struct SATAFISRegH2D {
    fis_type: u8,
    cflags: u8,
    command: u8,
    feature_lo: u8,

    lba_0: u8, // LBA 7:0
    lba_1: u8, // LBA 15:8
    lba_2: u8, // LBA 23:16
    dev_head: u8,

    lba_3: u8, // LBA 31:24
    lba_4: u8, // LBA 39:32
    lba_5: u8, // LBA 47:40
    feature_hi: u8,

    sector_count: u16,
    reserved: u8,
    control: u8,

    _padding: [u8; 48],
}

impl SATAFISRegH2D {
    fn set_lba(&mut self, lba: u64) {
        self.lba_0 = (lba >> 0) as u8;
        self.lba_1 = (lba >> 8) as u8;
        self.lba_2 = (lba >> 16) as u8;
        self.lba_3 = (lba >> 24) as u8;
        self.lba_4 = (lba >> 32) as u8;
        self.lba_5 = (lba >> 40) as u8;
    }
}

/// IDENTIFY DEVICE data
///
/// ATA8-ACS Table 29
#[repr(C)]
struct ATAIdentifyPacket {
    _1: [u16; 10],
    serial: [u8; 20], // words 10-19
    _2: [u16; 3],
    firmware: [u8; 8], // words 23-26
    model: [u8; 40],   // words 27-46
    _3: [u16; 13],
    lba_sectors: u32, // words 60-61
    _4: [u16; 38],
    lba48_sectors: u64, // words 100-103
}

impl AHCI {
    pub fn new(header: usize, _size: usize) -> Option<Self> {
        let ghc = addr_to_mut_ref(VirtAddr::new(header as u64));

        let (rfis_pa, rfis_va) = alloc_for_dma();

        let (cmd_list_pa, cmd_list_va) = alloc_for_dma();

        let (cmd_table_pa, cmd_table_va) = alloc_for_dma();

        let (data_pa, data_va) = alloc_for_dma();

        //log::info!("Allocated for AHCI");

        let received_fis = read_from_addr(rfis_va);
        let cmd_list = addr_to_array(cmd_list_va, 4096 / size_of::<AHCICommandHeader>());
        let cmd_table = addr_to_mut_ref(cmd_table_va);
        //let identify_data = unsafe { &*(data_va as *mut ATAIdentifyPacket) };
        let data = addr_to_array(data_va, BLOCK_SIZE);
        //log::info!("Read for AHCI");
        //let np = ghc.num_ports();
        let mut ahci = AHCI {
            ghc,
            received_fis,
            cmd_list,
            cmd_table,
            data,
            port: Vec::new(),
        };
        ahci.ghc.enable();

        //let mut port_num = 0;

        log::info!("AHCI: found {} ports", ahci.ghc.num_ports());

        for i in 0..ahci.ghc.num_ports() {
            if !ahci.ghc.has_port(i) {
                continue;
            }
            let sata_status = addr_to_mut_ref::<AHCIPort>(VirtAddr::from_ptr(ahci.ghc.port_ptr(i)))
                .sata_status
                .read();
            let ipm_active = sata_status.get_bits(8..12) == 1;
            let det_present = sata_status.get_bits(0..4) == 3;
            if ipm_active && det_present {
            } else {
                continue;
            }
            let port = addr_to_mut_ref::<AHCIPort>(VirtAddr::from_ptr(ahci.ghc.port_ptr(i)));

            // Disable Port First
            // ref: Linux ahci_stop_engine
            port.command.update(|c| {
                // ST
                c.set_bit(0, false);
            });
            // LIST_ON
            while port.command.read() | (1 << 15) == 1 {}
            // ref: Linux ahci_stop_fis_rx
            port.command.update(|c| {
                // FRE
                c.set_bit(4, false);
            });
            // FIS_ON
            while port.command.read() | (1 << 14) == 1 {}

            ahci.cmd_table.prdt[0].data_base_address = data_pa.as_u64();
            ahci.cmd_table.prdt[0].byte_count_i = (BLOCK_SIZE - 1) as u32;

            ahci.cmd_list[0].command_table_base_address = cmd_table_pa.as_u64();
            ahci.cmd_list[0].prdt_length = 1;
            ahci.cmd_list[0].prd_byte_count = 0;
            // cfl=4
            ahci.cmd_list[0].flags = 4;

            port.command_list_base_address.write(cmd_list_pa.as_u64());
            port.fis_base_address.write(rfis_pa.as_u64());

            // clear errors
            port.sata_error.write(0xffffffff);

            // ref: Linux ahci_power_up
            // spin up device
            port.command.update(|c| {
                // SUD
                *c |= 1 << 1;
            });
            // power up
            port.command.update(|c| {
                // ICC
                *c &= !(0xf << 28);
                *c |= 1 << 28;
            });

            // ref: Linux ahci_start_fis_rx
            // enable fre
            port.command.update(|c| {
                // FRE
                *c |= 1 << 4;
            });
            // flush
            port.command.read();

            // ref: Linux ahci_start_engine
            // enable port
            port.command.update(|c| {
                // ST
                *c |= 1 << 0;
            });
            // flush
            port.command.read();

            // wait for ST
            while port.command.read() | (1 << 0) == 0 {}

            let stat = port.sata_status.read();
            if stat == 0 {
                return None;
            }

            let fis = &mut ahci.cmd_table.cfis;
            // Register FIS from HBA to device
            fis.fis_type = FIS_REG_H2D;
            fis.cflags = 1 << 7;

            // 7.15 IDENTIFY DEVICE - ECh, PIO Data-In
            fis.command = CMD_IDENTIFY_DEVICE;
            fis.sector_count = 1;

            port.issue_command(0);
            port.spin_on_slot(0);
            let port = addr_to_mut_ref::<AHCIPort>(VirtAddr::from_ptr(ahci.ghc.port_ptr(i)));
            ahci.port.push(port);
        }
        Some(ahci)
    }

    pub fn get_hd_size(&mut self, hd: usize) -> usize {
        self.cmd_list[0].flags = 4;

        let fis = &mut self.cmd_table.cfis;
        // Register FIS from HBA to device
        fis.fis_type = FIS_REG_H2D;
        fis.cflags = 1 << 7;
        fis.command = CMD_IDENTIFY_DEVICE;
        fis.dev_head = 0x40;
        fis.control = 1;

        self.port[hd].issue_command(0);
        self.port[hd].spin_on_slot(0);

        let packet_ptr = self.data.as_ptr() as *const ATAIdentifyPacket;
        let packet = addr_to_mut_ref::<ATAIdentifyPacket>(VirtAddr::from_ptr(packet_ptr));
        packet.lba48_sectors as usize * 512
    }

    pub fn read_block(&mut self, hd: usize, id: u64, count: usize, buf: &mut [u8]) {
        // cfl=4
        self.cmd_list[0].flags = 4;

        let fis = &mut self.cmd_table.cfis;
        // Register FIS from HBA to device
        fis.fis_type = FIS_REG_H2D;
        fis.cflags = 1 << 7;
        // 7.25 READ DMA EXT - 25h, DMA
        fis.command = CMD_READ_DMA_EXT;
        fis.sector_count = count as u16;
        fis.dev_head = 0x40; // LBA
        fis.control = 0x80; // LBA48
        fis.set_lba(id);

        self.port[hd].issue_command(0);
        self.port[hd].spin_on_slot(0);

        let len = buf.len().min(BLOCK_SIZE);
        buf[..len].clone_from_slice(&self.data[0..len]);
    }

    pub fn write_block(&mut self, hd: usize, id: u64, count: usize, buf: &[u8]) {
        //panic!("write");
        // cfl=4
        self.cmd_list[0].flags = 4 | CommandHeaderFlags::WRITE.bits(); // device write

        let len = buf.len().min(BLOCK_SIZE);
        self.data[0..len].clone_from_slice(&buf[..len]);

        let fis = &mut self.cmd_table.cfis;
        // Register FIS from HBA to device
        fis.fis_type = FIS_REG_H2D;
        fis.cflags = 1 << 7;
        // ATA8-ACS
        // 7.63 WRITE DMA EXT - 35h, DMA
        fis.command = CMD_WRITE_DMA_EXT;
        fis.sector_count = count as u16;
        fis.dev_head = 0x40; // LBA
        fis.control = 0x80; // LBA48
        fis.set_lba(id);

        self.port[hd].issue_command(0);
        self.port[hd].spin_on_slot(0);
    }
}

impl Drop for AHCI {
    fn drop(&mut self) {
        dealloc_for_dma(VirtAddr::from_ptr(self.ghc as *const _));
        dealloc_for_dma(VirtAddr::from_ptr(self.cmd_list.as_ptr()));
        dealloc_for_dma(VirtAddr::from_ptr(self.cmd_table as *const _));
        dealloc_for_dma(VirtAddr::from_ptr(self.data.as_ptr()));
        dealloc_for_dma(VirtAddr::from_ptr(self.received_fis as *const _));
    }
}

pub const BLOCK_SIZE: usize = 512;

/*fn from_ata_string(data: &[u8]) -> String {
    let mut swapped_data = Vec::new();
    assert_eq!(data.len() % 2, 0);
    for i in (0..data.len()).step_by(2) {
        swapped_data.push(data[i + 1]);
        swapped_data.push(data[i]);
    }
    return String::from_utf8(swapped_data).unwrap();
}*/

static AHCI_CONS: Mutex<Vec<AHCI>> = Mutex::new(Vec::new());
pub static DISK_TO_CON: Mutex<BTreeMap<usize, usize>> = Mutex::new(BTreeMap::new());

pub fn init() {
    let devices = find_device_with_class(0x01, 0x06);

    let mut ahci_cons = AHCI_CONS.lock();

    for device in devices.iter() {
        if let Some(BAR::Memory(addr, len, _, _)) = device.bars[5] {
            assert!(len as usize <= 4096);
            let header = convert_physical_to_virtual(PhysAddr::new(addr)).as_u64() as usize;
            let size = len as usize;

            framework::drivers::pci::enable_device(device);

            log::info!("OK");

            if let Some(ahci) = AHCI::new(header, size) {
                ahci_cons.push(ahci);
            }
        }
    }

    log::info!("AHCI: found {} devices", ahci_cons.len());

    let mut disk_num = 0;

    for (idx, ahci_con) in ahci_cons.iter().enumerate() {
        for i in disk_num..disk_num + ahci_con.port.len() {
            DISK_TO_CON.lock().insert(i, idx);
        }
        disk_num += ahci_con.port.len();
    }

    log::info!("AHCI OK! Hard Disk Number:{}", disk_num);
}

fn find_hd(hd_id: usize) -> Option<usize> {
    if let Some(con) = DISK_TO_CON.lock().get(&hd_id) {
        Some(*con)
    } else {
        None
    }
}

pub fn read_block(hd: usize, start_sec: u64, buf: &mut [u8]) -> Option<()> {
    assert!(buf.len() % 512 == 0);
    let port_id = find_hd(hd)?;
    AHCI_CONS.lock()[port_id].read_block(hd, start_sec, buf.len() / 512, buf);
    Some(())
}

pub fn write_block(hd: usize, start_sec: u64, buf: &[u8]) -> Option<()> {
    assert!(buf.len() % 512 == 0);
    let port_id = find_hd(hd)?;
    AHCI_CONS.lock()[port_id].write_block(hd, start_sec, buf.len() / 512, buf);
    Some(())
}

pub fn get_hd_num() -> usize {
    DISK_TO_CON.lock().len()
}

pub fn get_hd_size(hd: usize) -> Option<usize> {
    let port_id = find_hd(hd)?;
    Some(AHCI_CONS.lock()[port_id].get_hd_size(hd))
}
