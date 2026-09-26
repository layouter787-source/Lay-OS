//! Preemptive round-robin scheduler with kernel context frames.

use crate::console;

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
}

static mut TASKS: [Task; MAX_TASKS] = [Task {
    id: 0,
    state: TaskState::Empty,
    context: 0,
}; MAX_TASKS];

#[repr(align(16))]
struct TaskStacks([[u8; STACK_SIZE]; MAX_TASKS]);

static mut STACKS: TaskStacks = TaskStacks([[0; STACK_SIZE]; MAX_TASKS]);
static mut CURRENT: usize = 0;
static mut TICKS: u64 = 0;

pub fn init() {
    unsafe {
        TASKS[0] = Task {
            id: 0,
            state: TaskState::Running,
            context: 0,
        };
        CURRENT = 0;
        TICKS = 0;

        create_task(1);
    }
}

unsafe fn create_task(id: u32) {
    let slot = 1;
    let stack_top = core::ptr::addr_of_mut!(STACKS.0[slot]) as *mut u8;
    let stack_top = stack_top.add(STACK_SIZE) as usize;
    let context = build_initial_context(stack_top, task_trampoline);

    TASKS[slot] = Task {
        id,
        state: TaskState::Ready,
        context,
    };
}

unsafe fn build_initial_context(stack_top: usize, entry: unsafe extern "C" fn() -> !) -> usize {
    // This layout exactly matches the registers pushed by irq0_stub,
    // followed by the CPU interrupt-return frame consumed by iretq.
    let mut sp = stack_top & !0xF;

    sp -= 8; write(sp, 0); // ss
    sp -= 8; write(sp, 0x202); // rflags
    sp -= 8; write(sp, 0x18); // cs
    sp -= 8; write(sp, entry as usize); // rip

    for _ in 0..11 {
        sp -= 8;
        write(sp, 0);
    }

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

pub fn tick(ticks: u64) {
    unsafe {
        TICKS = ticks;
    }
}

pub fn yield_now() {
    // Voluntary yielding is provided by the timer-driven preemptive path.
}

pub fn current_id() -> u32 {
    unsafe { TASKS[CURRENT].id }
}

pub fn ticks() -> u64 {
    unsafe { TICKS }
}

unsafe extern "C" fn task_trampoline() -> ! {
    console::write("scheduler: task 1 started\n");

    loop {
        for _ in 0..1_000_000 {
            core::hint::spin_loop();
        }
    }
}
