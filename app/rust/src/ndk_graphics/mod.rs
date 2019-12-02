#![allow(non_snake_case)]
extern crate jni;

use std::os::raw::{c_uint, c_void};
use std::ptr::null_mut;
use std::thread;

use jni::errors::Result;
use jni::objects::{GlobalRef, JObject, JValue};
use jni::sys::{jint, jlong};
use jni::JNIEnv;

use crate::dex;
use crate::dex::unwrap;
use crate::v8_jni;
use crate::v8_jni::attach_current_thread_as_daemon;

mod fractal;
pub mod graphics;
mod mandelbrot;

type RenderType = u32;
type JNIClosure = extern "C" fn(*mut c_void, callback: jlong, value: jlong);

extern "C" {
    fn send_message_to_looper(cb: JNIClosure, callback: jlong, data: jlong);
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

unsafe fn blend_bitmap<'a>(render_type: RenderType, image_view: JObject) -> Result<&'a str> {
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

#[repr(C)]
pub union InternalJval<'a> {
    pub s: &'a str,
    pub i: i32,
}

#[repr(C)]
pub struct JVal<'a> {
    pub t: i32,
    pub internal: InternalJval<'a>,
}

impl<'a> Drop for JVal<'a> {
    fn drop(&mut self) {
        adb_debug!(format!("JVal({:p}) dropped", &self));
    }
}

impl<'a> JVal<'a> {
    fn new_int(value: i32) -> Self {
        Self {
            internal: InternalJval { i: value },
            t: 0,
        }
    }
    fn new_str(value: &'a str) -> Self {
        Self {
            internal: InternalJval { s: value },
            t: 1,
        }
    }
    fn to_value(&self, env: &'a JNIEnv) -> JValue<'a> {
        unsafe {
            match self.t {
                0i32 => JValue::from(self.internal.i),
                1i32 => JValue::from(*env.new_string(self.internal.s).unwrap()),
                _ => JValue::from(JObject::null()),
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn Java_com_node_sample_GenerateImageActivity_blendBitmap(
    env: jni::JNIEnv,
    _class: JObject,
    image_view: JObject,
    render_type: u32,
    callback: JObject,
) {
    let callback = env.new_global_ref(callback).unwrap();
    let image_view_ref = env.new_global_ref(image_view).unwrap();

    thread::spawn(move || {
        let msg = unsafe {
            match blend_bitmap(render_type, image_view_ref.as_obj()) {
                Ok(msg) => msg,
                _ => "Failed to render!",
            }
        };

        let args = vec![JVal::new_str(msg)];

        unsafe {
            run_on_ui(
                &mut move |callback: jlong, data: jlong| {
                    let env = attach_current_thread_as_daemon();
                    let callback = Box::from_raw(callback as *mut GlobalRef);

                    let args: Vec<JValue> = {
                        let args = Box::from_raw(data as *mut Vec<JVal>);
                        args.iter().map(|item| item.to_value(&env)).collect()
                    };

                    unwrap(
                        &env,
                        env.call_method(
                            callback.as_obj(),
                            "invoke",
                            "(Ljava/lang/String;)V",
                            &args,
                        ),
                    );
                },
                callback,
                args,
            );
        }
    });
}

unsafe fn run_on_ui<F: FnMut(jlong, jlong)>(f: &mut F, callback: GlobalRef, args: Vec<JVal>) {
    send_message_to_looper(
        unpack_closure(f),
        Box::into_raw(Box::new(callback)) as jlong,
        Box::into_raw(Box::new(args)) as jlong,
    )
}

unsafe fn unpack_closure<F>(_closure: &mut F) -> JNIClosure
where
    F: FnMut(jlong, jlong),
{
    extern "C" fn trampoline<F>(ptr: *mut c_void, callback: jlong, data: jlong)
    where
        F: FnMut(jlong, jlong),
    {
        let closure: &mut F = unsafe { &mut *(ptr as *mut F) };
        (*closure)(callback, data);
    }

    trampoline::<F>
}
