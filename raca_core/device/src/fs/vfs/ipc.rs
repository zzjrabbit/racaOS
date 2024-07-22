use alloc::vec::Vec;

use super::inode::Inode;

pub struct IpcNode {
    queue: Vec<u8>,
}

impl Inode for IpcNode {
    fn when_mounted(&self, _path: alloc::string::String, _father: Option<super::inode::InodeRef>) {
        unimplemented!()
    }

    fn when_umounted(&self) {
        unimplemented!()
    }

    fn get_path(&self) -> alloc::string::String {
        "".into()
    }

    fn write_at(&self, _offset: usize, _buf: &[u8]) {
        ;
    }    
}

