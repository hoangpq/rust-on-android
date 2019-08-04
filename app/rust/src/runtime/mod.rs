use futures::Future;
use libc::c_char;
use tokio::runtime;

pub mod event_loop;
pub mod fetch;
pub mod isolate;
pub mod resource;
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
