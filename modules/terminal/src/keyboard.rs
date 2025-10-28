use crossbeam_queue::ArrayQueue;
use ostd::{
    arch::{
        device::io_port::ReadWriteAccess,
        kernel::{IRQ_CHIP, MappedIrqLine},
        trap::TrapFrame,
    },
    io::IoPort,
    irq::IrqLine,
};
use spin::{Lazy, Once};

use crate::wake_up;

static IRQ_LINE: Once<MappedIrqLine> = Once::new();
static IO_PORT: Lazy<IoPort<u8, ReadWriteAccess>> = Lazy::new(|| IoPort::acquire(0x60).unwrap());
pub static SCANCODE_QUEUE: Lazy<ArrayQueue<u8>> = Lazy::new(|| ArrayQueue::new(128));

pub fn init() {
    let mut irq_line = IrqLine::alloc()
        .and_then(|irq_line| IRQ_CHIP.get().unwrap().map_isa_pin_to(irq_line, 1))
        .unwrap();
    irq_line.on_active(keyboard_callback);

    IRQ_LINE.call_once(|| irq_line);
}

fn keyboard_callback(_frame: &TrapFrame) {
    let scan_code = IO_PORT.read();

    if SCANCODE_QUEUE.push(scan_code).is_err() {
        log::warn!("Scan code queue full. Dropping data.");
    }

    wake_up();
}
