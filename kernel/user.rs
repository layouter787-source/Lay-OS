//! First protected user address space.
//! One executable page at 0x0040_0000 and one stack page below 0x0050_0000.

pub const USER_ENTRY: usize = 0x0040_0000;
pub const USER_STACK_TOP: usize = 0x0050_0000;

const PML4: usize = 0x0010_0000;
const PDPT: usize = 0x0010_1000;
const PD: usize = 0x0010_2000;
const PT: usize = 0x0010_3000;

const PTE_PRESENT: u64 = 1;
const PTE_RW: u64 = 2;
const PTE_USER: u64 = 4;

// mov ax,0x2b; load DS/ES; getpid; write 'U'; yield; jump back.
pub static USER_CODE: [u8; 36] = [
    0x66, 0xB8, 0x2B, 0x00, 0x8E, 0xD8, 0x8E, 0xC0,
    0xB8, 0x01, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xBF, 0x55, 0x00, 0x00, 0x00,
    0xB8, 0x05, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0x00, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xEB, 0xDC,
];

pub fn init() {
    unsafe {
        // Copy while the boot huge-page identity mapping is still writable.
        let src = USER_CODE.as_ptr();
        let dst = USER_ENTRY as *mut u8;
        for index in 0..USER_CODE.len() {
            dst.add(index).write(src.add(index).read());
        }

        // User access requires U/S=1 through every paging level.
        (PML4 as *mut u64).write((PDPT as u64) | PTE_PRESENT | PTE_RW | PTE_USER);
        (PDPT as *mut u64).write((PD as u64) | PTE_PRESENT | PTE_RW | PTE_USER);

        // Replace the 4-6 MiB huge-page mapping with a 4 KiB page table.
        (PD as *mut u64)
            .add(2)
            .write((PT as u64) | PTE_PRESENT | PTE_RW | PTE_USER);

        let pt = PT as *mut u64;
        for index in 0..512 {
            pt.add(index).write(0);
        }

        // Executable user page is read/execute; no user write permission.
        pt.add(0).write((USER_ENTRY as u64) | PTE_PRESENT | PTE_USER);

        // User stack lives at [0x004FF000, 0x00500000).
        pt.add(0x100).write(
            ((USER_STACK_TOP - 0x1000) as u64)
                | PTE_PRESENT
                | PTE_RW
                | PTE_USER,
        );
    }
}
