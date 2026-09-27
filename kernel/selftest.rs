//! Boot-time deterministic subsystem checks.

use crate::{block, console, vfs};

pub fn run() {
    let mut block_in = [0u8; block::BLOCK_SIZE];
    block_in[0] = 0x4C;
    block_in[1] = 0x41;
    block_in[2] = 0x59;
    assert!(block::write(7, &block_in));

    let mut block_out = [0u8; block::BLOCK_SIZE];
    assert!(block::read(7, &mut block_out));
    assert!(block_out[0] == 0x4C && block_out[1] == 0x41 && block_out[2] == 0x59);

    assert!(vfs::create("/selftest.txt", b"ok"));
    let mut data = [0u8; 2];
    assert!(vfs::read("/selftest.txt", &mut data) == Some(2));
    assert!(data == *b"ok");

    console::write("selftest: storage + filesystem ok\n");
}
