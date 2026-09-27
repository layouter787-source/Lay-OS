//! Boot-time deterministic subsystem checks.

use crate::{block, console, vfs};

pub fn run() {
    assert!(block::available());

    let mut block_in = [0u8; block::BLOCK_SIZE];
    block_in[0] = 0x4C;
    block_in[1] = 0x41;
    block_in[2] = 0x59;
    assert!(block::write(400, &block_in));

    let mut block_out = [0u8; block::BLOCK_SIZE];
    assert!(block::read(400, &mut block_out));
    assert!(block_out[0] == 0x4C && block_out[1] == 0x41 && block_out[2] == 0x59);

    let mut data = [0u8; 2];
    assert!(vfs::read("/selftest.txt", &mut data) == Some(2));
    assert!(data == *b"ok");

    let mut welcome = [0u8; 25];
    assert!(vfs::read("/welcome.txt", &mut welcome) == Some(25));
    assert!(&welcome[..] == b"LAY OS filesystem online\n");

    if vfs::fresh_format() {
        console::write("selftest: persistent LayFS initialized\n");
    } else {
        console::write("selftest: persistent LayFS reload ok\n");
    }
}
