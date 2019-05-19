use futures::{future, Future, IntoFuture, Poll, Async};
use futures::stream::{FuturesUnordered, Stream};
use futures::future::{lazy, ok};
use futures::sync::mpsc;

use tokio::runtime;
use std::time::Instant;
use std::time::Duration;

use tokio::timer::Interval;
use std::sync::{Mutex, Arc};
use futures::sync::mpsc::{UnboundedSender, UnboundedReceiver};

use jni::{JNIEnv, JavaVM};
use jni::objects::{JClass, JObject, JStaticMethodID, GlobalRef};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use jni_log::LogPriority::ERROR;
use std::rc::Rc;
use std::fmt;
use serde_json::value::Value::Array;
use std::ptr::null;
use std::cell::RefCell;
use core::borrow::{Borrow, BorrowMut};

fn create_threadpool_runtime() -> tokio::runtime::Runtime {
    use tokio_threadpool::Builder as ThreadPoolBuilder;
    let mut threadpool_builder = ThreadPoolBuilder::new();
    threadpool_builder.panic_handler(|err| std::panic::resume_unwind(err));
    #[allow(deprecated)]
    runtime::Builder::new()
        .threadpool_builder(threadpool_builder)
        .build()
        .unwrap()
}

#[derive(Debug)]
pub struct Deno {
    pub rt: tokio::runtime::Runtime,
}

impl Deno {
    pub fn new() -> Self {
        Self { rt: create_threadpool_runtime() }
    }
}

pub fn create_runtime() -> *mut Deno {
    Box::into_raw(Box::new(Deno::new()))
}

pub type OpAsyncFuture = Box<Future<Item = (), Error = ()> + Send>;

struct Isolate {
    pub pending_ops: FuturesUnordered<OpAsyncFuture>,
    pub sender: UnboundedSender<Duration>,
    pub receiver: UnboundedReceiver<Duration>,
}

impl Isolate {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded::<Duration>();
        Self {
            pending_ops: FuturesUnordered::new(),
            sender,
            receiver,
        }
    }
}

unsafe impl Send for Isolate {}

impl Future for Isolate {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), ()> {
        loop {
            match self.receiver.poll() {
                Err(_) => panic!("unexpected op error"),
                Ok(Async::Ready(Some(data))) => {
                    adb_debug!(format!("$interval {}s", data.as_secs()));
                }
                _ => {}
            }
        }
    }
}

struct State<'a> {
    pub class: GlobalRef,
    pub method: Rc<JStaticMethodID<'a>>,
}

unsafe impl<'a> Send for State<'a> {}

fn get_state<'a>(env: &'static JNIEnv<'a>) -> State<'a> {
    let class = env.find_class("com/node/v8/V8Context").unwrap();
    let method = env.get_static_method_id("com/node/v8/V8Context", "showItemCount", "()V")
        .unwrap();

    State {
        class: env.new_global_ref(JObject::from(class)).unwrap(),
        method: Rc::new(method),
    }
}

extern "C" {
    fn CallStaticVoidMethod(env: *mut JNIEnv, obj: JObject, m_id: JStaticMethodID);
}

pub fn init_runtime(env: &'static JNIEnv, d: *mut Deno) {
    let mut d = unsafe { *Box::from_raw(d) };

    let mut isolate = Isolate::new();
    let sender = isolate.sender.clone();

    d.rt.executor().spawn(isolate);

    let state = get_state(env);
    let jvm = env.get_java_vm().unwrap();

    let lazy_future = lazy(move || {
        let delay = 5000 as u32;
        let delay = Duration::from_millis(delay.into());

        Interval::new(Instant::now() + delay, delay)
            .for_each(move |_| {

                let state = &state;
                let env = jvm.attach_current_thread_as_daemon().unwrap();
                unsafe {
                    CallStaticVoidMethod(
                        Box::into_raw(Box::new(env)),
                        state.class.as_obj(),
                        *state.method.borrow(),
                    );
                };

                sender.unbounded_send(delay).unwrap();
                future::ok(())
            })
            .into_future()
            .map_err(|_| panic!())

    });

    d.rt.block_on_all(lazy_future).unwrap();
}

pub fn create_interval<F>(
    cb: F,
    delay: u32,
) -> (impl Future<Item = (), Error = ()>, futures::sync::oneshot::Sender<()>)
where
    F: Fn() -> (),
{
    let (cancel_tx, cancel_rx) = futures::sync::oneshot::channel::<()>();
    let delay = Duration::from_millis(delay.into());
    let interval_task = future::lazy(move || {
        Interval::new(Instant::now() + delay, delay)
            .for_each(move |_| {
                cb();
                future::ok(())
            })
            .into_future()
            .map_err(|_| panic!())
    }).select(cancel_rx)
        .map(|_| ())
        .map_err(|_| ());

    (interval_task, cancel_tx)
}
