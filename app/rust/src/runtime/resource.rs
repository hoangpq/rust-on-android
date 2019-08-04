#![allow(non_snake_case)]
use jni::errors::Result;
use jni::objects::{JObject, JValue};
use jni::JNIEnv;
use jni::JavaVM;
use std::sync::Arc;

static JVM: Option<Arc<JavaVM>> = None;

pub fn load_resource<'a>(env: &'a JNIEnv) -> Result<String> {
    let result = env.call_static_method(
        "com/node/util/ResourceUtil",
        "readRawResource",
        "(Ljava/lang/String;)Ljava/lang/String;",
        &[JValue::from(JObject::from(env.new_string("isolate")?))],
    );

    let raw = result?.l()?;
    let java_str = env.get_string(raw.into())?;

    Ok(java_str.into())
}
