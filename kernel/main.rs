#![no_std]
#![no_main]

mod arch;
mod console;
mod gdt;
mod interrupts;
mod ipc;
mod syscalls;
mod memory;
mod scheduler;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn lay_kernel_main() -> ! {
    console::write("LAY OS 0.1\n");
    console::write("kernel: starting core services...\n");

    gdt::init();
    console::write("gdt: ok\n");

    memory::init();
    console::write("memory: allocator ready\n");

    scheduler::init();
    ipc::init();
    console::write("scheduler: ready\n");
    console::write("ipc: mailbox ready\n");

    interrupts::init();
    console::write("interrupts: idt/pic/pit ready\n");

    unsafe { arch::sti(); }
    console::write("kernel: interrupts enabled\n");
    console::write("LAY OS KERNEL ONLINE\n");

    loop {
        unsafe { arch::hlt(); }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    console::write("LAY KERNEL: PANIC\n");
    loop {
        unsafe { arch::hlt(); }
    }
}
