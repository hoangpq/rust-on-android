use libc::c_char;
use std::borrow::Cow;

#[no_mangle]
pub extern "C" fn adb_debug(p: *mut c_char) {
    adb_debug!(rust_str!(p));
}
