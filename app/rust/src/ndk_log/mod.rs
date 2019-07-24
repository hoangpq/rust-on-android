use libc;

#[derive(Clone, Copy)]
#[repr(isize)]
pub enum LogPriority {
    DEBUG = 3,
    ERROR = 6,
}

extern "C" {
    pub(crate) fn __android_log_print(
        prio: libc::c_int,
        tag: *const libc::c_char,
        fmt: *const libc::c_char,
        ...
    ) -> libc::c_int;
}

#[macro_export]
macro_rules! adb_debug {
    ($msg:expr) => {{
        unsafe {
            $crate::ndk_log::__android_log_print(
                $crate::ndk_log::LogPriority::DEBUG as libc::c_int,
                c_str!("Rust Runtime"),
                c_str!(format!("{:?}", $msg)),
            );
        };
    }};
}
