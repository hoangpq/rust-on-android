extern crate cast;

use std::borrow::Cow;
use std::slice;
use std::sync::{Arc, Once};

use jni::objects::{GlobalRef, JObject, JString, JValue};
use jni::strings::JavaStr;
use jni::sys::{jlong, jvalue};
use jni::{AttachGuard, JavaVM};
use v8::fun::CallbackInfo;

use crate::dex;
use crate::dex::{unwrap, unwrap_js};

static mut JVM: Option<Arc<JavaVM>> = None;
static INIT: Once = Once::new();

static STRING_CLASS: &str = "java/lang/String";
static INTEGER_CLASS: &str = "java/lang/Integer";
static OBJECT_CLASS: &str = "java/lang/Object";

static JNI_HELPER_CLASS: &str = "com/node/util/JNIHelper";

extern "C" {
    fn get_java_vm() -> *mut jni_sys::JavaVM;
}

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
    fn to_slice(&self) -> &[u8] {
        unsafe {
            assert!(!self.ptr.is_null());
            slice::from_raw_parts(self.ptr, self.len as usize)
        }
    }
    pub fn to_string(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.to_slice())
    }
    pub fn to_jstring(&self, env: &AttachGuard<'static>) -> jni::errors::Result<JString> {
        env.new_string(String::from_utf8_lossy(self.to_slice()))
    }
}

#[no_mangle]
pub extern "C" fn new_integer(val: i32) -> jvalue {
    JValue::from(val).to_jni()
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
pub unsafe extern "C" fn is_field(instance_ptr: jlong, field: string_t) -> bool {
    let global_ref = &mut *(instance_ptr as *mut GlobalRef);
    let env = attach_current_thread();

    let instance = unwrap(
        &env,
        dex::call_static_method(
            &env,
            JNI_HELPER_CLASS,
            "isField",
            "(Ljava/lang/Object;Ljava/lang/String;)Z",
            &[
                JValue::Object(global_ref.as_obj()),
                JValue::Object(JObject::from(field.to_jstring(&env).unwrap())),
            ],
        ),
    );

    match instance.z() {
        Ok(result) => result,
        Err(_) => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn is_method(instance_ptr: jlong, method: string_t) -> bool {
    let global_ref = &mut *(instance_ptr as *mut GlobalRef);
    let env = attach_current_thread();

    let instance = unwrap(
        &env,
        dex::call_static_method(
            &env,
            JNI_HELPER_CLASS,
            "isMethod",
            "(Ljava/lang/Object;Ljava/lang/String;)Z",
            &[
                JValue::Object(global_ref.as_obj()),
                JValue::Object(JObject::from(method.to_jstring(&env).unwrap())),
            ],
        ),
    );

    match instance.z() {
        Ok(result) => result,
        Err(_) => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn set_private_ptr() {}

#[no_mangle]
pub unsafe extern "C" fn get_current_activity() -> jlong {
    let env = attach_current_thread();
    let weak_ref = unwrap(
        &env,
        dex::call_static_method(
            &env,
            JNI_HELPER_CLASS,
            "getCurrentActivity",
            "()Ljava/lang/ref/WeakReference;",
            &[],
        ),
    );

    let instance = unwrap(&env, weak_ref.l());
    let instance_ref = unwrap(&env, env.new_global_ref(instance));
    Box::into_raw(Box::new(instance_ref)) as jlong
}

#[no_mangle]
pub unsafe extern "C" fn new_instance(class: string_t, args: *const value_t, argc: u32) -> jlong {
    let ctor_args = if argc > 0u32 { &[] } else { &[] };

    let class = class.to_string();
    let env = attach_current_thread();
    let instance = unwrap(&env, env.new_object(class, "()V", ctor_args));
    let instance_ref = unwrap(&env, env.new_global_ref(instance));
    Box::into_raw(Box::new(instance_ref)) as jlong
}

fn new_int(env: &AttachGuard<'static>, value: i32) -> JObject<'static> {
    unwrap(
        &env,
        env.new_object(INTEGER_CLASS, "(I)V", &[JValue::from(value)]),
    )
}

#[no_mangle]
pub unsafe extern "C" fn instance_call(
    instance_ptr: jlong,
    name: string_t,
    args: *const value_t,
    argc: u32,
    info: &CallbackInfo,
) {
    let global_ref = &mut *(instance_ptr as *mut GlobalRef);

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

            env.set_object_array_element(types, index as i32, new_int(&env, item.t.into()))
                .unwrap();
            env.set_object_array_element(values, index as i32, value)
                .unwrap();
        }

        (JObject::from(types), JObject::from(values))
    };

    let result = unwrap_js(&env, dex::call_static_method(
        &env,
        JNI_HELPER_CLASS,
        "callMethod",
        "(Ljava/lang/Object;Ljava/lang/String;[Ljava/lang/Integer;[Ljava/lang/Object;)Ljava/lang/Object;",
        &[
            JValue::Object(global_ref.as_obj()),
            JValue::Object(method_name),
            JValue::Object(types),
            JValue::Object(values),
        ],
    ));

    if let Some(JValue::Object(resp)) = result {
        let internal = unwrap(
            &env,
            env.call_method(resp, "getInternal", "()Ljava/lang/Object;", &[]),
        );

        let sig = unwrap(&env, env.get_field(resp, "sig", "I"));
        let sig = unwrap(&env, sig.i()) as u8;

        let has_error = unwrap(&env, env.get_field(resp, "hasError", "Z"));
        let has_error = unwrap(&env, has_error.z());

        if has_error {
            dex::throw_js_exception(&env, internal).unwrap();
            return;
        }

        return match sig {
            0u8 => {
                let value = unwrap(
                    &env,
                    dex::call_static_method(
                        &env,
                        JNI_HELPER_CLASS,
                        "intValue",
                        "(Ljava/lang/Object;)I",
                        &[internal],
                    ),
                );
                info.set_return_value(v8::new_number(value.i().unwrap()))
            }
            1u8 => {
                let value = unwrap(
                    &env,
                    dex::call_static_method(
                        &env,
                        JNI_HELPER_CLASS,
                        "longValue",
                        "(Ljava/lang/Object;)J",
                        &[internal],
                    ),
                );
                info.set_return_value(v8::new_number(cast::f64(value.j().unwrap())));
            }
            2u8 => {
                let value = unwrap(
                    &env,
                    dex::call_static_method(
                        &env,
                        JNI_HELPER_CLASS,
                        "doubleValue",
                        "(Ljava/lang/Object;)D",
                        &[internal],
                    ),
                );
                info.set_return_value(v8::new_number(value.d().unwrap()))
            }
            _ => info.set_return_value(v8::null()),
        };
    }

    info.set_return_value(v8::null())
}
