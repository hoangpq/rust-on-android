extern crate jni;
extern crate libc;

use jni::JNIEnv;
use jni::objects::JValue;
use jni::sys::{jint, jobject};

use std::os::raw::{c_int, c_void, c_uint};

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
