//! Kernel-owned Global Descriptor Table.

use crate::arch::IdtPointer;

#[repr(C, packed)]
struct GdtPointer {
    limit: u16,
    base: u64,
}

#[repr(align(8))]
struct Gdt([u64; 5]);

static GDT: Gdt = Gdt([
    0x0000000000000000,
    0x00CF9A000000FFFF,
    0x00CF92000000FFFF,
    0x00AF9A000000FFFF,
    0x00AF92000000FFFF,
]);

pub fn init() {
    let pointer = GdtPointer {
        limit: (core::mem::size_of::<Gdt>() - 1) as u16,
        base: core::ptr::addr_of!(GDT) as u64,
    };

    unsafe {
        core::arch::asm!("lgdt [{}]", in(reg) &pointer, options(readonly, nostack, preserves_flags));

        // Reload data segments and perform a far return to reload CS.
        core::arch::asm!(
            "push 0x18",
            "lea rax, [rip + 1f]",
            "push rax",
            "retfq",
            "1:",
            "mov ax, 0x20",
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            out("rax") _,
            options(preserves_flags)
        );
    }
}
