use std::sync::{Arc, Mutex};

use futures::{Async, Future};
use jni::JNIEnv;
use jni::objects::JObject;
use libc::c_char;
use tokio::runtime;

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
    fn eval_script(d: *const DenoC, name: *const c_char, script: *const c_char);
    fn lookup_and_eval_script(uuid: u32, script: *const c_char);
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
pub extern "C" fn Java_com_node_sample_MainActivity_invokeScript(_env: JNIEnv, _class: JObject) {
    unsafe {
        lookup_and_eval_script(
            0u32,
            c_str!(
                r#"
                clearTimer(i2s);
                setInterval(() => {
                    console.log(`3s interval`);
                }, 3000);
                "#
            ),
        )
    };
}
