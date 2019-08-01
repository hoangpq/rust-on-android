#![allow(non_snake_case)]
extern crate jni;
extern crate libc;

pub mod android;
mod fractal;
mod mandelbrot;
use crate::dex;

use jni::errors::Result;
use jni::objects::{GlobalRef, JClass, JObject, JValue};
use jni::sys::{jint, jstring};
use libc::{c_uint, c_void};
use std::{sync::mpsc, thread};

type RenderType = u32;

pub unsafe fn create_bitmap<'b>(
    env: &'b jni::JNIEnv,
    width: c_uint,
    height: c_uint,
) -> Result<JValue<'b>> {
    dex::call_static_method(
        env,
        "com/node/util/Util",
        "createBitmap",
        "(II)Landroid/graphics/Bitmap;",
        &[JValue::from(width as jint), JValue::from(height as jint)],
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_node_sample_MainActivity_getUtf8String(
    env: jni::JNIEnv,
    _class: JClass,
) -> jstring {
    let output = env
        .new_string("ｴｴｯ?工ｴｴｪｪ(๑̀⚬♊⚬́๑)ｪｪｴｴ工‼!!!")
        .expect("Couldn't create java string!");

    output.into_inner()
}

unsafe fn blend_bitmap<'a>(
    vm: jni::JavaVM,
    render_type: RenderType,
    image_view_ref: GlobalRef,
) -> Result<&'a str> {
    // Attach current thread
    let env = vm.attach_current_thread()?;

    // Get raw bitmap object
    let image_view = image_view_ref.as_obj();

    let bmp = create_bitmap(&env, 800, 800)?.l()?.into_inner();
    let mut info = android::AndroidBitmapInfo::new();

    // Get raw JNIEnv (without lifetime)
    let raw_env = env.get_native_interface();

    // Read bitmap info
    android::bitmap_get_info(raw_env, bmp, &mut info);
    let mut pixels = 0 as *mut c_void;

    // Lock pixel for draw
    android::bitmap_lock_pixels(raw_env, bmp, &mut pixels);

    let pixels =
        std::slice::from_raw_parts_mut(pixels as *mut u8, (info.stride * info.height) as usize);

    match render_type {
        0x000001 => mandelbrot::render(pixels, info.width as u32, info.height as u32),
        0x000002 => fractal::render(pixels, info.width as u32, info.height as u32),
        _ => {}
    };

    android::bitmap_unlock_pixels(raw_env, bmp);

    env.call_method(
        image_view,
        "setImageBitmap",
        "(Landroid/graphics/Bitmap;)V",
        &[JValue::from(JObject::from(bmp))],
    )?;

    Ok("Render successfully")
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_node_sample_GenerateImageActivity_blendBitmap(
    env: jni::JNIEnv,
    _class: JObject,
    image_view: JObject,
    render_type: u32,
    callback: JObject,
) {
    let vm = env.get_java_vm().unwrap();
    let image_view_ref = env.new_global_ref(image_view).unwrap();
    let (tx, rx) = mpsc::sync_channel::<&str>(1);

    thread::spawn(move || {
        let msg = match blend_bitmap(vm, render_type, image_view_ref) {
            Ok(msg) => msg,
            _ => "Failed to render!",
        };

        // Send to main thread
        tx.send(msg).unwrap();
    });

    // <- msg
    let msg = rx.recv().unwrap();
    if let Err(_) = env.call_method(
        callback,
        "invoke",
        "(Ljava/lang/String;)V",
        &[JValue::from(JObject::from(env.new_string(msg).unwrap()))],
    ) {
        env.exception_check().unwrap_or_else(|e| {
            adb_debug!(format!("{:?}", e));
            true
        });
    }
}
