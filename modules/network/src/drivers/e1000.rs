#![allow(unsafe_code)]

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem::size_of;
use core::sync::atomic::{Ordering, fence};
use driver::{DateTime, DmaList, Mmio};
use ostd::Pod;
use ostd::mm::{DmaCoherent, FrameAllocOptions, HasDaddr, PAGE_SIZE, VmIo, VmIoFill};
use ostd::sync::Mutex;
use pci::get_pci_devices;
use smoltcp::iface::{Config, Interface, PollResult, SocketSet};
use smoltcp::phy::{self, DeviceCapabilities};
use smoltcp::socket::dhcpv4::{Event, Socket};
use smoltcp::time::{Duration, Instant};
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpCidr};

use bit_field::*;
use bitflags::*;
use log::*;

pub fn init() {
    let pci_devices = get_pci_devices();
    for device in pci_devices.iter() {
        if device.vendor_id == 0x8086 && device.device_id == 0x100e {
            let bar0 = device.bars()[0];
            let (header, size) = bar0.unwrap().unwrap_mem();

            let mac = EthernetAddress::from_bytes(&[0x54, 0x51, 0x9F, 0x71, 0xC0, 0]);
            let driver = E1000::new(header, size, mac);
            log::info!("mac: {}", driver.mac);

            let mut driver = E1000Driver(Arc::new(Mutex::new(driver)));

            let now = Instant::from_secs(DateTime::default().unix_timestamp());
            let mut iface = Interface::new(
                Config::new(HardwareAddress::Ethernet(mac)),
                &mut driver,
                now,
            );

            let max_duration = Duration::from_secs(10);
            let mut dhcp_socket = Socket::new();
            dhcp_socket.set_max_lease_duration(Some(max_duration));

            let mut sockets = SocketSet::new(Vec::new());
            let dhcp_handle = sockets.add(dhcp_socket);

            loop {
                let timestamp = Instant::from_secs(DateTime::default().unix_timestamp());
                let poll = iface.poll(timestamp, &mut driver, &mut sockets);

                if poll == PollResult::None {
                    continue;
                }

                let event = sockets.get_mut::<Socket>(dhcp_handle).poll();
                match event {
                    None => {}
                    Some(Event::Configured(config)) => {
                        log::info!("IP Address: {}", config.address);
                        iface.update_ip_addrs(|addrs| {
                            addrs.clear();
                            addrs.push(IpCidr::Ipv4(config.address)).unwrap();
                        });

                        if let Some(router) = config.router {
                            log::info!("Default gateway: {}", router);
                            iface.routes_mut().add_default_ipv4_route(router).unwrap();
                        } else {
                            log::info!("No default gateway configured");
                            iface.routes_mut().remove_default_ipv4_route();
                        }

                        for (i, s) in config.dns_servers.iter().enumerate() {
                            log::info!("DNS Server {}:         {}", i, s);
                        }
                        break;
                    }
                    Some(Event::Deconfigured) => {
                        log::info!("DHCP lost config!");
                        iface.update_ip_addrs(|addrs| addrs.clear());
                        iface.routes_mut().remove_default_ipv4_route();
                        break;
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct E1000Driver(Arc<Mutex<E1000>>);

// At the beginning, all transmit descriptors have there status non-zero,
// so we need to track whether we are using the descriptor for the first time.
// When the descriptors wrap around, we set first_trans to false,
// and lookup status instead for checking whether it is empty.
pub struct E1000 {
    mac: EthernetAddress,
    registers: Vec<Mmio<u32>>,
    send_queue: DmaList<E1000SendDesc>,
    send_buffers: Vec<DmaCoherent>,
    recv_queue: DmaList<E1000RecvDesc>,
    recv_buffers: Vec<DmaCoherent>,
    first_trans: bool,
}

unsafe impl Send for E1000 {}
unsafe impl Sync for E1000 {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct E1000SendDesc {
    addr: u64,
    len: u16,
    cso: u8,
    cmd: u8,
    status: u8,
    css: u8,
    special: u8,
}

unsafe impl Pod for E1000SendDesc {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct E1000RecvDesc {
    addr: u64,
    len: u16,
    chksum: u16,
    status: u16,
    error: u8,
    special: u8,
}

unsafe impl Pod for E1000RecvDesc {}

bitflags! {
    #[derive(Debug)]
    struct E1000Status : u32 {
        const FD = 1 << 0;
        const LU = 1 << 1;
        const TXOFF = 1 << 4;
        const TBIMODE = 1 << 5;
        const SPEED_100M = 1 << 6;
        const SPEED_1000M = 1 << 7;
        const ASDV_100M = 1 << 8;
        const ASDV_1000M = 1 << 9;
        const MTXCKOK = 1 << 10;
        const PCI66 = 1 << 11;
        const BUS64 = 1 << 12;
        const PCIX_MODE = 1 << 13;
        const GIO_MASTER_ENABLE = 1 << 19;
    }
}

#[allow(dead_code)]
impl E1000 {
    pub fn new(header: usize, size: usize, mac: EthernetAddress) -> Self {
        assert_eq!(size_of::<E1000SendDesc>(), 16);
        assert_eq!(size_of::<E1000RecvDesc>(), 16);

        let send_queue = DmaList::new(256);
        let recv_queue = DmaList::new(256);

        let mut send_buffers = Vec::with_capacity(send_queue.len());
        let mut recv_buffers = Vec::with_capacity(recv_queue.len());

        log::info!(target: "kernel", "e1000 config space: {:x} {:x}", header, size);
        let e1000 = (0..size / 4).map(|n| Mmio::new(header + n * 4).unwrap());
        let e1000 = e1000.collect::<Vec<_>>();
        debug!(target: "kernel",
            "status before setup: {:#?}",
            E1000Status::from_bits_truncate(e1000[E1000_STATUS].read())
        );

        // 4.6 Software Initialization Sequence

        // 4.6.6 Transmit Initialization

        // Program the descriptor base address with the address of the region.
        e1000[E1000_TDBAL].write(&(send_queue.device_address() as u32)); // TDBAL
        e1000[E1000_TDBAH].write(&((send_queue.device_address() >> 32) as u32)); // TDBAH

        // Set the length register to the size of the descriptor ring.
        e1000[E1000_TDLEN].write(&(send_queue.size() as u32)); // TDLEN

        // If needed, program the head and tail registers.
        e1000[E1000_TDH].write(&0); // TDH
        e1000[E1000_TDT].write(&0); // TDT

        for i in 0..send_queue.len() {
            let buffer_frame = FrameAllocOptions::new().alloc_segment(1).unwrap();
            let buffer = DmaCoherent::map(buffer_frame.into(), false).unwrap();
            buffer.fill_zeros(0, PAGE_SIZE).unwrap();
            send_queue.with_value(i, |value: &mut E1000SendDesc| {
                value.addr = buffer.daddr() as u64;
            });
            send_buffers.push(buffer);
        }

        // EN | PSP | CT=0x10 | COLD=0x40
        e1000[E1000_TCTL].write(&((1 << 1) | (1 << 3) | (0x10 << 4) | (0x40 << 12))); // TCTL
        // IPGT=0xa | IPGR1=0x8 | IPGR2=0xc
        e1000[E1000_TIPG].write(&(0xa | (0x8 << 10) | (0xc << 20))); // TIPG

        // 4.6.5 Receive Initialization
        let mut ral: u32 = 0;
        let mut rah: u32 = 0;
        for i in 0..4 {
            ral |= (mac.as_bytes()[i] as u32) << (i * 8);
        }
        for i in 0..2 {
            rah |= (mac.as_bytes()[i + 4] as u32) << (i * 8);
        }

        e1000[E1000_RAL].write(&ral); // RAL
        // AV | AS=DA
        e1000[E1000_RAH].write(&(rah | (1 << 31))); // RAH

        // MTA
        for mmio in e1000.iter().take(E1000_RAL).skip(E1000_MTA) {
            mmio.write(&0);
        }

        // Program the descriptor base address with the address of the region.
        e1000[E1000_RDBAL].write(&(recv_queue.device_address() as u32)); // RDBAL
        e1000[E1000_RDBAH].write(&((recv_queue.device_address() >> 32) as u32)); // RDBAH

        // Set the length register to the size of the descriptor ring.
        e1000[E1000_RDLEN].write(&(recv_queue.size() as u32)); // RDLEN

        // If needed, program the head and tail registers. Note: the head and tail pointers are initialized (by hardware) to zero after a power-on or a software-initiated device reset.
        e1000[E1000_RDH].write(&0); // RDH

        // The tail pointer should be set to point one descriptor beyond the end.
        e1000[E1000_RDT].write(&((recv_queue.len() - 1) as u32)); // RDT

        // Receive buffers of appropriate size should be allocated and pointers to these buffers should be stored in the descriptor ring.
        for i in 0..recv_queue.len() {
            let buffer_frame = FrameAllocOptions::new().alloc_segment(1).unwrap();
            let buffer = DmaCoherent::map(buffer_frame.into(), false).unwrap();
            buffer.fill_zeros(0, PAGE_SIZE).unwrap();
            recv_queue.with_value(i, |value: &mut E1000RecvDesc| {
                value.addr = buffer.daddr() as u64;
            });
            recv_buffers.push(buffer);
        }

        // EN | BAM | BSIZE=3 | BSEX | SECRC
        // BSIZE=3 | BSEX means buffer size = 4096
        e1000[E1000_RCTL].write(&((1 << 1) | (1 << 15) | (3 << 16) | (1 << 25) | (1 << 26))); // RCTL

        debug!(
            "status after setup: {:#?}",
            E1000Status::from_bits_truncate(e1000[E1000_STATUS].read())
        );

        // enable interrupt
        // clear interrupt
        let data = e1000[E1000_ICR].read();
        e1000[E1000_ICR].write(&data);
        // RXT0
        e1000[E1000_IMS].write(&(1 << 7)); // IMS

        // clear interrupt
        let data = e1000[E1000_ICR].read();
        e1000[E1000_ICR].write(&data);

        E1000 {
            mac,
            registers: e1000,
            send_queue,
            send_buffers,
            recv_queue,
            recv_buffers,
            first_trans: true,
        }
    }

    pub fn handle_interrupt(&mut self) -> bool {
        let icr = self.registers[E1000_ICR].read();
        if icr != 0 {
            // clear it
            self.registers[E1000_ICR].write(&icr);
            true
        } else {
            false
        }
    }

    pub fn receive(&mut self) -> Option<Vec<u8>> {
        let tdt = self.registers[E1000_TDT].read() as usize;
        let index = tdt % self.send_queue.len();

        self.send_queue.with_value(index, |send_desc| {
            let mut rdt = self.registers[E1000_RDT].read() as usize;
            let index = (rdt + 1) % self.recv_queue.len();
            self.recv_queue.with_value(index, |recv_desc| {
                let transmit_avail = self.first_trans || send_desc.status.get_bit(0);
                let receive_avail = recv_desc.status.get_bit(0);

                if !(transmit_avail && receive_avail) {
                    return None;
                }

                let mut buffer = alloc::vec![0; recv_desc.len as usize];
                self.recv_buffers[index].read_bytes(0, &mut buffer).unwrap();

                recv_desc.status.set_bit(0, false);

                rdt = index;
                self.registers[E1000_RDT].write(&(rdt as u32));

                Some(buffer)
            })
        })
    }

    pub fn can_send(&self) -> bool {
        let tdt = self.registers[E1000_TDT].read();
        let index = (tdt as usize) % self.send_queue.len();
        let send_desc = &self.send_queue.read(index);
        self.first_trans || send_desc.status.get_bit(0)
    }

    pub fn send(&mut self, buffer: &[u8]) {
        let mut tdt = self.registers[E1000_TDT].read();
        let index = (tdt as usize) % self.send_queue.len();
        let send_desc = &mut self.send_queue.read(index);
        assert!(self.first_trans || send_desc.status.get_bit(0));

        self.send_buffers[index].write_bytes(0, buffer).unwrap();

        send_desc.len = buffer.len() as u16 + 4;
        send_desc.cmd = (1 << 3) | (1 << 1) | (1 << 0); // RS | IFCS | EOP
        send_desc.status = 0;
        fence(Ordering::SeqCst);

        tdt = (tdt + 1) % self.send_queue.len() as u32;
        self.registers[E1000_TDT].write(&tdt);
        fence(Ordering::SeqCst);

        // round
        if tdt == 0 {
            self.first_trans = false;
        }
    }
}

pub struct E1000RxToken(Vec<u8>);
pub struct E1000TxToken(E1000Driver);

impl phy::Device for E1000Driver {
    type RxToken<'a> = E1000RxToken;
    type TxToken<'a> = E1000TxToken;

    fn receive(&mut self, _inst: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        self.0
            .lock()
            .receive()
            .map(|vec| (E1000RxToken(vec), E1000TxToken(self.clone())))
    }

    fn transmit(&mut self, _inst: Instant) -> Option<Self::TxToken<'_>> {
        if self.0.lock().can_send() {
            Some(E1000TxToken(self.clone()))
        } else {
            None
        }
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1536;
        caps.max_burst_size = Some(64);
        caps
    }
}

impl phy::RxToken for E1000RxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.0)
    }
}

impl phy::TxToken for E1000TxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buffer = [0u8; PAGE_SIZE];
        let result = f(&mut buffer[..len]);

        let mut driver = (self.0).0.lock();
        driver.send(&buffer);

        result
    }
}

use consts::*;

#[allow(dead_code)]
mod consts {
    pub const E1000_STATUS: usize = 0x0008 / 4;
    pub const E1000_ERRD: usize = 0x0014 / 4;
    pub const E1000_ICR: usize = 0x00C0 / 4;
    pub const E1000_IMS: usize = 0x00D0 / 4;
    pub const E1000_IMC: usize = 0x00D8 / 4;
    pub const E1000_RCTL: usize = 0x0100 / 4;
    pub const E1000_TCTL: usize = 0x0400 / 4;
    pub const E1000_TIPG: usize = 0x0410 / 4;
    pub const E1000_RDBAL: usize = 0x2800 / 4;
    pub const E1000_RDBAH: usize = 0x2804 / 4;
    pub const E1000_RDLEN: usize = 0x2808 / 4;
    pub const E1000_RDH: usize = 0x2810 / 4;
    pub const E1000_RDT: usize = 0x2818 / 4;
    pub const E1000_TDBAL: usize = 0x3800 / 4;
    pub const E1000_TDBAH: usize = 0x3804 / 4;
    pub const E1000_TDLEN: usize = 0x3808 / 4;
    pub const E1000_TDH: usize = 0x3810 / 4;
    pub const E1000_TDT: usize = 0x3818 / 4;
    pub const E1000_MTA: usize = 0x5200 / 4;
    pub const E1000_RAL: usize = 0x5400 / 4;
    pub const E1000_RAH: usize = 0x5404 / 4;

    pub const E1000_ERRD_START: u32 = 1 << 0;
    pub const E1000_ERRD_END: u32 = 1 << 1;
    pub const E1000_ERRD_ADDR_SHIFT: u32 = 2;
    pub const E1000_ERRD_DATA_SHIFT: u32 = 16;
}
