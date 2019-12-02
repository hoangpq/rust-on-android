use std::os::raw::{c_int, c_uint, c_void};

use jni::objects::JObject;

#[repr(C)]
#[derive(Debug, Default)]
pub struct AndroidBitmapInfo {
    pub width: c_uint,
    pub height: c_uint,
    pub stride: c_uint,
    pub format: c_int,
    pub flags: c_uint,
}

impl AndroidBitmapInfo {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

mod graphic_sys {
    use std::os::raw::{c_int, c_void};

    use jni::sys::jobject;

    use crate::ndk_graphics::graphics::AndroidBitmapInfo;

    extern "C" {
        #[link_name = "AndroidBitmap_getInfo"]
        pub fn bitmap_get_info(
            env: *mut jni::sys::JNIEnv,
            bmp: jobject,
            info: *mut AndroidBitmapInfo,
        ) -> c_int;

        #[link_name = "AndroidBitmap_lockPixels"]
        pub fn bitmap_lock_pixels(
            env: *mut jni::sys::JNIEnv,
            bmp: jobject,
            pixels: *mut *mut c_void,
        ) -> c_int;

        #[link_name = "AndroidBitmap_unlockPixels"]
        pub fn bitmap_unlock_pixels(env: *mut jni::sys::JNIEnv, bmp: jobject) -> c_int;
    }
}

pub fn bitmap_get_info(env: &jni::JNIEnv, bmp: JObject, info: *mut AndroidBitmapInfo) {
    unsafe { graphic_sys::bitmap_get_info(env.get_native_interface(), bmp.into_inner(), info) };
}

pub fn bitmap_lock_pixels(env: &jni::JNIEnv, bmp: JObject, pixels: *mut *mut c_void) {
    unsafe {
        graphic_sys::bitmap_lock_pixels(env.get_native_interface(), bmp.into_inner(), pixels)
    };
}

pub fn bitmap_unlock_pixels(env: &jni::JNIEnv, bmp: JObject) -> c_int {
    unsafe { graphic_sys::bitmap_unlock_pixels(env.get_native_interface(), bmp.into_inner()) }
}
