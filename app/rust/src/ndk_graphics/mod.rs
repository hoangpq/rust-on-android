#![allow(non_snake_case)]
extern crate jni;

use std::os::raw::{c_uint, c_void};
use std::thread;

use jni::errors::Result;
use jni::objects::{JObject, JValue};
use jni::sys::{jint, jlong};
use jni::{JNIEnv, JavaVM};

use crate::dex;
use crate::dex::unwrap;
use crate::runtime::ui_thread::DenoTask;
use crate::v8_jni;

mod fractal;
pub mod graphics;
mod mandelbrot;

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

unsafe fn blend_bitmap<'a>(render_type: RenderType, image_view: JObject) -> Result<&'a str> {
    let v = vec![1, 2, 3];

    // Attach current thread
    let env = v8_jni::jvm().attach_current_thread()?;

    let bmp = create_bitmap(&env, 800, 800)?.l()?;
    let mut info = graphics::AndroidBitmapInfo::new();

    // Read bitmap info
    graphics::bitmap_get_info(&env, bmp, &mut info);
    let mut pixels = 0 as *mut c_void;

    // Lock pixel for draw
    graphics::bitmap_lock_pixels(&env, bmp, &mut pixels);

    let pixels =
        std::slice::from_raw_parts_mut(pixels as *mut u8, (info.stride * info.height) as usize);

    match render_type {
        0x000001 => mandelbrot::render(pixels, info.width as u32, info.height as u32),
        0x000002 => fractal::render(pixels, info.width as u32, info.height as u32),
        _ => {}
    };

    graphics::bitmap_unlock_pixels(&env, bmp);

    env.call_method(
        image_view,
        "setImageBitmap",
        "(Landroid/graphics/Bitmap;)V",
        &[JValue::from(JObject::from(bmp))],
    )?;

    Ok("Render successfully")
}

#[no_mangle]
pub extern "C" fn Java_com_node_sample_GenerateImageActivity_blendBitmap(
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
        let msg = unsafe {
            match blend_bitmap(render_type, image_view_ref.as_obj()) {
                Ok(msg) => msg,
                _ => "Failed to render!",
            }
        };

        run_on_ui_thread(
            move |env: &JNIEnv| {
                unwrap(
                    &env,
                    env.call_method(
                        callback.as_obj(),
                        "invoke",
                        "(Ljava/lang/String;)V",
                        &[JValue::Object(JObject::from(env.new_string(msg).unwrap()))],
                    ),
                );
            },
            &vm,
        );
    });
}

fn run_on_ui_thread<F: 'static>(f: F, vm: &JavaVM)
where
    F: Fn(&JNIEnv),
{
    if let Ok(env) = vm.attach_current_thread_as_daemon() {
        let task = Box::into_raw(DenoTask::new(f)) as jlong;
        unwrap(
            &env,
            dex::call_static_method(
                &env,
                "com/node/util/JNIHelper",
                "runTask",
                "(J)V",
                &[JValue::Long(task)],
            ),
        );
    }
}
