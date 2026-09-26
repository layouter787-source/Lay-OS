//! Generic block-device interface backed by a bounded RAM disk.

pub const BLOCK_SIZE: usize = 512;
pub const BLOCK_COUNT: usize = 128;

static mut DISK: [[u8; BLOCK_SIZE]; BLOCK_COUNT] = [[0; BLOCK_SIZE]; BLOCK_COUNT];

pub fn init() {
    unsafe { DISK = [[0; BLOCK_SIZE]; BLOCK_COUNT]; }
}

pub fn read(block: usize, out: &mut [u8; BLOCK_SIZE]) -> bool {
    if block >= BLOCK_COUNT { return false; }
    unsafe { *out = DISK[block]; }
    true
}

pub fn write(block: usize, input: &[u8; BLOCK_SIZE]) -> bool {
    if block >= BLOCK_COUNT { return false; }
    unsafe { DISK[block] = *input; }
    true
}
