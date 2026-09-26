//! First protected user address space.
//! One executable page at 0x0100_0000 and one stack page below 0x0110_0000.

pub const USER_ENTRY: usize = 0x0040_0000;
pub const USER_STACK_TOP: usize = 0x0050_0000;

const PML4: usize = 0x0010_0000;
const PDPT: usize = 0x0010_1000;
const PD: usize = 0x0010_2000;
const PT: usize = 0x0010_3000;

const PTE_PRESENT: u64 = 1;
const PTE_RW: u64 = 2;
const PTE_USER: u64 = 4;

// User process: getpid, print U/yield four times, then exit(0).
const USER_CODE_LEN: usize = 33;

fn user_byte(index: usize) -> u8 {
    match index {
        0 => 0xB9, 1 => 0x01, 2 => 0x00, 3 => 0x00, 4 => 0x00,
        5 => 0xB8, 6 => 0x01, 7 => 0x00, 8 => 0x00, 9 => 0x00,
        10 => 0xCD, 11 => 0x80,
        12 => 0xBF, 13 => 0x55, 14 => 0x00, 15 => 0x00, 16 => 0x00,
        17 => 0xB8, 18 => 0x05, 19 => 0x00, 20 => 0x00, 21 => 0x00,
        22 => 0xCD, 23 => 0x80,
        24 => 0x31, 25 => 0xFF, 26 => 0xB8, 27 => 0x06, 26 => 0x00, 27 => 0x00, 28 => 0x00,
        29 => 0xCD, 30 => 0x80,
        _ => 0,
    }
}

pub fn init() {
    crate::console::write("user: init begin\n");
    unsafe {
        let dst = USER_ENTRY as *mut u8;
        dst.write_volatile(0xCC);
        crate::console::write("user: destination write ok\n");

        for index in 0..USER_CODE_LEN {
            dst.add(index).write_volatile(user_byte(index));
        }
        crate::console::write("user: code copied\n");

        (PML4 as *mut u64).write((PDPT as u64) | PTE_PRESENT | PTE_RW | PTE_USER);
        (PDPT as *mut u64).write((PD as u64) | PTE_PRESENT | PTE_RW | PTE_USER);
        // 0x0040_0000 belongs to the third 2 MiB region: PD index 2.
        (PD as *mut u64).add(2).write((PT as u64) | PTE_PRESENT | PTE_RW | PTE_USER);

        let pt = PT as *mut u64;
        for index in 0..512 {
            pt.add(index).write(0);
        }

        pt.add(0).write((USER_ENTRY as u64) | PTE_PRESENT | PTE_USER);
        pt.add(0x0FF).write(
            ((USER_STACK_TOP - 0x1000) as u64) | PTE_PRESENT | PTE_RW | PTE_USER,
        );

        // Reload the active page-table root so the CPU drops the old huge-page TLB entry.
        core::arch::asm!("mov cr3, {}", in(reg) PML4, options(nostack, preserves_flags));
        crate::console::write("user: page tables ready\n");
    }
}
