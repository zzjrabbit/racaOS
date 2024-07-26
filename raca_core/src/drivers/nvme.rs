use framework::{
    drivers::{alloc_for_dma, pci::find_device_with_class},
    memory::{
        convert_physical_to_virtual, convert_virtual_to_physical, MemoryManager, FRAME_ALLOCATOR,
        KERNEL_PAGE_TABLE,
    },
};
use nvme_driver::*;
use pci::BAR;
use x86_64::{
    structures::paging::{Page, PageTableFlags, PhysFrame},
    PhysAddr, VirtAddr,
};

struct DmaAllocatorImp;

impl DmaAllocator for DmaAllocatorImp {
    fn dma_alloc(size: usize) -> usize {
        log::info!("size : {}", size);
        let (phys, addr) = alloc_for_dma((size + 4095) / 4096);
        let _ = <MemoryManager>::map_frame_to_page(
            PhysFrame::containing_address(phys),
            Page::containing_address(addr),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            &mut *KERNEL_PAGE_TABLE.lock(),
            &mut *FRAME_ALLOCATOR.lock(),
        );
        log::info!("Allocated: {:x}", addr);
        addr.as_u64() as usize
    }

    fn dma_dealloc(_addr: usize, _size: usize) -> usize {
        0
    }

    fn phys_to_virt(phys: usize) -> usize {
        convert_physical_to_virtual(PhysAddr::new(phys as u64)).as_u64() as usize
    }

    fn virt_to_phys(virt: usize) -> usize {
        convert_virtual_to_physical(VirtAddr::new(virt as u64)).as_u64() as usize
    }
}

struct IrqControllerImp;

impl IrqController for IrqControllerImp {
    fn enable_irq(_irq_num: usize) {}

    fn disable_irq(_irq_num: usize) {}
}

pub fn init() {
    let nvme_device = find_device_with_class(0x01, 0x08);
    if nvme_device.len() == 0 {
        log::error!("No NVME Device");
        return;
    }
    let nvme_device = &nvme_device[0];
    log::info!("NVME Bars :{:?}", nvme_device.bars);
    if let Some(BAR::Memory(addr, _, _, _)) = nvme_device.bars[0] {
        log::info!("OK");
        //assert!(len as usize <= 4096);
        let header = /*convert_physical_to_virtual(PhysAddr::new(addr)).as_u64() as usize*/ addr as usize;
        //let size = len as usize;

        log::info!("NVME Addr Phys: {:x} Virt: {:x}", addr, header);
        // NVME控制器的MMIO大小约有16KB,也就是4页
        for i in 0..4 {
            <MemoryManager>::map_frame_to_page(
                PhysFrame::containing_address(PhysAddr::new(addr + i * 4096)),
                Page::containing_address(VirtAddr::new(header as u64 + i * 4096)),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                &mut *KERNEL_PAGE_TABLE.lock(),
                &mut *FRAME_ALLOCATOR.lock(),
            )
            .unwrap();
        }

        framework::drivers::pci::enable_device(nvme_device);

        log::info!("OK");

        let nvme = NvmeInterface::<DmaAllocatorImp, IrqControllerImp>::new(header);
        log::info!("OK");
        for i in 0..5 {
            let mut read_buf = [0u8; 512];
            let buff = [i as u8; 512];
            let write_buf: &[u8] = &[i as u8; 512];
            nvme.write_block(i, &write_buf);
            nvme.read_block(i, &mut read_buf);
            framework::serial_println!("{:?}", read_buf);
            assert_eq!(read_buf, buff);
        }
    }
}
