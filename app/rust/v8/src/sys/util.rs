use std::ffi::CString;

pub unsafe fn to_c_str<T>(input: T) -> *const libc::c_char
where
    T: std::convert::Into<std::vec::Vec<u8>>,
{
    let c_str = CString::new(input).unwrap();
    let ptr = c_str.as_ptr();
    std::mem::forget(c_str);
    ptr
}
