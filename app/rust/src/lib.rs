#![feature(stmt_expr_attributes)]

#[macro_use]
extern crate itertools;
extern crate jni;
extern crate libc;

mod jni_graphics;
mod jni_log;

use std::cmp;
use std::thread;
use std::sync::mpsc;
use std::os::raw::c_void;
use std::time::Duration;
use itertools::Itertools;

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JValue};
use jni::sys::{jint, jlong, jstring, jobject};

use std::ffi::CString;

use jni_graphics::create_bitmap;
use jni_graphics::{Color, AndroidBitmapInfo};
use jni_graphics::{AndroidBitmap_getInfo, AndroidBitmap_lockPixels, AndroidBitmap_unlockPixels};

extern crate curl;
use curl::easy::Easy;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[no_mangle]
pub extern "C" fn init_module() {}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_com_node_sample_MainActivity_asyncComputation(
    env: JNIEnv,
    _class: JClass,
    callback: JObject,
) {
    let jvm = env.get_java_vm().unwrap();
    let callback = env.new_global_ref(callback).unwrap();
    let (tx, rx) = mpsc::channel();
    let _ = thread::spawn(move || {
        tx.send(()).unwrap();
        let env = jvm.attach_current_thread().unwrap();
        let callback = callback.as_obj();
        for i in 0..10_000 {
            let progress = i as jint;
            env.call_method(callback, "subscribe", "(I)V", &[progress.into()])
                .unwrap();
            thread::sleep(Duration::from_millis(300));
        }
    });
    rx.recv().unwrap();
}

fn generate_palette() -> Vec<Color> {
    let mut palette: Vec<Color> = vec![];
    let mut roffset = 24;
    let mut goffset = 16;
    let mut boffset = 0;
    for i in 0..256 {
        palette.push(Color {
            red: roffset,
            green: goffset,
            blue: boffset,
        });
        if i < 64 {
            roffset += 3;
        } else if i < 128 {
            goffset += 3;
        } else if i < 192 {
            boffset += 3;
        }
    }
    return palette;
}

pub fn draw_mandelbrot(
    buffer: &mut [u8],
    width: i64,
    height: i64,
    pixel_size: f64,
    x0: f64,
    y0: f64,
) {
    println!("Pixel size: {:?} - x0: {:?} - y0: {:?}", pixel_size, x0, y0);
    let palette: Vec<Color> = generate_palette();
    iproduct!((0..width), (0..height)).foreach(|(i, j)| {
        let cr = x0 + pixel_size * (i as f64);
        let ci = y0 + pixel_size * (j as f64);
        let (mut zr, mut zi) = (0.0, 0.0);

        let k = (0..256)
            .take_while(|_| {
                let (zrzi, zr2, zi2) = (zr * zi, zr * zr, zi * zi);
                zr = zr2 - zi2 + cr;
                zi = zrzi + zrzi + ci;
                zi2 + zr2 < 2.0
            })
            .count();
        let k = cmp::min(255, k) as u8;
        let idx = (4 * (j * width + i)) as usize;

        let result = palette.get(k as usize);
        match result {
            Some(color) => {
                buffer[idx] = color.red;
                buffer[idx + 1] = color.green;
                buffer[idx + 2] = color.blue;
                buffer[idx + 3] = 255;
            }
            None => {}
        }
    });
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_com_node_sample_MainActivity_getUtf8String(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let ptr = CString::new(
        "ｴｴｯ?工ｴｴｪｪ(๑̀⚬♊⚬́๑)ｪｪｴｴ工‼!!!".to_owned(),
    ).unwrap();
    let output = env.new_string(ptr.to_str().unwrap()).expect(
        "Couldn't create java string!",
    );
    output.into_inner()
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_com_node_sample_GenerateImageActivity_blendBitmap<'b>(
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
struct User<'a> {
    name: &'a str,
}

fn fetch_json() {
    let mut json = Vec::new();
    let mut handle = Easy::new();
    handle.ssl_verify_peer(false).unwrap();

    handle
        .url("https://my-json-server.typicode.com/typicode/demo/profile")
        .unwrap();
    {
        let mut transfer = handle.transfer();
        transfer
            .write_function(|data| {
                json.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }

    assert_eq!(200, handle.response_code().unwrap());
    // let s = std::str::from_utf8(&json).unwrap().to_string();
    let u: User = serde_json::from_slice(&json).unwrap();
    jni_log::debug(format!("{:?}", u.name));
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn onNodeServerLoaded(env: &JNIEnv, activity: JObject) {
    fetch_json();
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
        Err(e) => {
            jni_log::debug(format!("{:?}", e));
            panic!(e)
        }
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

    let runnable = env.call_static_method(
        "com/node/v8/V8Runnable",
        if t == 1 {
            "createTimeoutRunnable"
        } else {
            "createIntervalRunnable"
        },
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
                Err(e) => {
                    jni_log::debug(format!("{:?}", e));
                    panic!(e);
                }
            }
        }
        Err(e) => {
            jni_log::debug(format!("{:?}", e));
            panic!(e);
        }
    };
}
