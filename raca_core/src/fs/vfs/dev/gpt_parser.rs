use core::str::FromStr;

use crate::alloc::string::ToString;
use crate::fs::{
    get_root_partition_uuid,
    vfs::{
        dev::{partition::PartitionInode, ROOT_PARTITION},
        inode::{mount_to, InodeRef},
    },
};

use alloc::{format, sync::Arc, vec::Vec};
use gpt_disk_io::{
    gpt_disk_types::{BlockSize, GptPartitionEntryArrayLayout, GptPartitionEntrySize, Lba},
    BlockIo, Disk, DiskError,
};
use spin::RwLock;

struct InodeRefIO {
    inode: InodeRef,
}

impl InodeRefIO {
    pub fn new(inode: InodeRef) -> Self {
        Self { inode }
    }
}

impl BlockIo for InodeRefIO {
    type Error = usize;

    fn block_size(&self) -> gpt_disk_io::gpt_disk_types::BlockSize {
        BlockSize::from_usize(512).unwrap()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn num_blocks(&mut self) -> Result<u64, Self::Error> {
        Ok((self.inode.read().size() / 512) as u64)
    }

    fn read_blocks(
        &mut self,
        start_lba: gpt_disk_io::gpt_disk_types::Lba,
        dst: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.inode.read().read_at(start_lba.0 as usize * 512, dst);
        Ok(())
    }

    fn write_blocks(
        &mut self,
        start_lba: gpt_disk_io::gpt_disk_types::Lba,
        src: &[u8],
    ) -> Result<(), Self::Error> {
        self.inode.read().write_at(start_lba.0 as usize * 512, src);
        Ok(())
    }
}

pub fn parse_gpt_disk(
    disk_id: usize,
    disk: InodeRef,
    dev_fs: InodeRef,
) -> Result<(), DiskError<usize>> {
    let io = InodeRefIO::new(disk.clone());
    let mut gpt = Disk::new(io)?;

    let mut buf = Vec::new();
    for _ in 0..512 * 8 * 100 {
        buf.push(0);
    }
    let buf = buf.leak();

    let header = gpt.read_gpt_header(Lba(1), buf)?;

    let mut buf = Vec::new();
    for _ in 0..512 * 8 * 100 {
        buf.push(0);
    }
    let buf = buf.leak();

    let part_iter = gpt.gpt_partition_entry_array_iter(
        GptPartitionEntryArrayLayout {
            start_lba: header.partition_entry_lba.into(),
            entry_size: GptPartitionEntrySize::new(header.size_of_partition_entry.to_u32())
                .ok()
                .ok_or(DiskError::Io(0))?,
            num_entries: header.number_of_partition_entries.to_u32(),
        },
        buf,
    )?;

    let id_to_alpha = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", "u", "v", "w", "x", "y", "z",
    ];

    let root_partition_uuid = get_root_partition_uuid();

    for (partition_id, part) in part_iter.enumerate() {
        if let Ok(part) = part {
            if !part.is_used() {
                break;
            }
            let start_offset = part.starting_lba.to_u64() as usize * 512;
            let size = part.ending_lba.to_u64() as usize * 512;

            let partition = PartitionInode::new(start_offset, size, disk.clone());
            let partition = Arc::new(RwLock::new(partition));

            let alpha = id_to_alpha[disk_id];
            let partition_name = format!("hd{}{}", alpha, partition_id);

            mount_to(partition.clone(), dev_fs.clone(), partition_name.clone());

            let guid = part.clone().unique_partition_guid;
            let uuid = uuid::Uuid::from_str(guid.to_string().as_str()).unwrap();

            if root_partition_uuid == uuid {
                *ROOT_PARTITION.lock() = Some(partition.clone());
            }
        }
    }

    Ok(())
}
