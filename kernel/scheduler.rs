//! Preemptive round-robin scheduler with process lifecycle support.

use crate::{console, gdt, process, user};

const MAX_TASKS: usize = 4;
const STACK_SIZE: usize = 16 * 1024;

#[derive(Clone, Copy, PartialEq)]
enum TaskState { Empty, Ready, Running, Dead }

#[derive(Clone, Copy)]
struct Task { id: u32, state: TaskState, context: usize, user: bool }

static mut TASKS: [Task; MAX_TASKS] = [Task {
    id: 0, state: TaskState::Empty, context: 0, user: false,
}; MAX_TASKS];

#[repr(align(16))]
struct TaskStacks([[u8; STACK_SIZE]; MAX_TASKS]);
static mut STACKS: TaskStacks = TaskStacks([[0; STACK_SIZE]; MAX_TASKS]);
static mut CURRENT: usize = 0;
static mut TICKS: u64 = 0;
static mut EXIT_REPORTED: bool = false;

#[repr(align(16))]
struct ExitStack([u8; STACK_SIZE]);
static mut EXIT_STACK: ExitStack = ExitStack([0; STACK_SIZE]);

pub fn init() {
    unsafe {
        let kernel_context = build_kernel_context();
        TASKS[0] = Task { id: 0, state: TaskState::Running, context: kernel_context, user: false };
        CURRENT = 0;
        TICKS = 0;
        EXIT_REPORTED = false;

        let context = build_user_context(user::USER_STACK_TOP, user::USER_ENTRY);
        TASKS[1] = Task { id: 1, state: TaskState::Ready, context, user: true };

        if process::create(1, 0) {
            console::write("scheduler: process 1 linked\n");
        }
    }
}

#[no_mangle]
pub extern "C" fn kernel_resume() -> ! {
    console::write("scheduler: kernel resumed after process exit\n");
    loop { unsafe { crate::arch::hlt() } }
}

unsafe fn build_kernel_context() -> usize {
    let stack_top_addr = core::ptr::addr_of_mut!(EXIT_STACK.0).cast::<u8>().add(STACK_SIZE) as usize;
    let mut sp = stack_top_addr & !0xF;
    sp -= 8; write(sp, gdt::KERNEL_DATA as usize);
    sp -= 8; write(sp, stack_top_addr);
    sp -= 8; write(sp, 0x202);
    sp -= 8; write(sp, gdt::KERNEL_CODE as usize);
    sp -= 8; write(sp, kernel_resume as usize);
    for _ in 0..11 { sp -= 8; write(sp, 0); }
    sp
}

unsafe fn build_user_context(stack_top: usize, entry: usize) -> usize {
    let stack_top_addr = core::ptr::addr_of_mut!(STACKS.0[1])
        .cast::<u8>().add(STACK_SIZE) as usize;
    let mut sp = stack_top_addr & !0xF;

    sp -= 8; write(sp, gdt::USER_DATA as usize);
    sp -= 8; write(sp, stack_top);
    sp -= 8; write(sp, 0x202);
    sp -= 8; write(sp, gdt::USER_CODE as usize);
    sp -= 8; write(sp, entry);

    for _ in 0..11 { sp -= 8; write(sp, 0); }
    sp
}

unsafe fn write(address: usize, value: usize) {
    (address as *mut usize).write_volatile(value);
}

#[no_mangle]
pub unsafe extern "C" fn schedule_from_interrupt(saved_context: usize) -> usize {
    TASKS[CURRENT].context = saved_context;
    let previous = CURRENT;

    if TASKS[previous].state == TaskState::Running {
        TASKS[previous].state = TaskState::Ready;
        process::set_ready(TASKS[previous].id);
    }

    for step in 1..=MAX_TASKS {
        let candidate = (previous + step) % MAX_TASKS;
        if TASKS[candidate].state == TaskState::Ready
            && (candidate == 0 || !is_dead(TASKS[candidate].id))
        {
            CURRENT = candidate;
            TASKS[candidate].state = TaskState::Running;
            process::set_running(TASKS[candidate].id);
            return TASKS[candidate].context;
        }
    }

    TASKS[previous].state = TaskState::Running;
    CURRENT = previous;
    saved_context
}

pub fn exit_current(status: u64) -> usize {
    unsafe {
        let current = CURRENT;
        process::exit(TASKS[current].id, status);
        TASKS[current].state = TaskState::Dead;

        for step in 1..=MAX_TASKS {
            let candidate = (current + step) % MAX_TASKS;
            if TASKS[candidate].state == TaskState::Ready
                && (candidate == 0 || !is_dead(TASKS[candidate].id))
            {
                CURRENT = candidate;
                TASKS[candidate].state = TaskState::Running;
                process::set_running(TASKS[candidate].id);
                return TASKS[candidate].context;
            }
        }

        if !EXIT_REPORTED {
            EXIT_REPORTED = true;
            console::write("scheduler: no runnable user process\n");
        }

        // Process exit always returns through a dedicated kernel context.
        TASKS[0].state = TaskState::Running;
        CURRENT = 0;
        build_kernel_context()
    }
}

pub fn is_dead(pid: u32) -> bool {
    unsafe {
        for index in 0..MAX_TASKS {
            if TASKS[index].id == pid { return TASKS[index].state == TaskState::Dead; }
        }
    }
    false
}

pub fn tick(value: u64) { unsafe { TICKS = value; } }
pub fn current_id() -> u32 { unsafe { TASKS[CURRENT].id } }
pub fn ticks() -> u64 { unsafe { TICKS } }

pub fn describe() {
    unsafe {
        if TASKS[1].user { console::write("scheduler: user process 1 ready\n"); }
    }
}
