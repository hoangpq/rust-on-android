use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::stream::{FuturesUnordered, Stream};
use futures::Async::*;
use futures::{task, Future, Poll};

use crate::runtime::stream_cancel::TimerCancel;
use crate::runtime::timer::set_timeout;
use crate::runtime::{eval_script, string_to_ptr, DenoC, OpAsyncFuture};
use core::borrow::BorrowMut;
use libc::c_void;
use std::sync::{Once, ONCE_INIT};

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
    fn fire_callback(raw: *const DenoC, promise_id: u32);
    fn set_deno_data(deno: *const DenoC, user_data: *const libc::c_void);
    fn set_deno_resolver(deno: *const DenoC);
    fn stack_empty_check(deno: *const DenoC) -> bool;
    fn deno_lock(deno: *const DenoC);
    fn deno_unlock(deno: *const DenoC);
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

// Locker
struct LockerScope {
    deno: *const DenoC,
}

impl LockerScope {
    fn new(deno: *const DenoC) -> LockerScope {
        unsafe { deno_lock(deno) }
        LockerScope { deno }
    }
}

impl Drop for LockerScope {
    fn drop(&mut self) {
        unsafe { deno_unlock(self.deno) }
    }
}

fn next_uuid() -> u32 {
    let rid = NEXT_RID.fetch_add(1, Ordering::SeqCst);
    rid as u32
}

lazy_static! {
    static ref NEXT_RID: AtomicUsize = AtomicUsize::new(0);
}

static ISOLATE_INIT: Once = ONCE_INIT;

impl Isolate {
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

    pub unsafe fn initialize(&mut self) {
        set_deno_data(self.deno, self.as_raw_ptr());
        eval_script(
            self.deno,
            string_to_ptr(
                r#"
                const promiseTable = new Map();
                let nextPromiseId = 1;

                function isStackEmpty() {
                    return promiseTable.size === 0;
                }

                function createResolvable() {
                    let methods;
                    const cmdId = nextPromiseId++;
                    const promise = new Promise((resolve, reject) => {
                        methods = { resolve, reject, cmdId };
                    });
                    const promise_ = Object.assign(promise, methods);
                    promiseTable.set(cmdId, promise_);
                    return promise_;
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
                    const promise = createResolvable();
                    $fetch(url, promise.cmdId);
                    return promise.then(data => new Body(data));
                }

                let timerMap = new Map();
                let nextTimerId = 1;

                // timer implementation
                function setTimer(callback, delay, repeat, ...args) {
                    const timer = {
                        id: nextTimerId++,
                        callback,
                        repeat,
                        delay
                    };
                    timerMap.set(timer.id, timer);

                    const promise = createResolvable();
                    $newTimer(promise.cmdId, timer.id, timer.delay);

                    promise.then(() => {
                        Promise.resolve(timer.id).then(fire);
                        promiseTable.delete(promise.cmdId);
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
                    } else {
                        const promise = createResolvable();
                        $newTimer(promise.cmdId, timer.id, timer.delay, true);

                        promise.then(() => {
                            Promise.resolve(timer.id).then(fire);
                        });
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
        set_deno_resolver(self.deno);
    }

    pub unsafe fn execute(&mut self, script: &str) {
        ISOLATE_INIT.call_once(|| unsafe {
            self.initialize();
        });
        eval_script(self.deno, string_to_ptr(script));
    }

    #[inline]
    pub unsafe fn from_raw_ptr<'a>(ptr: *const c_void) -> &'a mut Self {
        let ptr = ptr as *mut _;
        &mut *ptr
    }

    #[inline]
    fn as_raw_ptr(&self) -> *const c_void {
        self as *const _ as *const c_void
    }

    extern "C" fn dispatch(
        data: *mut libc::c_void,
        deno: *const DenoC,
        promise_id: u32,
        timer_id: u32,
        delay: u32,
    ) {
        let isolate = unsafe { Isolate::from_raw_ptr(data) };
        let (task, trigger) = set_timeout(delay);
        let task = task.and_then(move |_| {
            unsafe { fire_callback(deno, promise_id) };
            Ok(vec![1u8].into_boxed_slice())
        });

        isolate.pending_ops.push(Box::new(task));
        isolate.have_unpolled_ops = true;
    }
}

impl Future for Isolate {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // Lock the current thread for V8.
        let _locker = LockerScope::new(self.deno);

        loop {
            self.have_unpolled_ops = false;
            #[allow(clippy::match_wild_err_arm)]
            match self.pending_ops.poll() {
                Err(_) => panic!("unexpected op error"),
                Ok(Ready(None)) => break,
                Ok(NotReady) => break,
                Ok(Ready(Some(_buf))) => {
                    // adb_debug!(format!("Buf: {:?}", buf));
                    break;
                }
            }
        }

        // We're idle if pending_ops is empty.
        if self.pending_ops.is_empty() {
            Ok(NotReady)
        } else {
            if self.have_unpolled_ops {
                task::current().notify();
            }
            Ok(NotReady)
        }
    }
}
