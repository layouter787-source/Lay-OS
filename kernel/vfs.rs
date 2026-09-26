//! LayFS/VFS foundation: fixed-size kernel filesystem backend.

const MAX_FILES: usize = 32;
const NAME_LEN: usize = 32;
const FILE_SIZE: usize = 4096;

#[derive(Clone, Copy)]
struct File {
    used: bool,
    name: [u8; NAME_LEN],
    len: usize,
    data: [u8; FILE_SIZE],
}

const EMPTY: File = File { used: false, name: [0; NAME_LEN], len: 0, data: [0; FILE_SIZE] };
static mut FILES: [File; MAX_FILES] = [EMPTY; MAX_FILES];

pub fn init() {
    unsafe { FILES = [EMPTY; MAX_FILES]; }
    let _ = create("/welcome.txt", b"LAY OS filesystem online\n");
}

pub fn create(path: &str, data: &[u8]) -> bool {
    unsafe {
        if find(path).is_some() { return false; }
        let bytes = path.as_bytes();
        if bytes.is_empty() || bytes.len() >= NAME_LEN || data.len() > FILE_SIZE { return false; }
        let slot = match FILES.iter().position(|f| !f.used) { Some(x) => x, None => return false };
        FILES[slot].used = true;
        FILES[slot].len = data.len();
        let mut i = 0;
        while i < bytes.len() { FILES[slot].name[i] = bytes[i]; i += 1; }
        i = 0;
        while i < data.len() { FILES[slot].data[i] = data[i]; i += 1; }
        true
    }
}

pub fn read(path: &str, out: &mut [u8]) -> Option<usize> {
    unsafe {
        let slot = find(path)?;
        let count = core::cmp::min(FILES[slot].len, out.len());
        let mut i = 0;
        while i < count { out[i] = FILES[slot].data[i]; i += 1; }
        Some(count)
    }
}

pub fn write(path: &str, data: &[u8]) -> bool {
    unsafe {
        let slot = match find(path) { Some(x) => x, None => return create(path, data) };
        if data.len() > FILE_SIZE { return false; }
        FILES[slot].len = data.len();
        let mut i = 0;
        while i < data.len() { FILES[slot].data[i] = data[i]; i += 1; }
        true
    }
}

pub fn list<F: FnMut(&str)>(mut visitor: F) {
    unsafe {
        for file in &FILES {
            if !file.used { continue; }
            let mut end = 0;
            while end < NAME_LEN && file.name[end] != 0 { end += 1; }
            if let Ok(name) = core::str::from_utf8(&file.name[..end]) { visitor(name); }
        }
    }
}

unsafe fn find(path: &str) -> Option<usize> {
    let bytes = path.as_bytes();
    for index in 0..MAX_FILES {
        if !FILES[index].used { continue; }
        let mut end = 0;
        while end < NAME_LEN && FILES[index].name[end] != 0 { end += 1; }
        if end == bytes.len() && FILES[index].name[..end] == bytes[..] { return Some(index); }
    }
    None
}
