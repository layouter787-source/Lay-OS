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
