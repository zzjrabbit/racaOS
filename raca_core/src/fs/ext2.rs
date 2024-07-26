use alloc::vec::Vec;
use core::fmt;

// Define the SuperBlock structure
#[repr(C)]
struct SuperBlock {
    s_inodes_count: u32,
    s_blocks_count: u32,
    s_r_blocks_count: u32,
    s_free_blocks_count: u32,
    s_free_inodes_count: u32,
    s_first_data_block: u32,
    s_log_block_size: u32,
    s_log_frag_size: u32,
    s_blocks_per_group: u32,
    s_frags_per_group: u32,
    s_inodes_per_group: u32,
    s_mtime: u32,
    s_wtime: u32,
    s_mnt_count: u16,
    s_max_mnt_count: u16,
    s_magic: u16,
    s_state: u16,
    s_errors: u16,
    s_minor_rev_level: u16,
    s_lastcheck: u32,
    s_checkinterval: u32,
    s_creator_os: u32,
    s_rev_level: u32,
    s_def_resuid: u16,
    s_def_resgid: u16,
    // More fields are present in the actual superblock
}

impl fmt::Debug for SuperBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SuperBlock {{ s_inodes_count: {}, s_blocks_count: {}, s_magic: 0x{:X} }}",
            self.s_inodes_count, self.s_blocks_count, self.s_magic
        )
    }
}

// Define the Group Descriptor structure
#[repr(C)]
struct GroupDesc {
    bg_block_bitmap: u32,
    bg_inode_bitmap: u32,
    bg_inode_table: u32,
    bg_free_blocks_count: u16,
    bg_free_inodes_count: u16,
    bg_used_dirs_count: u16,
    bg_pad: u16,
    bg_reserved: [u8; 12],
}

impl fmt::Debug for GroupDesc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GroupDesc {{ bg_block_bitmap: {}, bg_inode_bitmap: {}, bg_inode_table: {} }}",
            self.bg_block_bitmap, self.bg_inode_bitmap, self.bg_inode_table
        )
    }
}

// Define the Inode structure
#[repr(C)]
struct Inode {
    i_mode: u16,
    i_uid: u16,
    i_size: u32,
    i_atime: u32,
    i_ctime: u32,
    i_mtime: u32,
    i_dtime: u32,
    i_gid: u16,
    i_links_count: u16,
    i_blocks: u32,
    i_flags: u32,
    i_osd1: u32,
    i_block: [u32; 15],
    i_generation: u32,
    i_file_acl: u32,
    i_dir_acl: u32,
    i_faddr: u32,
    i_osd2: [u8; 12],
}

impl fmt::Debug for Inode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Inode {{ i_mode: {}, i_uid: {}, i_size: {}, i_blocks: {} }}",
            self.i_mode, self.i_uid, self.i_size, self.i_blocks
        )
    }
}

// Define the Ext2 File System structure
struct Ext2Fs {
    superblock: SuperBlock,
    group_desc_table: Vec<GroupDesc>,
    inodes: Vec<Inode>,
}

impl Ext2Fs {
    fn new(superblock: SuperBlock, group_desc_table: Vec<GroupDesc>, inodes: Vec<Inode>) -> Self {
        Ext2Fs {
            superblock,
            group_desc_table,
            inodes,
        }
    }

    fn read_inode(&self, inode_index: usize) -> Option<&Inode> {
        self.inodes.get(inode_index)
    }
}
