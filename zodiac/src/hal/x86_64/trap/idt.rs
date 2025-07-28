use core::mem::transmute;
use spin::Lazy;
use x86_64::structures::idt::{Entry, HandlerFunc, InterruptDescriptorTable, PageFaultErrorCode};

use crate::hal::trap::gdt::{DOUBLE_FAULT_IST_INDEX, PAGE_FAULT_IST_INDEX};

pub fn init() {
    IDT.load();
}

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    let entries = unsafe { &mut *((&raw mut idt).cast::<[Entry<HandlerFunc>; 256]>()) };
    unsafe {
        entries[0x00].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry00 as usize));
        entries[0x01].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry01 as usize));
        entries[0x02].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry02 as usize));
        entries[0x03].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry03 as usize));
        entries[0x04].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry04 as usize));
        entries[0x05].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry05 as usize));
        entries[0x06].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry06 as usize));
        entries[0x07].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry07 as usize));
        entries[0x08].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry08 as usize));
        entries[0x09].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry09 as usize));
        entries[0x0a].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry0a as usize));
        entries[0x0b].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry0b as usize));
        entries[0x0c].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry0c as usize));
        entries[0x0d].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry0d as usize));
        entries[0x0e].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry0e as usize));
        entries[0x0f].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry0f as usize));
        entries[0x10].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry10 as usize));
        entries[0x11].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry11 as usize));
        entries[0x12].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry12 as usize));
        entries[0x13].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry13 as usize));
        entries[0x14].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry14 as usize));
        entries[0x15].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry15 as usize));
        entries[0x16].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry16 as usize));
        entries[0x17].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry17 as usize));
        entries[0x18].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry18 as usize));
        entries[0x19].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry19 as usize));
        entries[0x1a].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry1a as usize));
        entries[0x1b].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry1b as usize));
        entries[0x1c].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry1c as usize));
        entries[0x1d].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry1d as usize));
        entries[0x1e].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry1e as usize));
        entries[0x1f].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry1f as usize));
        entries[0x20].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry20 as usize));
        entries[0x21].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry21 as usize));
        entries[0x22].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry22 as usize));
        entries[0x23].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry23 as usize));
        entries[0x24].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry24 as usize));
        entries[0x25].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry25 as usize));
        entries[0x26].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry26 as usize));
        entries[0x27].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry27 as usize));
        entries[0x28].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry28 as usize));
        entries[0x29].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry29 as usize));
        entries[0x2a].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry2a as usize));
        entries[0x2b].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry2b as usize));
        entries[0x2c].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry2c as usize));
        entries[0x2d].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry2d as usize));
        entries[0x2e].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry2e as usize));
        entries[0x2f].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry2f as usize));
        entries[0x30].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry30 as usize));
        entries[0x31].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry31 as usize));
        entries[0x32].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry32 as usize));
        entries[0x33].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry33 as usize));
        entries[0x34].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry34 as usize));
        entries[0x35].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry35 as usize));
        entries[0x36].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry36 as usize));
        entries[0x37].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry37 as usize));
        entries[0x38].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry38 as usize));
        entries[0x39].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry39 as usize));
        entries[0x3a].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry3a as usize));
        entries[0x3b].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry3b as usize));
        entries[0x3c].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry3c as usize));
        entries[0x3d].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry3d as usize));
        entries[0x3e].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry3e as usize));
        entries[0x3f].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry3f as usize));
        entries[0x40].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry40 as usize));
        entries[0x41].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry41 as usize));
        entries[0x42].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry42 as usize));
        entries[0x43].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry43 as usize));
        entries[0x44].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry44 as usize));
        entries[0x45].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry45 as usize));
        entries[0x46].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry46 as usize));
        entries[0x47].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry47 as usize));
        entries[0x48].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry48 as usize));
        entries[0x49].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry49 as usize));
        entries[0x4a].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry4a as usize));
        entries[0x4b].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry4b as usize));
        entries[0x4c].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry4c as usize));
        entries[0x4d].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry4d as usize));
        entries[0x4e].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry4e as usize));
        entries[0x4f].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry4f as usize));
        entries[0x50].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry50 as usize));
        entries[0x51].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry51 as usize));
        entries[0x52].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry52 as usize));
        entries[0x53].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry53 as usize));
        entries[0x54].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry54 as usize));
        entries[0x55].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry55 as usize));
        entries[0x56].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry56 as usize));
        entries[0x57].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry57 as usize));
        entries[0x58].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry58 as usize));
        entries[0x59].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry59 as usize));
        entries[0x5a].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry5a as usize));
        entries[0x5b].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry5b as usize));
        entries[0x5c].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry5c as usize));
        entries[0x5d].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry5d as usize));
        entries[0x5e].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry5e as usize));
        entries[0x5f].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry5f as usize));
        entries[0x60].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry60 as usize));
        entries[0x61].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry61 as usize));
        entries[0x62].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry62 as usize));
        entries[0x63].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry63 as usize));
        entries[0x64].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry64 as usize));
        entries[0x65].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry65 as usize));
        entries[0x66].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry66 as usize));
        entries[0x67].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry67 as usize));
        entries[0x68].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry68 as usize));
        entries[0x69].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry69 as usize));
        entries[0x6a].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry6a as usize));
        entries[0x6b].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry6b as usize));
        entries[0x6c].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry6c as usize));
        entries[0x6d].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry6d as usize));
        entries[0x6e].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry6e as usize));
        entries[0x6f].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry6f as usize));
        entries[0x70].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry70 as usize));
        entries[0x71].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry71 as usize));
        entries[0x72].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry72 as usize));
        entries[0x73].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry73 as usize));
        entries[0x74].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry74 as usize));
        entries[0x75].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry75 as usize));
        entries[0x76].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry76 as usize));
        entries[0x77].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry77 as usize));
        entries[0x78].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry78 as usize));
        entries[0x79].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry79 as usize));
        entries[0x7a].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry7a as usize));
        entries[0x7b].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry7b as usize));
        entries[0x7c].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry7c as usize));
        entries[0x7d].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry7d as usize));
        entries[0x7e].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry7e as usize));
        entries[0x7f].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry7f as usize));
        entries[0x80].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry80 as usize));
        entries[0x81].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry81 as usize));
        entries[0x82].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry82 as usize));
        entries[0x83].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry83 as usize));
        entries[0x84].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry84 as usize));
        entries[0x85].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry85 as usize));
        entries[0x86].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry86 as usize));
        entries[0x87].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry87 as usize));
        entries[0x88].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry88 as usize));
        entries[0x89].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry89 as usize));
        entries[0x8a].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry8a as usize));
        entries[0x8b].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry8b as usize));
        entries[0x8c].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry8c as usize));
        entries[0x8d].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry8d as usize));
        entries[0x8e].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry8e as usize));
        entries[0x8f].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry8f as usize));
        entries[0x90].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry90 as usize));
        entries[0x91].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry91 as usize));
        entries[0x92].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry92 as usize));
        entries[0x93].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry93 as usize));
        entries[0x94].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry94 as usize));
        entries[0x95].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry95 as usize));
        entries[0x96].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry96 as usize));
        entries[0x97].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry97 as usize));
        entries[0x98].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry98 as usize));
        entries[0x99].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry99 as usize));
        entries[0x9a].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry9a as usize));
        entries[0x9b].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry9b as usize));
        entries[0x9c].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry9c as usize));
        entries[0x9d].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry9d as usize));
        entries[0x9e].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry9e as usize));
        entries[0x9f].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentry9f as usize));
        entries[0xa0].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrya0 as usize));
        entries[0xa1].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrya1 as usize));
        entries[0xa2].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrya2 as usize));
        entries[0xa3].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrya3 as usize));
        entries[0xa4].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrya4 as usize));
        entries[0xa5].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrya5 as usize));
        entries[0xa6].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrya6 as usize));
        entries[0xa7].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrya7 as usize));
        entries[0xa8].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrya8 as usize));
        entries[0xa9].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrya9 as usize));
        entries[0xaa].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryaa as usize));
        entries[0xab].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryab as usize));
        entries[0xac].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryac as usize));
        entries[0xad].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryad as usize));
        entries[0xae].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryae as usize));
        entries[0xaf].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryaf as usize));
        entries[0xb0].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryb0 as usize));
        entries[0xb1].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryb1 as usize));
        entries[0xb2].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryb2 as usize));
        entries[0xb3].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryb3 as usize));
        entries[0xb4].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryb4 as usize));
        entries[0xb5].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryb5 as usize));
        entries[0xb6].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryb6 as usize));
        entries[0xb7].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryb7 as usize));
        entries[0xb8].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryb8 as usize));
        entries[0xb9].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryb9 as usize));
        entries[0xba].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryba as usize));
        entries[0xbb].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrybb as usize));
        entries[0xbc].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrybc as usize));
        entries[0xbd].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrybd as usize));
        entries[0xbe].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrybe as usize));
        entries[0xbf].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrybf as usize));
        entries[0xc0].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryc0 as usize));
        entries[0xc1].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryc1 as usize));
        entries[0xc2].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryc2 as usize));
        entries[0xc3].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryc3 as usize));
        entries[0xc4].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryc4 as usize));
        entries[0xc5].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryc5 as usize));
        entries[0xc6].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryc6 as usize));
        entries[0xc7].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryc7 as usize));
        entries[0xc8].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryc8 as usize));
        entries[0xc9].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryc9 as usize));
        entries[0xca].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryca as usize));
        entries[0xcb].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrycb as usize));
        entries[0xcc].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrycc as usize));
        entries[0xcd].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrycd as usize));
        entries[0xce].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryce as usize));
        entries[0xcf].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrycf as usize));
        entries[0xd0].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryd0 as usize));
        entries[0xd1].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryd1 as usize));
        entries[0xd2].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryd2 as usize));
        entries[0xd3].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryd3 as usize));
        entries[0xd4].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryd4 as usize));
        entries[0xd5].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryd5 as usize));
        entries[0xd6].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryd6 as usize));
        entries[0xd7].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryd7 as usize));
        entries[0xd8].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryd8 as usize));
        entries[0xd9].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryd9 as usize));
        entries[0xda].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryda as usize));
        entries[0xdb].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrydb as usize));
        entries[0xdc].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrydc as usize));
        entries[0xdd].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrydd as usize));
        entries[0xde].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryde as usize));
        entries[0xdf].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrydf as usize));
        entries[0xe0].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrye0 as usize));
        entries[0xe1].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrye1 as usize));
        entries[0xe2].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrye2 as usize));
        entries[0xe3].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrye3 as usize));
        entries[0xe4].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrye4 as usize));
        entries[0xe5].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrye5 as usize));
        entries[0xe6].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrye6 as usize));
        entries[0xe7].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrye7 as usize));
        entries[0xe8].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrye8 as usize));
        entries[0xe9].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentrye9 as usize));
        entries[0xea].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryea as usize));
        entries[0xeb].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryeb as usize));
        entries[0xec].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryec as usize));
        entries[0xed].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryed as usize));
        entries[0xee].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryee as usize));
        entries[0xef].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryef as usize));
        entries[0xf0].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryf0 as usize));
        entries[0xf1].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryf1 as usize));
        entries[0xf2].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryf2 as usize));
        entries[0xf3].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryf3 as usize));
        entries[0xf4].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryf4 as usize));
        entries[0xf5].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryf5 as usize));
        entries[0xf6].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryf6 as usize));
        entries[0xf7].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryf7 as usize));
        entries[0xf8].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryf8 as usize));
        entries[0xf9].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryf9 as usize));
        entries[0xfa].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryfa as usize));
        entries[0xfb].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryfb as usize));
        entries[0xfc].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryfc as usize));
        entries[0xfd].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryfd as usize));
        entries[0xfe].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryfe as usize));
        entries[0xff].set_handler_fn(transmute::<
            usize,
            extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
        >(intentryff as usize));
    }

    unsafe {
        idt.double_fault
            .set_handler_fn(transmute::<
                usize,
                extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame, u64) -> !,
            >(intentry08 as usize))
            .set_stack_index(DOUBLE_FAULT_IST_INDEX as u16);

        idt.page_fault
            .set_handler_fn(transmute::<
                usize,
                extern "x86-interrupt" fn(
                    x86_64::structures::idt::InterruptStackFrame,
                    PageFaultErrorCode,
                ) -> (),
            >(intentry0e as usize))
            .set_stack_index(PAGE_FAULT_IST_INDEX as u16);
    }

    idt
});

