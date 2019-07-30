#![allow(non_snake_case)]
extern crate jni;
extern crate libc;

use std::ffi::CString;
use std::sync::mpsc;
use std::{cmp, thread};

use itertools::Itertools;
use jni::errors::Result;
use jni::objects::{GlobalRef, JClass, JObject, JValue};
use jni::sys::{jint, jobject, jstring, JNIEnv};
use libc::{c_int, c_uint, c_void};

mod android;
use crate::dex;

pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub unsafe fn create_bitmap<'b>(
    env: &'b jni::JNIEnv<'b>,
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
pub unsafe extern "C" fn Java_com_node_sample_MainActivity_getUtf8String(
    env: jni::JNIEnv,
    _class: JClass,
) -> jstring {
    let ptr = CString::new(
        "ｴｴｯ?工ｴｴｪｪ(๑̀⚬♊⚬́๑)ｪｪｴｴ工‼!!!".to_owned(),
    )
    .unwrap();
    let output = env
        .new_string(ptr.to_str().unwrap())
        .expect("Couldn't create java string!");
    output.into_inner()
}

unsafe fn blend_bitmap<'a>(
    vm: jni::JavaVM,
    image_view_ref: GlobalRef,
    pixel_size: f64,
    x0: f64,
    y0: f64,
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

    draw_mandelbrot(
        pixels,
        info.width as i64,
        info.height as i64,
        pixel_size,
        x0,
        y0,
    );

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
    pixel_size: f64,
    x0: f64,
    y0: f64,
    callback: JObject,
) {
    let vm = env.get_java_vm().unwrap();
    let image_view_ref = env.new_global_ref(image_view).unwrap();
    let (tx, rx) = mpsc::sync_channel::<&str>(1);

    thread::spawn(move || {
        let msg = match blend_bitmap(vm, image_view_ref, pixel_size, x0, y0) {
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
