//! First protected user address space.
//! One executable page at 0x0020_0000 and one stack page below 0x0030_0000.

pub const USER_ENTRY: usize = 0x0020_0000;
pub const USER_STACK_TOP: usize = 0x0030_0000;

const PML4: usize = 0x0010_0000;
const PDPT: usize = 0x0010_1000;
const PD: usize = 0x0010_2000;
const PT: usize = 0x0010_3000;

const PTE_PRESENT: u64 = 1;
const PTE_RW: u64 = 2;
const PTE_USER: u64 = 4;

// User process: getpid, print U/yield four times, then exit(0).
const USER_CODE_LEN: usize = 54;

fn user_byte(index: usize) -> u8 {
    match index {
        0 => 0x66, 1 => 0xB8, 2 => 0x2B, 3 => 0x00, 4 => 0x8E, 5 => 0xD8, 6 => 0x8E, 7 => 0xC0,
        8 => 0xB9, 9 => 0x04, 10 => 0x00, 11 => 0x00, 12 => 0x00,
        13 => 0xB8, 14 => 0x01, 15 => 0x00, 16 => 0x00, 17 => 0xCD, 18 => 0x80,
        19 => 0xBF, 20 => 0x55, 21 => 0x00, 22 => 0x00, 23 => 0x00,
        24 => 0xB8, 25 => 0x05, 26 => 0x00, 27 => 0x00, 28 => 0x00, 29 => 0xCD, 30 => 0x80,
        31 => 0xB8, 32 => 0x00, 33 => 0x00, 34 => 0x00, 35 => 0x00, 36 => 0xCD, 37 => 0x80,
        38 => 0xFF, 39 => 0xC9, 40 => 0x75, 41 => 0xE9,
        42 => 0x31, 43 => 0xDB,
        44 => 0xB8, 45 => 0x06, 46 => 0x00, 47 => 0x00, 48 => 0x00, 49 => 0xCD, 50 => 0x80,
        51 => 0xEB, 52 => 0xFE, 53 => 0x00,
        _ => 0,
    }
}

pub fn init() {
    crate::console::write("user: init begin\n");
    unsafe {
        if crate::memory::alloc_page() != Some(USER_ENTRY) {
            crate::console::write("user: page allocation failed\n");
            return;
        }
        let dst = USER_ENTRY as *mut u8;
        dst.write_volatile(0xCC);
        crate::console::write("user: destination write ok\n");
        let src = USER_CODE.as_ptr();
        for index in 0..USER_CODE.len() {
            dst.add(index).write(src.add(index).read());
        }

        crate::console::write("user: code copied\n");

        (PML4 as *mut u64).write((PDPT as u64) | PTE_PRESENT | PTE_RW | PTE_USER);
        (PDPT as *mut u64).write((PD as u64) | PTE_PRESENT | PTE_RW | PTE_USER);
        (PD as *mut u64).add(1).write((PT as u64) | PTE_PRESENT | PTE_RW | PTE_USER);

        let pt = PT as *mut u64;
        for index in 0..512 { pt.add(index).write(0); }

        pt.add(0).write((USER_ENTRY as u64) | PTE_PRESENT | PTE_USER);
        pt.add(0x0FF).write(
            ((USER_STACK_TOP - 0x1000) as u64) | PTE_PRESENT | PTE_RW | PTE_USER,
        );
        crate::console::write("user: page tables ready\n");
    }
}
