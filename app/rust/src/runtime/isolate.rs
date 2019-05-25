use futures::stream::{FuturesUnordered, Stream};
use futures::sync::mpsc;
use futures::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::{task, Async, Future, Poll};
use std::time::Duration;

use futures::Async::*;

use runtime::{create_thread_pool_runtime, evalScript, ptr_to_string, string_to_ptr};
use std::sync::{Arc, Mutex};

pub type OpAsyncFuture = Box<Future<Item = String, Error = ()> + Send>;

pub struct Isolate {
    pub pending_ops: FuturesUnordered<OpAsyncFuture>,
    pub sender: UnboundedSender<Duration>,
    pub receiver: UnboundedReceiver<Duration>,
    pub have_unpolled_ops: bool,
    pub deno_share: Arc<Mutex<*const libc::c_void>>,
    deno: *const libc::c_void,
}

impl Isolate {
    pub fn new(deno: *const libc::c_void) -> Self {
        let (sender, receiver) = mpsc::unbounded::<Duration>();
        Self {
            deno,
            sender,
            receiver,
            pending_ops: FuturesUnordered::new(),
            have_unpolled_ops: false,
            deno_share: Arc::new(Mutex::new(deno)),
        }
    }
    pub unsafe fn execute(&self, script: &str) -> Option<String> {
        return ptr_to_string(evalScript(self.deno, string_to_ptr(script)));
    }
    pub unsafe fn static_execute(deno: *const libc::c_void, script: &str) -> Option<String> {
        return ptr_to_string(evalScript(deno, string_to_ptr(script)));
    }
}

unsafe impl Send for Isolate {}
unsafe impl Sync for Isolate {}

impl Drop for Isolate {
    fn drop(&mut self) {
        adb_debug!(format!("{:?} dropped", &self as *const _));
    }
}

impl Future for Isolate {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            adb_debug!("Loop");
            self.have_unpolled_ops = false;
            #[allow(clippy::match_wild_err_arm)]
            match self.pending_ops.poll() {
                Err(_) => panic!("unexpected op error"),
                Ok(Ready(None)) => break,
                Ok(NotReady) => break,
                Ok(Ready(Some(buf))) => {
                    adb_debug!(buf);
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
