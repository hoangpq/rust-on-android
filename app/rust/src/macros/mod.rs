use std::ffi::CString;

use libc::c_char;

pub unsafe fn to_c_str<T>(input: T) -> *const c_char
where
    T: std::convert::Into<std::vec::Vec<u8>>,
{
    let c_str = CString::new(input).unwrap();
    let ptr = c_str.as_ptr();
    std::mem::forget(c_str);
    ptr
}

#[macro_export]
macro_rules! rust_str {
    ( $p:expr ) => {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(
            $p as *const u8,
            libc::strlen($p),
        ))
    };
}

#[macro_export]
macro_rules! c_str {
    ( $str:expr ) => {
        $crate::macros::to_c_str($str)
    };
}

#[macro_export]
macro_rules! boxed {
    ( $str:expr ) => {
        $str.to_owned().into_boxed_str().into_boxed_bytes()
    };
}
