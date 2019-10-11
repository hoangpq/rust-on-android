use crate::dex;
use jni::objects::{GlobalRef, JObject, JValue};
use jni::{AttachGuard, JavaVM};
use jni_sys::{jint, jobject, jvalue};
use std::borrow::Cow;
use std::slice;
use std::sync::{Arc, Once};

static mut JVM: Option<Arc<JavaVM>> = None;
static INIT: Once = Once::new();

#[repr(C)]
pub struct string_t {
    ptr: *const u8,
    len: u32,
}

#[repr(C)]
pub struct value_t {
    value: jvalue,
    t: i8,
}

impl value_t {
    pub fn new_int(value: jvalue) -> value_t {
        value_t { value, t: 0 }
    }

    pub fn new_void() -> value_t {
        value_t {
            value: JValue::Void.to_jni(),
            t: -1,
        }
    }
}

impl string_t {
    pub fn to_string(&self) -> Cow<'_, str> {
        let data = unsafe {
            assert!(!self.ptr.is_null());
            slice::from_raw_parts(self.ptr, self.len as usize)
        };
        String::from_utf8_lossy(data)
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
pub unsafe extern "C" fn new_class(class_name: string_t) -> GlobalRef {
    let env = attach_current_thread();
    let class_name = class_name.to_string();
    let class_name = env.new_string(class_name.replace("/", "."));

    let class = env.call_static_method(
        "java/lang/Class",
        "forName",
        "(Ljava/lang/String;)Ljava/lang/Class;",
        &[JValue::from(JObject::from(class_name.unwrap()))],
    );

    env.new_global_ref(class.unwrap().l().unwrap()).unwrap()
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
    args: *const value_t,
    argc: u32,
) -> value_t {
    assert!(!args.is_null());

    let name = name.to_string();
    let env = attach_current_thread();
    let args = slice::from_raw_parts(args, argc as usize);

    let args = args
        .iter()
        .map(|item| match item.t {
            0 => JValue::Int(item.value.i),
            _ => JValue::Void,
        })
        .collect::<Vec<JValue>>();

    let result = env.call_method(instance.as_obj(), name, "(I)I", &args);

    match result {
        Ok(value) => match value {
            JValue::Int(_i) => value_t::new_int(value.to_jni()),
            _ => value_t::new_void(),
        },
        _ => value_t::new_void(),
    }
}
