extern crate ws;
extern crate jni;
extern crate libc;
extern crate image;
extern crate imageproc;
extern crate num_complex;

#[macro_use]
extern crate itertools;

use std::cmp;
use std::thread;
use std::ffi::CStr;
use std::sync::mpsc;
use std::os::raw::{c_int, c_void, c_uint};
use std::time::Duration;
use num_complex::Complex;
use itertools::Itertools;

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jint, jlong, jobject, jstring, jbyteArray};

#[repr(C)]
#[derive(Debug)]
pub struct AndroidBitmapInfo {
    pub width: c_uint,
    pub height: c_uint,
    pub stride: c_uint,
    pub format: c_int,
    pub flags: c_uint, // 0 for now
}

pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
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
    let jvm = env.get_java_vm().unwrap();
    let callback = env.new_global_ref(callback).unwrap();
    let (tx, rx) = mpsc::channel();
    let _ = thread::spawn(move || {
        tx.send(()).unwrap();
        let env = jvm.attach_current_thread().unwrap();
        let callback = callback.as_obj();
        for i in 0..100 {
            let progress = i as jint;
            env.call_method(
                callback, "subscribe",
                "(I)V", &[progress.into()])
                .unwrap();
            thread::sleep(Duration::from_millis(1000));
        }
    });
    rx.recv().unwrap();
}


#[allow(non_snake_case)]
extern "C" {
    pub fn AndroidBitmap_getInfo(env: *mut JNIEnv, jbitmap: jobject, info: *mut AndroidBitmapInfo) -> c_int;
    pub fn AndroidBitmap_lockPixels(env: *mut JNIEnv, jbitmap: jobject, addrPtr: *mut *mut c_void) -> c_int;
    pub fn AndroidBitmap_unlockPixels(env: *mut JNIEnv, jbitmap: jobject) -> c_int;
}

fn generate_palette() -> Vec<Color> {
    let mut palette: Vec<Color> = vec![];
    let mut roffset = 24;
    let mut goffset = 16;
    let mut boffset = 0;
    for i in 0..256 {
        palette.push(Color { red: roffset, green: goffset, blue: boffset });
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
    buffer: &mut [u8], width: i64, height: i64,
    pixel_size: f64, x0: f64, y0: f64,
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
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_com_node_sample_GenerateImageActivity_blendBitmap(
    env: *mut JNIEnv,
    _class: JClass,
    bitmap: jobject,
) {
    let mut info = AndroidBitmapInfo {
        width: 0,
        height: 0,
        stride: 0,
        format: 0,
        flags: 0,
    };
    let ret = unsafe {
        AndroidBitmap_getInfo(env, bitmap, &mut info)
    };


    let mut pixels = 0 as *mut c_void;
    let ret = unsafe {
        AndroidBitmap_lockPixels(env, bitmap, &mut pixels)
    };

    let mut pixels = unsafe {
        ::std::slice::from_raw_parts_mut(
            pixels as *mut u8,
            (info.stride * info.height) as usize,
        )
    };

    draw_mandelbrot(pixels, info.width as i64,
                    info.height as i64,
                    0.005, -2.0, -1.5);

    /*for pixel in pixels.chunks_mut(4) {
        let threshold = 127.5;
        let (r, g, b) = (pixel[0], pixel[1], pixel[2]);
        if 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32 >= threshold {
            pixel[0] = 255;
            pixel[1] = 255;
            pixel[2] = 255;
        } else {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
        }
    }*/

    unsafe {
        AndroidBitmap_unlockPixels(env, bitmap);
    }
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
            *pixel = image::Luma([i as u8]);
        }
        imgbuf.save(raw_path).unwrap();
        let env = jvm.attach_current_thread().unwrap();
        let callback = callback.as_obj();
        env.call_method(callback, "subscribe", "()V",
                        &[]).unwrap();
    });
    handle.join().unwrap();
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_com_node_sample_MainActivity_connectWS(
    env: JNIEnv,
    _class: JClass,
    callback: JObject,
) {
    let jvm = env.get_java_vm().unwrap();
    let callback = env.new_global_ref(callback).unwrap();
    let t = thread::spawn(move || {
        let env = jvm.attach_current_thread().unwrap();
        let callback = callback.as_obj();
        ws::connect("ws://echo.websocket.org", |out| {
            println!("Connected...");
            let watcher = |msg: ws::Message| {
                if msg.as_text().unwrap() == "Stopped!" {
                    env.call_method(callback, "subscribe", "()V", &[]).unwrap();
                }
                let jmsg: JObject = JObject::from(env.new_string(msg.as_text().unwrap()).unwrap());
                env.call_method(callback, "subscribe", "(Ljava/lang/String;)V",
                                &[JValue::from(jmsg)]).unwrap();
                Ok(())
            };

            thread::spawn(move || {
                let mut i = 1;
                loop {
                    if i > 10 {
                        out.send("Stopped!").unwrap();
                        out.close(ws::CloseCode::Normal);
                        return;
                    }
                    let formatted_msg = format!("Send message to WebSocket {} times", i);
                    out.send(formatted_msg).unwrap();
                    thread::sleep(Duration::from_millis(3000));
                    i += 1;
                }
            });
            watcher
        }).unwrap()
    });
}
