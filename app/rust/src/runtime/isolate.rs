use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::stream::{FuturesUnordered, Stream};
use futures::Async::*;
use futures::{task, Future, Poll};

use crate::runtime::stream_cancel::TimerCancel;
use crate::runtime::timer::set_timeout;
use crate::runtime::{eval_script, string_to_ptr, DenoC, OpAsyncFuture};

#[allow(non_camel_case_types)]
type deno_recv_cb = unsafe extern "C" fn(
    isolate: *mut Isolate,
    d: *const DenoC,
    timer_id: u32,
    duration: u32,
) -> u32;

extern "C" {
    fn set_deno_data(raw: *const DenoC, data: *mut Isolate) -> *mut Isolate;
    fn deno_init(recv_cb: deno_recv_cb) -> *const DenoC;
    fn fire_callback(raw: *const DenoC, timer_id: u32);
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
                let timerMap = new Map();
                let nextTimerId = 1;

                function setTimer(cb, delay, repeat, ...args) {
                    // const callback = () => args.length === 0 ? cb : cb.bind(null, ...args);
                    const timer = {
                        id: nextTimerId++,
                        callback: cb,
                        repeat,
                        delay
                    };
                    timerMap.set(timer.id, timer);
                    $newTimer(timer.id, timer.delay);
                    return timer.id;
                }

                function setTimeout(callback, delay, ...args) {
                    return setTimer(callback, delay, false, ...args);
                }

                function setInterval(callback, delay, ...args) {
                    return setTimer(callback, delay, true, ...args);
                }

                function clearInterval(timerId) {
                    if (timerMap.has(timerId)) {
                        timerMap.delete(timerId);
                    }
                }

                function fire(timerId) {
                    if (!timerMap.has(timerId)) return;
                    Promise.resolve(timerId)
                        .then(function(id) {
                            if (timerMap.has(id)) {
                                const timer = timerMap.get(id);
                                const callback = timer.callback;
                                callback();
                                if (!timer.repeat) {
                                    timeMap.delete(timer.id);
                                    return;
                                }
                                // schedule interval
                                $newTimer(timer.id, timer.delay);
                            }
                        });
                }

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
        "#,
            ),
        );

        eval_script(self.deno, string_to_ptr(script));
    }

    unsafe extern "C" fn new_timer(
        ptr: *mut Isolate,
        d: *const DenoC,
        timer_id: u32,
        delay: u32,
    ) -> u32 {
        // let uid = Isolate::next_uuid();
        let isolate = Isolate::from_c(ptr);
        let (task, trigger) = set_timeout(delay);

        isolate.pending_ops.push(Box::new(task.and_then(move |_| {
            fire_callback(d, timer_id);
            Ok(vec![1u8].into_boxed_slice())
        })));
        isolate.have_unpolled_ops = true;

        return timer_id;
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
