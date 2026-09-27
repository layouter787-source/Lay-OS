//! Low-level x86_64 primitives used by the Lay OS kernel.

#[inline(always)]
pub unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    value
}

#[inline(always)]
pub unsafe fn lidt(idt: &IdtPointer) {
    core::arch::asm!(
        "lidt [{}]",
        in(reg) idt,
        options(readonly, nostack, preserves_flags)
    );
}

#[inline(always)]
pub unsafe fn sti() {
    core::arch::asm!("sti", options(nomem, nostack));
}

#[inline(always)]
pub unsafe fn cli() {
    core::arch::asm!("cli", options(nomem, nostack));
}

#[inline(always)]
pub unsafe fn hlt() {
    core::arch::asm!("hlt", options(nomem, nostack));
}

#[repr(C, packed)]
pub struct IdtPointer {
    pub limit: u16,
    pub base: u64,
}


pub unsafe fn reload_cr3() {
    core::arch::asm!("mov rax, cr3", "mov cr3, rax", out("rax") _, options(nostack, preserves_flags));
}


pub fn read_cr2() -> usize {
    let value: usize;
    unsafe { core::arch::asm!("mov {}, cr2", out(reg) value, options(nostack, preserves_flags)); }
    value
}
