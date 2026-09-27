//! Physical page-frame allocator driven by the firmware (E820) memory map.
//! Reserved boot structures (kernel image, initial page tables, real-mode
//! data) live below `RESERVED_BELOW` and are never handed out.
//! See docs/MEMORY_MAP.md for the boot-loader-to-kernel data contract.

const PAGE_SIZE: usize = 4096;
const RESERVED_BELOW: usize = 0x0020_0000; // 2 MiB safety margin for boot/kernel data.

/// Frame bitmap capacity: 32768 frames * 4 KiB = 128 MiB of trackable RAM.
/// Enough for the current development target (QEMU default RAM); memory
/// above this ceiling is simply not tracked yet (see docs/AI_HANDOFF.md).
const MAX_FRAMES: usize = 32 * 1024;
const BITMAP_WORDS: usize = MAX_FRAMES / 64;

// 1 = frame reserved/used, 0 = frame free.
static mut BITMAP: [u64; BITMAP_WORDS] = [u64::MAX; BITMAP_WORDS];
static mut TOTAL_FRAMES: usize = 0;
static mut FREE_FRAMES: usize = 0;
static mut NEXT_HINT: usize = 0;

const MEMMAP_COUNT_ADDR: usize = 0x8FF0;
const MEMMAP_ENTRIES_ADDR: usize = 0x9000;
const MEMMAP_MAX_ENTRIES: usize = 64;
const MEMMAP_ENTRY_SIZE: usize = 24;
const E820_TYPE_USABLE: u32 = 1;

#[inline]
fn set_used(frame: usize) -> bool {
    unsafe {
        let word = frame / 64;
        let bit = frame % 64;
        let mask = 1u64 << bit;
        let was_free = BITMAP[word] & mask == 0;
        BITMAP[word] |= mask;
        was_free
    }
}

#[inline]
fn set_free(frame: usize) -> bool {
    unsafe {
        let word = frame / 64;
        let bit = frame % 64;
        let mask = 1u64 << bit;
        let was_used = BITMAP[word] & mask != 0;
        BITMAP[word] &= !mask;
        was_used
    }
}

#[inline]
fn is_used(frame: usize) -> bool {
    unsafe {
        let word = frame / 64;
        let bit = frame % 64;
        BITMAP[word] & (1u64 << bit) != 0
    }
}

/// One raw E820-style entry as written by boot/boot.asm: base (u64),
/// length (u64), region type (u32), ACPI 3.0 attributes (u32, ignored).
struct MemRegion {
    base: u64,
    length: u64,
    region_type: u32,
}

unsafe fn read_region(index: usize) -> MemRegion {
    let ptr = (MEMMAP_ENTRIES_ADDR + index * MEMMAP_ENTRY_SIZE) as *const u8;
    let base = core::ptr::read_unaligned(ptr as *const u64);
    let length = core::ptr::read_unaligned(ptr.add(8) as *const u64);
    let region_type = core::ptr::read_unaligned(ptr.add(16) as *const u32);
    MemRegion { base, length, region_type }
}

fn free_range(start: usize, end: usize) {
    let start_frame = start / PAGE_SIZE;
    let end_frame = (end / PAGE_SIZE).min(MAX_FRAMES);
    let mut frame = start_frame;
    while frame < end_frame {
        if set_free(frame) {
            unsafe {
                TOTAL_FRAMES += 1;
                FREE_FRAMES += 1;
            }
        }
        frame += 1;
    }
}

pub fn init() {
    unsafe {
        for word in BITMAP.iter_mut() {
            *word = u64::MAX;
        }
        TOTAL_FRAMES = 0;
        FREE_FRAMES = 0;
        NEXT_HINT = RESERVED_BELOW / PAGE_SIZE;

        let count = core::ptr::read_volatile(MEMMAP_COUNT_ADDR as *const u16) as usize;
        let count = count.min(MEMMAP_MAX_ENTRIES);

        for i in 0..count {
            let region = read_region(i);
            if region.region_type != E820_TYPE_USABLE {
                continue;
            }
            let mut start = region.base as usize;
            let end = start.saturating_add(region.length as usize);
            if start < RESERVED_BELOW {
                start = RESERVED_BELOW;
            }
            if end <= start {
                continue;
            }
            free_range(start, end);
        }

        // No usable firmware map (older firmware, or a boot loader that
        // never ran detect_memory): fall back to the original fixed
        // 2..32 MiB window so the kernel still boots.
        if TOTAL_FRAMES == 0 {
            free_range(RESERVED_BELOW, 0x0200_0000);
        }
    }
}

pub fn alloc_page() -> Option<usize> {
    unsafe {
        if FREE_FRAMES == 0 {
            return None;
        }
        let mut frame = NEXT_HINT;
        for _ in 0..MAX_FRAMES {
            if frame >= MAX_FRAMES {
                frame = 0;
            }
            if !is_used(frame) {
                set_used(frame);
                FREE_FRAMES -= 1;
                NEXT_HINT = frame + 1;
                return Some(frame * PAGE_SIZE);
            }
            frame += 1;
        }
        None
    }
}

pub fn free_page(addr: usize) {
    unsafe {
        let frame = addr / PAGE_SIZE;
        if frame < MAX_FRAMES && set_free(frame) {
            FREE_FRAMES += 1;
        }
    }
}

pub fn used_bytes() -> usize {
    unsafe { (TOTAL_FRAMES - FREE_FRAMES) * PAGE_SIZE }
}

pub fn free_bytes() -> usize {
    unsafe { FREE_FRAMES * PAGE_SIZE }
}

pub fn total_bytes() -> usize {
    unsafe { TOTAL_FRAMES * PAGE_SIZE }
}
