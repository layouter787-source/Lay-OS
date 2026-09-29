//! Boot-time deterministic subsystem checks.

use crate::{block, console, memory, vfs};

fn memory_check() {
    let free_before = memory::free_bytes();
    assert!(memory::total_bytes() > 0);
    assert!(free_before > 2 * 4096);

    let a = memory::alloc_page();
    let b = memory::alloc_page();
    assert!(a.is_some() && b.is_some());
    let (a, b) = (a.unwrap(), b.unwrap());

    // Distinct, page-aligned, and never inside the boot/kernel reservation.
    assert!(a != b);
    assert!(a % 4096 == 0 && b % 4096 == 0);
    assert!(a >= 0x0020_0000 && b >= 0x0020_0000);
    assert!(memory::free_bytes() == free_before - 2 * 4096);

    // Frames must be writable RAM.
    unsafe {
        (a as *mut u32).write_volatile(0xA5A5_5A5A);
        assert!((a as *const u32).read_volatile() == 0xA5A5_5A5A);
    }

    // Invalid frees (reserved low memory, unaligned) must be ignored.
    memory::free_page(0x1000);
    memory::free_page(a + 1);
    assert!(memory::free_bytes() == free_before - 2 * 4096);

    memory::free_page(a);
    memory::free_page(b);
    // Double free must be ignored too.
    memory::free_page(a);
    assert!(memory::free_bytes() == free_before);

    // Freed low frames are reused first.
    let again = memory::alloc_page();
    assert!(again == Some(a));
    memory::free_page(a);
    assert!(memory::free_bytes() == free_before);

    // Fixed-address reservation: low memory and already-used frames are refused.
    assert!(!memory::reserve_page(0x1000));
    assert!(memory::reserve_page(a));
    assert!(!memory::reserve_page(a));
    assert!(memory::free_bytes() == free_before - 4096);
    memory::free_page(a);
    assert!(memory::free_bytes() == free_before);
}

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

    memory_check();
    console::write("selftest: page allocator ok\n");
}
