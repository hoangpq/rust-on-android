use futures::stream::Stream;
use futures::{future, Async, Future, IntoFuture, Poll};
use jni::JNIEnv;
use std::time::{Duration, Instant};
use tokio::timer::Interval;

use core::borrow::BorrowMut;
use runtime::isolate::Isolate;
use runtime::{create_thread_pool_runtime, initIsolate, invokeFunction, ptr_to_string};
use std::fmt;
use tokio_timer::Delay;

#[no_mangle]
pub unsafe extern "C" fn adb_debug(p: *mut libc::c_char) {
    if let Some(msg) = ptr_to_string(p) {
        adb_debug!(msg);
    }
}

#[no_mangle]
pub unsafe fn create_isolate(raw: *const libc::c_void) -> *mut Isolate {
    Box::into_raw(Box::new(Isolate::new(raw)))
}

#[no_mangle]
pub unsafe extern "C" fn create_timer(isolate: &mut Isolate, f: *const libc::c_void) {
    let isolate = isolate.borrow_mut();
    /*let f = future::poll_fn(move || {
        Ok(Async::Ready("hello world".to_string()))
    });*/
    // isolate.tasks.push(Box::new(f));
}

pub unsafe fn init_event_loop(_env: &'static JNIEnv) {
    let main_future = futures::lazy(move || {
        let mut isolate = *Box::from_raw(initIsolate());
        isolate.vexecute(
            r#"
               $timeout((msg) => {});
               const data = { msg: 'Hello, World!'};
               $log(data.msg);
            "#,
        );
        isolate
    });

    let mut rt = create_thread_pool_runtime();
    rt.block_on_all(main_future).unwrap();
}
