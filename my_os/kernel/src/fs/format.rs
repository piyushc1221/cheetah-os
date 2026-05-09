pub const FS_START_LBA: u32 = 2048;
pub const BLOCK_SIZE: usize = 512;
pub const MAGIC: &[u8; 8] = b"TINYFS1\0";
pub const MAX_FILES: usize = 16;
pub const ENTRY_SIZE: usize = 64;
pub const TABLE_SECTORS: u32 = 2;
pub const FILE_TABLE_LBA: u32 = FS_START_LBA + 1;
pub const DATA_START_LBA: u32 = FS_START_LBA + 1 + TABLE_SECTORS;

pub fn make_superblock(buf: &mut [u8; BLOCK_SIZE]) {
    buf.fill(0);
    buf[0..8].copy_from_slice(MAGIC);
    buf[8..12].copy_from_slice(&(BLOCK_SIZE as u32).to_le_bytes());
    buf[12..16].copy_from_slice(&(MAX_FILES as u32).to_le_bytes());
    buf[16..20].copy_from_slice(&TABLE_SECTORS.to_le_bytes());
    buf[20..24].copy_from_slice(&DATA_START_LBA.to_le_bytes());
}

pub fn valid_superblock(buf: &[u8; BLOCK_SIZE]) -> bool {
    &buf[0..8] == MAGIC
}
