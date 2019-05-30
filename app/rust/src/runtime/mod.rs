pub mod fetch;
pub mod isolate;
pub mod stream_cancel;
pub mod timer;
pub mod util;

use std::ffi::CString;
use std::slice;
use tokio::runtime;

#[repr(C)]
pub struct DenoC {
    _unused: [u8; 0],
}

unsafe impl Send for DenoC {}
unsafe impl Sync for DenoC {}

#[allow(non_snake_case)]
extern "C" {
    fn eval_script_void(deno: *const DenoC, script: *const libc::c_char);
    fn eval_script(deno: *const DenoC, script: *const libc::c_char) -> *mut libc::c_char;
}

unsafe fn ptr_to_string(raw: *mut libc::c_char) -> Option<String> {
    Some(
        std::str::from_utf8_unchecked(slice::from_raw_parts(raw as *const u8, libc::strlen(raw)))
            .to_string(),
    )
}

unsafe fn string_to_ptr(s: &str) -> *const libc::c_char {
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
