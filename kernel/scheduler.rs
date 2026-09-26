//! Preemptive round-robin scheduler with a first user process.

use crate::{console, gdt, user};

const MAX_TASKS: usize = 4;
const STACK_SIZE: usize = 16 * 1024;

#[derive(Clone, Copy, PartialEq)]
enum TaskState {
    Empty,
    Ready,
    Running,
    Dead,
}

#[derive(Clone, Copy)]
struct Task {
    id: u32,
    state: TaskState,
    context: usize,
    user: bool,
}

static mut TASKS: [Task; MAX_TASKS] = [Task {
    id: 0,
    state: TaskState::Empty,
    context: 0,
    user: false,
}; MAX_TASKS];

#[repr(align(16))]
struct TaskStacks([[u8; STACK_SIZE]; MAX_TASKS]);

static mut STACKS: TaskStacks = TaskStacks([[0; STACK_SIZE]; MAX_TASKS]);
static mut CURRENT: usize = 0;
static mut TICKS: u64 = 0;

pub fn init() {
    console::write("scheduler: init begin\n");

    unsafe {
        TASKS[0] = Task {
            id: 0,
            state: TaskState::Running,
            context: 0,
            user: false,
        };

        CURRENT = 0;
        TICKS = 0;
        console::write("scheduler: kernel task ready\n");

        let stack_start = core::ptr::addr_of_mut!(STACKS.0[1]).cast::<u8>() as usize;
        let stack_end = stack_start + STACK_SIZE;
        console::write("scheduler: stack start=");
        console::write_hex(stack_start);
        console::write(" end=");
        console::write_hex(stack_end);
        console::write("\n");

        let context = build_user_context(user::USER_STACK_TOP, user::USER_ENTRY);
        console::write("scheduler: user context built\n");

        TASKS[1] = Task {
            id: 1,
            state: TaskState::Ready,
            context,
            user: true,
        };
        console::write("scheduler: task table ready\n");
    }
}

unsafe fn build_user_context(stack_top: usize, entry: usize) -> usize {
    let stack_start = core::ptr::addr_of_mut!(STACKS.0[1]).cast::<u8>();
    let mut sp = stack_start.add(STACK_SIZE) as usize;
    sp &= !0xF;

    console::write("scheduler: writing context at ");
    console::write_hex(sp);
    console::write("\n");

    sp -= 8;
    write(sp, gdt::USER_DATA as usize);
    sp -= 8;
    write(sp, stack_top);
    sp -= 8;
    write(sp, 0x202);
    sp -= 8;
    write(sp, gdt::USER_CODE as usize);
    sp -= 8;
    write(sp, entry);

    for _ in 0..11 {
        sp -= 8;
        write(sp, 0);
    }

    console::write("scheduler: context writes complete at ");
    console::write_hex(sp);
    console::write("\n");
    sp
}

unsafe fn write(address: usize, value: usize) {
    (address as *mut usize).write(value);
}

#[no_mangle]
pub unsafe extern "C" fn schedule_from_interrupt(saved_context: usize) -> usize {
    TASKS[CURRENT].context = saved_context;

    let previous = CURRENT;
    if TASKS[previous].state == TaskState::Running {
        TASKS[previous].state = TaskState::Ready;
    }

    for step in 1..=MAX_TASKS {
        let candidate = (previous + step) % MAX_TASKS;
        if TASKS[candidate].state == TaskState::Ready {
            CURRENT = candidate;
            TASKS[candidate].state = TaskState::Running;
            return TASKS[candidate].context;
        }
    }

    TASKS[previous].state = TaskState::Running;
    CURRENT = previous;
    saved_context
}

pub fn tick(value: u64) {
    unsafe { TICKS = value; }
}

pub fn current_id() -> u32 {
    unsafe { TASKS[CURRENT].id }
}

pub fn ticks() -> u64 {
    unsafe { TICKS }
}

pub fn describe() {
    unsafe {
        if TASKS[1].user {
            console::write("scheduler: user process 1 ready\n");
        }
    }
}
