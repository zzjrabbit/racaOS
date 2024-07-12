use alloc::vec::Vec;

#[repr(C, packed)]
struct BootSector {
    jmp_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sector_count: u16,
    num_fats: u8,
    root_entry_count: u16,
    total_sectors_16: u16,
    media: u8,
    fat_size_16: u16,
    sectors_per_track: u16,
    num_heads: u16,
    hidden_sectors: u32,
    total_sectors_32: u32,
    fat_size_32: u32,
    ext_flags: u16,
    fs_version: u16,
    root_cluster: u32,
    fs_info: u16,
    bk_boot_sec: u16,
    reserved: [u8; 12],
    drive_number: u8,
    reserved1: u8,
    boot_signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    fs_type: [u8; 8],
}

struct Fat32 {
    boot_sector: BootSector,
    fat: Vec<u32>,
    data: Vec<u8>,
}

impl Fat32 {
    fn new(boot_sector: BootSector, fat: Vec<u32>, data: Vec<u8>) -> Self {
        Fat32 {
            boot_sector,
            fat,
            data,
        }
    }

    fn read_cluster(&self, cluster: u32) -> &[u8] {
        let cluster_size = self.boot_sector.bytes_per_sector as usize * self.boot_sector.sectors_per_cluster as usize;
        let start = (cluster as usize - 2) * cluster_size;
        let end = start + cluster_size;
        &self.data[start..end]
    }
}
