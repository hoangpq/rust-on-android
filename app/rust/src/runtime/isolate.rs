use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::{Future, Poll, task};
use futures::Async::*;
use futures::stream::{FuturesUnordered, Stream};

use crate::runtime::{DenoC, eval_script, string_to_ptr};
use crate::runtime::stream_cancel::TimerCancel;
use crate::runtime::timer::{set_interval, set_timeout};

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
    fn set_deno_data(raw: *const DenoC, data: *mut Isolate) -> *mut Isolate;
    fn deno_init(recv_cb: deno_recv_cb) -> *const DenoC;
    fn invoke_function(raw: *const DenoC, f: *const libc::c_void, timer_id: u32);
}

pub struct Isolate {
    pub deno: *const DenoC,
    pub have_unpolled_ops: bool,
    pub pending_ops: FuturesUnordered<OpAsyncFuture>,
    pub timers: HashMap<u32, TimerCancel>,
}

unsafe impl Send for Isolate {}
unsafe impl Sync for Isolate {}

lazy_static! {
    static ref NEXT_RID: AtomicUsize = AtomicUsize::new(0);
}

impl Isolate {
    #[inline]
    fn next_uuid() -> u32 {
        let rid = NEXT_RID.fetch_add(1, Ordering::SeqCst);
        rid as u32
    }
    #[inline]
    pub fn from_c<'a>(ptr: *mut Isolate) -> &'a mut Isolate {
        let isolate_box = unsafe { Box::from_raw(ptr) };
        Box::leak(isolate_box)
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

    pub unsafe fn execute(&mut self, script: &str) {
        eval_script(
            self.deno,
            string_to_ptr(
                r#"
                const promiseTable = new Map();
                let nextPromiseId = 1;

                function createResolvable() {
                    let methods;
                    const promise = new Promise((resolve, reject) => {
                        methods = { resolve, reject };
                    });
                    return Object.assign(promise, methods);
                }

                function resolvePromise(promiseId, value) {
                    if (promiseTable.has(promiseId)) {
                        try {
                            let promise = promiseTable.get(promiseId);
                            promise.resolve(value);
                            promiseTable.delete(promiseId);
                        } catch (e) {
                            $log(e.message);
                        }
                    }
                }

                function fetch(url) {
                    const cmdId = nextPromiseId++;
                    const promise = createResolvable();
                    promiseTable.set(cmdId, promise);
                    $fetch(url, cmdId);
                    return promise;
                }
        "#,
            ),
        );

        eval_script(self.deno, string_to_ptr(script));
    }

    unsafe extern "C" fn new_timer(
        ptr: *mut Isolate,
        d: *const DenoC,
        cb: *mut libc::c_void,
        duration: u32,
        interval: bool,
    ) -> u32 {
        let uid = Isolate::next_uuid();
        let isolate = Isolate::from_c(ptr);
        if interval {
            let (task, trigger) = set_interval(
                move || {
                    invoke_function(d, cb, 0);
                },
                duration,
            );
            isolate.timers.insert(uid, trigger);
            isolate.pending_ops.push(Box::new(task));
        } else {
            let (task, trigger) = set_timeout(
                move || {
                    invoke_function(d, cb, uid);
                },
                duration,
            );
            isolate.timers.insert(uid, trigger);
            isolate.pending_ops.push(Box::new(task));
        };
        isolate.have_unpolled_ops = true;
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
                Ok(Ready(Some(buf))) => {}
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
    let isolate = Isolate::from_c(ptr);
    let _ = isolate.timers.remove(&timer_id);
    adb_debug!(format!("Timer {} removed", timer_id));
}
