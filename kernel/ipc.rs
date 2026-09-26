//! Minimal fixed-size kernel IPC mailbox layer.

const QUEUE_SIZE: usize = 16;

#[derive(Clone, Copy)]
pub struct Message {
    pub sender: u32,
    pub value: u64,
}

static mut QUEUE: [Message; QUEUE_SIZE] = [Message { sender: 0, value: 0 }; QUEUE_SIZE];
static mut HEAD: usize = 0;
static mut TAIL: usize = 0;
static mut COUNT: usize = 0;

pub fn init() {
    unsafe {
        HEAD = 0;
        TAIL = 0;
        COUNT = 0;
    }
}

pub fn send(sender: u32, value: u64) -> bool {
    unsafe {
        if COUNT == QUEUE_SIZE {
            return false;
        }
        QUEUE[TAIL] = Message { sender, value };
        TAIL = (TAIL + 1) % QUEUE_SIZE;
        COUNT += 1;
        true
    }
}

pub fn receive() -> Option<Message> {
    unsafe {
        if COUNT == 0 {
            return None;
        }
        let message = QUEUE[HEAD];
        HEAD = (HEAD + 1) % QUEUE_SIZE;
        COUNT -= 1;
        Some(message)
    }
}
