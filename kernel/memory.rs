//! Very small physical-frame/bump allocator for the first kernel milestone.

const HEAP_START: usize = 0x0010_0000;
const HEAP_END: usize = 0x0100_0000;
const PAGE_SIZE: usize = 4096;

static mut NEXT: usize = HEAP_START;

pub fn init() {
    unsafe { NEXT = HEAP_START; }
}

pub fn alloc_page() -> Option<usize> {
    unsafe {
        let current = NEXT;
        let next = current.checked_add(PAGE_SIZE)?;
        if next > HEAP_END {
            return None;
        }
        NEXT = next;
        Some(current)
    }
}

pub fn used_bytes() -> usize {
    unsafe { NEXT - HEAP_START }
}
