//! Kernel GDT plus user-mode segments and a TSS for privilege transitions.

use core::mem::size_of;

#[repr(C, packed)]
struct GdtPointer {
    limit: u16,
    base: u64,
}

#[repr(C, packed)]
pub struct Tss64 {
    _reserved0: u32,
    pub rsp: [u64; 3],
    _reserved1: u64,
    pub ist: [u64; 7],
    _reserved2: u64,
    _reserved3: u16,
    pub iomap: u16,
}

pub const KERNEL_CODE: u16 = 0x18;
pub const KERNEL_DATA: u16 = 0x20;
pub const USER_DATA: u16 = 0x2B;
pub const USER_CODE: u16 = 0x33;
pub const TSS_SELECTOR: u16 = 0x38;

pub const TASK0_STACK_TOP: usize = 0x90000;
pub const USER_KERNEL_STACK_TOP: usize = 0x88000;

#[repr(align(16))]
struct Gdt([u64; 9]);

static mut GDT: Gdt = Gdt([
    0x0000000000000000,
    0x00CF9A000000FFFF,
    0x00CF92000000FFFF,
    0x00AF9A000000FFFF,
    0x00AF92000000FFFF,
    0x00CFF2000000FFFF, // user data, DPL3
    0x00AFFA000000FFFF, // user code, DPL3, 64-bit
    0,
    0,
]);

static mut TSS: Tss64 = Tss64 {
    _reserved0: 0,
    rsp: [0; 3],
    _reserved1: 0,
    ist: [0; 7],
    _reserved2: 0,
    _reserved3: 0,
    iomap: size_of::<Tss64>() as u16,
};

pub fn init() {
    unsafe {
        TSS.rsp[0] = USER_KERNEL_STACK_TOP as u64;

        let base = core::ptr::addr_of!(TSS) as u64;
        let limit = (size_of::<Tss64>() - 1) as u64;

        // 64-bit available TSS descriptor (type 0x9, P=1).
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
