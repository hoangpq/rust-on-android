use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Mutex;

use jni::objects::{JObject, JValue};
use jni::strings::{JNIStr, JNIString};
use jni_sys::{jlong, jvalue};

use crate::dex::unwrap;
use crate::v8_jni::util::{attach_current_thread_as_daemon, data_t, jvm, value_t};

mod util;

lazy_static! {
    static ref STRING_TABLE: Mutex<HashMap<jlong, String>> = Mutex::new(HashMap::new());
}

#[no_mangle]
pub extern "C" fn _rust_new_string(data: *const c_char) -> jlong {
    let slice = unsafe { CStr::from_ptr(data) };
    let data = slice.to_string_lossy().into_owned();
    Box::into_raw(Box::new(data.clone())) as jlong
}

#[no_mangle]
pub extern "C" fn _rust_get_string(ptr: jlong) -> String {
    let data = unsafe { &mut *(ptr as *mut String) };
    let clone = data.clone();
    let _ = unsafe { Box::from_raw(ptr as *mut String) };
    clone
}
