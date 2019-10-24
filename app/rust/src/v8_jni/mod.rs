use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{Arc, Once};

use jni::{AttachGuard, JNIEnv, JavaVM};
use jni_sys::jlong;

mod util;

extern "C" {
    fn get_java_vm() -> *mut jni::sys::JavaVM;
}

static mut JVM: Option<Arc<JavaVM>> = None;
static INIT: Once = Once::new();

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

#[allow(dead_code)]
pub fn attach_current_thread_as_daemon() -> JNIEnv<'static> {
    jvm()
        .attach_current_thread_as_daemon()
        .expect("failed to attach jvm thread")
}

#[no_mangle]
pub extern "C" fn _rust_new_string(data: *const c_char) -> jlong {
    let slice = unsafe { CStr::from_ptr(data) };
    let data = slice.to_string_lossy().into_owned();
    Box::into_raw(Box::new(data.clone())) as jlong
}

#[no_mangle]
pub extern "C" fn _rust_get_string(ptr: jlong) -> String {
    let data = unsafe { Box::from_raw(ptr as *mut String) };
    data.to_string()
}
