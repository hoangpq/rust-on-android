use std::slice;

use jni::{
    errors::Result,
    objects::JString,
    objects::{JString},
    strings::JavaStr
};

pub fn jni_string_to_string(env: &JNIEnv, s: JString) -> String {
    let s = JavaStr::from_env(env, s).unwrap();
    let raw = s.get_raw();
    unsafe {
        std::str::from_utf8_unchecked(slice::from_raw_parts(raw as *const u8, libc::strlen(raw)))
            .to_string()
    }
}
