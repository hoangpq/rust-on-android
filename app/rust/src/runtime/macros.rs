use libc::c_char;
use std::ffi::CString;

#[macro_export]
macro_rules! rust_str {
    ( $p:expr ) => {
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                $p as *const u8,
                libc::strlen($p),
            ))
        }
    };
}

#[macro_export]
macro_rules! c_str {
    ( $str:expr ) => {
        $crate::runtime::macros::to_c_str($str)
    };
}

#[macro_export]
macro_rules! boxed {
    ( $str:expr ) => {
        $str.to_owned().into_boxed_str().into_boxed_bytes()
    };
}
