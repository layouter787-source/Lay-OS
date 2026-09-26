//! Timer-driven kernel scheduler foundation.

const MAX_TASKS: usize = 32;

#[derive(Clone, Copy)]
struct Task {
    id: u32,
    active: bool,
}

static mut TASKS: [Task; MAX_TASKS] = [Task { id: 0, active: false }; MAX_TASKS];
static mut CURRENT: usize = 0;
static mut TICKS: u64 = 0;

pub fn init() {
    unsafe {
        TASKS[0] = Task { id: 0, active: true };
        CURRENT = 0;
        TICKS = 0;
    }
}

pub fn tick(ticks: u64) {
    unsafe {
        TICKS = ticks;
        if ticks % 10 == 0 {
            select_next();
        }
    }
}

pub fn yield_now() {
    unsafe { select_next(); }
}

pub fn current_id() -> u32 {
    unsafe { TASKS[CURRENT].id }
}

unsafe fn select_next() {
    for step in 1..=MAX_TASKS {
        let candidate = (CURRENT + step) % MAX_TASKS;
        if TASKS[candidate].active {
            CURRENT = candidate;
            return;
        }
    }
}
