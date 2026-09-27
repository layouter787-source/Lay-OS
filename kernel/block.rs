//! Persistent 512-byte block device backed by the primary ATA/IDE disk.
//! Lay storage uses a reserved persistent region on the boot disk.

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
const ATA_MASTER: u8 = 0xE0;
const ATA_CMD_READ: u8 = 0x20;
const ATA_CMD_WRITE: u8 = 0x30;
const ATA_CMD_IDENTIFY: u8 = 0xEC;
const ATA_BSY: u8 = 0x80;
const ATA_DRQ: u8 = 0x08;
const ATA_ERR: u8 = 0x01;

static mut READY: bool = false;

pub fn init() {
    unsafe {
        READY = identify();
    }
}

pub fn available() -> bool {
    unsafe { READY }
}

pub fn read(block: usize, out: &mut [u8; BLOCK_SIZE]) -> bool {
    if block >= BLOCK_COUNT || !available() { return false; }
    unsafe { ata_transfer(block as u32, out, false) }
}

pub fn write(block: usize, input: &[u8; BLOCK_SIZE]) -> bool {
    if block >= BLOCK_COUNT || !available() { return false; }
    let mut buffer = *input;
    unsafe { ata_transfer(block as u32, &mut buffer, true) }
}

unsafe fn identify() -> bool {
    crate::arch::outb(ATA_DRIVE, ATA_MASTER);
    ata_delay();
    if wait_not_busy() == 0 { return false; }

    crate::arch::outb(ATA_SECTOR_COUNT, 0);
    crate::arch::outb(ATA_LBA0, 0);
    crate::arch::outb(ATA_LBA1, 0);
    crate::arch::outb(ATA_LBA2, 0);
    crate::arch::outb(ATA_COMMAND, ATA_CMD_IDENTIFY);

    let status = wait_not_busy();
    if status == 0 || status & ATA_ERR != 0 { return false; }
    if !wait_drq() { return false; }

    for _ in 0..256 {
        let _ = read_data_word();
    }
    ata_delay();
    true
}

unsafe fn ata_transfer(lba: u32, buffer: &mut [u8; BLOCK_SIZE], write: bool) -> bool {
    // Finish any previous PIO command before programming the next one.
    let before = wait_not_busy();
    if before == 0 || before & ATA_ERR != 0 {
        crate::console::write("ata: controller not ready
");
        return false;
    }

    crate::arch::outb(ATA_DRIVE, ATA_MASTER | ((lba >> 24) as u8 & 0x0F));
    ata_delay();

    // Re-check the selected device before issuing a new command.
    let selected = wait_not_busy();
    if selected == 0 || selected & ATA_ERR != 0 {
        crate::console::write("ata: select failed
");
        return false;
    }

    crate::arch::outb(ATA_SECTOR_COUNT, 1);
    crate::arch::outb(ATA_LBA0, lba as u8);
    crate::arch::outb(ATA_LBA1, (lba >> 8) as u8);
    crate::arch::outb(ATA_LBA2, (lba >> 16) as u8);
    crate::arch::outb(ATA_COMMAND, if write { ATA_CMD_WRITE } else { ATA_CMD_READ });

    if !wait_drq() {
        crate::console::write(if write { "ata: write DRQ timeout
" } else { "ata: read DRQ timeout
" });
        return false;
    }

    if write {
        for i in 0..256 {
            let lo = buffer[i * 2] as u16;
            let hi = (buffer[i * 2 + 1] as u16) << 8;
            write_data_word(lo | hi);
        }

        // Poll the regular status register after the PIO data phase.
        // This acknowledges the command completion and lets the next
        // command start from a clean ATA state.
        let status = wait_not_busy();
        ata_delay();

        if status == 0 || status & ATA_ERR != 0 {
            crate::console::write("ata: write completion error
");
            return false;
        }
        true
    } else {
        for i in 0..256 {
            let word = read_data_word();
            buffer[i * 2] = word as u8;
            buffer[i * 2 + 1] = (word >> 8) as u8;
        }

        let status = wait_not_busy();
        ata_delay();

        if status == 0 || status & ATA_ERR != 0 {
            crate::console::write("ata: read completion error
");
            return false;
        }
        true
    }
}

unsafe fn wait_not_busy() -> u8 {
    let mut status = 0u8;
    for _ in 0..100_000 {
        status = crate::arch::inb(ATA_STATUS);
        if status & ATA_BSY == 0 { break; }
    }
    status
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
    // ATA specifies roughly 400 ns after drive/head or command register
    // transitions. Four alternate-status reads are the traditional PIO
    // implementation and do not clear the regular status/interrupt state.
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
