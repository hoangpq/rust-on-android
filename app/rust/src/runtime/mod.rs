use std::ffi::CString;
use std::slice;

use futures::{Async, Future};
use jni::objects::JObject;
use jni::JNIEnv;
use libc::c_char;
use std::sync::{Arc, Mutex};
use tokio::runtime;

pub mod console;
pub mod fetch;
pub mod isolate;
pub mod stream_cancel;
pub mod timer;
pub mod util;

#[repr(C)]
pub struct DenoC {
    _unused: [u8; 0],
}

#[allow(non_snake_case)]
extern "C" {
    fn eval_script(deno: *const DenoC, script: *const c_char);
    fn lookup_deno_and_eval_script(uuid: u32, script: *const c_char);
}

pub unsafe fn ptr_to_string(raw: *mut c_char) -> Option<String> {
    Some(
        std::str::from_utf8_unchecked(slice::from_raw_parts(raw as *const u8, libc::strlen(raw)))
            .to_string(),
    )
}

unsafe fn string_to_ptr<T>(s: T) -> *const c_char
where
    T: std::convert::Into<std::vec::Vec<u8>>,
{
    let s = CString::new(s).unwrap();
    let p = s.as_ptr();
    std::mem::forget(s);
    p
}

fn create_thread_pool_runtime() -> tokio::runtime::Runtime {
    use tokio_threadpool::Builder as ThreadPoolBuilder;
    let mut thread_pool_builder = ThreadPoolBuilder::new();
    thread_pool_builder.panic_handler(|err| std::panic::resume_unwind(err));
    #[allow(deprecated)]
    runtime::Builder::new()
        .threadpool_builder(thread_pool_builder)
        .build()
        .unwrap()
}

pub type Buf = Box<[u8]>;
pub type OpAsyncFuture = Box<dyn Future<Item = Buf, Error = ()> + Send>;

#[derive(Clone)]
pub struct Worker {
    inner: Arc<Mutex<isolate::Isolate>>,
}

impl Worker {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(isolate::Isolate::new())),
        }
    }

    fn execute(&mut self, script: &str) {
        let mut isolate = self.inner.lock().unwrap();
        isolate.execute(script);
    }
}

impl Future for Worker {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let mut isolate = self.inner.lock().unwrap();
        isolate.poll().map_err(|err| adb_debug!(err))
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_com_node_sample_MainActivity_invokeScript(env: JNIEnv, _class: JObject) {
    unsafe {
        lookup_deno_and_eval_script(
            0u32,
            string_to_ptr(
                r#"
                clearTimer(i2s);
                setInterval(() => {
                    console.log(`3s interval`);
                }, 3000);

                console.log(promiseTable.size);
                "#,
            ),
        )
    };
}
