use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::Mutex;

use jni::JNIEnv;
use jni::objects::{GlobalRef, JMethodID, JObject, JString};
use jni::sys;
use jni_sys::jobject;

use crate::runtime::ptr_to_string;

#[derive(Clone)]
struct Event {
    instance: GlobalRef,
    method_id: JMethodID<'static>,
}

unsafe impl Send for Event {}

lazy_static! {
    static ref EVENT_TABLE: Mutex<HashMap<String, Event>> = Mutex::new(HashMap::new());
}

extern "C" {
    fn c_dispatch_event(
        env: *mut sys::JNIEnv,
        instance: sys::jobject,
        mid: sys::jmethodID,
        data: jobject,
    );
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_com_node_sample_MainActivity_addEventListener(
    env: JNIEnv<'static>,
    instance: JObject,
    event_name: JString,
) {
    let s = env.get_string(event_name).unwrap().as_ptr();
    let c_str = unsafe { CStr::from_ptr(s) };
    let s = c_str.to_str().unwrap().to_string();

    let mut table = EVENT_TABLE.lock().unwrap();
    let class = env.get_object_class(instance).unwrap();
    let method_id: JMethodID<'static> = env
        .get_method_id(class, format!("{}Listener", s), "(Ljava/lang/Object;)V")
        .unwrap();

    table.insert(
        s,
        Event {
            method_id,
            instance: env.new_global_ref(instance).unwrap(),
        },
    );
}

#[no_mangle]
pub extern "C" fn dispatch_event(
    env: JNIEnv,
    event_name: *const libc::c_char,
    msg: *const libc::c_char,
) {
    let mut table = EVENT_TABLE.lock().unwrap();

    let event_name = unsafe { ptr_to_string(event_name).unwrap() };
    if let Some(event) = table.get_mut(&event_name) {
        let obj = event.instance.as_obj().into_inner();
        let method_id = event.method_id.into_inner();
        let msg = unsafe { ptr_to_string(msg).unwrap() };

        let s = env.new_string(msg).unwrap();
        unsafe { c_dispatch_event(env.get_native_interface(), obj, method_id, s.into_inner()) };
    }
}
