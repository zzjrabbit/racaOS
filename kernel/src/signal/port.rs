use crate::object::{HandleValue, KernelObject, Signal};
use crate::task::scheduler::SCHEDULER;
use crate::task::thread::Thread;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

crate::kernel_object! {
    pub struct Port {
        inner: Mutex<PortInner> = Mutex::new(PortInner {bindings: Vec::new(), packets: Vec::new(), waiter: None}),
    }

    fn new() {}
}

struct PortInner {
    bindings: Vec<(Arc<dyn KernelObject>, HandleValue)>,
    packets: Vec<PortPacket>,
    waiter: Option<(Signal, Arc<Thread>)>,
}

impl Port {
    pub fn bind_to(self: &Arc<Self>, object: Arc<dyn KernelObject>, handle: HandleValue) {
        let port = self.clone();

        self.inner.lock().bindings.push((object.clone(), handle));
        let another_object = object.clone();
        object.add_signal_callback(Box::new(move |signal| {
            log::info!("OK {:?}", signal);
            if signal.is_empty() {
                return false;
            }

            let mut inner = port.inner.lock();

            let (_, source) = inner
                .bindings
                .iter()
                .find(|(ko, _)| ko.id() == another_object.id())
                .cloned()
                .unwrap();

            inner.packets.push(PortPacket { signal, source });

            if let Some((signal_waiting, waiter)) = inner.waiter.clone() {
                log::info!("waiter");
                if signal.contains(signal_waiting) {
                    //waiter.wake_up();
                    SCHEDULER.add_thread(&waiter);
                }
            }

            false
        }));
    }

    pub fn wait_for_packet(&self, signal: Signal) -> PortPacket {
        log::info!("wait for {:?}", signal);

        let mut inner = self.inner.lock();

        if let Some((id, packet)) = inner
            .packets
            .iter()
            .enumerate()
            .find(|(_, packet)| packet.signal.contains(signal))
            .map(|(id, packet)| (id, packet.clone()))
        {
            inner.packets.remove(id);
            log::info!("got");
            return packet.clone();
        }

        let current_thread = SCHEDULER.current_thread().upgrade().unwrap();
        //current_thread.begin_sleep();

        inner.waiter = Some((signal, current_thread.clone()));

        drop(inner);

        unsafe {
            core::arch::asm!("int 0x21");
        }

        let mut inner = self.inner.lock();

        log::info!("trying to get {:?} {:?}", signal, inner.packets);

        let (id, packet) = inner
            .packets
            .iter()
            .enumerate()
            .find(|(_, packet)| packet.signal.contains(signal))
            .map(|(id, packet)| (id, packet.clone()))
            .unwrap();

        inner.packets.remove(id);
        inner.waiter = None;
        packet
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct PortPacket {
    signal: Signal,
    source: HandleValue,
}
