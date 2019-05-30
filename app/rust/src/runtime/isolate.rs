use futures::stream::{FuturesUnordered, Stream};
use futures::Async::*;
use futures::{task, Future, Poll};
use std::collections::HashMap;

use crate::runtime::fetch::fetch_async;
use crate::runtime::timer::{set_interval, set_timeout};
use crate::runtime::{eval_script, eval_script_void, ptr_to_string, string_to_ptr, DenoC};

pub type OpAsyncFuture = Box<Future<Item = (), Error = ()>>;

#[allow(non_camel_case_types)]
type deno_recv_cb = unsafe extern "C" fn(
    isolate: *mut Isolate,
    d: *const DenoC,
    cb: *mut libc::c_void,
    duration: u32,
    interval: bool,
) -> u32;

extern "C" {
    fn set_deno_callback(raw: *const DenoC, recv_cb: deno_recv_cb);
    fn set_deno_data(raw: *const DenoC, data: *mut Isolate) -> *mut Isolate;
    fn lock_deno_isolate(raw: *const libc::c_void);
    fn deno_init(recv_cb: deno_recv_cb) -> *const DenoC;
    fn invoke_function(raw: *const DenoC, f: *const libc::c_void);
}

pub struct Isolate {
    deno: *const DenoC,
    pub have_unpolled_ops: bool,
    pub pending_ops: FuturesUnordered<OpAsyncFuture>,
    pub timers: HashMap<u32, futures::sync::oneshot::Sender<()>>,
}

unsafe impl Send for Isolate {}
unsafe impl Sync for Isolate {}

fn from_c<'a>(ptr: *mut Isolate) -> &'a mut Isolate {
    let isolate_box = unsafe { Box::from_raw(ptr) };
    Box::leak(isolate_box)
}

static mut UUID: u32 = 0;

impl Isolate {
    #[inline]
    fn next_uuid() -> u32 {
        unsafe {
            UUID = UUID + 1;
            UUID
        }
    }
    pub unsafe fn new<'a>() -> &'a mut Self {
        let deno = deno_init(Self::new_timer);
        let isolate_box = Box::new(Self {
            deno,
            have_unpolled_ops: false,
            pending_ops: FuturesUnordered::new(),
            timers: HashMap::new(),
        });
        let isolate_ptr: &'a mut Isolate = Box::leak(isolate_box);
        set_deno_data(deno, isolate_ptr);
        isolate_ptr
    }

    pub unsafe fn vexecute(&mut self, script: &str) {
        self.pending_ops.push(fetch_async());
        eval_script_void(self.deno, string_to_ptr(script));
    }

    pub unsafe fn execute(&self, script: &str) -> Option<String> {
        return ptr_to_string(eval_script(self.deno, string_to_ptr(script)));
    }

    unsafe extern "C" fn new_timer(
        ptr: *mut Isolate,
        d: *const DenoC,
        cb: *mut libc::c_void,
        duration: u32,
        interval: bool,
    ) -> u32 {
        let uid = Isolate::next_uuid();
        let isolate = from_c(ptr);
        if interval {
            let (task, trigger) = set_interval(
                move || {
                    invoke_function(d, cb);
                },
                duration,
            );
            isolate.timers.insert(uid, trigger);
            isolate.pending_ops.push(Box::new(task));
        } else {
            let (task, trigger) = set_timeout(
                move || {
                    invoke_function(d, cb);
                },
                duration,
            );
            isolate.timers.insert(uid, trigger);
            isolate.pending_ops.push(Box::new(task));
        };
        return uid;
    }
}

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
                Ok(Ready(Some(_buf))) => {
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

#[no_mangle]
fn remove_timer(ptr: *mut Isolate, timer_id: u32) {
    let isolate = from_c(ptr);
    if let Some(timer) = isolate.timers.remove(&timer_id) {
        timer.send(()).unwrap();
        adb_debug!(format!("Timer {} removed", timer_id));
    }
}
