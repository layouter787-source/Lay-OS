//! Kernel GDT plus user-mode segments and a TSS for privilege transitions.

use core::mem::size_of;

#[repr(C, packed)]
struct GdtPointer {
    limit: u16,
    base: u64,
}

const TSS_SIZE: usize = 104;
const USER_KERNEL_STACK_SIZE: usize = 16 * 1024;

#[repr(align(16))]
struct TssStorage([u8; TSS_SIZE]);

#[repr(align(16))]
struct UserKernelStack([u8; USER_KERNEL_STACK_SIZE]);

pub const KERNEL_CODE: u16 = 0x18;
pub const KERNEL_DATA: u16 = 0x20;
pub const USER_DATA: u16 = 0x2B;
pub const USER_CODE: u16 = 0x33;
pub const TSS_SELECTOR: u16 = 0x38;

#[repr(align(16))]
struct Gdt([u64; 9]);

static mut GDT: Gdt = Gdt([
    0x0000000000000000,
    0x00CF9A000000FFFF,
    0x00CF92000000FFFF,
    0x00AF9A000000FFFF,
    0x00AF92000000FFFF,
    0x00CFF2000000FFFF,
    0x00AFFA000000FFFF,
    0,
    0,
]);

static mut TSS: TssStorage = TssStorage([0; TSS_SIZE]);
static mut USER_KERNEL_STACK: UserKernelStack =
    UserKernelStack([0; USER_KERNEL_STACK_SIZE]);

unsafe fn write_u16(ptr: *mut u8, offset: usize, value: u16) {
    ptr.add(offset).cast::<u16>().write_unaligned(value);
}

unsafe fn write_u64(ptr: *mut u8, offset: usize, value: u64) {
    ptr.add(offset).cast::<u64>().write_unaligned(value);
}

pub fn init() {
    unsafe {
        let tss = TSS.0.as_mut_ptr();

        // Long-mode TSS layout:
        // RSP0 is at byte offset 4 and I/O map base at byte offset 102.
        let user_stack_top =
            core::ptr::addr_of!(USER_KERNEL_STACK) as usize + USER_KERNEL_STACK_SIZE;
        write_u64(tss, 4, user_stack_top as u64);
        write_u16(tss, 102, TSS_SIZE as u16);

        let base = core::ptr::addr_of!(TSS) as u64;
        let limit = (TSS_SIZE - 1) as u64;

        // 64-bit available TSS descriptor, type 0x9, present.
        GDT.0[7] =
            (limit & 0xFFFF)
            | ((base & 0xFFFF) << 16)
            | (((base >> 16) & 0xFF) << 32)
            | (0x89 << 40)
            | (((limit >> 16) & 0x0F) << 48)
            | (((base >> 24) & 0xFF) << 56);
        GDT.0[8] = (base >> 32) & 0xFFFF_FFFF;

        let pointer = GdtPointer {
            limit: (size_of::<Gdt>() - 1) as u16,
            base: core::ptr::addr_of!(GDT) as u64,
        };

        core::arch::asm!(
            "lgdt [{}]",
            in(reg) &pointer,
            options(readonly, nostack, preserves_flags)
        );

        // Reload CS with the kernel 64-bit descriptor.
        core::arch::asm!(
            "push 0x18",
            "lea rax, [rip + 1f]",
            "push rax",
            "retfq",
            "1:",
            out("rax") _,
            options(preserves_flags)
        );

        core::arch::asm!(
            "mov ax, 0x20",
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            "mov ax, 0x38",
            "ltr ax",
            out("rax") _,
            options(preserves_flags)
        );
    }
}
