//! Primitivas de baixo nível x86_64 usadas pelo kernel.

#[inline(always)]
pub unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}

#[inline(always)]
pub unsafe fn lidt(idt: &IdtPointer) {
    core::arch::asm!("lidt [{}]", in(reg) idt, options(readonly, nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn sti() {
    core::arch::asm!("sti", options(nomem, nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn cli() {
    core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn hlt() {
    core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
}

#[repr(C, packed)]
pub struct IdtPointer {
    pub limit: u16,
    pub base: u64,
}
