use futures::stream::Stream;
use futures::{future, Future, IntoFuture};
use jni::JNIEnv;
use std::time::{Duration, Instant};
use tokio::timer::Interval;

use runtime::isolate::Isolate;
use runtime::{create_thread_pool_runtime, initDeno, invokeFunction, ptr_to_string};
use tokio_timer::Delay;

#[no_mangle]
pub unsafe extern "C" fn adb_debug(p: *mut libc::c_char) {
    if let Some(msg) = ptr_to_string(p) {
        adb_debug!(msg);
    }
}

#[no_mangle]
pub unsafe fn create_isolate(ptr: *const libc::c_void) -> *mut Isolate {
    Box::into_raw(Box::new(Isolate::new(ptr)))
}

#[no_mangle]
pub unsafe extern "C" fn create_timer(isolate: *mut Isolate, f: *const libc::c_void) {
    (*isolate).have_unpolled_ops = false;
}

pub unsafe fn init_event_loop(_env: &'static JNIEnv, d: *const libc::c_void) {
    let mut isolate = *Box::from_raw(initDeno(d));
    let main_future = futures::lazy(move || {
        isolate.execute(
            r#"
               $timeout((msg) => {
                  try {
                    $log(msg);
                  } catch (e) {
                    $log(e.message);
                  }
               });
               const data = { msg: 'Hello, World!'};
               $log(data.msg);
            "#,
        );
        isolate
    });

    let mut rt = create_thread_pool_runtime();

    let when = Instant::now() + Duration::from_millis(100);
    let task = Delay::new(when)
        .and_then(|_| {
            adb_debug!("Hello world!");
            Ok(())
        })
        .map_err(|e| panic!("delay error; err={:?}", e));

    rt.spawn(task);
    rt.block_on_all(main_future).unwrap();
}
