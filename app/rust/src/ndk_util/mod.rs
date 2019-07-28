use std::slice;

use jni::{objects::JString, strings::JavaStr, JNIEnv};

pub fn jni_string_to_string(env: &JNIEnv, s: JString) -> String {
    let java_str = JavaStr::from_env(env, s).unwrap();
    let raw = java_str.get_raw();
    unsafe {
        std::str::from_utf8_unchecked(slice::from_raw_parts(raw as *const u8, libc::strlen(raw)))
            .to_string()
    }
}
