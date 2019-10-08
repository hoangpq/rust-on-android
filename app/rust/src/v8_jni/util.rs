use jni::errors::*;
use jni::objects::{JObject, JValue};
use jni::{AttachGuard, JNIEnv, JavaVM};
use jni_sys::{jint, jobject};
use libc::c_char;
use std::sync::{Arc, Once};

static mut JVM: Option<Arc<JavaVM>> = None;
static INIT: Once = Once::new();

extern "C" {
    fn get_java_vm() -> *mut jni_sys::JavaVM;
}

pub fn jvm() -> &'static Arc<JavaVM> {
    INIT.call_once(|| unsafe {
        if let Ok(vm) = JavaVM::from_raw(get_java_vm()) {
            JVM = Some(Arc::new(vm));
        }
    });

    unsafe { JVM.as_ref().unwrap() }
}

#[allow(dead_code)]
pub fn attach_current_thread() -> AttachGuard<'static> {
    jvm()
        .attach_current_thread()
        .expect("failed to attach jvm thread")
}

#[no_mangle]
pub extern "C" fn new_integer(value: i32) -> jobject {
    let env = attach_current_thread();
    env.new_object("java/lang/Integer", "(I)V", &[JValue::Int(value)])
        .unwrap()
        .into_inner()
}
