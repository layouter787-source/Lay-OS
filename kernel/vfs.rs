//! Persistent LayFS foundation backed by the ATA block device.

use crate::block;

const MAX_FILES: usize = 32;
const NAME_LEN: usize = 31;
const FILE_SIZE: usize = 4096;
const FS_START: usize = 7;
const META_BLOCKS: usize = 4;
const META_START: usize = FS_START + 1;
const DATA_START: usize = META_START + META_BLOCKS;
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
static mut FRESH: bool = false;
const FORMAT_BLOCKS: usize = DATA_START + 2 * BLOCKS_PER_FILE;
static mut FORMAT_BUFFER: [u8; FORMAT_BLOCKS * block::BLOCK_SIZE] = [0; FORMAT_BLOCKS * block::BLOCK_SIZE];

pub fn init() {
    unsafe {
        READY = false;
        FRESH = false;
        ENTRIES = [EMPTY; MAX_FILES];
        crate::console::write("filesystem: reading superblock...\n");
        if load() {
            READY = true;
            return;
        }
        crate::console::write("filesystem: formatting...\n");
        ENTRIES[0].used = true;
        ENTRIES[0].len = 25;
        ENTRIES[0].name[..12].copy_from_slice(b"/welcome.txt");
        ENTRIES[1].used = true;
        ENTRIES[1].len = 2;
        ENTRIES[1].name[..13].copy_from_slice(b"/selftest.txt");
        format();
        FRESH = true;
        READY = true;
    }
}

pub fn persistent() -> bool {
    unsafe { READY }
}

pub fn fresh_format() -> bool {
    unsafe { FRESH }
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
    FORMAT_BUFFER = [0u8; FORMAT_BLOCKS * block::BLOCK_SIZE];

    let raw = &mut FORMAT_BUFFER[..];
    raw[..8].copy_from_slice(MAGIC);
    raw[8..12].copy_from_slice(&(block::BLOCK_SIZE as u32).to_le_bytes());
    raw[12..16].copy_from_slice(&(MAX_FILES as u32).to_le_bytes());

    for mb in 0..META_BLOCKS {
        let base = (1 + mb) * block::BLOCK_SIZE;
        for n in 0..8 {
            let slot = mb * 8 + n;
            if slot >= MAX_FILES { break; }
            let off = base + n * 64;
            raw[off] = if ENTRIES[slot].used { 1 } else { 0 };
            let mut i = 0;
            while i < NAME_LEN {
                raw[off + 1 + i] = ENTRIES[slot].name[i];
                i += 1;
            }
            raw[off + 32..off + 36].copy_from_slice(&(ENTRIES[slot].len as u32).to_le_bytes());
        }
    }

    let welcome = (DATA_START - FS_START) * block::BLOCK_SIZE;
    raw[welcome..welcome + 25].copy_from_slice(b"LAY OS filesystem online\\n");
    let selftest = (DATA_START + BLOCKS_PER_FILE - FS_START) * block::BLOCK_SIZE;
    raw[selftest..selftest + 2].copy_from_slice(b"ok");

    assert!(block::write_many(FS_START, &mut raw[..FORMAT_BLOCKS * block::BLOCK_SIZE]));
}

unsafe fn load() -> bool {
    let blocks = 1 + META_BLOCKS;
    FORMAT_BUFFER = [0u8; FORMAT_BLOCKS * block::BLOCK_SIZE];
    if !block::read_many(FS_START, &mut FORMAT_BUFFER[..blocks * block::BLOCK_SIZE]) { return false; }
    let raw = &FORMAT_BUFFER[..];

    if &raw[..8] != MAGIC { return false; }
    for mb in 0..META_BLOCKS {
        let base = (1 + mb) * block::BLOCK_SIZE;
        for n in 0..8 {
            let slot = mb * 8 + n;
            if slot >= MAX_FILES { break; }
            let off = base + n * 64;
            ENTRIES[slot].used = raw[off] != 0;
            ENTRIES[slot].len = u32::from_le_bytes([raw[off + 32], raw[off + 33], raw[off + 34], raw[off + 35]]) as usize;
            let mut i = 0;
            while i < NAME_LEN {
                ENTRIES[slot].name[i] = raw[off + 1 + i];
                i += 1;
            }
            if ENTRIES[slot].len > FILE_SIZE { return false; }
        }
    }
    true
}

unsafe fn save_metadata() {
    for mb in 0..META_BLOCKS {
        let base = mb * block::BLOCK_SIZE;
        let dst = &mut FORMAT_BUFFER[base..base + block::BLOCK_SIZE];
        dst.fill(0);
        for n in 0..8 {
            let slot = mb * 8 + n;
            if slot >= MAX_FILES { break; }
            let off = n * 64;
            dst[off] = if ENTRIES[slot].used { 1 } else { 0 };
            let mut i = 0;
            while i < NAME_LEN {
                dst[off + 1 + i] = ENTRIES[slot].name[i];
                i += 1;
            }
            dst[off + 32..off + 36].copy_from_slice(&(ENTRIES[slot].len as u32).to_le_bytes());
        }
    }
    let _ = block::write_many(META_START, &mut FORMAT_BUFFER[..META_BLOCKS * block::BLOCK_SIZE]);
}

unsafe fn write_file(slot: usize, data: &[u8]) -> bool {
    let blocks_needed = core::cmp::max(1, (data.len() + block::BLOCK_SIZE - 1) / block::BLOCK_SIZE);
    FORMAT_BUFFER[..blocks_needed * block::BLOCK_SIZE].fill(0);
    FORMAT_BUFFER[..data.len()].copy_from_slice(data);
    block::write_many(DATA_START + slot * BLOCKS_PER_FILE, &mut FORMAT_BUFFER[..blocks_needed * block::BLOCK_SIZE])
}

unsafe fn read_file(slot: usize, out: &mut [u8]) -> bool {
    let blocks_needed = core::cmp::max(1, (out.len() + block::BLOCK_SIZE - 1) / block::BLOCK_SIZE);
    if blocks_needed > BLOCKS_PER_FILE { return false; }
    let mut raw = [0u8; FILE_SIZE];
    if !block::read_many(DATA_START + slot * BLOCKS_PER_FILE, &mut raw[..blocks_needed * block::BLOCK_SIZE]) { return false; }
    out.copy_from_slice(&raw[..out.len()]);
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
