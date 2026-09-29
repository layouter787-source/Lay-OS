//! First protected user address space.
//! One executable page at USER_ENTRY (shared by every user process) and one
//! private stack page per process slot just below USER_STACK_TOP.
//! The third-level page table (PT) comes from the real page allocator, and
//! the fixed code/stack frames are reserved so the allocator never reuses them.

pub const USER_ENTRY: usize = 0x0040_0000;
pub const USER_STACK_TOP: usize = 0x0050_0000;

/// User process slots (pid 1..=MAX_USER_SLOTS); each gets its own stack page.
pub const MAX_USER_SLOTS: usize = 3;

const PML4: usize = 0x0010_0000;
const PDPT: usize = 0x0010_1000;
const PD: usize = 0x0010_2000;

const PTE_PRESENT: u64 = 1;
const PTE_RW: u64 = 2;
const PTE_USER: u64 = 4;

/// Top-of-stack address for user process `slot` (1-based). Slot 1 keeps the
/// original layout (stack page 0x4FF000, top 0x500000); each next slot sits
/// one page lower.
pub fn stack_top(slot: usize) -> usize {
    USER_STACK_TOP - slot.saturating_sub(1) * 0x1000
}

// User process: getpid, print U, then exit(0).
const USER_CODE_LEN: usize = 33;

fn user_byte(index: usize) -> u8 {
    match index {
        0 => 0xB9, 1 => 0x01, 2 => 0x00, 3 => 0x00, 4 => 0x00,
        5 => 0xB8, 6 => 0x01, 7 => 0x00, 8 => 0x00, 9 => 0x00,
        10 => 0xCD, 11 => 0x80,
        12 => 0xBF, 13 => 0x55, 14 => 0x00, 15 => 0x00, 16 => 0x00,
        17 => 0xB8, 18 => 0x05, 19 => 0x00, 20 => 0x00, 21 => 0x00,
        22 => 0xCD, 23 => 0x80,
        24 => 0x31, 25 => 0xFF, 26 => 0xB8, 27 => 0x06, 28 => 0x00, 29 => 0x00, 30 => 0x00,
        31 => 0xCD, 32 => 0x80,
        _ => 0,
    }
}

pub fn init() {
    crate::console::write("user: init begin\n");
    unsafe {
        // These frames live at fixed physical addresses inside normal RAM.
        // Claim them first so alloc_page can never hand them out later.
        crate::memory::reserve_page(USER_ENTRY);
        let mut slot = 1;
        while slot <= MAX_USER_SLOTS {
            crate::memory::reserve_page(stack_top(slot) - 0x1000);
            slot += 1;
        }

        let dst = USER_ENTRY as *mut u8;
        dst.write_volatile(0xCC);
        crate::console::write("user: destination write ok\n");

        for index in 0..USER_CODE_LEN {
            dst.add(index).write_volatile(user_byte(index));
        }
        crate::console::write("user: code copied\n");

        // The PT frame comes from the real frame allocator: memory::init()
        // has already run by the time user::init() executes (kernel/main.rs),
        // and the frame is identity-mapped by the boot loader's first-1-GiB
        // mapping.
        let pt = crate::memory::alloc_page()
            .expect("user: out of memory allocating the user page table");

        (PML4 as *mut u64).write((PDPT as u64) | PTE_PRESENT | PTE_RW | PTE_USER);
        (PDPT as *mut u64).write((PD as u64) | PTE_PRESENT | PTE_RW | PTE_USER);
        // 0x0040_0000 belongs to the third 2 MiB region: PD index 2.
        (PD as *mut u64).add(2).write((pt as u64) | PTE_PRESENT | PTE_RW | PTE_USER);

        let pt_ptr = pt as *mut u64;
        for index in 0..512 {
            pt_ptr.add(index).write(0);
        }

        // Shared code page (read/execute for user mode).
        pt_ptr.add(0).write((USER_ENTRY as u64) | PTE_PRESENT | PTE_USER);

        // One private, writable stack page per process slot.
        let mut slot = 1;
        while slot <= MAX_USER_SLOTS {
            let page = stack_top(slot) - 0x1000;
            pt_ptr
                .add((page - USER_ENTRY) / 0x1000)
                .write((page as u64) | PTE_PRESENT | PTE_RW | PTE_USER);
            slot += 1;
        }

        // Reload the active page-table root so the CPU drops the old huge-page TLB entry.
        core::arch::asm!("mov cr3, {}", in(reg) PML4, options(nostack, preserves_flags));
        crate::console::write("user: page tables ready\n");
    }
}
