use std::ffi::CString;
use std::os::raw;

#[allow(non_camel_case_types)]
pub type c_int = raw::c_int;
#[allow(non_camel_case_types)]
pub type c_char = raw::c_char;

#[derive(Clone, Copy)]
#[repr(isize)]
pub enum LogPriority {
    DEBUG = 3,
    ERROR = 6,
}

extern "C" {
    pub fn __android_log_print(prio: c_int, tag: *const c_char, fmt: *const c_char, ...) -> c_int;
}

pub fn log(msg: String, prio: LogPriority) {
    let msg = CString::new(msg).expect("CString::new failed");
    let tag = CString::new("Rust Runtime").expect("CString::new failed");
    unsafe {
        __android_log_print(prio as c_int, tag.as_ptr(), msg.as_ptr());
    }
}

#[macro_export]
macro_rules! adb_debug {
    ($msg:expr) => {{
        $crate::jni_log::log(format!("{:?}", $msg), $crate::jni_log::LogPriority::DEBUG);
    }};
}
