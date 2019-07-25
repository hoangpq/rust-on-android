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
