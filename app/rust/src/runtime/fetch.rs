use futures::Future;
use reqwest::r#async::Response;
use serde::Deserialize;

use crate::runtime::isolate::Isolate;
use crate::runtime::{ptr_to_string, string_to_ptr, Buf, DenoC, OpAsyncFuture};
use std::os::raw::c_char;

extern "C" {
    fn resolve_promise(d: *const DenoC, promise_id: u32, data: *const c_char) -> *mut c_char;
}

lazy_static! {
    pub static ref CLIENT: reqwest::r#async::Client = reqwest::r#async::Client::new();
}

#[derive(Deserialize, Debug)]
pub struct User {
    name: String,
}

unsafe fn string_to_boxed_bytes(ptr: *mut c_char) -> Buf {
    let s = ptr_to_string(ptr).unwrap();
    let boxed_str = s.to_owned().into_boxed_str();
    boxed_str.into_boxed_bytes()
}

pub fn fetch_async(d: *const DenoC, url: String, promise_id: u32) -> OpAsyncFuture {
    let json = |mut res: Response| res.json::<User>();

    Box::new(
        CLIENT
            .get(&url)
            .send()
            .and_then(json)
            .and_then(move |user: User| unsafe {
                let ptr = resolve_promise(d, promise_id, string_to_ptr(user.name));
                Ok(string_to_boxed_bytes(ptr))
            })
            .map_err(|_| ()),
    )
}

#[no_mangle]
fn fetch(ptr: *mut Isolate, url: *mut c_char, promise_id: u32) {
    if let Some(url) = unsafe { ptr_to_string(url) } {
        let isolate = Isolate::from_c(ptr);
        isolate
            .pending_ops
            .push(fetch_async(isolate.deno, url, promise_id));
    };
}
