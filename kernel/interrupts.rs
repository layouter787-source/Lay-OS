//! IDT, PIC, PIT and syscall entry.
//! 70% lifecycle build: timer and syscall paths share the scheduler context.

use crate::arch::{inb, lidt, outb, IdtPointer};

const IDT_ENTRIES: usize = 256;
const PIC1: u16 = 0x20;
const PIC2: u16 = 0xA0;
const PIC1_DATA: u16 = 0x21;
const PIC2_DATA: u16 = 0xA1;
const PIT_COMMAND: u16 = 0x43;
const PIT_CHANNEL0: u16 = 0x40;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16, selector: u16, options: u16, offset_mid: u16,
    offset_high: u32, reserved: u32,
}

impl IdtEntry {
    const EMPTY: Self = Self {
        offset_low: 0, selector: 0, options: 0, offset_mid: 0,
        offset_high: 0, reserved: 0,
    };
    fn new(handler: unsafe extern "C" fn()) -> Self { Self::with_options(handler, 0x8E00) }
    fn with_options(handler: unsafe extern "C" fn(), options: u16) -> Self {
        let address = handler as usize as u64;
        Self {
            offset_low: address as u16, selector: 0x18, options,
            offset_mid: (address >> 16) as u16, offset_high: (address >> 32) as u32,
            reserved: 0,
        }
    }
}

#[repr(align(16))]
struct Idt([IdtEntry; IDT_ENTRIES]);
static mut IDT: Idt = Idt([IdtEntry::EMPTY; IDT_ENTRIES]);
static mut TICKS: u64 = 0;

unsafe extern "C" {
    fn irq0_stub();
    fn irq1_stub();
    fn exception_stub();
    fn syscall_stub();
    fn general_protection_stub();
    fn page_fault_stub();
}

#[no_mangle]
pub extern "C" fn general_protection_handler() -> ! {
    crate::console::write("LAY KERNEL: general protection fault\n");
    loop { unsafe { crate::arch::hlt() } }
}

#[no_mangle]
pub extern "C" fn page_fault_handler() -> ! {
    crate::console::write("LAY KERNEL: page fault\n");
    loop { unsafe { crate::arch::hlt() } }
}

#[no_mangle]
pub extern "C" fn exception_handler() -> ! {
    crate::console::write("LAY KERNEL: CPU exception\
");
    loop { unsafe { crate::arch::hlt() } }
}

#[no_mangle]
pub extern "C" fn timer_handler(saved_context: usize) -> usize {
    unsafe { TICKS = TICKS.wrapping_add(1); }
    let ticks = unsafe { TICKS };
    crate::scheduler::tick(ticks);
    let next_context = unsafe { crate::scheduler::schedule_from_interrupt(saved_context) };
    unsafe { outb(PIC1, 0x20); }
    next_context
}

#[no_mangle]
pub extern "C" fn keyboard_handler() {
    unsafe {
        let scancode = inb(0x60);
        crate::shell::input_scancode(scancode);
        outb(PIC1, 0x20);
    }
}

#[no_mangle]
pub unsafe extern "C" fn syscall_handler(saved_context: usize) -> usize {
    let regs = saved_context as *mut u64;
    let number = *regs.add(10);
    let arg1 = *regs.add(7);
    let arg2 = *regs.add(4);

    *regs.add(10) = crate::syscalls::dispatch(number, arg1, arg2);

    if number == crate::syscalls::SYS_YIELD {
        crate::scheduler::schedule_from_interrupt(saved_context)
    } else if number == crate::syscalls::SYS_EXIT {
        crate::scheduler::exit_current(arg2)
    } else {
        saved_context
    }
}

pub fn init() {
    unsafe {
        IDT.0[0] = IdtEntry::new(exception_stub);
        IDT.0[6] = IdtEntry::new(exception_stub);
        IDT.0[8] = IdtEntry::new(exception_stub);
        IDT.0[10] = IdtEntry::new(exception_stub);
        IDT.0[11] = IdtEntry::new(exception_stub);
        IDT.0[12] = IdtEntry::new(exception_stub);
        IDT.0[13] = IdtEntry::new(general_protection_stub);
        IDT.0[14] = IdtEntry::new(page_fault_stub);
        IDT.0[17] = IdtEntry::new(exception_stub);
        IDT.0[32] = IdtEntry::new(irq0_stub);
        IDT.0[33] = IdtEntry::new(irq1_stub);
        IDT.0[0x80] = IdtEntry::with_options(syscall_stub, 0xEE00);
        let pointer = IdtPointer {
            limit: (core::mem::size_of::<Idt>() - 1) as u16,
            base: core::ptr::addr_of!(IDT) as u64,
        };
        lidt(&pointer);
        remap_pic();
        init_pit(100);
    }
}

unsafe fn remap_pic() {
    let master_mask = inb(PIC1_DATA);
    let slave_mask = inb(PIC2_DATA);
    outb(PIC1, 0x11); io_wait();
    outb(PIC2, 0x11); io_wait();
    outb(PIC1_DATA, 0x20); io_wait();
    outb(PIC2_DATA, 0x28); io_wait();
    outb(PIC1_DATA, 0x04); io_wait();
    outb(PIC2_DATA, 0x02); io_wait();
    outb(PIC1_DATA, 0x01); io_wait();
    outb(PIC2_DATA, 0x01); io_wait();
    outb(PIC1_DATA, master_mask & !0x03);
    outb(PIC2_DATA, slave_mask | 0xFF);
}

unsafe fn init_pit(hz: u32) {
    let divisor = (1_193_182u32 / hz.max(1)).min(65_535);
    outb(PIT_COMMAND, 0x36);
    outb(PIT_CHANNEL0, divisor as u8);
    outb(PIT_CHANNEL0, (divisor >> 8) as u8);
}

unsafe fn io_wait() { outb(0x80, 0); }
