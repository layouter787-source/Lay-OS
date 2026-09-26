//! Minimal kernel process lifecycle for the first native user process.

#[derive(Clone, Copy, PartialEq)]
pub enum State { Free, Running, Ready, Exited }

static mut PID1_STATE: State = State::Free;
static mut PID1_STATUS: u64 = 0;

pub fn init() {
    crate::console::write("process: init begin\n");
    unsafe {
        PID1_STATE = State::Free;
        PID1_STATUS = 0;
    }
    crate::console::write("process: table ready\n");
}

pub fn create(pid: u32, parent: u32) -> bool {
    if pid != 1 || parent != 0 { return false; }
    unsafe { PID1_STATE = State::Ready; }
    true
}

pub fn set_running(pid: u32) {
    if pid == 1 {
        unsafe { PID1_STATE = State::Running; }
    }
}

pub fn set_ready(pid: u32) {
    if pid == 1 {
        unsafe {
            if PID1_STATE == State::Running { PID1_STATE = State::Ready; }
        }
    }
}

pub fn exit(pid: u32, status: u64) {
    if pid != 1 { return; }
    unsafe {
        PID1_STATE = State::Exited;
        PID1_STATUS = status;
    }
    crate::console::write("process: pid 1 exited status ");
    write_u64(status);
    crate::console::write("\n");
}

pub fn state(pid: u32) -> State {
    if pid == 1 {
        unsafe { PID1_STATE }
    } else {
        State::Free
    }
}

#[allow(dead_code)]
pub fn exit_status(pid: u32) -> u64 {
    if pid == 1 { unsafe { PID1_STATUS } } else { 0 }
}

fn write_u64(mut value: u64) {
    let mut buf = [0u8; 20];
    let mut n = 0;
    if value == 0 { crate::console::put_byte(b'0'); return; }
    while value > 0 {
        buf[n] = b'0' + (value % 10) as u8;
        n += 1;
        value /= 10;
    }
    while n > 0 {
        n -= 1;
        crate::console::put_byte(buf[n]);
    }
}
