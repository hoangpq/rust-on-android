extern crate jni;

extern crate libc;
extern crate image;
extern crate num_complex;

use num_complex::Complex;
use std::ffi::{CString, CStr};

use std::thread;
use std::time::Duration;
use std::sync::mpsc;

use std::{slice, mem};
use libc::c_void;

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString};
use jni::sys::jint;


#[repr(C)]
pub struct Array {
    data: *const c_void,
    len: libc::size_t,
}

impl Array {
    unsafe fn as_u32_slice(&self) -> &[u32] {
        assert!(!self.data.is_null());
        slice::from_raw_parts(self.data as *const u32, self.len as usize)
    }

    fn from_vec<T>(mut vec: Vec<T>) -> Array {
        vec.shrink_to_fit();
        let array = Array {
            data: vec.as_ptr() as *const libc::c_void,
            len: vec.len() as libc::size_t,
        };
        mem::forget(vec);
        array
    }
}

#[no_mangle]
pub extern "C" fn init_module() {}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_com_node_sample_MainActivity_asyncComputation(
    env: JNIEnv,
    _class: JClass,
    callback: JObject,
) {
    // `JNIEnv` cannot be sent across thread boundaries. To be able to use JNI
    // functions in other threads, we must first obtain the `JavaVM` interface
    // which, unlike `JNIEnv` is `Send`.
    let jvm = env.get_java_vm().unwrap();

    // We need to obtain global reference to the `callback` object before sending
    // it to the thread, to prevent it from being collected by the GC.
    let callback = env.new_global_ref(callback).unwrap();

    // Use channel to prevent the Java program to finish before the thread
    // has chance to start.
    let (tx, rx) = mpsc::channel();

    let _ = thread::spawn(move || {
        // Signal that the thread has started.
        tx.send(()).unwrap();

        // Use the `JavaVM` interface to attach a `JNIEnv` to the current thread.
        let env = jvm.attach_current_thread().unwrap();

        // Then use the `callback` with this newly obtained `JNIEnv`.
        let callback = callback.as_obj();

        for i in 0..100 {
            let progress = i as jint;
            // Now we can use all available `JNIEnv` functionality normally.
            env.call_method(callback, "asyncCallback", "(I)V", &[progress.into()])
                .unwrap();
            thread::sleep(Duration::from_millis(1000));
        }

        // The current thread is detached automatically when `env` goes out of scope.
    });

    // Wait until the thread has started.
    rx.recv().unwrap();
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_com_node_sample_GenerateImageActivity_generateJuliaFractal(
    env: JNIEnv,
    _class: JClass,
    path: JString,
    callback: JObject,
) {
    let jvm = env.get_java_vm().unwrap();
    let callback = env.new_global_ref(callback).unwrap();

    let pattern = env.get_string(path).expect("invalid pattern string").as_ptr();
    let c_str = unsafe { CStr::from_ptr(pattern) };
    let raw_path = c_str.to_str().unwrap();

    let handle = thread::spawn(move || {
        let max_iterations = 256u16;

        let imgx = 800;
        let imgy = 800;

        let scalex = 4.0 / imgx as f32;
        let scaley = 4.0 / imgy as f32;

        // Create a new ImgBuf with width: imgx and height: imgy
        let mut imgbuf = image::GrayImage::new(imgx, imgy);

        // Iterate over the coordinates and pixels of the image
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let cy = y as f32 * scaley - 2.0;
            let cx = x as f32 * scalex - 2.0;

            let mut z = Complex::new(cx, cy);
            let c = Complex::new(-0.4, 0.6);

            let mut i = 0;

            for t in 0..max_iterations {
                if z.norm() > 2.0 {
                    break;
                }
                z = z * z + c;
                i = t;
            }
            // Create an 8bit pixel of type Luma and value i
            // and assign in to the pixel at position (x, y)
            *pixel = image::Luma([i as u8]);
        }

        // let v = imgbuf.into_raw();
        // copy(v.as_ptr(), pixels as *mut u8, v.len());
        imgbuf.save(raw_path).unwrap();

        // Use the `JavaVM` interface to attach a `JNIEnv` to the current thread.
        let env = jvm.attach_current_thread().unwrap();
        let callback = callback.as_obj();

        env.call_method(callback, "subscribe", "()V",
                        &[]).unwrap();
    });

    handle.join().unwrap();
}
