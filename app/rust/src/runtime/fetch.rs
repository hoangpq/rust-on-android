use futures::Future;
use futures::stream::Stream;
use libc::{c_char, c_void};
use reqwest::r#async::Response;

use crate::runtime::{DenoC, OpAsyncFuture};
use crate::runtime::isolate::Isolate;

extern "C" {
    fn resolve(d: *const DenoC, promise_id: u32, data: *const c_char);
}

lazy_static! {
    pub static ref CLIENT: reqwest::r#async::Client = reqwest::r#async::Client::new();
}

pub fn fetch_async(d: *const DenoC, url: &str, promise_id: u32) -> OpAsyncFuture {
    let raw_transform = |res: Response| {
        res.into_body()
            .concat2()
            .map(|body| String::from_utf8(body.to_vec()).ok())
    };

    adb_debug!(format!("Send -> {}", url));
    let d = unsafe { d.as_ref() };

    Box::new(
        CLIENT
            .get(url)
            .send()
            .and_then(raw_transform)
            .and_then(move |body| match body {
                Some(body) => unsafe {
                    let d = d.unwrap();
                    resolve(d, promise_id, c_str!(body.clone()));
                    Ok(boxed!(body.clone()))
                },
                None => Ok(boxed!("{}")),
            })
            .map_err(|e| adb_debug!(e)),
    )
}

#[no_mangle]
fn fetch(isolate_ptr: *const c_void, url: *mut c_char, promise_id: u32) {
    let url = rust_str!(url);
    let isolate = unsafe { Isolate::from_raw_ptr(isolate_ptr) };
    isolate
        .pending_ops
        .push(fetch_async(isolate.deno, url, promise_id));
}
