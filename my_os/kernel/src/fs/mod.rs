pub mod disk;
pub mod format;
pub mod inode;

use self::format::{
    make_superblock, valid_superblock, BLOCK_SIZE, DATA_START_LBA, FILE_TABLE_LBA, MAX_FILES,
    TABLE_SECTORS,
};
use self::inode::{entry_range, FileEntry};

pub use self::format::FS_START_LBA;

static mut FILES: [FileEntry; MAX_FILES] = [FileEntry::empty(); MAX_FILES];

#[derive(Debug)]
pub enum FsError {
    Disk,
    NotFound,
    NoSpace,
    NameTooLong,
    FileTooLarge,
}

pub fn init() -> Result<(), FsError> {
    let mut sector = [0u8; BLOCK_SIZE];
    disk::read_sector(FS_START_LBA, &mut sector).map_err(|_| FsError::Disk)?;

    if !valid_superblock(&sector) {
        format_disk()?;
    }

    load_table()
}

pub fn list() {
    unsafe {
        let mut printed = false;
        for entry in FILES.iter() {
            if entry.used {
                entry.print_name();
                crate::println!("  {} bytes", entry.size);
                printed = true;
            }
        }
        if !printed {
            crate::println!("(empty)");
        }
    }
}

pub fn read_file(name: &str, output: &mut [u8; BLOCK_SIZE]) -> Result<usize, FsError> {
    unsafe {
        let index = find_file(name).ok_or(FsError::NotFound)?;
        let entry = FILES[index];
        disk::read_sector(DATA_START_LBA + entry.start_block, output).map_err(|_| FsError::Disk)?;
        Ok(entry.size as usize)
    }
}

pub fn write_file(name: &str, data: &[u8]) -> Result<(), FsError> {
    if data.len() > BLOCK_SIZE {
        return Err(FsError::FileTooLarge);
    }
    if name.as_bytes().is_empty() || name.as_bytes().len() > 31 {
        return Err(FsError::NameTooLong);
    }

    unsafe {
        let index = match find_file(name) {
            Some(index) => index,
            None => find_free_entry().ok_or(FsError::NoSpace)?,
        };

        let mut sector = [0u8; BLOCK_SIZE];
        sector[0..data.len()].copy_from_slice(data);
        disk::write_sector(DATA_START_LBA + index as u32, &sector).map_err(|_| FsError::Disk)?;

        FILES[index].used = true;
        FILES[index].start_block = index as u32;
        FILES[index].size = data.len() as u32;
        if !FILES[index].set_name(name) {
            return Err(FsError::NameTooLong);
        }

        store_table()
    }
}

fn format_disk() -> Result<(), FsError> {
    let mut superblock = [0u8; BLOCK_SIZE];
    make_superblock(&mut superblock);
    disk::write_sector(FS_START_LBA, &superblock).map_err(|_| FsError::Disk)?;

    unsafe {
        FILES = [FileEntry::empty(); MAX_FILES];
    }
    store_table()
}

fn load_table() -> Result<(), FsError> {
    let mut table = [0u8; BLOCK_SIZE * TABLE_SECTORS as usize];
    let mut sector = [0u8; BLOCK_SIZE];

    for table_sector in 0..TABLE_SECTORS {
        disk::read_sector(FILE_TABLE_LBA + table_sector, &mut sector).map_err(|_| FsError::Disk)?;
        let start = table_sector as usize * BLOCK_SIZE;
        table[start..start + BLOCK_SIZE].copy_from_slice(&sector);
    }

    unsafe {
        for index in 0..MAX_FILES {
            FILES[index] = FileEntry::from_bytes(&table[entry_range(index)]);
        }
    }

    Ok(())
}

fn store_table() -> Result<(), FsError> {
    let mut table = [0u8; BLOCK_SIZE * TABLE_SECTORS as usize];

    unsafe {
        for index in 0..MAX_FILES {
            FILES[index].write_bytes(&mut table[entry_range(index)]);
        }
    }

    for table_sector in 0..TABLE_SECTORS {
        let mut sector = [0u8; BLOCK_SIZE];
        let start = table_sector as usize * BLOCK_SIZE;
        sector.copy_from_slice(&table[start..start + BLOCK_SIZE]);
        disk::write_sector(FILE_TABLE_LBA + table_sector, &sector).map_err(|_| FsError::Disk)?;
    }

    Ok(())
}

unsafe fn find_file(name: &str) -> Option<usize> {
    for (index, entry) in FILES.iter().enumerate() {
        if entry.used && entry.name_equals(name) {
            return Some(index);
        }
    }
    None
}

unsafe fn find_free_entry() -> Option<usize> {
    for (index, entry) in FILES.iter().enumerate() {
        if !entry.used {
            return Some(index);
        }
    }
    None
}
