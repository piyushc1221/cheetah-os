use crate::io;

const ATA_DATA: u16 = 0x1f0;
const ATA_SECTOR_COUNT: u16 = 0x1f2;
const ATA_LBA_LOW: u16 = 0x1f3;
const ATA_LBA_MID: u16 = 0x1f4;
const ATA_LBA_HIGH: u16 = 0x1f5;
const ATA_DRIVE: u16 = 0x1f6;
const ATA_COMMAND: u16 = 0x1f7;
const ATA_STATUS: u16 = 0x1f7;

const CMD_READ_SECTORS: u8 = 0x20;
const CMD_WRITE_SECTORS: u8 = 0x30;
const CMD_CACHE_FLUSH: u8 = 0xe7;

#[derive(Debug)]
pub enum DiskError {
    AtaError,
}

pub fn read_sector(lba: u32, buffer: &mut [u8; 512]) -> Result<(), DiskError> {
    unsafe {
        select_sector(lba);
        io::outb(ATA_COMMAND, CMD_READ_SECTORS);
        wait_data()?;

        for index in 0..256 {
            let word = io::inw(ATA_DATA);
            buffer[index * 2] = word as u8;
            buffer[index * 2 + 1] = (word >> 8) as u8;
        }
    }
    Ok(())
}

pub fn write_sector(lba: u32, buffer: &[u8; 512]) -> Result<(), DiskError> {
    unsafe {
        select_sector(lba);
        io::outb(ATA_COMMAND, CMD_WRITE_SECTORS);
        wait_data()?;

        for index in 0..256 {
            let low = buffer[index * 2] as u16;
            let high = (buffer[index * 2 + 1] as u16) << 8;
            io::outw(ATA_DATA, high | low);
        }

        wait_not_busy()?;
        io::outb(ATA_COMMAND, CMD_CACHE_FLUSH);
        wait_not_busy()?;
    }
    Ok(())
}

unsafe fn select_sector(lba: u32) {
    wait_not_busy().ok();
    io::outb(ATA_DRIVE, 0xe0 | ((lba >> 24) as u8 & 0x0f));
    io::outb(ATA_SECTOR_COUNT, 1);
    io::outb(ATA_LBA_LOW, lba as u8);
    io::outb(ATA_LBA_MID, (lba >> 8) as u8);
    io::outb(ATA_LBA_HIGH, (lba >> 16) as u8);
}

unsafe fn wait_not_busy() -> Result<(), DiskError> {
    for _ in 0..100000 {
        let status = io::inb(ATA_STATUS);
        if status & 0x80 == 0 {
            return Ok(());
        }
    }
    Err(DiskError::AtaError)
}

unsafe fn wait_data() -> Result<(), DiskError> {
    for _ in 0..100000 {
        let status = io::inb(ATA_STATUS);
        if status & 0x01 != 0 {
            return Err(DiskError::AtaError);
        }
        if status & 0x08 != 0 {
            return Ok(());
        }
    }
    Err(DiskError::AtaError)
}
