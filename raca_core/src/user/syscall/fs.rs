use core::alloc::Layout;

use crate::{
    fs::{
        operation::OpenMode,
        vfs::inode::{FileInfo, InodeTy},
    },
    user::get_current_process,
};
use alloc::{string::String, vec, vec::Vec};
use framework::{
    memory::{addr_to_array, addr_to_mut_ref, write_for_syscall},
    ref_to_mut,
};

use x86_64::VirtAddr;

pub fn open(buf_addr: usize, buf_len: usize, open_mode: usize) -> usize {
    let mut buf = vec![0; buf_len];

    if let Err(_) = get_current_process().read().page_table.read(
        VirtAddr::new(buf_addr as u64),
        buf_len,
        &mut buf,
    ) {
        panic!("Read error at {:x}!", buf_addr);
    }

    let path = String::from(core::str::from_utf8(buf.as_slice()).unwrap());

    let open_mode = match open_mode {
        0 => OpenMode::Read,
        1 => OpenMode::Write,
        _ => return 0,
    };

    let fd = crate::fs::operation::open(path.clone(), open_mode);
    if let Some(fd) = fd {
        fd
    } else {
        0
    }
}

pub fn write(fd: usize, buf_addr: usize, buf_len: usize) -> usize {
    let mut buf = vec![0; buf_len];

    if let Err(_) = get_current_process().read().page_table.read(
        VirtAddr::new(buf_addr as u64),
        buf_len,
        &mut buf,
    ) {
        panic!("Read error at {:x}!", buf_addr);
    }

    if let Some(_) = crate::fs::operation::write(fd, buf.as_slice()) {
        buf_len
    } else {
        0
    }
}

pub fn read(fd: usize, buf_addr: usize, buf_len: usize) -> usize {
    let mut buf = vec![0; buf_len];

    let ok = crate::fs::operation::read(fd, buf.as_mut()).is_some();

    write_for_syscall(VirtAddr::new(buf_addr as u64), buf.as_slice());

    let mut buf = vec![0; buf_len];

    if let Err(_) = get_current_process().read().page_table.read(
        VirtAddr::new(buf_addr as u64),
        buf_len,
        &mut buf,
    ) {
        panic!("Read error at {:x}!", buf_addr);
    }

    ok as usize * buf_len
}

pub fn close(fd: usize) -> usize {
    if let Some(_) = crate::fs::operation::close(fd) {
        1
    } else {
        0
    }
}

pub fn lseek(fd: usize, offset: usize) -> usize {
    if let Some(_) = crate::fs::operation::lseek(fd, offset) {
        1
    } else {
        0
    }
}

pub fn fsize(fd: usize) -> usize {
    if let Some(size) = crate::fs::operation::fsize(fd) {
        size
    } else {
        0
    }
}

pub fn open_pipe(buf_addr: usize) -> usize {
    let buffer: &mut [usize] = addr_to_array::<usize>(VirtAddr::new(buf_addr as u64), 2);
    if let Some(_) = crate::fs::operation::open_pipe(buffer) {
        1
    } else {
        0
    }
}

pub fn dir_item_num(path_addr: usize, path_len: usize) -> usize {
    let mut buf = vec![0; path_len];

    if let Err(_) = get_current_process().read().page_table.read(
        VirtAddr::new(path_addr as u64),
        path_len,
        &mut buf,
    ) {
        panic!("Read error at {:x}!", path_addr);
    }

    let path = String::from(core::str::from_utf8(buf.as_slice()).unwrap());

    let file_infos = crate::fs::operation::list_dir(path);

    file_infos.len()
}

pub fn list_dir(path_addr: usize, path_len: usize, buf_addr: usize) -> usize {
    let mut buf = vec![0; path_len];

    if let Err(_) = get_current_process().read().page_table.read(
        VirtAddr::new(path_addr as u64),
        path_len,
        &mut buf,
    ) {
        panic!("Read error at {:x}!", path_addr);
    }

    let path = String::from(core::str::from_utf8(buf.as_slice()).unwrap());

    #[derive(Clone)]
    #[allow(dead_code)]
    struct TemporyInfo {
        name: &'static [u8],
        ty: InodeTy,
    }

    let file_infos: Vec<TemporyInfo> = {
        let infos = crate::fs::operation::list_dir(path);
        let mut new_infos = Vec::new();
        for info in infos.iter() {
            let FileInfo { name, ty } = info;
            let new_name = ref_to_mut(&*get_current_process().read())
                .heap
                .allocate(Layout::from_size_align(name.len(), 8).unwrap())
                .unwrap();
            let new_name = addr_to_array(VirtAddr::new(new_name), name.len());
            new_name[..name.len()].copy_from_slice(name.as_bytes());
            new_infos.push(TemporyInfo {
                name: new_name,
                ty: *ty,
            });
        }
        new_infos
    };

    write_for_syscall(VirtAddr::new(buf_addr as u64), file_infos.as_slice());

    0
}

pub fn change_cwd(path_addr: usize, path_len: usize) -> usize {
    let mut buf = vec![0; path_len];

    if let Err(_) = get_current_process().read().page_table.read(
        VirtAddr::new(path_addr as u64),
        path_len,
        &mut buf,
    ) {
        panic!("Read error at {:x}!", path_addr);
    }

    let path = String::from(core::str::from_utf8(buf.as_slice()).unwrap());

    crate::fs::operation::change_cwd(path);

    0
}

pub fn get_cwd() -> usize {
    let path = crate::fs::operation::get_cwd();
    let new_path_ptr = ref_to_mut(&*get_current_process().read())
                .heap
                .allocate(Layout::from_size_align(path.len(), 8).unwrap())
                .unwrap();
    let new_path = addr_to_array(VirtAddr::new(new_path_ptr), path.len());
    new_path[..path.len()].copy_from_slice(path.as_bytes());
    let ret_struct_ptr = ref_to_mut(&*get_current_process().read())
                .heap
                .allocate(Layout::from_size_align(16, 8).unwrap())
                .unwrap();
    let path_ptr = addr_to_mut_ref(VirtAddr::new(ret_struct_ptr));
    *path_ptr = new_path_ptr;
    let len_ptr = addr_to_mut_ref(VirtAddr::new(ret_struct_ptr + 8));
    *len_ptr = path.len();
    ret_struct_ptr as usize
}

