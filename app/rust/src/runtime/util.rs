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
use main;
use jni::objects::{JClass, JObject};

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

pub fn init_runtime(env: &'static JNIEnv, cc: JObject, d: *mut Deno) {
    let mut d = unsafe { *Box::from_raw(d) };

    let mut isolate = Isolate::new();
    let sender = isolate.sender.clone();

    let vm = env.get_java_vm().unwrap();
    let ccRef = env.new_global_ref(cc).unwrap();

    d.rt.executor().spawn(isolate);

    let lazy_future = lazy(move || {
        let delay = 5000 as u32;
        let delay = Duration::from_millis(delay.into());
        match vm.attach_current_thread() {
            Ok(env) => {},
            _ => {}
        }

        Interval::new(Instant::now() + delay, delay)
            .for_each(move |_| {
                adb_debug!(format!("$interval {}s", delay.as_secs()));
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
