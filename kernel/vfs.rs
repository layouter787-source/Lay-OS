//! Persistent LayFS foundation backed by the ATA block device.

use crate::block;

const MAX_FILES: usize = 32;
const NAME_LEN: usize = 31;
const FILE_SIZE: usize = 4096;
const META_BLOCKS: usize = 4;
const DATA_START: usize = 1 + META_BLOCKS;
const BLOCKS_PER_FILE: usize = FILE_SIZE / block::BLOCK_SIZE;
const MAGIC: &[u8; 8] = b"LAYFS01\0";

#[derive(Clone, Copy)]
struct Entry {
    used: bool,
    name: [u8; NAME_LEN],
    len: usize,
}

const EMPTY: Entry = Entry { used: false, name: [0; NAME_LEN], len: 0 };
static mut ENTRIES: [Entry; MAX_FILES] = [EMPTY; MAX_FILES];
static mut READY: bool = false;

pub fn init() {
    unsafe {
        READY = false;
        ENTRIES = [EMPTY; MAX_FILES];
        crate::console::write("filesystem: reading superblock...\n");
        if load() {
            crate::console::write("filesystem: superblock valid\n");
            READY = true;
            return;
        }
        crate::console::write("filesystem: formatting...\n");
        format();
        crate::console::write("filesystem: format complete\n");
        READY = true;
        crate::console::write("filesystem: creating welcome...\n");
        let _ = create("/welcome.txt", b"LAY OS filesystem online\n");
        crate::console::write("filesystem: welcome complete\n");
    }
}

pub fn persistent() -> bool {
    unsafe { READY }
}

pub fn create(path: &str, data: &[u8]) -> bool {
    if data.len() > FILE_SIZE || !valid_path(path) { return false; }
    unsafe {
        if find(path).is_some() { return false; }
        let slot = match free_slot() { Some(x) => x, None => return false };
        ENTRIES[slot].used = true;
        ENTRIES[slot].len = data.len();
        let bytes = path.as_bytes();
        let mut i = 0;
        while i < bytes.len() { ENTRIES[slot].name[i] = bytes[i]; i += 1; }
        if !write_file(slot, data) {
            ENTRIES[slot] = EMPTY;
            return false;
        }
        save_metadata();
        true
    }
}

pub fn read(path: &str, out: &mut [u8]) -> Option<usize> {
    unsafe {
        let slot = find(path)?;
        let count = core::cmp::min(ENTRIES[slot].len, out.len());
        if read_file(slot, &mut out[..count]) { Some(count) } else { None }
    }
}

pub fn write(path: &str, data: &[u8]) -> bool {
    if data.len() > FILE_SIZE || !valid_path(path) { return false; }
    unsafe {
        let slot = match find(path) { Some(x) => x, None => return create(path, data) };
        if !write_file(slot, data) { return false; }
        ENTRIES[slot].len = data.len();
        save_metadata();
        true
    }
}

pub fn list<F: FnMut(&str)>(mut visitor: F) {
    unsafe {
        for file in &ENTRIES {
            if !file.used { continue; }
            let mut end = 0;
            while end < NAME_LEN && file.name[end] != 0 { end += 1; }
            if let Ok(name) = core::str::from_utf8(&file.name[..end]) { visitor(name); }
        }
    }
}

unsafe fn format() {
    let mut block_data = [0u8; block::BLOCK_SIZE];
    block_data[..8].copy_from_slice(MAGIC);
    block_data[8..12].copy_from_slice(&(block::BLOCK_SIZE as u32).to_le_bytes());
    block_data[12..16].copy_from_slice(&(MAX_FILES as u32).to_le_bytes());
    let _ = block::write(0, &block_data);
    save_metadata();
}

unsafe fn load() -> bool {
    let mut header = [0u8; block::BLOCK_SIZE];
    if !block::read(0, &mut header) { return false; }
    if &header[..8] != MAGIC { return false; }

    let mut raw = [0u8; block::BLOCK_SIZE];
    for mb in 0..META_BLOCKS {
        if !block::read(1 + mb, &mut raw) { return false; }
        for n in 0..8 {
            let slot = mb * 8 + n;
            let off = n * 64;
            if slot >= MAX_FILES { break; }
            ENTRIES[slot].used = raw[off] != 0;
            ENTRIES[slot].len = u32::from_le_bytes([raw[off + 32], raw[off + 33], raw[off + 34], raw[off + 35]]) as usize;
            let mut i = 0;
            while i < NAME_LEN { ENTRIES[slot].name[i] = raw[off + 1 + i]; i += 1; }
            if ENTRIES[slot].len > FILE_SIZE { return false; }
        }
    }
    true
}

unsafe fn save_metadata() {
    let mut raw = [0u8; block::BLOCK_SIZE];
    for mb in 0..META_BLOCKS {
        raw = [0u8; block::BLOCK_SIZE];
        for n in 0..8 {
            let slot = mb * 8 + n;
            if slot >= MAX_FILES { break; }
            let off = n * 64;
            raw[off] = if ENTRIES[slot].used { 1 } else { 0 };
            let mut i = 0;
            while i < NAME_LEN { raw[off + 1 + i] = ENTRIES[slot].name[i]; i += 1; }
            raw[off + 32..off + 36].copy_from_slice(&(ENTRIES[slot].len as u32).to_le_bytes());
        }
        let _ = block::write(1 + mb, &raw);
    }
}

unsafe fn write_file(slot: usize, data: &[u8]) -> bool {
    let mut raw = [0u8; block::BLOCK_SIZE];
    let blocks_needed = core::cmp::max(1, (data.len() + block::BLOCK_SIZE - 1) / block::BLOCK_SIZE);
    for b in 0..blocks_needed {
        raw = [0u8; block::BLOCK_SIZE];
        let start = b * block::BLOCK_SIZE;
        let end = core::cmp::min(start + block::BLOCK_SIZE, data.len());
        if start < end { raw[..end - start].copy_from_slice(&data[start..end]); }
        if !block::write(DATA_START + slot * BLOCKS_PER_FILE + b, &raw) { return false; }
    }
    true
}

unsafe fn read_file(slot: usize, out: &mut [u8]) -> bool {
    let mut raw = [0u8; block::BLOCK_SIZE];
    for b in 0..BLOCKS_PER_FILE {
        let start = b * block::BLOCK_SIZE;
        if start >= out.len() { break; }
        if !block::read(DATA_START + slot * BLOCKS_PER_FILE + b, &mut raw) { return false; }
        let end = core::cmp::min(start + block::BLOCK_SIZE, out.len());
        out[start..end].copy_from_slice(&raw[..end - start]);
    }
    true
}

unsafe fn find(path: &str) -> Option<usize> {
    let bytes = path.as_bytes();
    for index in 0..MAX_FILES {
        if !ENTRIES[index].used { continue; }
        let mut end = 0;
        while end < NAME_LEN && ENTRIES[index].name[end] != 0 { end += 1; }
        if end == bytes.len() && ENTRIES[index].name[..end] == bytes[..] { return Some(index); }
    }
    None
}

unsafe fn free_slot() -> Option<usize> {
    for i in 0..MAX_FILES {
        if !ENTRIES[i].used { return Some(i); }
    }
    None
}

fn valid_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    !bytes.is_empty() && bytes.len() <= NAME_LEN && bytes[0] == b'/'
}
