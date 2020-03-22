use jni::sys::{jlong, jobject};
use jni::{JNIEnv, NativeMethod};
use libc::c_void;

use crate::v8_jni::jvm;

pub struct DenoTask {
    f: Box<dyn FnMut(&JNIEnv)>,
}

impl DenoTask {
    pub fn new<F: 'static>(f: F) -> Box<DenoTask>
    where
        F: Fn(&JNIEnv),
    {
        Box::new(DenoTask { f: Box::new(f) })
    }
}

impl Drop for DenoTask {
    fn drop(&mut self) {
        adb_debug!(format!("Task({:p}) dropped)", &self));
    }
}

macro_rules! jni_method {
    ( $method:tt, $signature:expr ) => {{
        NativeMethod {
            name: stringify!($method).into(),
            sig: $signature.into(),
            fn_ptr: $method as *mut c_void,
        }
    }};
}

#[no_mangle]
pub fn invoke(env: JNIEnv, _obj: jobject, task: jlong) {
    let mut task = unsafe { Box::from_raw(task as *mut DenoTask) };
    if let Ok(env) = jvm().attach_current_thread_as_daemon() {
        (task.f)(&env);
    }
}

#[no_mangle]
unsafe fn register_native(env: &mut jni::sys::JNIEnv) {
    let class_name: &str = "com/node/util/DenoRunnable";
    if let Ok(env) = JNIEnv::from_raw(env) {
        let jni_methods = [jni_method!(invoke, "(J)V")];

        env.register_native_methods(class_name, &jni_methods)
            .unwrap();
    }
}
