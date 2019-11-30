#![allow(non_snake_case)]
extern crate jni;

use std::os::raw::{c_int, c_uint, c_void};
use std::thread;

use jni::errors::Result;
use jni::objects::{GlobalRef, JClass, JObject, JValue};
use jni::sys::{jint, jlong, jstring};

use crate::dex;
use crate::dex::unwrap;
use crate::v8_jni::attach_current_thread_as_daemon;

mod fractal;
pub mod graphics;
mod mandelbrot;

type RenderType = u32;
type JNICallback = extern "C" fn(*mut c_void, data: jlong);

extern "C" {
    fn send_jni_callback_message(cb: JNICallback, data: jlong);
}

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
    vm: &jni::JavaVM,
    render_type: RenderType,
    image_view_ref: GlobalRef,
) -> Result<&'a str> {
    // Attach current thread
    let env = vm.attach_current_thread()?;

    // Get raw bitmap object
    let image_view = image_view_ref.as_obj();

    let bmp = create_bitmap(&env, 800, 800)?.l()?.into_inner();
    let mut info = graphics::AndroidBitmapInfo::new();
    let raw_env = env.get_native_interface();

    // Read bitmap info
    graphics::bitmap_get_info(raw_env, bmp, &mut info);
    let mut pixels = 0 as *mut c_void;

    // Lock pixel for draw
    graphics::bitmap_lock_pixels(raw_env, bmp, &mut pixels);

    let pixels =
        std::slice::from_raw_parts_mut(pixels as *mut u8, (info.stride * info.height) as usize);

    match render_type {
        0x000001 => mandelbrot::render(pixels, info.width as u32, info.height as u32),
        0x000002 => fractal::render(pixels, info.width as u32, info.height as u32),
        _ => {}
    };

    graphics::bitmap_unlock_pixels(raw_env, bmp);

    env.call_method(
        image_view,
        "setImageBitmap",
        "(Landroid/graphics/Bitmap;)V",
        &[JValue::from(JObject::from(bmp))],
    )?;

    Ok("Render successfully")
}

struct JNICallbackData<'a> {
    callback: GlobalRef,
    msg: &'a str,
}

impl JNICallbackData<'_> {
    pub fn dumps(callback: GlobalRef, msg: &str) -> jlong {
        Box::into_raw(Box::new(JNICallbackData { callback, msg })) as jlong
    }
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
    let callback = env.new_global_ref(callback).unwrap();
    let image_view_ref = env.new_global_ref(image_view).unwrap();

    thread::spawn(move || {
        let env = vm.attach_current_thread().unwrap();
        let msg = match blend_bitmap(&vm, render_type, image_view_ref) {
            Ok(msg) => msg,
            _ => "Failed to render!",
        };

        let jni_data = JNICallbackData::dumps(callback, msg);
        let mut callback = move |data: jlong| {
            let env = attach_current_thread_as_daemon();
            let data = Box::from_raw(data as *mut JNICallbackData);

            unwrap(
                &env,
                env.call_method(
                    (*data).callback.as_obj(),
                    "invoke",
                    "(Ljava/lang/String;)V",
                    &[JValue::from(JObject::from(
                        env.new_string(data.msg).unwrap(),
                    ))],
                ),
            );
        };

        send_jni_callback_message(unpack_closure(&mut callback), jni_data);
    });
}

unsafe fn unpack_closure<F>(closure: &mut F) -> JNICallback
where
    F: FnMut(jlong),
{
    extern "C" fn trampoline<F>(ptr: *mut c_void, data: jlong)
    where
        F: FnMut(jlong),
    {
        let closure: &mut F = unsafe { &mut *(ptr as *mut F) };
        (*closure)(data);
    }

    trampoline::<F>
}
