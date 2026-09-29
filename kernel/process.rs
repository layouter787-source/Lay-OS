//! Process table. Scheduler slot N is pid N (pid 0 is the kernel itself).
//! The scheduler owns execution state; this table owns lifecycle metadata
//! (state, parent, exit status) and is updated through create/exit so both
//! stay consistent.

use crate::console;

pub const MAX_PROCESSES: usize = 4;

#[derive(Clone, Copy, PartialEq)]
pub enum State { Free, Running, Ready, Exited }

#[derive(Clone, Copy)]
struct Process { state: State, parent: u32, status: u64 }

static mut TABLE: [Process; MAX_PROCESSES] =
    [Process { state: State::Free, parent: 0, status: 0 }; MAX_PROCESSES];

pub fn init() {
    unsafe {
        let mut i = 0;
        while i < MAX_PROCESSES {
            TABLE[i] = Process { state: State::Free, parent: 0, status: 0 };
            i += 1;
        }
    }
    console::write("process: table ready\n");
}

fn valid(pid: u32) -> bool {
    pid >= 1 && (pid as usize) < MAX_PROCESSES
}

/// Registers a new process. Fails for an invalid pid or one that is still alive.
pub fn create(pid: u32, parent: u32) -> bool {
    if !valid(pid) || pid == parent {
        return false;
    }
    let idx = pid as usize;
    unsafe {
        if TABLE[idx].state != State::Free && TABLE[idx].state != State::Exited {
            return false;
        }
        TABLE[idx].state = State::Ready;
        TABLE[idx].parent = parent;
        TABLE[idx].status = 0;
    }
    console::write("process: pid ");
    console::write_dec(pid as usize);
    console::write(" created parent ");
    console::write_dec(parent as usize);
    console::write("\n");
    true
}

pub fn set_running(pid: u32) {
    if valid(pid) {
        unsafe {
            if TABLE[pid as usize].state != State::Exited {
                TABLE[pid as usize].state = State::Running;
            }
        }
    }
}

pub fn set_ready(pid: u32) {
    if valid(pid) {
        unsafe {
            if TABLE[pid as usize].state != State::Exited {
                TABLE[pid as usize].state = State::Ready;
            }
        }
    }
}

pub fn exit(pid: u32, status: u64) {
    if !valid(pid) {
        return;
    }
    unsafe {
        TABLE[pid as usize].state = State::Exited;
        TABLE[pid as usize].status = status;
    }
    console::write("process: pid ");
    console::write_dec(pid as usize);
    console::write(" exited status ");
    if status == 0 {
        console::write("0");
    } else {
        console::write("nonzero");
    }
    console::write("\n");
}

pub fn state(pid: u32) -> State {
    if !valid(pid) {
        return State::Free;
    }
    unsafe { TABLE[pid as usize].state }
}

pub fn parent(pid: u32) -> u32 {
    if !valid(pid) {
        return 0;
    }
    unsafe { TABLE[pid as usize].parent }
}

#[allow(dead_code)]
pub fn status(pid: u32) -> u64 {
    if !valid(pid) {
        return 0;
    }
    unsafe { TABLE[pid as usize].status }
}

pub fn state_name(state: State) -> &'static str {
    match state {
        State::Free => "free",
        State::Running => "running",
        State::Ready => "ready",
        State::Exited => "exited",
    }
}
