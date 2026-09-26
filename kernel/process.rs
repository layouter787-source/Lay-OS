//! Kernel process table and lifecycle state.

const MAX_PROCESSES: usize = 8;

#[derive(Clone, Copy, PartialEq)]
pub enum State { Free, Running, Ready, Exited }

#[derive(Clone, Copy)]
struct Process {
    pid: u32,
    parent: u32,
    state: State,
    exit_status: u64,
}

static mut TABLE: [Process; MAX_PROCESSES] = [Process {
    pid: 0, parent: 0, state: State::Free, exit_status: 0,
}; MAX_PROCESSES];

pub fn init() {
    unsafe {
        TABLE[0] = Process { pid: 0, parent: 0, state: State::Running, exit_status: 0 };
    }
    crate::console::write("process: table ready\n");
}

pub fn create(pid: u32, parent: u32) -> bool {
    unsafe {
        for slot in TABLE.iter_mut() {
            if slot.state == State::Free {
                *slot = Process { pid, parent, state: State::Ready, exit_status: 0 };
                return true;
            }
        }
    }
    false
}

pub fn set_running(pid: u32) {
    unsafe {
        for slot in TABLE.iter_mut() {
            if slot.pid == pid && slot.state != State::Free { slot.state = State::Running; }
        }
    }
}

pub fn set_ready(pid: u32) {
    unsafe {
        for slot in TABLE.iter_mut() {
            if slot.pid == pid && slot.state == State::Running { slot.state = State::Ready; }
        }
    }
}

pub fn exit(pid: u32, status: u64) {
    unsafe {
        for slot in TABLE.iter_mut() {
            if slot.pid == pid && slot.state != State::Free {
                slot.state = State::Exited;
                slot.exit_status = status;
                crate::console::write("process: pid ");
                write_u32(pid);
                crate::console::write(" exited status ");
                write_u64(status);
                crate::console::write("\n");
                return;
            }
        }
    }
}

pub fn state(pid: u32) -> State {
    unsafe {
        for slot in TABLE.iter() {
            if slot.pid == pid { return slot.state; }
        }
    }
    State::Free
}

fn write_u32(mut value: u32) {
    let mut buf = [0u8; 10];
    let mut n = 0;
    if value == 0 { crate::console::put_byte(b'0'); return; }
    while value > 0 { buf[n] = b'0' + (value % 10) as u8; n += 1; value /= 10; }
    while n > 0 { n -= 1; crate::console::put_byte(buf[n]); }
}

fn write_u64(mut value: u64) {
    let mut buf = [0u8; 20];
    let mut n = 0;
    if value == 0 { crate::console::put_byte(b'0'); return; }
    while value > 0 { buf[n] = b'0' + (value % 10) as u8; n += 1; value /= 10; }
    while n > 0 { n -= 1; crate::console::put_byte(buf[n]); }
}
