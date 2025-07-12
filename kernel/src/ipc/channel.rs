use alloc::{
    collections::vec_deque::VecDeque,
    sync::{Arc, Weak},
    vec::Vec,
};

use crate::{
    error::*,
    kernel_object,
    object::{Handle, KObjectBase, KernelObject, KoID, Signal},
};

use spin::Mutex;

#[derive(Default)]
pub struct MessagePacket {
    pub data: Vec<u8>,
    pub handles: Vec<Handle>,
}

impl MessagePacket {
    pub fn new(data: Vec<u8>, handles: Vec<Handle>) -> Self {
        Self {
            data: data,
            handles: handles,
        }
    }
}

kernel_object! {
    pub struct Channel {
        peer: Weak<Channel>,
        recv_queue: Mutex<VecDeque<MessagePacket>>,
    }

    fn peer(&self) -> RcResult<Arc<dyn KernelObject>> {
        let peer = self.peer.upgrade().ok_or(RcError::PeerClosed)?;
        Ok(peer)
    }

    fn related_koid(&self) -> KoID {
        self.peer.upgrade().map(|p| p.id()).unwrap_or(0)
    }
}

impl Channel {
    #[allow(unsafe_code)]
    pub fn create() -> (Arc<Self>, Arc<Self>) {
        let mut channel0 = Arc::new(Channel {
            base: KObjectBase::default(),
            peer: Weak::default(),
            recv_queue: Default::default(),
        });
        let channel1 = Arc::new(Channel {
            base: KObjectBase::default(),
            peer: Arc::downgrade(&channel0),
            recv_queue: Default::default(),
        });

        unsafe {
            Arc::get_mut_unchecked(&mut channel0).peer = Arc::downgrade(&channel1);
        }
        (channel0, channel1)
    }

    pub fn read(&self) -> RcResult<MessagePacket> {
        let mut recv_queue = self.recv_queue.lock();
        if let Some(_msg) = recv_queue.front() {
            let msg = recv_queue.pop_front().unwrap();

            if recv_queue.len() == 0 {
                //log::info!("read all");
                self.clear_signal(Signal::READABLE);
            }

            return Ok(msg);
        }

        //log::info!("unreadable");
        self.clear_signal(Signal::READABLE);
        if self.peer_closed() {
            Err(RcError::PeerClosed)
        } else {
            Err(RcError::ShouldWait)
        }
    }

    pub fn write(&self, msg: MessagePacket) -> RcResult<()> {
        let peer = self.peer.upgrade().ok_or(RcError::PeerClosed)?;
        peer.push_general(msg);
        Ok(())
    }

    fn push_general(&self, msg: MessagePacket) {
        let mut send_queue = self.recv_queue.lock();
        send_queue.push_back(msg);
        self.set_signal(Signal::READABLE);
    }

    fn peer_closed(&self) -> bool {
        self.peer.strong_count() == 0
    }
}
