use alloc::{string::ToString, sync::Arc};
use fat32::Fat32Volume;
use limine::request::KernelFileRequest;
use spin::{Lazy, Mutex, RwLock};
use uuid::Uuid;
use vfs::{
    dev::ROOT_PARTITION,
    inode::{mount_to, Inode, InodeRef},
    root::RootFS,
};

//mod ext2;
mod fat32;
pub mod operation;
pub mod vfs;

pub static ROOT: Lazy<Mutex<InodeRef>> =
    Lazy::new(|| Mutex::new(Arc::new(RwLock::new(RootFS::new()))));

#[used]
static KERNEL_FILE_REQUEST: KernelFileRequest = KernelFileRequest::new();

pub fn get_root_partition_uuid() -> Uuid {
    let kernel_file_response = KERNEL_FILE_REQUEST.get_response().unwrap();
    Uuid::from(kernel_file_response.file().gpt_partition_id().unwrap())
}

pub fn init() {
    //crate::drivers::ahci::init();
    ROOT.lock().read().when_mounted("/".to_string(), None);

    framework::serial_println!("Root Partition: {:?}", get_root_partition_uuid());

    vfs::dev::init();

    let root_partition = ROOT_PARTITION.lock().clone().unwrap().clone();
    let root_fs = Arc::new(RwLock::new(Fat32Volume::new(root_partition.clone())));

    let dev_fs = ROOT.lock().read().open("dev".into()).unwrap();

    *ROOT.lock() = root_fs.clone();

    root_fs.read().when_mounted("/".to_string(), None);
    dev_fs.read().when_umounted();
    mount_to(dev_fs.clone(), root_fs.clone(), "dev".to_string());
}