core::arch::global_asm!(include_str!("entry.asm"));
unsafe extern "C" {
    fn intentry00();
    fn intentry01();
    fn intentry02();
    fn intentry03();
    fn intentry04();
    fn intentry05();
    fn intentry06();
    fn intentry07();
    fn intentry08();
    fn intentry09();
    fn intentry0a();
    fn intentry0b();
    fn intentry0c();
    fn intentry0d();
    fn intentry0e();
    fn intentry0f();
    fn intentry10();
    fn intentry11();
    fn intentry12();
    fn intentry13();
    fn intentry14();
    fn intentry15();
    fn intentry16();
    fn intentry17();
    fn intentry18();
    fn intentry19();
    fn intentry1a();
    fn intentry1b();
    fn intentry1c();
    fn intentry1d();
    fn intentry1e();
    fn intentry1f();
    fn intentry20();
    fn intentry21();
    fn intentry22();
    fn intentry23();
    fn intentry24();
    fn intentry25();
    fn intentry26();
    fn intentry27();
    fn intentry28();
    fn intentry29();
    fn intentry2a();
    fn intentry2b();
    fn intentry2c();
    fn intentry2d();
    fn intentry2e();
    fn intentry2f();
    fn intentry30();
    fn intentry31();
    fn intentry32();
    fn intentry33();
    fn intentry34();
    fn intentry35();
    fn intentry36();
    fn intentry37();
    fn intentry38();
    fn intentry39();
    fn intentry3a();
    fn intentry3b();
    fn intentry3c();
    fn intentry3d();
    fn intentry3e();
    fn intentry3f();
    fn intentry40();
    fn intentry41();
    fn intentry42();
    fn intentry43();
    fn intentry44();
    fn intentry45();
    fn intentry46();
    fn intentry47();
    fn intentry48();
    fn intentry49();
    fn intentry4a();
    fn intentry4b();
    fn intentry4c();
    fn intentry4d();
    fn intentry4e();
    fn intentry4f();
    fn intentry50();
    fn intentry51();
    fn intentry52();
    fn intentry53();
    fn intentry54();
    fn intentry55();
    fn intentry56();
    fn intentry57();
    fn intentry58();
    fn intentry59();
    fn intentry5a();
    fn intentry5b();
    fn intentry5c();
    fn intentry5d();
    fn intentry5e();
    fn intentry5f();
    fn intentry60();
    fn intentry61();
    fn intentry62();
    fn intentry63();
    fn intentry64();
    fn intentry65();
    fn intentry66();
    fn intentry67();
    fn intentry68();
    fn intentry69();
    fn intentry6a();
    fn intentry6b();
    fn intentry6c();
    fn intentry6d();
    fn intentry6e();
    fn intentry6f();
    fn intentry70();
    fn intentry71();
    fn intentry72();
    fn intentry73();
    fn intentry74();
    fn intentry75();
    fn intentry76();
    fn intentry77();
    fn intentry78();
    fn intentry79();
    fn intentry7a();
    fn intentry7b();
    fn intentry7c();
    fn intentry7d();
    fn intentry7e();
    fn intentry7f();
    fn intentry80();
    fn intentry81();
    fn intentry82();
    fn intentry83();
    fn intentry84();
    fn intentry85();
    fn intentry86();
    fn intentry87();
    fn intentry88();
    fn intentry89();
    fn intentry8a();
    fn intentry8b();
    fn intentry8c();
    fn intentry8d();
    fn intentry8e();
    fn intentry8f();
    fn intentry90();
    fn intentry91();
    fn intentry92();
    fn intentry93();
    fn intentry94();
    fn intentry95();
    fn intentry96();
    fn intentry97();
    fn intentry98();
    fn intentry99();
    fn intentry9a();
    fn intentry9b();
    fn intentry9c();
    fn intentry9d();
    fn intentry9e();
    fn intentry9f();
    fn intentrya0();
    fn intentrya1();
    fn intentrya2();
    fn intentrya3();
    fn intentrya4();
    fn intentrya5();
    fn intentrya6();
    fn intentrya7();
    fn intentrya8();
    fn intentrya9();
    fn intentryaa();
    fn intentryab();
    fn intentryac();
    fn intentryad();
    fn intentryae();
    fn intentryaf();
    fn intentryb0();
    fn intentryb1();
    fn intentryb2();
    fn intentryb3();
    fn intentryb4();
    fn intentryb5();
    fn intentryb6();
    fn intentryb7();
    fn intentryb8();
    fn intentryb9();
    fn intentryba();
    fn intentrybb();
    fn intentrybc();
    fn intentrybd();
    fn intentrybe();
    fn intentrybf();
    fn intentryc0();
    fn intentryc1();
    fn intentryc2();
    fn intentryc3();
    fn intentryc4();
    fn intentryc5();
    fn intentryc6();
    fn intentryc7();
    fn intentryc8();
    fn intentryc9();
    fn intentryca();
    fn intentrycb();
    fn intentrycc();
    fn intentrycd();
    fn intentryce();
    fn intentrycf();
    fn intentryd0();
    fn intentryd1();
    fn intentryd2();
    fn intentryd3();
    fn intentryd4();
    fn intentryd5();
    fn intentryd6();
    fn intentryd7();
    fn intentryd8();
    fn intentryd9();
    fn intentryda();
    fn intentrydb();
    fn intentrydc();
    fn intentrydd();
    fn intentryde();
    fn intentrydf();
    fn intentrye0();
    fn intentrye1();
    fn intentrye2();
    fn intentrye3();
    fn intentrye4();
    fn intentrye5();
    fn intentrye6();
    fn intentrye7();
    fn intentrye8();
    fn intentrye9();
    fn intentryea();
    fn intentryeb();
    fn intentryec();
    fn intentryed();
    fn intentryee();
    fn intentryef();
    fn intentryf0();
    fn intentryf1();
    fn intentryf2();
    fn intentryf3();
    fn intentryf4();
    fn intentryf5();
    fn intentryf6();
    fn intentryf7();
    fn intentryf8();
    fn intentryf9();
    fn intentryfa();
    fn intentryfb();
    fn intentryfc();
    fn intentryfd();
    fn intentryfe();
    fn intentryff();
}
