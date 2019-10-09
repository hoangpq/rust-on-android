use crate::dex;
use jni::errors::*;
use jni::objects::JValue;
use jni::{AttachGuard, JavaVM};
use jni_sys::jvalue;
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
pub extern "C" fn new_integer(val: i32) -> jvalue {
    JValue::from(val).to_jni()
}

#[no_mangle]
pub extern "C" fn static_call(val: JValue) -> Result<()> {
    let env = attach_current_thread();
    dex::call_static_method(&env, "com/node/util/Util", "testMethod", "(I)I", &[val])?;

    Ok(())
}
