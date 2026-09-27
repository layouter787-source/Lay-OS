#![no_std]
#![no_main]

mod arch;
mod block;
mod console;
mod gdt;
mod interrupts;
mod ipc;
mod memory;
mod process;
mod runtime;
mod scheduler;
mod shell;
mod selftest;
mod vfs;
mod syscalls;
mod user;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn lay_kernel_main() -> ! {
    console::write("LAY OS 0.1\n");
    console::write("kernel: starting core services...\n");

    gdt::init();
    console::write("gdt: kernel + user + tss ok\n");

    memory::init();
    console::write("memory: page allocator ready\n");

    block::init();
    vfs::init();
    console::write("storage: block layer + LayFS ready\n");
    selftest::run();

    process::init();

    user::init();
    console::write("memory: user address space ready\n");

    scheduler::init();
    scheduler::describe();

    ipc::init();
    console::write("ipc: mailbox ready\n");

    interrupts::init();
    console::write("interrupts: idt/pic/pit ready\n");

    unsafe { arch::sti() };
    console::write("LAY OS KERNEL ONLINE\n");

    loop { unsafe { arch::hlt() } }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    console::write("LAY KERNEL: PANIC\n");
    loop { unsafe { arch::hlt() } }
}
