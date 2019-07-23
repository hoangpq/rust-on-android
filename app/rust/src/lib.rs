#![allow(improper_ctypes)]
#![feature(stmt_expr_attributes)]

extern crate bytes;
extern crate core;
extern crate curl;
extern crate futures;
#[macro_use]
extern crate itertools;
extern crate jni;
extern crate jni_sys;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate reqwest;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate tokio;
extern crate tokio_threadpool;
extern crate tokio_timer;
extern crate v8;
extern crate v8_macros;

use std::ffi::CString;
use std::mem;

use jni::JNIEnv;
use jni::objects::{JObject, JValue};
use jni::sys::{jint, jlong, jobject};
use libc::{c_char, size_t};
use v8_macros::v8_fn;

pub mod ndk_util;
#[macro_use]
pub mod ndk_log;
#[macro_use]
pub mod ndk_graphics;
#[macro_use]
pub mod dex;
#[macro_use]
pub mod runtime;
pub mod buffer;

#[no_mangle]
pub unsafe extern "C" fn get_android_version(env: &JNIEnv) -> i32 {
    // Android Version
    env.get_static_field("android/os/Build$VERSION", "SDK_INT", "I")
        .unwrap()
        .i()
        .unwrap() as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn createTimeoutHandler(env: &JNIEnv) -> jobject {
    let result = env
        .call_static_method(
            "com/node/v8/V8Utils",
            "getHandler",
            "()Landroid/os/Handler;",
            &[],
        )
        .expect("Can not create handler!");

    match result.l() {
        Ok(v) => v.into_inner(),
        Err(e) => panic!(e),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn postDelayed(env: &JNIEnv, handler: JObject, f: jlong, d: jlong, t: jint) {
    let ctx = env
        .call_static_method(
            "com/node/v8/V8Context",
            "getCurrent",
            "()Lcom/node/v8/V8Context;",
            &[],
        )
        .expect("Can not get current context");

    let timer_sig = if t == 1 {
        "createTimeoutRunnable"
    } else {
        "createIntervalRunnable"
    };

    let runnable = env
        .call_static_method(
            "com/node/v8/V8Runnable",
            timer_sig,
            "(Lcom/node/v8/V8Context;JJ)Lcom/node/v8/V8Runnable;",
            &[ctx, JValue::from(f), JValue::from(d)],
        )
        .expect("Can not create Runnable by factory!");

    match runnable.l() {
        Ok(v) => {
            let result = env.call_method(
                handler,
                "postDelayed",
                "(Ljava/lang/Runnable;J)Z",
                &[JValue::Object(v), JValue::from(d)],
            );

            match result {
                Ok(_v) => {}
                Err(e) => panic!(e),
            }
        }
        Err(e) => panic!(e),
    };
}

type Buf = *mut u8;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn workerSendBytes(_buf: Buf, _len: size_t, _cb: v8::Value) -> *const c_char {
    let _contents: *mut u8;
    unsafe {
        let ab: v8::ArrayBuffer = v8::ArrayBuffer::New(&"💖".as_bytes());
        let _cb: v8::Function = v8::Function::Cast(_cb);
        _cb.Call(vec![ab]);

        _contents = mem::transmute(_buf);
        let slice: &[u8] = std::slice::from_raw_parts(_contents, _len as usize);
        let name = buffer::load_user_buf(slice).unwrap();
        let s = CString::new(name).unwrap();
        let ptr = s.as_ptr();
        mem::forget(s);
        ptr
    }
}

#[v8_fn]
pub fn test_fn(args: &v8::CallbackInfo) {
    let f: v8::Function = v8::Function::Cast(args.Get(0));
    f.Call(vec![] as Vec<v8::Value>);
    args.SetReturnValue(v8::String::NewFromUtf8("Send 💖 to JS world!"));
}

#[allow(dead_code)]
fn main() {}
