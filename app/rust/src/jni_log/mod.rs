use std::os::raw;
use std::ffi::CString;

#[allow(non_camel_case_types)]
pub type c_int = raw::c_int;
#[allow(non_camel_case_types)]
pub type c_char = raw::c_char;

#[derive(Clone, Copy)]
#[repr(isize)]
pub enum LogPriority {
    DEBUG = 3,
}

extern "C" {
    pub fn __android_log_print(prio: c_int,
                               tag: *const c_char,
                               fmt: *const c_char,
                               ...)
                               -> c_int;
}

pub fn debug(msg: String) {
    let tag = CString::new("Rust").expect("CString::new failed");
    let msg = CString::new(msg).expect("CString::new failed");
    unsafe {
        __android_log_print(LogPriority::DEBUG as c_int,
                            tag.as_ptr(), msg.as_ptr());
    }
}
