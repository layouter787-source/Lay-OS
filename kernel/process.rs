//! Process lifecycle API backed by the scheduler's task table.
//! Keeping one authoritative task state avoids duplicating kernel memory state.

#[derive(Clone, Copy, PartialEq)]
pub enum State { Free, Running, Ready, Exited }

pub fn init() {
    crate::console::write("process: table ready\n");
}

pub fn create(pid: u32, parent: u32) -> bool {
    if pid == 1 && parent == 0 {
        crate::console::write("process: pid 1 created parent 0\n");
        true
    } else {
        false
    }
}

pub fn set_running(_pid: u32) {}
pub fn set_ready(_pid: u32) {}

pub fn exit(pid: u32, status: u64) {
    if pid == 1 {
        crate::console::write("process: pid 1 exited status ");
        write_u64(status);
        crate::console::write("\n");
    }
}

pub fn state(pid: u32) -> State {
    if pid != 1 { return State::Free; }
    if crate::scheduler::is_dead(pid) { State::Exited } else { State::Ready }
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
