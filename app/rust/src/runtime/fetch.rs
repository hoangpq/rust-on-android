use futures::future;
use futures::stream::Stream;
use futures::{FutureExt, TryFutureExt};
use libc::{c_char, c_void};

use crate::runtime::isolate::Isolate;
use crate::runtime::{Buf, DenoC, OpAsyncFuture};

extern "C" {
    fn resolve(d: *const DenoC, promise_id: u32, data: *const c_char);
}

lazy_static! {
    pub static ref CLIENT: reqwest::Client = reqwest::Client::new();
}

pub fn fetch_async(d: *const DenoC, url: &str, promise_id: u32) -> OpAsyncFuture {
    adb_debug!(format!("Send -> {}", url));
    let deno_ref = unsafe { d.as_ref() };

    let fut = CLIENT
        .get(url)
        .send()
        .and_then(move |resp| {
            resp.text().and_then(move |val| {
                let deno_ref = deno_ref.unwrap();
                unsafe { resolve(deno_ref, promise_id, c_str!(val.clone())) };
                future::ready(Ok(boxed!(val.clone())))
            })
        })
        .then(|res| future::ready(boxed!("{}")));

    Box::pin(fut)
}

#[no_mangle]
fn fetch(isolate_ptr: *const c_void, url: *mut c_char, promise_id: u32) {
    let url = unsafe { rust_str!(url) };
    let isolate = unsafe { Isolate::from_raw_ptr(isolate_ptr) };
    isolate
        .pending_ops
        .push(fetch_async(isolate.deno, url, promise_id));
    isolate.have_unpolled_ops = true;
}
