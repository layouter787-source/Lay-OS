//! VGA text console plus QEMU debug output.

const VGA: usize = 0xB8000;
const DEBUG_PORT: u16 = 0xE9;
const WIDTH: usize = 80;
const HEIGHT: usize = 25;

static mut ROW: usize = 0;
static mut COL: usize = 0;

pub fn write(text: &str) {
    for byte in text.bytes() {
        put_byte(byte);
    }
}

pub fn put_byte(byte: u8) {
    unsafe {
        crate::arch::outb(DEBUG_PORT, byte);

        if byte == b'\n' {
            COL = 0;
            ROW = (ROW + 1) % HEIGHT;
            return;
        }

        let offset = (ROW * WIDTH + COL) * 2;
        (VGA as *mut u8).add(offset).write_volatile(byte);
        (VGA as *mut u8).add(offset + 1).write_volatile(0x07);

        COL += 1;
        if COL >= WIDTH {
            COL = 0;
            ROW = (ROW + 1) % HEIGHT;
        }
    }
}


pub fn write_hex(mut value: usize) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    put_byte(b"0"[0]); put_byte(b"x"[0]);
    let mut shift = (core::mem::size_of::<usize>() * 8).saturating_sub(4);
    loop {
        put_byte(HEX[((value >> shift) & 0xF) as usize]);
        if shift == 0 { break; }
        shift -= 4;
    }
}

/// Prints an unsigned value in decimal, no leading zeros.
pub fn write_dec(mut value: usize) {
    if value == 0 {
        put_byte(b'0');
        return;
    }
    let mut buf = [0u8; 20];
    let mut n = 0;
    while value > 0 {
        buf[n] = b'0' + (value % 10) as u8;
        n += 1;
        value /= 10;
    }
    while n > 0 {
        n -= 1;
        put_byte(buf[n]);
    }
}
