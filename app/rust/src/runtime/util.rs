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

pub unsafe fn init_event_loop(_env: &'static JNIEnv) {
    let main_future = futures::lazy(move || {
        let mut isolate = Isolate::new();
        isolate.vexecute(
            r#"
               $timeout((msg) => {}, 6e3);
               const data = { msg: 'Hello, World!'};
               $timeout((msg) => {}, 9e3);
               $log(data.msg);
            "#,
        );
        isolate
    });

    let mut rt = create_thread_pool_runtime();
    rt.block_on_all(main_future).unwrap();
}
