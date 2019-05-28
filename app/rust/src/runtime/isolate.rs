use futures::stream::{FuturesUnordered, Stream};
use futures::sync::mpsc;
use futures::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::sync::oneshot;
use futures::{future, task, Async, Future, Poll};
use std::time::{Duration, Instant};

use futures::Async::*;

use core::borrow::BorrowMut;
use futures::future::IntoFuture;
use runtime::{
    create_thread_pool_runtime, evalScript, evalScriptVoid, ptr_to_string, string_to_ptr,
};
use std::sync::{Arc, Mutex};
use tokio_timer::Delay;

pub type OpAsyncFuture = Box<Future<Item = (), Error = ()> + Send>;

#[allow(non_camel_case_types)]
type deno_recv_cb = unsafe extern "C" fn(isolate: *mut Isolate, f: *mut libc::c_void, d: i32);

extern "C" {
    fn set_deno_callback(raw: *const libc::c_void, recv_cb: deno_recv_cb);
    fn set_deno_data(raw: *const libc::c_void, data: *mut Isolate) -> *mut Isolate;
    fn lock_deno_isolate(raw: *const libc::c_void) -> *const isolate;
    fn deno_init(recv_cb: deno_recv_cb) -> *const libc::c_void;
    fn invoke_function(raw: *const libc::c_void, f: *const libc::c_void);
}

pub fn panic_on_error<I, E, F>(f: F) -> impl Future<Item = I, Error = ()>
where
    F: Future<Item = I, Error = E>,
    E: std::fmt::Debug,
{
    f.map_err(|err| panic!("Future got unexpected error: {:?}", err))
}

fn create_future<T>(v: T) -> impl Future<Item = T, Error = ()>
where
    T: std::fmt::Debug,
{
    future::ok(v).and_then(|value: T| {
        adb_debug!(format!("result: {:?}", value));
        Ok(value)
    })
}

#[repr(C)]
pub struct isolate {
    _unused: [u8; 0],
}

unsafe impl Send for isolate {}
unsafe impl Sync for isolate {}

pub struct Isolate {
    deno: *const libc::c_void,
    pub have_unpolled_ops: bool,
    pub pending_ops: FuturesUnordered<OpAsyncFuture>,
}

impl Isolate {
    pub unsafe fn new<'a>() -> &'a mut Self {
        let deno = deno_init(Self::new_timer);
        let isolate_box = Box::new(Self {
            deno,
            have_unpolled_ops: false,
            pending_ops: FuturesUnordered::new(),
        });
        let isolate_ptr: &'a mut Isolate = Box::leak(isolate_box);
        set_deno_data(deno, isolate_ptr);
        isolate_ptr
    }

    pub unsafe fn vexecute(&mut self, script: &str) {
        evalScriptVoid(self.deno, string_to_ptr(script));
    }

    pub unsafe fn execute(&self, script: &str) -> Option<String> {
        return ptr_to_string(evalScript(self.deno, string_to_ptr(script)));
    }

    pub unsafe fn static_execute(deno: *const libc::c_void, script: &str) -> Option<String> {
        return ptr_to_string(evalScript(deno, string_to_ptr(script)));
    }

    extern "C" fn new_timer(ptr: *mut Isolate, cb: *mut libc::c_void, duration: i32) {
        let isolate_box = unsafe { Box::from_raw(ptr) };
        let isolate = Box::leak(isolate_box);

        let duration = Duration::from_millis(duration as u64);
        let delay_task = Delay::new(Instant::now() + duration)
            .and_then(move |_| {
                unsafe {
                    // share_deno_isolate.lock().unwrap();
                    // invoke_function(isolate.deno, cb)
                };
                adb_debug!(format!("$timeout {:?}s", duration.as_secs()));
                Ok(())
            })
            .map_err(|e| panic!("delay error; err={:?}", e));

        isolate.pending_ops.push(Box::new(delay_task));
    }
}

unsafe impl Send for Isolate {}
unsafe impl Sync for Isolate {}

impl Future for Isolate {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            self.have_unpolled_ops = false;
            #[allow(clippy::match_wild_err_arm)]
            match self.pending_ops.poll() {
                Err(_) => panic!("unexpected op error"),
                Ok(Ready(None)) => break,
                Ok(NotReady) => break,
                Ok(Ready(Some(buf))) => {
                    // adb_debug!(format!("buf: {:?}", buf));
                    break;
                }
            }
        }

        // We're idle if pending_ops is empty.
        if self.pending_ops.is_empty() {
            Ok(futures::Async::Ready(()))
        } else {
            if self.have_unpolled_ops {
                task::current().notify();
            }
            Ok(futures::Async::NotReady)
        }
    }
}
