use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    sync::Once,
};

use futures::channel::oneshot;
use futures::stream::{FuturesUnordered, Stream};
use futures::{task, Future, FutureExt, StreamExt, TryFutureExt, TryStreamExt};
use tokio::time::Sleep;

use crate::runtime::timer::{invoke_timer_cb, min_timeout};
use crate::runtime::{eval_script, Buf, DenoC, OpAsyncFuture, TimerFuture};

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
    pub timer: Pin<Box<Sleep>>,
}

unsafe impl Send for Isolate {}

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
            timer: Box::pin(tokio::time::sleep(Default::default())),
        };
    }

    pub fn reset(&mut self) {
        let timeout = min_timeout(self);
        adb_debug!(format!("Scheduling new timeout: {}ms", timeout));

        self.timer
            .as_mut()
            .reset(tokio::time::Instant::now() + Duration::from_millis(timeout.into()));
    }

    pub unsafe fn initialize(&mut self) {
        set_deno_data(self.deno, self.as_raw_ptr());
        eval_script(
            self.deno,
            c_str!("isolate.js"),
            c_str!(
                r#"
                /** Text decoder */
                function TextDecoder() {}

                TextDecoder.prototype.decode = function(octets) {
                    var string = '';
                    var i = 0;
                    while (i < octets.length) {
                        var octet = octets[i];
                        var bytesNeeded = 0;
                        var codePoint = 0;
                        if (octet <= 0x7f) {
                            bytesNeeded = 0;
                            codePoint = octet & 0xff;
                        } else if (octet <= 0xdf) {
                            bytesNeeded = 1;
                            codePoint = octet & 0x1f;
                        } else if (octet <= 0xef) {
                            bytesNeeded = 2;
                            codePoint = octet & 0x0f;
                        } else if (octet <= 0xf4) {
                            bytesNeeded = 3;
                            codePoint = octet & 0x07;
                        }
                        if (octets.length - i - bytesNeeded > 0) {
                            var k = 0;
                            while (k < bytesNeeded) {
                                octet = octets[i + k + 1];
                                codePoint = (codePoint << 6) | (octet & 0x3f);
                                k += 1;
                            }
                        } else {
                            codePoint = 0xfffd;
                            bytesNeeded = octets.length - i;
                        }
                        string += String.fromCodePoint(codePoint);
                        i += bytesNeeded + 1;
                    }
                    return string;
                };

                ArrayBuffer.prototype.toJSON = function() {
                    const ar = new Uint8Array(this);
                    return new TextDecoder().decode(ar);
                }
                
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
                function setTimer(timerId, callback, delay, repeat, ...args) {
                  const timer = {
                    id: timerId,
                    callback,
                    repeat,
                    delay,
                    due: Date.now() + delay,
                  };

                  // Add promise to microtask queue
                  timerMap.set(timer.id, timer);
                }

                function fire(id) {
                  if (!timerMap.has(id)) return;
                  const timer = timerMap.get(id);
                
                  if (timer.due <= Date.now()) {
                    const due = Date.now() + timer.delay;
                    timer.callback();
                    if (timer.repeat) {
                      timer.due = due;
                    } else {
                      timerMap.delete(timer.id);
                    }
                  }
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
                
                // For Java <-> JS bridge
                
                let uiTaskId = 1;
                const uiTaskMap = new Map();
                
                function registerUITask() {
                  let methods;
                  
                  const cmdId = uiTaskId++;
                  const promise = new Promise((resolve, reject) => {
                    methods = { resolve, reject, cmdId };
                  });
                  
                  const promise_ = Object.assign(promise, methods);
                  uiTaskMap.set(cmdId, promise_);
                  
                  // Remove the promise
                  promise_.finally(() => {
                    uiTaskMap.delete(promise_.cmdId);
                  });
                  
                  return {
                    promise: promise,
                    uiTaskId: cmdId
                  };
                }
                
                function resolverUITask(cmdId, data) {
                  Promise.resolve(cmdId).then(id => {
                    if (!uiTaskMap.has(id)) return;
                    const task = uiTaskMap.get(id);
                    task.resolve(data);
                  });
                }
                
                // global timer callback
                function globalTimerCallback() {
                  if (timerMap.size > 0) {
                    for (const key of timerMap.keys()) {
                      Promise.resolve(key).then(fire);  
                    }
                  }
                }
                
                // register global timer
                $newGlobalTimer(() => {
                  let minTimeout = Number.MAX_SAFE_INTEGER;
                  for (const [key, value] of timerMap) {
                    minTimeout = Math.min(minTimeout, value.delay);
                  }
                  return minTimeout;
                }, globalTimerCallback);
                
                const slice = Array.prototype.slice;
                
                function javaFunction(target, prop) {
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
                        context.name = 'activity';
                        return new Proxy(context, javaHandler);
                    }
                    return function wrapper() {
                        const instance = new Java(name, slice.call(arguments));
                        instance.name = name;
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
    pub unsafe fn from_raw_ptr<'a>(ptr: *const libc::c_void) -> &'a mut Self {
        let ptr = ptr as *mut _;
        &mut *ptr
    }

    #[inline]
    fn as_raw_ptr(&self) -> *const libc::c_void {
        self as *const _ as *const libc::c_void
    }

    extern "C" fn dispatch(data: *mut libc::c_void, promise_id: u32, delay: u32) {
        let isolate = unsafe { Isolate::from_raw_ptr(data) };
        let deno = unsafe { isolate.deno.as_ref() };
    }
}

impl Future for Isolate {
    type Output = std::io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Lock the current thread for V8.
        let _locker = LockerScope::new(self.deno);
        self.have_unpolled_ops = false;

        loop {
            match self.pending_ops.poll_next_unpin(cx) {
                Poll::Ready(None) => break,
                Poll::Pending => break,
                Poll::Ready(Some(buf)) => {
                    adb_debug!(format!("{:?}", String::from_utf8_lossy(&buf)));
                    // TODO: refactor to dispatch
                    break;
                }
            }
        }

        match self.timer.poll_unpin(cx) {
            Poll::Ready(_) => {
                invoke_timer_cb(&mut self);
                self.reset();
                Poll::Pending
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
