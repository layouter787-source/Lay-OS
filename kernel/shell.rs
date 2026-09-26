//! Minimal keyboard-driven Lay Shell.

use crate::{console, vfs};

const INPUT_SIZE: usize = 128;
static mut INPUT: [u8; INPUT_SIZE] = [0; INPUT_SIZE];
static mut LEN: usize = 0;

pub fn init() {
    console::write("Lay Shell 0.1\n");
    console::write("Type help for commands.\n");
    prompt();
}

pub fn input_scancode(scancode: u8) {
    if scancode & 0x80 != 0 { return; }
    let ch = match scancode_to_ascii(scancode) { Some(c) => c, None => return };

    unsafe {
        match ch {
            b'\n' => {
                console::write("\n");
                execute(&INPUT[..LEN]);
                LEN = 0;
                prompt();
            }
            8 => {
                if LEN != 0 {
                    LEN -= 1;
                    console::write("\x08 \x08");
                }
            }
            _ if LEN < INPUT_SIZE => {
                INPUT[LEN] = ch;
                LEN += 1;
                console::put_byte(ch);
            }
            _ => {}
        }
    }
}

fn prompt() { console::write("lay> "); }

fn execute(command: &[u8]) {
    if command == b"help" {
        console::write("help  ls  cat /welcome.txt  write /file text\n");
    } else if command == b"ls" {
        vfs::list(|name| { console::write(name); console::write("\n"); });
    } else if command.starts_with(b"cat ") {
        let mut buffer = [0u8; 4096];
        match core::str::from_utf8(&command[4..]) {
            Ok(path) => match vfs::read(path, &mut buffer) {
                Some(len) => { for byte in &buffer[..len] { console::put_byte(*byte); } }
                None => console::write("cat: file not found\n"),
            },
            Err(_) => console::write("cat: invalid path\n"),
        }
    } else if command.starts_with(b"write ") {
        let rest = &command[6..];
        let mut split = 0;
        while split < rest.len() && rest[split] != b' ' { split += 1; }
        if split == 0 || split == rest.len() {
            console::write("usage: write /path text\n");
        } else if let Ok(path) = core::str::from_utf8(&rest[..split]) {
            if vfs::write(path, &rest[split + 1..]) { console::write("ok\n"); }
            else { console::write("write: failed\n"); }
        } else {
            console::write("write: invalid path\n");
        }
    } else if !command.is_empty() {
        console::write("unknown command\n");
    }
}

fn scancode_to_ascii(code: u8) -> Option<u8> {
    match code {
        0x02 => Some(b'1'), 0x03 => Some(b'2'), 0x04 => Some(b'3'), 0x05 => Some(b'4'),
        0x06 => Some(b'5'), 0x07 => Some(b'6'), 0x08 => Some(b'7'), 0x09 => Some(b'8'),
        0x0A => Some(b'9'), 0x0B => Some(b'0'), 0x10 => Some(b'q'), 0x11 => Some(b'w'),
        0x12 => Some(b'e'), 0x13 => Some(b'r'), 0x14 => Some(b't'), 0x15 => Some(b'y'),
        0x16 => Some(b'u'), 0x17 => Some(b'i'), 0x18 => Some(b'o'), 0x19 => Some(b'p'),
        0x1E => Some(b'a'), 0x1F => Some(b's'), 0x20 => Some(b'd'), 0x21 => Some(b'f'),
        0x22 => Some(b'g'), 0x23 => Some(b'h'), 0x24 => Some(b'j'), 0x25 => Some(b'k'),
        0x26 => Some(b'l'), 0x2C => Some(b'z'), 0x2D => Some(b'x'), 0x2E => Some(b'c'),
        0x2F => Some(b'v'), 0x30 => Some(b'b'), 0x31 => Some(b'n'), 0x32 => Some(b'm'),
        0x39 => Some(b' '), 0x0E => Some(8), 0x1C => Some(b'\n'), _ => None,
    }
}
