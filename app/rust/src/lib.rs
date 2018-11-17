extern crate ws;
extern crate jni;
extern crate libc;
extern crate image;
extern crate imageproc;

#[macro_use]
extern crate itertools;

use std::cmp;
use std::thread;
use std::sync::mpsc;
use std::os::raw::{c_int, c_void, c_uint};
use std::time::Duration;
use itertools::Itertools;

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JValue};
use jni::sys::{jint, jobject};

#[repr(C)]
#[derive(Debug, Default)]
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
    pub fn AndroidBitmap_getInfo(env: *mut jni::sys::JNIEnv, jbitmap: jobject, info: *mut AndroidBitmapInfo) -> c_int;
    pub fn AndroidBitmap_lockPixels(env: *mut jni::sys::JNIEnv, jbitmap: jobject, addrPtr: *mut *mut c_void) -> c_int;
    pub fn AndroidBitmap_unlockPixels(env: *mut jni::sys::JNIEnv, jbitmap: jobject) -> c_int;
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
pub unsafe extern "system" fn Java_com_node_sample_GenerateImageActivity_blendBitmap<'b>(
    env: JNIEnv<'b>, _class: JClass, bitmap: JObject, pixel_size: f64, x0: f64, y0: f64, callback: JObject,
) {
    let jvm = env.get_java_vm().unwrap();
    let bitmap = env.new_global_ref(bitmap).unwrap();

    let _callback = env.new_global_ref(callback).unwrap();

    thread::spawn(move || {
        let env = jvm.attach_current_thread().unwrap();
        let jenv = env.get_native_interface();
        let bitmap = bitmap.as_obj().into_inner();
        let callback = _callback.as_obj();

        let mut info = AndroidBitmapInfo { ..Default::default() };
        AndroidBitmap_getInfo(jenv, bitmap, &mut info);

        let mut pixels = 0 as *mut c_void;
        AndroidBitmap_lockPixels(jenv, bitmap, &mut pixels);

        let pixels = ::std::slice::from_raw_parts_mut(
            pixels as *mut u8, (info.stride * info.height) as usize);

        draw_mandelbrot(pixels, info.width as i64, info.height as i64, pixel_size, x0, y0);
        AndroidBitmap_unlockPixels(jenv, bitmap);

        env.call_method(callback, "subscribe", "()V", &[]).unwrap();
    });
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
    thread::spawn(move || {
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
                        out.close(ws::CloseCode::Normal).unwrap();
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
