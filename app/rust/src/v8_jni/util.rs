use crate::dex;
use jni::objects::{GlobalRef, JObject, JValue};
use jni::{AttachGuard, JavaVM};
use jni_sys::{jint, jvalue};
use std::sync::{Arc, Once};

static mut JVM: Option<Arc<JavaVM>> = None;
static INIT: Once = Once::new();

#[repr(C)]
pub struct string_t {
    ptr: *mut u8,
    len: u32,
}

impl string_t {
    pub fn to_string(&self) -> String {
        let data = unsafe { Vec::from_raw_parts(self.ptr, self.len as usize, self.len as usize) };
        String::from_utf8_lossy(&data).into_owned()
    }
}

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
pub unsafe extern "C" fn new_instance(class: string_t) -> GlobalRef {
    let class = class.to_string();

    let env = attach_current_thread();
    let instance = env.new_object(class, "()V", &[]);
    env.new_global_ref(instance.unwrap()).unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn instance_call(
    instance: GlobalRef,
    name: string_t,
    argc: u32,
    argv: *mut JValue,
) -> jvalue {
    let name = name.to_string();
    /*let argv = Vec::from_raw_parts(argv, argc as usize, argc as usize);
    let argv = argv
        .iter()
        .map(|value| JValue::Int(value.i))
        .collect::<Vec<JValue>>();*/

    let env = attach_current_thread();
    // let result = env.call_method(instance.as_obj(), name, "(I)I", &argv);
    // result.unwrap().to_jni()

    JValue::from(1).to_jni()
}
