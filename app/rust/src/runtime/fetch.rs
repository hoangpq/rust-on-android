use futures::Future;
use reqwest::r#async::Response;
use serde::Deserialize;

use crate::runtime::{DenoC, ptr_to_string, string_to_ptr};
use crate::runtime::isolate::Isolate;

extern "C" {
    fn resolve_promise(d: *const DenoC, promise_id: u32, data: *const libc::c_char);
}

lazy_static! {
    pub static ref CLIENT: reqwest::r#async::Client = reqwest::r#async::Client::new();
}

#[derive(Deserialize, Debug)]
pub struct User {
    name: String,
}

pub fn fetch_async(
    d: *const DenoC,
    url: String,
    promise_id: u32,
) -> Box<impl Future<Item = (), Error = ()>> {
    let json = |mut res: Response| res.json::<User>();

    Box::new(
        CLIENT
            .get(&url)
            .send()
            .and_then(json)
            .and_then(move |user| unsafe {
                // adb_debug!(format!("Fetch {} -> {}", url, user.name));
                resolve_promise(d, promise_id, string_to_ptr(user.name));
                Ok(())
            })
            .map_err(|_| ()),
    )
}

#[no_mangle]
fn fetch(ptr: *mut Isolate, url: *mut libc::c_char, promise_id: u32) {
    if let Some(url) = unsafe { ptr_to_string(url) } {
        let isolate = Isolate::from_c(ptr);
        isolate
            .pending_ops
            .push(fetch_async(isolate.deno, url, promise_id));
    };
}
