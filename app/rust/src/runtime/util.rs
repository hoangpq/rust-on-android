use std::ffi::CStr;

use libc::c_char;

#[no_mangle]
pub extern "C" fn adb_debug(msg: *mut c_char) {
    let msg = unsafe { CStr::from_ptr(msg) };
    let as_str = msg.to_str().expect("The message is always valid UTF8");
    adb_debug!(as_str);
}
