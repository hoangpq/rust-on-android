use crate::dex;
use jni::objects::JValue::Object;
use jni::objects::{GlobalRef, JList, JObject, JValue};
use jni::{AttachGuard, JavaVM};
use jni_sys::{jint, jobject, jsize, jvalue, JNIEnv};
use std::any::Any;
use std::borrow::Cow;
use std::slice;
use std::sync::{Arc, Once};
use v8::fun::CallbackInfo;

static mut JVM: Option<Arc<JavaVM>> = None;
static INIT: Once = Once::new();

static ARRAY_LIST_CLASS: &str = "java/util/ArrayList";
static STRING_CLASS: &str = "java/lang/String";
static INTEGER_CLASS: &str = "java/lang/Integer";
static OBJECT_CLASS: &str = "java/lang/Object";

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

fn new_int(env: &AttachGuard<'static>, value: i32) -> JObject<'static> {
    env.new_object(INTEGER_CLASS, "(I)V", &[JValue::from(value)])
        .unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn instance_call(
    instance: GlobalRef,
    name: string_t,
    args: *const value_t,
    argc: u32,
    info: &CallbackInfo,
) {
    assert!(!args.is_null());

    let name = name.to_string();
    let env = attach_current_thread();
    let args = slice::from_raw_parts(args, argc as usize);

    let method_name = JObject::from(env.new_string(name.to_string()).unwrap());

    let (types, values) = {
        let types = env
            .new_object_array(args.len() as i32, INTEGER_CLASS, JObject::null())
            .unwrap();

        let values = env
            .new_object_array(args.len() as i32, OBJECT_CLASS, JObject::null())
            .unwrap();

        for (index, item) in args.iter().enumerate() {
            let value = match item.t {
                0 => new_int(&env, item.value.i),
                _ => JObject::null(),
            };
            env.set_object_array_element(types, index as i32, new_int(&env, item.t.into()));
            env.set_object_array_element(values, index as i32, value);
        }

        (JObject::from(types), JObject::from(values))
    };

    if let Ok(value) = dex::call_static_method(
        &env,
        "com/node/util/JNIHelper",
        "callMethod",
        "(Ljava/lang/Object;Ljava/lang/String;[Ljava/lang/Integer;[Ljava/lang/Object;)Ljava/lang/Object;",
        &[
            JValue::Object(instance.as_obj()),
            JValue::Object(method_name),
            JValue::Object(types),
            JValue::Object(values),
        ],
    ) {
        if let JValue::Object(resp) = value {
            let internal = env.call_method(resp, "getInternal", "()Ljava/lang/Object;", &[]).unwrap();
            let sig = env.get_field(resp, "sig", "I").unwrap().i().unwrap() as u8;

            return match sig {
                0u8 => info.set_return_value(v8::new_number(internal.to_jni().i)),
                _ => info.set_return_value(v8::null())
            }
        }
    }

    info.set_return_value(v8::null())
}
