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
pub static USER_CODE: [u8; 54] = [
    0x66, 0xB8, 0x2B, 0x00, 0x8E, 0xD8, 0x8E, 0xC0,
    0xB9, 0x04, 0x00, 0x00, 0x00,
    0xB8, 0x01, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xBF, 0x55, 0x00, 0x00, 0x00,
    0xB8, 0x05, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0x00, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xFF, 0xC9,
    0x75, 0xE9,
    0x31, 0xDB,
    0xB8, 0x06, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xEB, 0xFE,
];

pub fn init() {
    crate::console::write("user: init begin\n");
    unsafe {
        if crate::memory::alloc_page() != Some(USER_ENTRY) {
            crate::console::write("user: page allocation failed\n");
            return;
        }
        let src = USER_CODE.as_ptr();
        let dst = USER_ENTRY as *mut u8;
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
