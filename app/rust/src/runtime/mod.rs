use std::pin::Pin;

use futures::Future;
use libc::c_char;
use tokio::runtime;
use tokio::time::Sleep;

pub mod event_loop;
pub mod fetch;
pub mod isolate;
pub mod timer;
pub mod ui_thread;
pub mod util;

#[repr(C)]
pub struct DenoC {
    _unused: [u8; 0],
}

#[allow(non_snake_case)]
extern "C" {
    fn eval_script(d: *const DenoC, name: *const c_char, script: *const c_char);
}

pub type Buf = Box<[u8]>;

pub type OpAsyncFuture = Pin<Box<dyn Future<Output = Buf> + Send>>;
pub type TimerFuture = Pin<Box<dyn Future<Output = Buf>>>;
