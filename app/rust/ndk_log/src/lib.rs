#[macro_use]
extern crate lazy_static;
extern crate utf8_util;

use libc;
use utf8_util::Utf8;

#[derive(Clone, Copy)]
#[repr(isize)]
pub enum LogPriority {
    DEBUG = 3,
    ERROR = 6,
}

extern "C" {
    pub fn __android_log_print(
        prio: libc::c_int,
        tag: *const libc::c_char,
        fmt: *const libc::c_char,
        ...
    ) -> libc::c_int;
}

#[macro_export]
macro_rules! adb_debug {
    ($msg:expr) => {{
        use std::ffi::CString;
        unsafe {
            $crate::__android_log_print(
                $crate::LogPriority::DEBUG as libc::c_int,
                CString::new("Rust Runtime")
                    .expect("CString::new failed")
                    .as_ptr(),
                CString::new(format!("{:?}", $msg))
                    .expect("CString::new failed")
                    .as_ptr(),
            );
        };
    }};
}
