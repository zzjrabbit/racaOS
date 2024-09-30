use {
    super::*,
    crate::{error::*, object::*},
    alloc::{
        collections::VecDeque,
        sync::{Arc, Weak},
        vec::Vec,
    },
    rc_object_derive::kernel_object,
    spin::Mutex,
};

#[derive(Default)]
pub struct MessagePacket {
    pub data: Vec<u8>,
    pub handles: Vec<Handle>,
}

#[kernel_object(fn peer(&self) -> RcResult<Arc<dyn KernelObject>> {
    let peer = self.peer.upgrade().ok_or(RcError::PEER_CLOSED)?;
    Ok(peer)
}
fn related_koid(&self) -> KoID {
    self.peer.upgrade().map(|p| p.id()).unwrap_or(0)
})]
pub struct Channel {
    peer: Weak<Channel>,
    recv_queue: Mutex<VecDeque<T>>,
}

type T = MessagePacket;

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
        // no other reference of `channel0`
        unsafe {
            Arc::get_mut_unchecked(&mut channel0).peer = Arc::downgrade(&channel1);
        }
        (channel0, channel1)
    }

    pub fn read(&self) -> RcResult<T> {
        let mut recv_queue = self.recv_queue.lock();
        if let Some(_msg) = recv_queue.front() {
            let msg = recv_queue.pop_front().unwrap();
            return Ok(msg);
        }
        if self.peer_closed() {
            Err(RcError::PEER_CLOSED)
        } else {
            Err(RcError::SHOULD_WAIT)
        }
    }

    pub fn write(&self, msg: T) -> RcResult {
        let peer = self.peer.upgrade().ok_or(RcError::PEER_CLOSED)?;
        peer.push_general(msg);
        Ok(())
    }

    fn push_general(&self, msg: T) {
        let mut send_queue = self.recv_queue.lock();
        send_queue.push_back(msg);
    }

    fn peer_closed(&self) -> bool {
        self.peer.strong_count() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basics() {
        let (end0, end1) = Channel::create();
        assert!(Arc::ptr_eq(
            &end0.peer().unwrap().downcast_arc().unwrap(),
            &end1
        ));
        assert_eq!(end0.related_koid(), end1.id());

        drop(end1);
        assert_eq!(end0.peer().unwrap_err(), RcError::PEER_CLOSED);
        assert_eq!(end0.related_koid(), 0);
    }

    #[test]
    fn read_write() {
        let (channel0, channel1) = Channel::create();
        // write a message to each other
        channel0
            .write(MessagePacket {
                data: Vec::from("hello 1"),
                handles: Vec::new(),
            })
            .unwrap();

        channel1
            .write(MessagePacket {
                data: Vec::from("hello 0"),
                handles: Vec::new(),
            })
            .unwrap();

        // read message should success
        let recv_msg = channel1.read().unwrap();
        assert_eq!(recv_msg.data.as_slice(), b"hello 1");
        assert!(recv_msg.handles.is_empty());

        let recv_msg = channel0.read().unwrap();
        assert_eq!(recv_msg.data.as_slice(), b"hello 0");
        assert!(recv_msg.handles.is_empty());

        // read more message should fail.
        assert_eq!(channel0.read().err(), Some(RcError::SHOULD_WAIT));
        assert_eq!(channel1.read().err(), Some(RcError::SHOULD_WAIT));
    }
}
