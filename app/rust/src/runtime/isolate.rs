use std::{
    sync::atomic::{AtomicUsize, Ordering},
    sync::Once,
};

use futures::stream::{FuturesUnordered, Stream};
use futures::Async::*;
use futures::{task, Future, Poll};
use libc::c_void;

use crate::runtime::timer::set_timeout;
use crate::runtime::{eval_script, DenoC, OpAsyncFuture};

#[allow(non_camel_case_types)]
type deno_recv_cb = unsafe extern "C" fn(data: *mut libc::c_void, promise_id: u32, duration: u32);

extern "C" {
    fn deno_init(recv_cb: deno_recv_cb, uuid: u32) -> *const DenoC;
    fn fire_callback(raw: *const DenoC, promise_id: u32);
    fn set_deno_data(deno: *const DenoC, user_data: *const libc::c_void);
    fn set_deno_resolver(deno: *const DenoC);
    fn deno_lock(deno: *const DenoC);
    fn deno_unlock(deno: *const DenoC);
}

pub struct Isolate {
    uuid: u32,
    pub deno: *const DenoC,
    pub have_unpolled_ops: bool,
    pub pending_ops: FuturesUnordered<OpAsyncFuture>,
}

unsafe impl Send for Isolate {}

impl Drop for Isolate {
    fn drop(&mut self) {
        adb_debug!(format!("Isolate {:p} dropped", &self));
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

static ISOLATE_INIT: Once = Once::new();

impl Isolate {
    pub fn new() -> Self {
        let uuid = next_uuid();
        return Self {
            uuid,
            deno: unsafe { deno_init(Self::dispatch, uuid) },
            have_unpolled_ops: false,
            pending_ops: FuturesUnordered::new(),
        };
    }

    pub unsafe fn initialize(&mut self) {
        set_deno_data(self.deno, self.as_raw_ptr());
        eval_script(
            self.deno,
            c_str!("isolate.js"),
            c_str!(
                r#"
                function assert(cond, msg = 'assert') {
                    if (!cond) {
                        throw Error(msg);
                    }
                }

                const EPOCH = Date.now();
                const APOCALYPSE = 2 ** 32 - 2;

                // Timeout values > TIMEOUT_MAX are set to 1.
                const TIMEOUT_MAX = 2 ** 31 - 1;

                function getTime() {
                    // TODO: use a monotonic clock.
                    const now = Date.now() - EPOCH;
                    assert(now >= 0 && now < APOCALYPSE);
                    return now;
                }

                const promiseTable = new Map();
                let nextPromiseId = 1;

                function isStackEmpty() {
                  return false;
                }

                Promise.prototype.finally = function finallyPolyfill(callback) {
                  let constructor = this.constructor;

                  return this.then(function(value) {
                    return constructor.resolve(callback()).then(function() {
                      return value;
                    });
                  }, function(reason) {
                    return constructor.resolve(callback()).then(function() {
                      throw reason;
                    });
                  });
                };

                function createResolvable() {
                  let methods;
                  const cmdId = nextPromiseId++;
                  const promise = new Promise((resolve, reject) => {
                    methods = { resolve, reject, cmdId };
                  });
                  const promise_ = Object.assign(promise, methods);
                  promiseTable.set(cmdId, promise_);

                  // Remove the promise
                  promise.finally(() => {
                    promiseTable.delete(promise.cmdId);
                  });

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
                async function setTimer(timerId, callback, delay, repeat, ...args) {
                  const timer = {
                    id: timerId,
                    callback,
                    repeat,
                    delay
                  };

                  // console.log(getTime());

                  // Add promise to microtask queue
                  timerMap.set(timer.id, timer);
                  const promise = createResolvable();

                  // Send message to tokio backend
                  $newTimer(promise.cmdId, timer.delay);

                  // Wait util promise resolve
                  await promise;
                  Promise.resolve(timer.id).then(fire);
                }

                async function fire(id) {
                  if (!timerMap.has(id)) return;

                  const timer = timerMap.get(id);
                  const callback = timer.callback;
                  callback();

                  if (!timer.repeat) {
                    timeMap.delete(timer.id);
                    return;
                  }

                  // Add new timer (setInterval fake)
                  const promise = createResolvable();
                  $newTimer(promise.cmdId, timer.delay, true);

                  await promise;
                  Promise.resolve(timer.id).then(fire);
                }

                function setTimeout(callback, delay) {
                  const timerId = nextTimerId++;
                  setTimer(timerId, callback, delay, false);
                  return timerId;
                }

                function setInterval(callback, delay) {
                  const timerId = nextTimerId++;
                  setTimer(timerId, callback, delay, true);
                  return timerId;
                }

                function _clearTimer(id) {
                  id = Number(id);
                  const timer = timerMap.get(id);
                  if (timer === undefined) {
                    return;
                  }
                  timerMap.delete(timer.id);
                }

                function clearInterval(id) {
                  _clearTimer(id);
                }

                function clearTimeout(id) {
                  _clearTimer(id);
                }
                
                const slice = Array.prototype.slice;
                
                function javaFunction(target, prop) {
                    // console.log(target.isMethod(prop));
                    // console.log(target.isField(prop));
                    return function invoke() {
                        return $invokeJavaFn(target, prop, slice.call(arguments));
                    }
                }

                const javaHandler = {
                    get(target, prop, receiver) {
                        return javaFunction(target, prop);
                    }
                };
                
                const java = {
                  import(name) {
                    if (name === 'context') {
                        const context = new Java('context', []);
                        // context.testMethod('#99ffff');
                        // return new Proxy(context, javaHandler);
                        return context;
                    }
                    return function wrapper() {
                        const instance = new Java(name, slice.call(arguments));
                        return new Proxy(instance, javaHandler);
                    }
                  }
                };
                
        "#
            ),
        );
        set_deno_resolver(self.deno);
    }

    pub fn execute(&mut self, script: &str) {
        ISOLATE_INIT.call_once(|| unsafe {
            self.initialize();
        });
        unsafe { eval_script(self.deno, c_str!("worker.js"), c_str!(script)) };
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

    extern "C" fn dispatch(data: *mut libc::c_void, promise_id: u32, delay: u32) {
        let isolate = unsafe { Isolate::from_raw_ptr(data) };
        let (task, _trigger) = set_timeout(delay);

        let deno = unsafe { isolate.deno.as_ref() };
        let task = task.and_then(move |_| {
            let deno = deno.unwrap();
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
                    break;
                }
            }
        }

        // We're idle if pending_ops is empty.
        if self.pending_ops.is_empty() {
            Ok(Ready(()))
        } else {
            if self.have_unpolled_ops {
                task::current().notify();
            }
            Ok(NotReady)
        }
    }
}
