//! Persistent 512-byte block device backed by the secondary ATA/IDE disk.

pub const BLOCK_SIZE: usize = 512;
pub const BLOCK_COUNT: usize = 4096;

const ATA_DATA: u16 = 0x1F0;
const ATA_SECTOR_COUNT: u16 = 0x1F2;
const ATA_LBA0: u16 = 0x1F3;
const ATA_LBA1: u16 = 0x1F4;
const ATA_LBA2: u16 = 0x1F5;
const ATA_DRIVE: u16 = 0x1F6;
const ATA_STATUS: u16 = 0x1F7;
const ATA_COMMAND: u16 = 0x1F7;
const ATA_ALT_STATUS: u16 = 0x3F6;
const ATA_SLAVE: u8 = 0xF0;
const ATA_CMD_READ: u8 = 0x20;
const ATA_CMD_WRITE: u8 = 0x30;
const ATA_CMD_IDENTIFY: u8 = 0xEC;
const ATA_BSY: u8 = 0x80;
const ATA_DRQ: u8 = 0x08;
const ATA_ERR: u8 = 0x01;

static mut READY: bool = false;

pub fn init() {
    unsafe { READY = identify(); }
}

pub fn available() -> bool {
    unsafe { READY }
}

pub fn read(block: usize, out: &mut [u8; BLOCK_SIZE]) -> bool {
    read_many(block, out)
}

pub fn write(block: usize, input: &[u8; BLOCK_SIZE]) -> bool {
    write_many(block, input)
}

pub fn read_many(start: usize, out: &mut [u8]) -> bool {
    transfer_many(start, out, false)
}

pub fn write_many(start: usize, input: &[u8]) -> bool {
    transfer_many(start, input, true)
}

fn transfer_many(start: usize, buffer: &mut [u8], write: bool) -> bool {
    if buffer.is_empty() || buffer.len() % BLOCK_SIZE != 0 { return false; }
    let sectors = buffer.len() / BLOCK_SIZE;
    if start >= BLOCK_COUNT || sectors == 0 || sectors > 127 || start + sectors > BLOCK_COUNT || !available() {
        return false;
    }
    unsafe { ata_transfer_many(start as u32, buffer, write) }
}

unsafe fn identify() -> bool {
    crate::arch::outb(ATA_DRIVE, ATA_SLAVE);
    crate::arch::outb(ATA_SECTOR_COUNT, 0);
    crate::arch::outb(ATA_LBA0, 0);
    crate::arch::outb(ATA_LBA1, 0);
    crate::arch::outb(ATA_LBA2, 0);
    crate::arch::outb(ATA_COMMAND, ATA_CMD_IDENTIFY);

    if !wait_not_busy_ok() { return false; }
    if !wait_drq() { return false; }
    for _ in 0..256 { let _ = read_data_word(); }
    wait_not_busy_ok()
}

unsafe fn ata_transfer_many(lba: u32, buffer: &mut [u8], write: bool) -> bool {
    if !wait_not_busy_ok() { return false; }

    crate::arch::outb(ATA_DRIVE, ATA_SLAVE | ((lba >> 24) as u8 & 0x0F));
    ata_delay();
    let sectors = (buffer.len() / BLOCK_SIZE) as u8;
    crate::arch::outb(ATA_SECTOR_COUNT, sectors);
    crate::arch::outb(ATA_LBA0, lba as u8);
    crate::arch::outb(ATA_LBA1, (lba >> 8) as u8);
    crate::arch::outb(ATA_LBA2, (lba >> 16) as u8);
    crate::arch::outb(ATA_COMMAND, if write { ATA_CMD_WRITE } else { ATA_CMD_READ });

    if !wait_drq() { return false; }

    let count = buffer.len() / BLOCK_SIZE;
    for sector in 0..count {
        let base = sector * BLOCK_SIZE;
        if write {
            for i in 0..256 {
                let lo = buffer[base + i * 2] as u16;
                let hi = (buffer[base + i * 2 + 1] as u16) << 8;
                write_data_word(lo | hi);
            }
        } else {
            for i in 0..256 {
                let word = read_data_word();
                buffer[base + i * 2] = word as u8;
                buffer[base + i * 2 + 1] = (word >> 8) as u8;
            }
        }
        if sector + 1 < count && !wait_drq() { return false; }
    }

    wait_not_busy_ok()
}

unsafe fn wait_not_busy_ok() -> bool {
    let mut status = 0u8;
    for _ in 0..100_000 {
        status = crate::arch::inb(ATA_STATUS);
        if status & ATA_ERR != 0 { return false; }
        if status & ATA_BSY == 0 { return true; }
    }
    false
}

unsafe fn wait_drq() -> bool {
    for _ in 0..100_000 {
        let status = crate::arch::inb(ATA_STATUS);
        if status & ATA_ERR != 0 { return false; }
        if status & ATA_BSY == 0 && status & ATA_DRQ != 0 { return true; }
    }
    false
}

unsafe fn ata_delay() {
    let _ = crate::arch::inb(ATA_ALT_STATUS);
    let _ = crate::arch::inb(ATA_ALT_STATUS);
    let _ = crate::arch::inb(ATA_ALT_STATUS);
    let _ = crate::arch::inb(ATA_ALT_STATUS);
}

unsafe fn read_data_word() -> u16 {
    crate::arch::inw(ATA_DATA)
}

unsafe fn write_data_word(value: u16) {
    crate::arch::outw(ATA_DATA, value)
}
