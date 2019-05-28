use runtime::isolate::Isolate;
use std::ffi::CString;
use std::slice;
use tokio::runtime;

pub mod isolate;
pub mod util;

#[allow(non_snake_case)]
extern "C" {
    fn initIsolate() -> *mut libc::c_void;
    fn evalScriptVoid(deno: *const libc::c_void, script: *const libc::c_char);
    fn evalScript(deno: *const libc::c_void, script: *const libc::c_char) -> *mut libc::c_char;
    fn invokeFunction(deno: *const libc::c_void, f: *const libc::c_void);
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
