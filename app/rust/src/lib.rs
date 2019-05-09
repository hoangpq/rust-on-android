#![allow(improper_ctypes)]
#![feature(stmt_expr_attributes)]

#[macro_use]
extern crate itertools;
extern crate jni;
extern crate libc;
extern crate core;
extern crate curl;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate bytes;

#[macro_use]
pub mod jni_log;
pub mod v8;
#[macro_use]
mod jni_graphics;
mod buffer;

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JValue};
use jni::sys::{jint, jlong, jobject};
use std::ffi::CString;

use libc::size_t;
use std::os::raw::{c_void, c_char};
use std::{mem, thread};
use std::sync::mpsc;

use core::borrow::BorrowMut;

use jni_graphics::{create_bitmap, draw_mandelbrot};
use jni_graphics::{ AndroidBitmapInfo};
use jni_graphics::{AndroidBitmap_getInfo, AndroidBitmap_lockPixels, AndroidBitmap_unlockPixels};

use curl::easy::Easy;
use v8::{Function, ArrayBuffer, Value, CallbackInfo};

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_com_node_sample_MainActivity_asyncComputation(
    env: JNIEnv,
    _class: JClass,
    _ctx: JObject,
) {
    let script = env.new_string(r#"
        try {
            let count = 0;
            const $ref = $createRef();
            function update() {
                $invokeRef($ref, ++count);
            }
            setInterval(update, 1e3);
        } catch (e) {
            $log(e.message);
        }
    "#);

    match script {
        Ok(s) => {

            let obj: JObject = s.into();
            let result = env.call_method(
                _ctx,
                "eval",
                "(Ljava/lang/String;)Lcom/node/v8/V8Context$V8Result;",
                &[JValue::from(obj)]);

            match result {
                Ok(r) => adb_debug!(r),
                Err(e) => adb_debug!(e)
            };
        },
        Err(e) => adb_debug!(e)
    };
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_com_node_sample_GenerateImageActivity_blendBitmap<'b>(
    env: JNIEnv<'b>,
    _class: JObject,
    imageView: JObject,
    pixel_size: f64,
    x0: f64,
    y0: f64,
) {
    let jvm = env.get_java_vm().unwrap();
    let imageViewRef = env.new_global_ref(imageView).unwrap();
    let _classRef = env.new_global_ref(_class).unwrap();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        // attach current thread
        let env = jvm.attach_current_thread().unwrap();
        let jenv = env.get_native_interface();
        let imageView = imageViewRef.as_obj();
        // create new bitmap
        let jbitmap = create_bitmap(&env, 800, 800);
        let bitmap = jbitmap.l().unwrap().into_inner();
        let mut info = AndroidBitmapInfo { ..Default::default() };
        // Read bitmap info
        AndroidBitmap_getInfo(jenv, bitmap, &mut info);
        let mut pixels = 0 as *mut c_void;
        // Lock pixel for draw
        AndroidBitmap_lockPixels(jenv, bitmap, &mut pixels);
        let pixels = ::std::slice::from_raw_parts_mut(
            pixels as *mut u8,
            (info.stride * info.height) as usize,
        );
        draw_mandelbrot(
            pixels,
            info.width as i64,
            info.height as i64,
            pixel_size,
            x0,
            y0,
        );
        AndroidBitmap_unlockPixels(jenv, bitmap);
        // detach current thread
        env.call_method(
            imageView,
            "setImageBitmap",
            "(Landroid/graphics/Bitmap;)V",
            &[JValue::from(JObject::from(bitmap))],
        ).unwrap();
        tx.send(()).unwrap();
    });
    rx.recv().unwrap();
    env.call_method(_class, "showToast", "()V", &[]).unwrap();
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn getAndroidVersion(env: &JNIEnv) -> i32 {
    // Android Version
    env.get_static_field("android/os/Build$VERSION", "SDK_INT", "I")
        .unwrap()
        .i()
        .unwrap() as i32
}


#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn onNodeServerLoaded(env: &JNIEnv, activity: JObject) {
    env.call_method(activity, "onNodeServerLoaded", "()V", &[])
        .unwrap();
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn createTimeoutHandler(env: &JNIEnv) -> jobject {
    let result = env.call_static_method(
        "com/node/v8/V8Utils",
        "getHandler",
        "()Landroid/os/Handler;",
        &[],
    ).expect("Can not create handler!");

    match result.l() {
        Ok(v) => v.into_inner(),
        Err(e) => panic!(e),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn postDelayed(env: &JNIEnv, handler: JObject, f: jlong, d: jlong, t: jint) {
    let ctx = env.call_static_method(
        "com/node/v8/V8Context",
        "getCurrent",
        "()Lcom/node/v8/V8Context;",
        &[],
    ).expect("Can not get current context");

    let timer_sig = if t == 1 {
        "createTimeoutRunnable"
    } else {
        "createIntervalRunnable"
    };

    let runnable = env.call_static_method(
        "com/node/v8/V8Runnable",
        timer_sig,
        "(Lcom/node/v8/V8Context;JJ)Lcom/node/v8/V8Runnable;",
        &[ctx, JValue::from(f), JValue::from(d)],
    ).expect("Can not create Runnable by factory!");

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

#[allow(dead_code)]
fn fetch_user() -> User {
    let mut handle = Easy::new();
    handle.ssl_verify_peer(false).unwrap();

    handle
        .url("https://my-json-server.typicode.com/typicode/demo/profile")
        .unwrap();

    let mut json = Vec::new();
    {
        let mut transfer = handle.transfer();
        transfer
            .borrow_mut()
            .write_function(|data| {
                json.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }

    let json = json.to_owned();
    assert_eq!(200, handle.response_code().unwrap());
    serde_json::from_slice(&json).unwrap()
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn workerSendBytes(_buf: *mut u8, _len: size_t, _cb: Value) -> *const c_char {
    let _contents: *mut u8;
    unsafe {
        let ab: ArrayBuffer = ArrayBuffer::New(&"💖".as_bytes());
        let _cb: Function = Function::Cast(_cb);
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

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Perform(args: &CallbackInfo) {
    let f: Function = Function::Cast(args.Get(0));
    f.Call(vec![] as Vec<Value>);
    args.SetReturnValue(v8::String::NewFromUtf8("Send 💖 to JS world!"));
}

#[allow(dead_code)]
fn main() {}
