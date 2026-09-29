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
// 1 = frame is real, allocatable RAM (reported usable by firmware and above
// the boot reservation). Guards free_page against invalid/foreign frees.
static mut USABLE: [u64; BITMAP_WORDS] = [0; BITMAP_WORDS];
static mut TOTAL_FRAMES: usize = 0;
static mut FREE_FRAMES: usize = 0;
static mut NEXT_HINT: usize = 0;

const MEMMAP_COUNT_ADDR: usize = 0x8FF0;
const MEMMAP_ENTRIES_ADDR: usize = 0x9000;
const MEMMAP_MAX_ENTRIES: usize = 64;
const MEMMAP_ENTRY_SIZE: usize = 24;
const E820_TYPE_USABLE: u32 = 1;

#[inline]
fn set_used(frame: usize) {
    unsafe {
        BITMAP[frame / 64] |= 1u64 << (frame % 64);
    }
}

#[inline]
fn set_free(frame: usize) -> bool {
    unsafe {
        let word = frame / 64;
        let mask = 1u64 << (frame % 64);
        let was_used = BITMAP[word] & mask != 0;
        BITMAP[word] &= !mask;
        was_used
    }
}

#[inline]
fn is_used(frame: usize) -> bool {
    unsafe { BITMAP[frame / 64] & (1u64 << (frame % 64)) != 0 }
}

#[inline]
fn mark_usable(frame: usize) {
    unsafe {
        USABLE[frame / 64] |= 1u64 << (frame % 64);
    }
}

#[inline]
fn is_usable(frame: usize) -> bool {
    unsafe { USABLE[frame / 64] & (1u64 << (frame % 64)) != 0 }
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
    let end_frame = core::cmp::min(end / PAGE_SIZE, MAX_FRAMES);
    let mut frame = start_frame;
    while frame < end_frame {
        if set_free(frame) {
            mark_usable(frame);
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
        let mut i = 0;
        while i < BITMAP_WORDS {
            BITMAP[i] = u64::MAX;
            USABLE[i] = 0;
            i += 1;
        }
        TOTAL_FRAMES = 0;
        FREE_FRAMES = 0;
        NEXT_HINT = RESERVED_BELOW / PAGE_SIZE;

        let count = core::ptr::read_volatile(MEMMAP_COUNT_ADDR as *const u16) as usize;
        let count = core::cmp::min(count, MEMMAP_MAX_ENTRIES);

        let mut index = 0;
        while index < count {
            let region = read_region(index);
            index += 1;
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
        let mut tried = 0;
        while tried < MAX_FRAMES {
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
            tried += 1;
        }
        None
    }
}

/// Claims a specific frame that the kernel uses at a fixed address (for
/// example the user code/stack pages), so `alloc_page` can never hand it out
/// later. Returns false if the frame is not free, usable RAM.
pub fn reserve_page(addr: usize) -> bool {
    unsafe {
        let frame = addr / PAGE_SIZE;
        if addr % PAGE_SIZE != 0 || frame >= MAX_FRAMES || !is_usable(frame) || is_used(frame) {
            return false;
        }
        set_used(frame);
        FREE_FRAMES -= 1;
        true
    }
}

/// Returns a frame to the allocator. Ignores anything that is not a
/// page-aligned, currently-allocated frame of real usable RAM, so a bad or
/// double free can never corrupt the accounting or hand out reserved memory.
pub fn free_page(addr: usize) {
    unsafe {
        let frame = addr / PAGE_SIZE;
        if addr % PAGE_SIZE != 0 || frame >= MAX_FRAMES || !is_usable(frame) || !is_used(frame) {
            return;
        }
        set_free(frame);
        FREE_FRAMES += 1;
        if frame < NEXT_HINT {
            NEXT_HINT = frame;
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
