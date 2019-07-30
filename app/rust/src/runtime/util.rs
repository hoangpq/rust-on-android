use libc::c_char;

#[no_mangle]
pub extern "C" fn adb_debug(p: *mut c_char) {
    adb_debug!(rust_str!(p));
}
