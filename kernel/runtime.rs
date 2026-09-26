//! Minimal freestanding compiler runtime for Lay OS.
//! These symbols replace libc/compiler-rt routines emitted by core.

#[no_mangle]
pub unsafe extern "C" fn memcpy(
    dest: *mut u8,
    src: *const u8,
    n: usize,
) -> *mut u8 {
    for i in 0..n {
        let byte = core::ptr::read_volatile(src.add(i));
        core::ptr::write_volatile(dest.add(i), byte);
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memset(dest: *mut u8, value: i32, n: usize) -> *mut u8 {
    let byte = value as u8;
    for i in 0..n {
        core::ptr::write_volatile(dest.add(i), byte);
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(
    left: *const u8,
    right: *const u8,
    n: usize,
) -> i32 {
    for i in 0..n {
        let a = core::ptr::read_volatile(left.add(i));
        let b = core::ptr::read_volatile(right.add(i));
        if a != b {
            return if a < b { -1 } else { 1 };
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn bcmp(
    left: *const u8,
    right: *const u8,
    n: usize,
) -> i32 {
    memcmp(left, right, n)
}

#[no_mangle]
pub unsafe extern "C" fn memmove(
    dest: *mut u8,
    src: *const u8,
    n: usize,
) -> *mut u8 {
    if (dest as usize) <= (src as usize) {
        memcpy(dest, src, n);
    } else {
        for i in (0..n).rev() {
            let byte = core::ptr::read_volatile(src.add(i));
            core::ptr::write_volatile(dest.add(i), byte);
        }
    }
    dest
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
