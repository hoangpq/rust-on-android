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
    ( $str:expr ) => {{
        use std::ffi::CString;
        CString::new($str)
            .expect("c_str macro::new failed")
            .as_ptr()
    }};
}

#[macro_export]
macro_rules! boxed {
    ( $str:expr ) => {
        $str.to_owned().into_boxed_str().into_boxed_bytes()
    };
}
