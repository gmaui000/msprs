use ffmpeg_sys_next as ffmpeg;
use libc;
use std::ffi::{CStr, CString};

/// Converts a C-style string pointer to a Rust `String`.
///
/// # Safety
///
/// The `c_str` pointer must point to a valid C-style string, meaning it must be null-terminated.
/// Dereferencing an invalid pointer will lead to undefined behavior.
/// The memory pointed to by `c_str` must not be deallocated or modified during the lifetime of the returned `String`.
pub unsafe fn c_str_to_string(c_str: *const libc::c_char) -> String {
    CStr::from_ptr(c_str).to_str().unwrap().to_string()
}

pub fn str_to_c_str(str: &str) -> CString {
    CString::new(str).expect("could not alloc CString")
}

pub fn ffmpeg_error(func: &str, code: i32) -> String {
    unsafe {
        let msg = c_str_to_string(ffmpeg::strerror(code));
        format!(
            "[ffmpeg_error] code: {}, func: {}, msg: {}",
            code, func, msg
        )
    }
}
