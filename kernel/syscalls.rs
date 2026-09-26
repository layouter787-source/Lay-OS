//! Lay OS system-call ABI.
//! Entry point: software interrupt 0x80.
//! rax = syscall number, rbx/rdi/rsi = arguments.

pub const SYS_YIELD: u64 = 0;
pub const SYS_GETPID: u64 = 1;
pub const SYS_GETTICKS: u64 = 2;
pub const SYS_IPC_SEND: u64 = 3;
pub const SYS_IPC_RECV: u64 = 4;

pub fn dispatch(number: u64, arg1: u64, arg2: u64) -> u64 {
    match number {
        SYS_YIELD => {
            crate::scheduler::yield_now();
            0
        }
        SYS_GETPID => crate::scheduler::current_id() as u64,
        SYS_GETTICKS => crate::scheduler::ticks(),
        SYS_IPC_SEND => {
            if crate::ipc::send(arg1 as u32, arg2) { 0 } else { u64::MAX }
        }
        SYS_IPC_RECV => match crate::ipc::receive() {
            Some(message) => ((message.sender as u64) << 32) | (message.value & 0xFFFF_FFFF),
            None => u64::MAX,
        },
        _ => u64::MAX,
    }
}
