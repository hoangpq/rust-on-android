use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::stream::{FuturesUnordered, Stream};
use futures::Async::*;
use futures::{task, Future, Poll};

use crate::runtime::stream_cancel::TimerCancel;
use crate::runtime::timer::set_timeout;
use crate::runtime::{eval_script, string_to_ptr, DenoC, OpAsyncFuture};
use libc::c_void;

#[allow(non_camel_case_types)]
type deno_recv_cb = unsafe extern "C" fn(
    data: *mut libc::c_void,
    d: *const DenoC,
    promise_id: u32,
    timer_id: u32,
    duration: u32,
);

extern "C" {
    fn deno_init(recv_cb: deno_recv_cb) -> *const DenoC;
    fn fire_callback(raw: *const DenoC, promise_id: u32, timer_id: u32);
}

pub struct Isolate {
    uuid: u32,
    pub deno: *const DenoC,
    pub have_unpolled_ops: bool,
    pub pending_ops: FuturesUnordered<OpAsyncFuture>,
    pub timers: HashMap<u32, TimerCancel>,
}

unsafe impl Send for Isolate {}

impl Drop for Isolate {
    fn drop(&mut self) {
        adb_debug!(format!("{:p} dropped", self.as_raw_ptr()));;
    }
}

fn next_uuid() -> u32 {
    let rid = NEXT_RID.fetch_add(1, Ordering::SeqCst);
    rid as u32
}

lazy_static! {
    static ref NEXT_RID: AtomicUsize = AtomicUsize::new(0);
}

impl Isolate {
    #[inline]
    pub fn from_c<'a>(ptr: *mut Isolate) -> &'a mut Isolate {
        let isolate_box = unsafe { Box::from_raw(ptr) };
        Box::leak(isolate_box)
    }

    pub unsafe fn new() -> Self {
        let mut core_isolate = Self {
            deno: deno_init(Self::dispatch),
            uuid: next_uuid(),
            have_unpolled_ops: false,
            pending_ops: FuturesUnordered::new(),
            timers: HashMap::new(),
        };
        core_isolate
    }

    pub unsafe fn execute(&mut self, script: &str) {
        eval_script(
            self.deno,
            self.as_raw_ptr(),
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

                function resolve(promiseId, value) {
                    if (promiseTable.has(promiseId)) {
                        try {
                            let promise = promiseTable.get(promiseId);
                            promise.resolve(value);
                            promiseTable.delete(promiseId);
                        } catch (e) {
                            console.log(e.message);
                        }
                    }
                }

                class Body {
                    constructor(data) {
                        this._data = data;
                    }
                    text() {
                        return Promise.resolve(this._data);
                    }
                    json() {
                        try {
                            return Promise.resolve(this._data).then(JSON.parse);
                        } catch (e) {
                            throw new Error(`Can't not parse json data`);
                        }
                    }
                }

                function fetch(url) {
                    const cmdId = nextPromiseId++;
                    const promise = createResolvable();
                    promiseTable.set(cmdId, promise);
                    $fetch(url, cmdId);
                    return promise.then(data => new Body(data));
                }

                let timerMap = new Map();
                let nextTimerId = 1;

                // timer implementation
                function setTimer(callback, delay, repeat, ...args) {

                    console.log(`Create timer`);

                    const timer = {
                        id: nextTimerId++,
                        callback,
                        repeat,
                        delay
                    };
                    timerMap.set(timer.id, timer);

                    const cmdId = nextPromiseId++;
                    const promise = createResolvable();
                    promiseTable.set(cmdId, promise);
                    $newTimer(cmdId, timer.id, timer.delay);

                    promise.then(() => {
                        Promise.resolve(timer.id).then(fire);
                        promiseTable.delete(promiseId)
                    });

                    return timer.id;
                }

                function fire(id) {
                    const timer = timerMap.get(id);
                    const callback = timer.callback;
                    callback();

                    if (!timer.repeat) {
                        timeMap.delete(timer.id);
                        return;
                    }
                }

                function setTimeout(callback, delay) {
                    return setTimer(callback, delay, false);
                }

                function setInterval(callback, delay) {
                    return setTimer(callback, delay, true);
                }

        "#,
            ),
        );

        eval_script(self.deno, self.as_raw_ptr(), string_to_ptr(script));
    }

    #[inline]
    unsafe fn from_raw_ptr<'a>(ptr: *const c_void) -> &'a mut Self {
        let ptr = ptr as *mut _;
        &mut *ptr
    }

    #[inline]
    fn as_raw_ptr(&self) -> *const c_void {
        self as *const _ as *const c_void
    }

    unsafe extern "C" fn dispatch(
        data: *mut libc::c_void,
        deno: *const DenoC,
        promise_id: u32,
        timer_id: u32,
        delay: u32,
    ) {
        let isolate = unsafe { Isolate::from_raw_ptr(data) };
        let (task, trigger) = set_timeout(delay);
        isolate.pending_ops.push(Box::new(task.and_then(move |_| {
            fire_callback(deno, promise_id, timer_id);
            Ok(vec![1u8].into_boxed_slice())
        })));
        isolate.have_unpolled_ops = true;
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
                Ok(Ready(Some(buf))) => {
                    // adb_debug!(format!("Buf: {:?}", buf));
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
