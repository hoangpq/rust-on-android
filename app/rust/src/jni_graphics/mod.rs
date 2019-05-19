extern crate jni;
extern crate libc;

use jni::JNIEnv;
use jni::objects::{JValue, JClass};
use jni::sys::{jint, jobject, jstring};

use std::os::raw::{c_int, c_void, c_uint};
use std::cmp;
use std::ffi::CString;
use itertools::Itertools;

#[repr(C)]
#[derive(Debug, Default)]
pub struct AndroidBitmapInfo {
    pub width: c_uint,
    pub height: c_uint,
    pub stride: c_uint,
    pub format: c_int,
    pub flags: c_uint, // 0 for now
}

impl Drop for AndroidBitmapInfo {
    fn drop(&mut self) {
        adb_debug!(format!("Drop bitmap info {:?}", self));
    }
}

pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[allow(non_snake_case)]
extern "C" {
    pub fn AndroidBitmap_getInfo(
        env: *mut jni::sys::JNIEnv,
        jbitmap: jobject,
        info: *mut AndroidBitmapInfo,
    ) -> c_int;
    pub fn AndroidBitmap_lockPixels(
        env: *mut jni::sys::JNIEnv,
        jbitmap: jobject,
        addrPtr: *mut *mut c_void,
    ) -> c_int;
    pub fn AndroidBitmap_unlockPixels(env: *mut jni::sys::JNIEnv, jbitmap: jobject) -> c_int;
}

pub unsafe fn create_bitmap<'b>(env: &'b JNIEnv<'b>, width: c_uint, height: c_uint) -> JValue<'b> {
    let config = env.call_static_method(
        "android/graphics/Bitmap$Config",
        "nativeToConfig",
        "(I)Landroid/graphics/Bitmap$Config;",
        &[JValue::from(5)],
    ).unwrap();

    let jbitmap = env.call_static_method(
        "android/graphics/Bitmap",
        "createBitmap",
        "(IILandroid/graphics/Bitmap$Config;)Landroid/graphics/Bitmap;",
        &[
            JValue::from(width as jint),
            JValue::from(height as jint),
            config,
        ],
    ).unwrap();

    jbitmap
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
pub unsafe extern "C" fn Java_com_node_sample_MainActivity_getUtf8String(
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
