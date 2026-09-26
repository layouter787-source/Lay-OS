//! Lay OS syscall ABI.
//! RAX = syscall number, RBX = argument 1, RDI = argument 2.

pub const SYS_YIELD: u64 = 0;
pub const SYS_GETPID: u64 = 1;
pub const SYS_GETTICKS: u64 = 2;
pub const SYS_IPC_SEND: u64 = 3;
pub const SYS_IPC_RECV: u64 = 4;
pub const SYS_WRITE_CHAR: u64 = 5;
pub const SYS_EXIT: u64 = 6;

pub fn dispatch(number: u64, _arg1: u64, arg2: u64) -> u64 {
    match number {
        SYS_YIELD => 0,
        SYS_GETPID => crate::scheduler::current_id() as u64,
        SYS_GETTICKS => crate::scheduler::ticks(),
        SYS_IPC_SEND => {
            if crate::ipc::send(crate::scheduler::current_id(), arg2) { 0 } else { u64::MAX }
        }
        SYS_IPC_RECV => match crate::ipc::receive() {
            Some(message) => ((message.sender as u64) << 32) | (message.value & 0xFFFF_FFFF),
            None => u64::MAX,
        },
        SYS_WRITE_CHAR => {
            crate::console::put_byte(arg2 as u8);
            0
        }
        SYS_EXIT => 0,
        _ => u64::MAX,
    }
}
