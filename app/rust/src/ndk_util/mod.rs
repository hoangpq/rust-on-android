use std::slice;

use jni::{objects::JString, strings::JavaStr, JNIEnv};

pub fn jni_string_to_string(env: &JNIEnv, s: JString) -> String {
    JavaStr::from_env(env, s).unwrap().into()
}
